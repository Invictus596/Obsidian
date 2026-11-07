//! `obsidian-ui` — the browser's UI shell.
//!
//! This crate owns the user-facing shell of Obsidian: window management,
//! chrome (address bar, tabs, bookmarks), and the overlay that composites
//! CEF-rendered web content.
//!
//! ## Status
//!
//! Phase 2 scaffolding. The UI shell will eventually drive a native Rust
//! windowing toolkit and composite CEF off-screen / on-screen rendering.

/// Initialize the UI shell.
///
/// A dummy placeholder for Phase 2. In the final architecture this function
/// will create the application window, set up the UI toolkit, and wire up
/// event handling between the shell and the engine core.
pub fn init() -> Result<(), UiInitError> {
    log::info!("Initializing Obsidian UI shell (dummy Phase 2 placeholder)...");

    log::info!("Obsidian UI shell initialized.");

    Ok(())
}

/// Errors that can occur while initializing the UI shell.
#[derive(Debug, thiserror::Error)]
pub enum UiInitError {
    #[error("failed to create application window: {0}")]
    WindowCreationFailed(String),

    #[error("UI toolkit not available: {0}")]
    ToolkitUnavailable(String),
}

// `thiserror` is used for ergonomic error derivation. Add it to
// [dependencies] in obsidian-ui/Cargo.toml when building for real.
