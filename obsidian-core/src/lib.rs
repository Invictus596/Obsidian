//! `obsidian-core` — OS-native WebView browser engine lifecycle.
//!
//! This crate is the low-level bridge between the Rust shell and the OS-native
//! WebView via the `wry` crate (Edge WebView2 on Windows, WebKitGTK on Linux,
//! Android WebView on mobile). It owns the WebView lifecycle, navigation,
//! resizing, JavaScript evaluation, and the custom Rust WebExtensions runtime.
//!
//! ## Status
//!
//! Phase 4.2 — Custom Rust WebExtensions runtime: Manifest V3 parser,
//! `chrome.*` IPC polyfill, and content script injector.

mod extensions;

pub use extensions::{ExtensionManager, ExtensionManifest, LoadedExtension};

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use winit::window::Window;
use wry::Rect;

/// A message sent from the WebView's JavaScript IPC to Rust.
#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessage {
    pub channel: String,
    pub payload: serde_json::Value,
}

/// Initialization script injected into every page. Listens for `mouseover`
/// on `<a>` tags and sends the hovered `href` back to Rust via
/// `window.ipc.postMessage(...)` for the floating link preview pill.
const LINK_PREVIEW_INIT_SCRIPT: &str = r#"
(function() {
    window.ipc = window.ipc || { postMessage: function(p) {} };
    document.addEventListener('mouseover', function(e) {
        var a = e.target.closest('a[href]');
        if (a) {
            window.ipc.postMessage({ channel: 'link-hover', payload: { href: a.href } });
        }
    });
    document.addEventListener('mouseout', function(e) {
        var a = e.target.closest('a[href]');
        if (a) {
            window.ipc.postMessage({ channel: 'link-leave', payload: { href: a.href } });
        }
    });
})();
"#;

/// Handle to an embedded OS-native WebView.
pub struct BrowserEngine {
    webview: Arc<Mutex<Option<wry::WebView>>>,
    extension_manager: Arc<Mutex<ExtensionManager>>,
}

impl BrowserEngine {
    /// Create a new `BrowserEngine` embedded as a child of the given winit
    /// `Window`, with initial bounds matching the UI viewport.
    pub fn new(
        window: &Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<Self, EngineInitError> {
        info!(
            "Creating wry WebView at ({}, {}) {}x{}",
            x, y, width, height
        );

        let extension_manager = Arc::new(Mutex::new(ExtensionManager::new()));
        let extension_manager_clone = extension_manager.clone();
        let webview_ref: Arc<Mutex<Option<wry::WebView>>> =
            Arc::new(Mutex::new(None));
        let webview_ref_clone = webview_ref.clone();

        let bounds = Rect {
            position: wry::dpi::Position::Logical(
                winit::dpi::LogicalPosition::new(x as f64, y as f64).into(),
            ),
            size: wry::dpi::Size::Logical(
                winit::dpi::LogicalSize::new(width as f64, height as f64).into(),
            ),
        };

        // Generate the chrome.* IPC polyfill script.
        let polyfill_script = extension_manager.lock().unwrap().generate_polyfill_script();

        // Combined initialization script: link preview + chrome polyfill.
        let init_script = format!("{}\n{}", LINK_PREVIEW_INIT_SCRIPT, polyfill_script);

        let webview = wry::WebViewBuilder::new()
            .with_bounds(bounds)
            .with_initialization_script(&init_script)
            .with_ipc_handler(move |request: wry::http::Request<String>| {
                let msg = request.body();
                let parsed: IpcMessage = match serde_json::from_str(msg) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Failed to parse IPC message: {}", e);
                        return;
                    }
                };

                match parsed.channel.as_str() {
                    "link-hover" => {
                        if let Some(href) = parsed.payload.get("href").and_then(|v| v.as_str()) {
                            info!("Link hover: {}", href);
                            if let Ok(guard) = webview_ref_clone.lock() {
                                if let Some(wv) = guard.as_ref() {
                                    let _ = wv.evaluate_script(&format!(
                                        "window.__obsidianHoveredUrl = '{}';",
                                        href.replace("'", "\\'")
                                    ));
                                }
                            }
                        }
                    }
                    "link-leave" => {
                        if let Ok(guard) = webview_ref_clone.lock() {
                            if let Some(wv) = guard.as_ref() {
                                let _ = wv.evaluate_script("window.__obsidianHoveredUrl = '';");
                            }
                        }
                    }
                    "chrome_api" => {
                        let message = parsed.payload.clone();
                        let mut em_guard = extension_manager_clone.lock().unwrap();
                        let response = em_guard.handle_chrome_api(message);
                        drop(em_guard);

                        let call_id = parsed.payload
                            .get("callId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if let Ok(guard) = webview_ref_clone.lock() {
                            if let Some(wv) = guard.as_ref() {
                                let result_json = serde_json::to_string(
                                    &response.get("result").cloned().unwrap_or(serde_json::Value::Null)
                                ).unwrap_or("null".to_string());
                                let _ = wv.evaluate_script(&format!(
                                    "window.__chromeResponse('{}', {}, {});",
                                    call_id.replace("'", "\\'"),
                                    response.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
                                    result_json
                                ));
                            }
                        }
                    }
                    _ => {
                        error!("Unknown IPC channel: {}", parsed.channel);
                    }
                }
            })
            .build_as_child(window)
            .map_err(|e| {
                error!("Failed to create wry WebView: {}", e);
                EngineInitError::WebViewCreationFailed(e.to_string())
            })?;

        // Store the webview in the shared ref so the IPC handler can access it.
        *webview_ref.lock().unwrap() = Some(webview);

        info!("wry WebView created successfully.");
        Ok(Self {
            webview: webview_ref,
            extension_manager,
        })
    }

    /// Load an unpacked extension from a directory.
    pub fn load_extension(&self, path: &Path) -> Result<String, String> {
        let ext = self
            .extension_manager
            .lock()
            .unwrap()
            .load_unpacked(path)?;
        Ok(format!("Loaded extension '{}' (id: {})", ext.manifest.name, ext.extension_id))
    }

    /// Navigate the WebView to a URL.
    pub fn navigate(&self, url: &str) -> Result<(), EngineInitError> {
        info!("Navigating WebView to {}", url);
        self.webview
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .load_url(url)
            .map_err(|e| {
                error!("Failed to load URL: {}", e);
                EngineInitError::NavigationFailed(e.to_string())
            })?;
        Ok(())
    }

    /// Resize and reposition the WebView to match the UI viewport.
    pub fn resize(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<(), EngineInitError> {
        info!("Resizing WebView to ({}, {}) {}x{}", x, y, width, height);
        let bounds = Rect {
            position: wry::dpi::Position::Logical(
                winit::dpi::LogicalPosition::new(x as f64, y as f64).into(),
            ),
            size: wry::dpi::Size::Logical(
                winit::dpi::LogicalSize::new(width as f64, height as f64).into(),
            ),
        };
        self.webview
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .set_bounds(bounds)
            .map_err(|e| {
                error!("Failed to resize WebView: {}", e);
                EngineInitError::ResizeFailed(e.to_string())
            })?;
        Ok(())
    }

    /// Evaluate JavaScript in the WebView. Used for back/forward/reload
    /// via `history.back()`, `history.forward()`, `location.reload()`.
    pub fn eval(&self, js: &str) -> Result<(), EngineInitError> {
        info!("Evaluating JS: {}", js);
        self.webview
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .evaluate_script(js)
            .map_err(|e| {
                error!("Failed to eval JS: {}", e);
                EngineInitError::JsEvalFailed(e.to_string())
            })?;
        Ok(())
    }

    /// Navigate back one entry in history.
    pub fn go_back(&self) -> Result<(), EngineInitError> {
        self.eval("history.back()")
    }

    /// Navigate forward one entry in history.
    pub fn go_forward(&self) -> Result<(), EngineInitError> {
        self.eval("history.forward()")
    }

    /// Reload the current page.
    pub fn reload(&self) -> Result<(), EngineInitError> {
        self.eval("location.reload()")
    }

    /// Get a reference to the extension manager (for querying scripts, etc.).
    pub fn extension_manager(&self) -> Arc<Mutex<ExtensionManager>> {
        self.extension_manager.clone()
    }
}

/// Errors that can occur while operating the browser engine.
#[derive(Debug, thiserror::Error)]
pub enum EngineInitError {
    #[error("failed to create WebView: {0}")]
    WebViewCreationFailed(String),

    #[error("failed to navigate: {0}")]
    NavigationFailed(String),

    #[error("failed to resize WebView: {0}")]
    ResizeFailed(String),

    #[error("failed to eval JS: {0}")]
    JsEvalFailed(String),

    #[error("invalid command-line arguments: {0}")]
    InvalidArguments(String),
}

/// Resolve the expected path for extension directories relative to the
/// workspace root.
pub fn extension_dir() -> PathBuf {
    PathBuf::from("tests/dummy_extension")
}
