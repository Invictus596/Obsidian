//! `obsidian-core` — CEF C-API bindings and browser engine lifecycle.
//!
//! This crate is the low-level bridge between the Rust shell and the Chromium
//! Embedded Framework (CEF). It owns the CEF process lifecycle, command-line
//! argument handling, and the FFI glue over the C-API.
//!
//! ## Status
//!
//! Phase 1 scaffolding. A real CEF integration requires pre-compiled CEF
//! binaries (see the project README) and `cef-sys`-style FFI bindings.

/// Initialize the Obsidian browser engine core.
///
/// This is a dummy placeholder for Phase 1 of the project. Once pre-compiled
/// CEF binaries are available, this function will:
///
/// * parse and validate CEF command-line arguments,
/// * initialize the CEF framework (`cef_initialize`),
/// * spawn the browser / renderer / GPU subprocesses, and
/// * return a handle to the running engine for the UI shell to drive.
pub fn init() -> Result<(), EngineInitError> {
    log::info!("Initializing Obsidian engine core (dummy Phase 1 placeholder)...");

    // Simulate successful engine initialization.
    log::info!("Obsidian engine core initialized.");

    Ok(())
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

// `thiserror` is used above for ergonomic error derivation. It is a
// development-only convenience and is re-exported-free; add it to
// [dependencies] when the crate is built for real.
