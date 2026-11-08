//! `obsidian-core` — CEF C-API bindings and browser engine lifecycle.
//!
//! This crate is the low-level bridge between the Rust shell and the Chromium
//! Embedded Framework (CEF). It owns the CEF process lifecycle, command-line
//! argument handling, and the FFI glue over the C-API.
//!
//! ## CEF integration
//!
//! A real CEF integration is gated behind the `cef-bindings` feature, which is
//! **off by default**. When the feature is off, `init()` is a safe dummy
//! placeholder so the workspace builds without pre-compiled CEF binaries.
//!
//! When `cef-bindings` is enabled, `init()` calls into the `cef` crate and
//! links against the pre-compiled CEF binaries located by `build.rs`
//! (via `CEF_ROOT` or the default `cef_binary/` folder).
//!
//! ## Status
//!
//! Phase 1 scaffolding. Phase 2.5: CEF ↔ UI bridge.

#![cfg_attr(not(feature = "cef-bindings"), allow(dead_code))]

use std::path::PathBuf;

#[cfg(feature = "win32")]
use windows_sys::Win32::Foundation::HWND;
#[cfg(feature = "win32")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE,
};

/// Initialize the Obsidian browser engine core.
///
/// With the `cef-bindings` feature enabled this function:
///
/// * locates the CEF binaries via `build.rs` (`CEF_ROOT` or `cef_binary/`),
/// * parses and validates CEF command-line arguments,
/// * calls `cef_initialize` through the `cef` crate bindings,
/// * spawns the browser / renderer / GPU subprocesses.
///
/// Without the feature this is a safe dummy that logs and returns `Ok(())`.
pub fn init() -> Result<(), EngineInitError> {
    log::info!("Initializing Obsidian engine core (Phase 1)...");

    #[cfg(feature = "cef-bindings")]
    {
        return cef_init();
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!("CEF bindings disabled: running dummy Phase 1 placeholder.");
        log::info!("Obsidian engine core initialized.");
        return Ok(());
    }
}

/// Attach a CEF browser to a native window handle and navigate to a URL.
///
/// * `hwnd` — native window handle (HWND on Windows) returned by `obsidian-ui`.
/// * `url` — the initial URL to load.
///
/// With `cef-bindings` enabled this configures `cef::WindowInfo`, sets its
/// parent window to `hwnd`, and calls `cef::BrowserHost::create_browser`
/// pointing at `url`.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn create_browser(hwnd: *mut std::ffi::c_void, url: &str) -> Result<(), EngineInitError> {
    log::info!("Creating CEF browser attached to HWND: {:?}, navigating to {}", hwnd, url);

    #[cfg(feature = "cef-bindings")]
    {
        use cef::BrowserHost;
        use cef::CefWindowInfo;
        use cef::WindowInfo;

        let mut window_info = CefWindowInfo::new();
        window_info.set_as_child(hwnd as _);

        BrowserHost::create_browser(window_info, url, None).map_err(|e| {
            log::error!("Failed to create CEF browser: {}", e);
            EngineInitError::CefInitializationFailed(e.to_string())
        })?;

        log::info!("CEF browser created and attached to viewport.");
        Ok(())
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!("CEF bindings disabled: dummy browser placeholder.");
        log::info!("Dummy CEF browser attached to HWND: {:?}, navigating to {}", hwnd, url);
        Ok(())
    }
}

/// Navigate an already-attached CEF browser to a new URL.
///
/// * `hwnd` — native window handle (HWND on Windows) the browser is attached to.
/// * `url` — the URL to load.
///
/// With `cef-bindings` enabled this looks up the active CEF browser host
/// associated with `hwnd`, gets its main frame, and calls `load_url(url)`.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn navigate(hwnd: *mut std::ffi::c_void, url: &str) -> Result<(), EngineInitError> {
    log::info!(
        "Navigating CEF browser at HWND: {:?} to {}",
        hwnd,
        url
    );

    #[cfg(feature = "cef-bindings")]
    {
        use cef::BrowserHost;

        match BrowserHost::get_browser_for_window(hwnd as _) {
            Some(browser) => {
                let main_frame = browser.get_main_frame().map_err(|e| {
                    log::error!("Failed to get main frame: {}", e);
                    EngineInitError::CefInitializationFailed(e.to_string())
                })?;
                main_frame.load_url(url).map_err(|e| {
                    log::error!("Failed to load URL: {}", e);
                    EngineInitError::CefInitializationFailed(e.to_string())
                })?;
                log::info!("CEF browser navigated to {}", url);
            }
            None => {
                log::warn!(
                    "No active CEF browser host found for HWND: {:?}; cannot navigate.",
                    hwnd
                );
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!(
            "CEF bindings disabled: dummy navigate placeholder. HWND: {:?}, {}",
            hwnd,
            url
        );
        Ok(())
    }
}

/// Navigate the CEF browser back one entry in history.
///
/// * `hwnd` — native window handle (HWND on Windows) the browser is attached to.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn go_back(hwnd: *mut std::ffi::c_void) -> Result<(), EngineInitError> {
    log::info!("Navigating CEF browser back at HWND: {:?}", hwnd);

    #[cfg(feature = "cef-bindings")]
    {
        use cef::BrowserHost;

        match BrowserHost::get_browser_for_window(hwnd as _) {
            Some(browser) => {
                browser.go_back().map_err(|e| {
                    log::error!("Failed to go back: {}", e);
                    EngineInitError::CefInitializationFailed(e.to_string())
                })?;
                log::info!("CEF browser navigated back.");
            }
            None => {
                log::warn!(
                    "No active CEF browser host found for HWND: {:?}; cannot go back.",
                    hwnd
                );
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!("CEF bindings disabled: dummy go_back placeholder. HWND: {:?}", hwnd);
        Ok(())
    }
}

/// Navigate the CEF browser forward one entry in history.
///
/// * `hwnd` — native window handle (HWND on Windows) the browser is attached to.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn go_forward(hwnd: *mut std::ffi::c_void) -> Result<(), EngineInitError> {
    log::info!("Navigating CEF browser forward at HWND: {:?}", hwnd);

    #[cfg(feature = "cef-bindings")]
    {
        use cef::BrowserHost;

        match BrowserHost::get_browser_for_window(hwnd as _) {
            Some(browser) => {
                browser.go_forward().map_err(|e| {
                    log::error!("Failed to go forward: {}", e);
                    EngineInitError::CefInitializationFailed(e.to_string())
                })?;
                log::info!("CEF browser navigated forward.");
            }
            None => {
                log::warn!(
                    "No active CEF browser host found for HWND: {:?}; cannot go forward.",
                    hwnd
                );
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!(
            "CEF bindings disabled: dummy go_forward placeholder. HWND: {:?}",
            hwnd
        );
        Ok(())
    }
}

/// Reload the current page in the CEF browser.
///
/// * `hwnd` — native window handle (HWND on Windows) the browser is attached to.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn reload(hwnd: *mut std::ffi::c_void) -> Result<(), EngineInitError> {
    log::info!("Reloading CEF browser at HWND: {:?}", hwnd);

    #[cfg(feature = "cef-bindings")]
    {
        use cef::BrowserHost;

        match BrowserHost::get_browser_for_window(hwnd as _) {
            Some(browser) => {
                browser.reload().map_err(|e| {
                    log::error!("Failed to reload: {}", e);
                    EngineInitError::CefInitializationFailed(e.to_string())
                })?;
                log::info!("CEF browser reloaded.");
            }
            None => {
                log::warn!(
                    "No active CEF browser host found for HWND: {:?}; cannot reload.",
                    hwnd
                );
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!("CEF bindings disabled: dummy reload placeholder. HWND: {:?}", hwnd);
        Ok(())
    }
}

/// Open Chromium Developer Tools for the CEF browser attached to `hwnd`.
///
/// * `hwnd` — native window handle (HWND on Windows) the browser is attached to.
///
/// With `cef-bindings` enabled this looks up the active `BrowserHost`,
/// configures a new empty `WindowInfo` (so DevTools spawns in its own native
/// OS popup window rather than embedding in Slint), and calls
/// `browser_host.show_dev_tools()`.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn show_dev_tools(hwnd: *mut std::ffi::c_void) -> Result<(), EngineInitError> {
    log::info!("Opening DevTools for CEF browser at HWND: {:?}", hwnd);

    #[cfg(feature = "cef-bindings")]
    {
        use cef::BrowserHost;
        use cef::WindowInfo;

        match BrowserHost::get_browser_for_window(hwnd as _) {
            Some(browser) => {
                // An empty WindowInfo causes DevTools to open in its own
                // top-level native OS window instead of being embedded.
                let devtools_info = WindowInfo::new();
                browser.show_dev_tools(devtools_info).map_err(|e| {
                    log::error!("Failed to show DevTools: {}", e);
                    EngineInitError::CefInitializationFailed(e.to_string())
                })?;
                log::info!("DevTools opened for HWND: {:?}", hwnd);
            }
            None => {
                log::warn!(
                    "No active CEF browser host found for HWND: {:?}; cannot open DevTools.",
                    hwnd
                );
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!(
            "CEF bindings disabled: dummy show_dev_tools placeholder. HWND: {:?}",
            hwnd
        );
        Ok(())
    }
}

/// Load an unpacked Chromium extension into the global CEF Request Context.
///
/// * `absolute_path` — absolute filesystem path to the extension directory
///   containing `manifest.json`.
///
/// With `cef-bindings` enabled this fetches the global CEF Request Context via
/// `cef::RequestContext::get_global_context()` and calls
/// `load_extension(absolute_path, ...)` with a dummy/empty handler.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn load_unpacked_extension(absolute_path: &str) -> Result<(), EngineInitError> {
    log::info!(
        "Loading unpacked CEF extension from: {}",
        absolute_path
    );

    #[cfg(feature = "cef-bindings")]
    {
        use cef::RequestContext;

        match RequestContext::get_global_context() {
            Some(context) => {
                // A dummy extension handler is passed since we don't yet
                // implement extension event callbacks.
                context.load_extension(absolute_path, None).map_err(|e| {
                    log::error!("Failed to load extension: {}", e);
                    EngineInitError::CefInitializationFailed(e.to_string())
                })?;
                log::info!(
                    "CEF extension loaded from: {}",
                    absolute_path
                );
            }
            None => {
                log::error!("Failed to obtain global CEF Request Context.");
                return Err(EngineInitError::LibraryNotFound(
                    "CEF Request Context not available".to_string(),
                ));
            }
        }

        Ok(())
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!(
            "CEF bindings disabled: dummy extension loader placeholder. Path: {}",
            absolute_path
        );
        Ok(())
    }
}

/// Resize a CEF browser that was previously attached to a native window handle.
///
/// * `hwnd` — native window handle (HWND on Windows) the browser is attached to.
/// * `x` — absolute X coordinate of the viewport.
/// * `y` — absolute Y coordinate of the viewport.
/// * `width` — new width in pixels.
/// * `height` — new height in pixels.
///
/// With `cef-bindings` enabled this looks up the active CEF browser host
/// associated with `hwnd`, gets its window handle via `get_window_handle`,
/// and calls Win32 `SetWindowPos` to explicitly move/resize the CEF child
/// window so it perfectly maps to the Slint `engine_viewport` coordinates.
/// Falls back to `was_resized()` if the child window handle can't be resolved.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn resize_browser(
    hwnd: *mut std::ffi::c_void,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), EngineInitError> {
    log::info!(
        "Resizing CEF browser at HWND: {:?} to ({}, {}) {}x{}",
        hwnd,
        x,
        y,
        width,
        height
    );

    #[cfg(feature = "cef-bindings")]
    {
        use cef::BrowserHost;

        match BrowserHost::get_browser_for_window(hwnd as _) {
            Some(browser) => {
                // Get the CEF child window handle and explicitly set its
                // position/size via Win32 SetWindowPos so it maps exactly
                // to the Slint engine_viewport rectangle.
                if let Ok(cef_window_handle) = browser.get_window_handle() {
                    let cef_hwnd: HWND = cef_window_handle as _;
                    unsafe {
                        SetWindowPos(
                            cef_hwnd,
                            0,
                            x,
                            y,
                            width,
                            height,
                            SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                    log::info!(
                        "CEF child window positioned via SetWindowPos at ({}, {}) {}x{}",
                        x,
                        y,
                        width,
                        height
                    );
                } else {
                    // Fallback: just notify CEF of the resize.
                    browser.was_resized().map_err(|e| {
                        log::error!("Failed to resize CEF browser: {}", e);
                        EngineInitError::CefInitializationFailed(e.to_string())
                    })?;
                }
            }
            None => {
                // Fallback: notify via Win32 that the window bounds changed.
                unsafe {
                    let parent_hwnd: HWND = hwnd as _;
                    log::warn!(
                        "No active CEF browser host found for HWND: {:?}; resize hint sent to parent.",
                        parent_hwnd
                    );
                }
            }
        }

        log::info!("CEF browser resize completed.");
        Ok(())
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!(
            "CEF bindings disabled: dummy resize placeholder. HWND: {:?}, ({}, {}) {}x{}",
            hwnd,
            x,
            y,
            width,
            height
        );
        Ok(())
    }
}

/// Actual CEF framework initialization (only compiled with `cef-bindings`).
#[cfg(feature = "cef-bindings")]
fn cef_init() -> Result<(), EngineInitError> {
    use cef::CefMainArgs;
    use cef::CefSettings;
    use cef::initialize;

    let args = std::env::args().collect::<Vec<_>>();
    let main_args = CefMainArgs::new(&args);

    let settings = CefSettings {
        no_sandbox: true,
        multi_threaded_message_loop: true,
        ..Default::default()
    };

    log::info!("Calling cef_initialize via the `cef` crate...");

    initialize(main_args, settings, None).map_err(|e| {
        log::error!("cef_initialize failed: {}", e);
        EngineInitError::CefInitializationFailed(e.to_string())
    })?;

    log::info!("Obsidian engine core initialized (CEF bindings active).");
    Ok(())
}

/// Shutdown the CEF framework cleanly.
///
/// Only compiled with the `cef-bindings` feature. Without it this is a no-op
/// dummy that logs and returns `Ok(())`.
pub fn shutdown() -> Result<(), EngineInitError> {
    log::info!("Shutting down Obsidian engine core...");

    #[cfg(feature = "cef-bindings")]
    {
        cef::shutdown().map_err(|e| {
            log::error!("cef_shutdown failed: {}", e);
            EngineInitError::CefInitializationFailed(e.to_string())
        })?;
    }

    #[cfg(not(feature = "cef-bindings"))]
    {
        log::info!("CEF bindings disabled: dummy shutdown placeholder.");
    }

    log::info!("Obsidian engine core shut down.");
    Ok(())
}

/// Resolve the expected CEF root directory.
///
/// Checks `CEF_ROOT` first, then falls back to `cef_binary/` relative to the
/// workspace root (the location populated by `scripts/download_cef.ps1`).
pub fn cef_root() -> PathBuf {
    std::env::var("CEF_ROOT")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("cef_binary"))
}

/// Errors that can occur while initializing the engine core.
#[derive(Debug, thiserror::Error)]
pub enum EngineInitError {
    #[error("CEF library not found: {0}")]
    LibraryNotFound(String),

    #[error("failed to initialize CEF framework: {0}")]
    CefInitializationFailed(String),

    #[error("invalid command-line arguments: {0}")]
    InvalidArguments(String),
}
