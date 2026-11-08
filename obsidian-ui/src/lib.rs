//! `obsidian-ui` — the browser's UI shell.
//!
//! Phase 2: Arc/Helium-inspired fluid, borderless UI shell built with Slint + winit.
//!
//! This crate owns window management, chrome, and the Slint UI overlay that
//! composites CEF-rendered web content. The UI lives in a borderless winit
//! window; the Slint template is defined in `src/ui/main.slint`.

use log::{info, warn};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::dpi::LogicalSize;
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

/// Initialize the UI shell: spawn a borderless winit window and render the
/// Slint UI inside it.
#[allow(deprecated)]
pub fn init() -> Result<EngineViewportHandle, UiInitError> {
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
    info!("Native window handle acquired: {:?}", handle);

    // Load and render the Slint UI inside the winit window.
    setup_slint_ui(&window, &event_loop);

    info!("Obsidian UI shell initialized (borderless, compact-mode).");

    Ok(EngineViewportHandle { handle })
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

/// Set up Slint UI rendering inside the winit window.
fn setup_slint_ui(_window: &Window, _event_loop: &EventLoop<()>) {
    // Slint integrates with winit via `slint::platform::winit::WindowAdapter`:
    //
    // use slint::platform::winit::WindowAdapter;
    // let adapter = WindowAdapter::new(window, event_loop.create_proxy(), false);
    // let ui = MainWindow::new();
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
}
