//! `obsidian-core` — OS-native WebView browser engine lifecycle.
//!
//! This crate is the low-level bridge between the Rust shell and the OS-native
//! WebView via the `wry` crate (Edge WebView2 on Windows, WebKitGTK on Linux,
//! Android WebView on mobile). It owns the WebView lifecycle, navigation,
//! resizing, and JavaScript evaluation.
//!
//! ## Status
//!
//! Phase 4.1 — wry OS-native WebView (~10MB target binary) paired with a
//! custom Rust WebExtensions runtime.

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
    webview: wry::WebView,
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

        let bounds = Rect {
            position: wry::dpi::Position::Logical(
                winit::dpi::LogicalPosition::new(x as f64, y as f64).into(),
            ),
            size: wry::dpi::Size::Logical(
                winit::dpi::LogicalSize::new(width as f64, height as f64).into(),
            ),
        };

        let webview = wry::WebViewBuilder::new()
            .with_bounds(bounds)
            .with_initialization_script(LINK_PREVIEW_INIT_SCRIPT)
            .build_as_child(window)
            .map_err(|e| {
                error!("Failed to create wry WebView: {}", e);
                EngineInitError::WebViewCreationFailed(e.to_string())
            })?;

        info!("wry WebView created successfully.");
        Ok(Self { webview })
    }

    /// Navigate the WebView to a URL.
    pub fn navigate(&self, url: &str) -> Result<(), EngineInitError> {
        info!("Navigating WebView to {}", url);
        self.webview
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
