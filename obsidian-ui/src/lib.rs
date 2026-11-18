//! `obsidian-ui` — ultra-minimal compact UI shell.
//!
//! Phase 4.1: Arc/Helium-inspired fluid, borderless UI shell built with Slint
//! + winit, embedding an OS-native WebView via `wry`.
//!
//! This crate owns window management, chrome, and the Slint UI overlay that
//! composites WebView-rendered web content. The UI lives in a borderless
//! winit window; the Slint template is defined in `src/ui/main.slint`.

use log::{info, warn};
use obsidian_core::BrowserEngine;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

use thiserror::Error;

// The Slint UI component is compiled from `src/ui/main.slint` at build time
// by `slint-build` and made available via the `include_slint!` macro.
// slint::include_slint!("ui/main.slint");

// ---------------------------------------------------------------------------
// Native window handle for wry WebView embedding
// ---------------------------------------------------------------------------

/// Native window handle type (HWND on Windows, NSView on macOS, etc.).
pub type NativeWindowHandle = RawWindowHandle;

/// Initialize the UI shell: spawn a borderless winit window.
///
/// Returns the winit `Window` and `EventLoop`. The caller (`obsidian-bin`)
/// is responsible for creating a `obsidian_core::BrowserEngine` attached to
/// this window, then running the event loop. Resize events are forwarded to
/// the WebView's `set_bounds()` automatically.
#[allow(deprecated)]
pub fn init() -> Result<(Window, EventLoop<()>), UiInitError> {
    info!("Initializing Obsidian UI shell (Phase 4.1: ultra-compact UI)...");

    let event_loop = EventLoop::new().map_err(|e| {
        UiInitError::EventLoopFailed(e.to_string())
    })?;

    let window_attributes = WindowAttributes::default()
        .with_title("Obsidian")
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .with_decorations(false) // borderless
        .with_transparent(true); // translucent backdrop for WebView composition

    let window = event_loop
        .create_window(window_attributes)
        .map_err(|e| UiInitError::WindowCreationFailed(e.to_string()))?;

    // Capture the native window handle (needed for wry WebView embedding).
    let handle: RawWindowHandle = window
        .window_handle()
        .map_err(|e| UiInitError::ToolkitUnavailable(e.to_string()))?
        .into();
    info!("Native window handle acquired: {:?}", handle);

    // Load and render the Slint UI inside the winit window.
    setup_slint_ui(&window);

    info!("Obsidian UI shell initialized (borderless, compact-mode).");

    Ok((window, event_loop))
}

/// Expose the native window handle of the engine viewport so `obsidian-core`
/// can attach the wry WebView to it.
pub fn engine_viewport_handle() -> Option<RawWindowHandle> {
    // In the full implementation, this handle is stored in a static/lazy cell
    // after `init()` creates the window. For scaffolding, callers should use
    // the `Window` returned by `init()` directly.
    None
}

/// Run the winit event loop, forwarding resize/sidebar events to the WebView.
///
/// When the window is resized or moved, the new geometry (x, y, width, height)
/// is passed to `webview.set_bounds()` so the WebView stays in sync with the
/// Slint `engine_viewport` rectangle. The sidebar width is derived from the
/// Slint `sidebar_open` state (not directly observable from Rust in this
/// scaffolding, so we use the full window width minus the sidebar when open).
#[allow(deprecated)]
pub fn run_event_loop(
    window: &Window,
    event_loop: EventLoop<()>,
    webview: &BrowserEngine,
) -> Result<(), UiInitError> {
    info!("Starting winit event loop with WebView bounds forwarding...");

    // Initial bounds: full window below the 36px top bar, minus sidebar when open.
    let initial_x = 220; // sidebar width when open
    let initial_y = 36;  // top bar height
    let size = window.inner_size();
    let initial_width = size.width.saturating_sub(220);
    let initial_height = size.height.saturating_sub(36);

    webview
        .resize(initial_x as i32, initial_y as i32, initial_width, initial_height)
        .map_err(|e| UiInitError::WebViewError(e.to_string()))?;

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let position = window
                    .outer_position()
                    .unwrap_or(PhysicalPosition::new(0, 0));
                info!(
                    "Viewport resized to ({}, {}) {}x{} — updating WebView bounds",
                    position.x, position.y, size.width, size.height
                );
                if let Err(e) = webview.resize(
                    position.x as i32 + 220,
                    position.y as i32 + 36,
                    size.width.saturating_sub(220),
                    size.height.saturating_sub(36),
                ) {
                    warn!("Failed to resize WebView: {}", e);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Moved(position),
                ..
            } => {
                let size = window.outer_size();
                info!(
                    "Viewport moved to ({}, {}) {}x{} — updating WebView bounds",
                    position.x, position.y, size.width, size.height
                );
                if let Err(e) = webview.resize(
                    position.x as i32 + 220,
                    position.y as i32 + 36,
                    size.width.saturating_sub(220),
                    size.height.saturating_sub(36),
                ) {
                    warn!("Failed to resize WebView: {}", e);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            _ => {}
        }
    }).map_err(|e| UiInitError::EventLoopFailed(e.to_string()))?;

    Ok(())
}

/// Set up Slint UI rendering inside the winit window.
fn setup_slint_ui(_window: &Window) {
    // Slint integrates with winit via `slint::platform::winit::WindowAdapter`:
    //
    // use slint::platform::winit::WindowAdapter;
    // let adapter = WindowAdapter::new(window, event_loop.create_proxy(), false);
    // let ui = MainWindow::new();
    //
    // Wire the Slint callbacks:
    //
    // ui.on_window_drag_requested(move || {
    //     let _ = window.drag_window();
    // });
    // ui.on_request_navigation(move |url| {
    //     let _ = webview.navigate(&url);
    // });
    // ui.on_request_back(move || { let _ = webview.go_back(); });
    // ui.on_request_forward(move || { let _ = webview.go_forward(); });
    // ui.on_request_reload(move || { let _ = webview.reload(); });
    // ui.on_request_devtools(move || { /* webview.devtools() */ });
    // ui.on_request_load_extension(move || { /* webview.load_extension() */ });
    // ui.on_request_sidebar_toggle(move || { /* toggle sidebar width */ });
    //
    // ui.show_with_adapter(adapter);
    //
    // The MainWindow component is generated from src/ui/main.slint via
    // slint::include_slint!("ui/main.slint");.
    //
    // This wiring is scaffolding; the adapter will be connected once
    // slint-platform-winit integration is fully configured.
    warn!("Slint UI rendering is scaffolding; wire up WindowAdapter next.");
}

/// Errors that can occur while initializing the UI shell.
#[derive(Debug, Error)]
pub enum UiInitError {
    #[error("failed to create application window: {0}")]
    WindowCreationFailed(String),

    #[error("failed to create winit event loop: {0}")]
    EventLoopFailed(String),

    #[error("UI toolkit not available: {0}")]
    ToolkitUnavailable(String),

    #[error("webview error: {0}")]
    WebViewError(String),
}
