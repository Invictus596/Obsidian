use obsidian_core::{create_browser as core_create_browser, init as core_init};
use obsidian_ui::{init as ui_init, run_event_loop};

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
    let (viewport, event_loop) = ui_init().map_err(|e| {
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

    // Phase 3: start the UI event loop. Resize events are forwarded to
    // obsidian_core::resize_browser() so the CEF browser stays in sync.
    run_event_loop(event_loop, hwnd).map_err(|e| {
        log::error!("Event loop failed: {}", e);
        e
    })?;

    Ok(())
}
