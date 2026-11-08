use obsidian_core::{create_browser as core_create_browser, init as core_init};
use obsidian_ui::init as ui_init;
use obsidian_ui::EngineViewportHandle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging. RUST_LOG controls verbosity (e.g. `RUST_LOG=debug`).
    env_logger::init();

    log::info!("========================================");
    log::info!("  Obsidian — Rust-native browser shell");
    log::info!("  Unconquered speed over Chromium");
    log::info!("========================================");

    // Phase 1: initialize the CEF engine core.
    core_init().map_err(|e| {
        log::error!("Failed to initialize engine core: {}", e);
        e
    })?;

    // Phase 2: initialize the UI shell (borderless winit window + Slint UI).
    let viewport: EngineViewportHandle = ui_init().map_err(|e| {
        log::error!("Failed to initialize UI shell: {}", e);
        e
    })?;

    // Extract the native window handle as an OS-agnostic raw pointer.
    let hwnd = viewport.as_raw_pointer();
    log::info!("Native viewport handle: {:?}", hwnd);

    // Phase 2.5: attach the CEF browser to the UI viewport and navigate.
    core_create_browser(hwnd, "https://duckduckgo.com").map_err(|e| {
        log::error!("Failed to create CEF browser: {}", e);
        e
    })?;

    log::info!("Obsidian is up and running.");

    // Start the UI event loop. In the full architecture this loop drives both
    // the Slint UI and the CEF message pump. For scaffolding, the window has
    // already been created and shown inside `obsidian_ui::init()`.
    //
    // event_loop.run() would go here once Slint ↔ winit integration is wired.

    Ok(())
}
