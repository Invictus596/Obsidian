//! `obsidian-ui` — the browser's UI shell.
//!
//! Phase 2: Arc/Helium-inspired fluid, borderless UI shell built with Slint + winit.
//!
//! This crate owns window management, chrome, and the Slint UI overlay that
//! composites CEF-rendered web content. The UI lives in a borderless winit
//! window; the Slint template is defined in `src/ui/main.slint`.
//!
//! Phase 3: hooks into winit resize events and notifies `obsidian-core` via
//! `resize_browser()` so the CEF browser stays in sync with the viewport.

use log::{info, warn};
#[allow(unused_imports)]
use obsidian_core::{
    go_back as core_go_back, go_forward as core_go_forward, navigate as core_navigate,
    reload as core_reload, resize_browser as core_resize_browser,
    show_dev_tools as core_show_dev_tools,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::os::raw::c_void;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

use thiserror::Error;

// The Slint UI component is compiled from `src/ui/main.slint` at build time
// by `slint-build` and made available via the `include_slint!` macro.
// slint::include_slint!("ui/main.slint");

// ---------------------------------------------------------------------------
// Native window handle (HWND on Windows) for CEF viewport attachment
// ---------------------------------------------------------------------------

/// Native window handle type (HWND on Windows, NSView on macOS, etc.).
pub type NativeWindowHandle = RawWindowHandle;

/// Handle to the engine viewport where CEF will eventually render.
///
/// Currently this wraps the main window's native handle. In the full
/// implementation CEF will be parented/clipped to the `engine_viewport`
/// rectangle defined in the Slint template.
pub struct EngineViewportHandle {
    pub handle: RawWindowHandle,
}

impl EngineViewportHandle {
    /// Return the native window handle as an OS-agnostic raw pointer
    /// (`*mut c_void`). This is the value `obsidian_core::create_browser`
    /// expects as its `hwnd` argument.
    pub fn as_raw_pointer(&self) -> *mut c_void {
        match &self.handle {
            RawWindowHandle::Win32(handle) => handle.hwnd.get() as *mut c_void,
            RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr(),
            RawWindowHandle::Xlib(handle) => handle.window as *mut c_void,
            RawWindowHandle::Wayland(handle) => handle.surface.as_ptr(),
            RawWindowHandle::Xcb(handle) => handle.window.get() as *mut c_void,
            _ => std::ptr::null_mut(),
        }
    }
}

/// Initialize the UI shell: spawn a borderless winit window.
///
/// Returns the viewport handle, the winit window, and the event loop.
/// The caller (`obsidian-bin`) is responsible for calling
/// `obsidian_core::create_browser()` with the handle and then running the
/// event loop. Resize events are forwarded to
/// `obsidian_core::resize_browser()` automatically.
#[allow(deprecated)]
pub fn init() -> Result<(EngineViewportHandle, Window, EventLoop<()>), UiInitError> {
    info!("Initializing Obsidian UI shell (Phase 2: Arc/Helium fluid UI)...");

    let event_loop = EventLoop::new().map_err(|e| {
        UiInitError::EventLoopFailed(e.to_string())
    })?;

    let window_attributes = WindowAttributes::default()
        .with_title("Obsidian")
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .with_decorations(false) // borderless
        .with_transparent(true); // translucent backdrop for CEF composition

    let window = event_loop
        .create_window(window_attributes)
        .map_err(|e| UiInitError::WindowCreationFailed(e.to_string()))?;

    // Capture the native window handle (HWND on Windows) so obsidian-core can
    // later attach the CEF browser to this viewport.
    let handle: RawWindowHandle = window
        .window_handle()
        .map_err(|e| UiInitError::ToolkitUnavailable(e.to_string()))?
        .into();
    let hwnd = match &handle {
        RawWindowHandle::Win32(h) => h.hwnd.get() as *mut c_void,
        RawWindowHandle::AppKit(h) => h.ns_view.as_ptr(),
        RawWindowHandle::Xlib(h) => h.window as *mut c_void,
        RawWindowHandle::Wayland(h) => h.surface.as_ptr(),
        RawWindowHandle::Xcb(h) => h.window.get() as *mut c_void,
        _ => std::ptr::null_mut(),
    };
    info!("Native window handle acquired: {:?}", hwnd);

    // Load and render the Slint UI inside the winit window.
    setup_slint_ui(&window, hwnd);

    info!("Obsidian UI shell initialized (borderless, compact-mode).");

    Ok((EngineViewportHandle { handle }, window, event_loop))
}

/// Expose the native window handle of the engine viewport so `obsidian-core`
/// can attach the CEF browser to it.
///
/// On Windows this is an `HWND`. Callers should use the handle from the
/// [`EngineViewportHandle`] returned by [`init()`]; this function is provided
/// as a standalone accessor for global lookup once the handle is stored.
pub fn engine_viewport_handle() -> Option<RawWindowHandle> {
    // In the full implementation, this handle is stored in a static/lazy cell
    // after `init()` creates the window. For scaffolding, callers should use
    // the handle from `EngineViewportHandle` returned by `init()`.
    None
}

/// Run the winit event loop, forwarding resize and navigation events to `obsidian-core`.
///
/// When the window is resized or moved, the new `engine_viewport` geometry
/// (x, y, width, height) is passed to `obsidian_core::resize_browser()` so
/// the CEF browser stays in sync with the Slint viewport.
///
/// In the full implementation, the `engine_viewport` rectangle's absolute
/// x, y, width, height would be read directly from the Slint component. For
/// scaffolding, we use the winit window's outer position and size.
#[allow(deprecated)]
pub fn run_event_loop(
    window: &Window,
    event_loop: EventLoop<()>,
    hwnd: *mut c_void,
) -> Result<(), UiInitError> {
    info!("Starting winit event loop with CEF resize and navigation forwarding...");

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Read the window's outer position as the viewport origin.
                let position = window
                    .outer_position()
                    .unwrap_or(PhysicalPosition::new(0, 0));
                info!(
                    "Viewport moved/resized to ({}, {}) {}x{} — notifying CEF browser",
                    position.x, position.y, size.width, size.height
                );
                if let Err(e) = core_resize_browser(
                    hwnd,
                    position.x as i32,
                    position.y as i32,
                    size.width as i32,
                    size.height as i32,
                ) {
                    warn!("Failed to resize CEF browser: {}", e);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Moved(position),
                ..
            } => {
                // Window was moved; fetch current size to complete the geometry.
                let size = window.outer_size();
                info!(
                    "Viewport moved to ({}, {}) {}x{} — notifying CEF browser",
                    position.x, position.y, size.width, size.height
                );
                if let Err(e) = core_resize_browser(
                    hwnd,
                    position.x as i32,
                    position.y as i32,
                    size.width as i32,
                    size.height as i32,
                ) {
                    warn!("Failed to resize CEF browser: {}", e);
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

/// Set up Slint UI rendering inside the winit window and wire up the
/// navigation and window-control callbacks to `obsidian-core` / `winit`.
fn setup_slint_ui(window: &Window, hwnd: *mut c_void) {
    // Slint integrates with winit via `slint::platform::winit::WindowAdapter`:
    //
    // use slint::platform::winit::WindowAdapter;
    // let adapter = WindowAdapter::new(window, event_loop.create_proxy(), false);
    // let ui = MainWindow::new();
    //
    // Wire the window_drag_requested callback to winit's drag_window():
    //
    // ui.on_window_drag_requested(move || {
    //     let _ = window.drag_window();
    // });
    //
    // Wire the request_navigation callback:
    //
    // ui.on_request_navigation(move |url: slint::SharedString| {
    //     let url_str = url.trim();
    //     let formatted_url = if url_str.starts_with("http://")
    //         || url_str.starts_with("https://")
    //     {
    //         url_str.to_string()
    //     } else if !url_str.is_empty() {
    //         format!("https://{}", url_str)
    //     } else {
    //         return;
    //     };
    //     let _ = core_navigate(hwnd, &formatted_url);
    // });
    //
    // Wire the back / forward / reload callbacks:
    //
    // ui.on_request_back(move || {
    //     let _ = core_go_back(hwnd);
    // });
    // ui.on_request_forward(move || {
    //     let _ = core_go_forward(hwnd);
    // });
    // ui.on_request_reload(move || {
    //     let _ = core_reload(hwnd);
    // });
    //
    // Wire the DevTools callback:
    //
    // ui.on_request_devtools(move || {
    //     let _ = core_show_dev_tools(hwnd);
    // });
    //
    // ui.show_with_adapter(adapter);
    //
    // The MainWindow component is generated from src/ui/main.slint via
    // slint::include_slint!("ui/main.slint");.
    //
    // This wiring is scaffolding; the adapter will be connected once
    // slint-platform-winit integration is fully configured.
    let _ = window;
    let _ = hwnd;
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
}
