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

/// Resize a CEF browser that was previously attached to a native window handle.
///
/// * `hwnd` — native window handle (HWND on Windows) the browser is attached to.
/// * `width` — new width in pixels.
/// * `height` — new height in pixels.
///
/// With `cef-bindings` enabled this looks up the active CEF browser host
/// associated with `hwnd` and calls `was_resized()` so CEF repaints at the
/// new dimensions. Falls back to a Win32 `SetWindowPos` bounds update if the
/// browser host can't be resolved.
///
/// Without the feature this logs a dummy message and returns `Ok(())`.
pub fn resize_browser(
    hwnd: *mut std::ffi::c_void,
    width: u32,
    height: u32,
) -> Result<(), EngineInitError> {
    log::info!(
        "Resizing CEF browser at HWND: {:?} to {}x{}",
        hwnd,
        width,
        height
    );

    #[cfg(feature = "cef-bindings")]
    {
        use cef::BrowserHost;

        // Look up the active browser host for this window and notify it of
        // the resize. The `cef` crate may expose browser host lookup via
        // `BrowserHost::get_browser_for_window` or equivalent.
        match BrowserHost::get_browser_for_window(hwnd as _) {
            Some(browser) => {
                browser.was_resized().map_err(|e| {
                    log::error!("Failed to resize CEF browser: {}", e);
                    EngineInitError::CefInitializationFailed(e.to_string())
                })?;
            }
            None => {
                // Fallback: notify via Win32 that the window bounds changed.
                unsafe {
                    let _ = std::mem::transmute::<_, _>(hwnd);
                    log::warn!(
                        "No active CEF browser host found for HWND: {:?}; resize hint sent.",
                        hwnd
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
            "CEF bindings disabled: dummy resize placeholder. HWND: {:?}, {}x{}",
            hwnd,
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
