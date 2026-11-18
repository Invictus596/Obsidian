use obsidian_core::BrowserEngine;
use obsidian_ui::{init as ui_init, run_event_loop};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging. RUST_LOG controls verbosity (e.g. `RUST_LOG=debug`).
    env_logger::init();

    log::info!("========================================");
    log::info!("  Obsidian — Rust-native browser shell");
    log::info!("  Unconquered speed over Chromium");
    log::info!("========================================");

    // Phase 2: initialize the UI shell (borderless winit window + Slint UI).
    let (window, event_loop) = ui_init().map_err(|e| {
        log::error!("Failed to initialize UI shell: {}", e);
        e
    })?;

    log::info!("Native window handle acquired for WebView embedding.");

    // Phase 4.1: create the wry WebView attached as a child of the winit window.
    let initial_x = 220; // sidebar width when open
    let initial_y = 36;  // top bar height
    let size = window.inner_size();
    let initial_width = size.width.saturating_sub(220);
    let initial_height = size.height.saturating_sub(36);

    let webview = BrowserEngine::new(&window, initial_x, initial_y, initial_width, initial_height)
        .map_err(|e| {
            log::error!("Failed to create WebView: {}", e);
            e
        })?;

    // Navigate to the default URL.
    webview.navigate("https://duckduckgo.com").map_err(|e| {
        log::error!("Failed to navigate: {}", e);
        e
    })?;

    // Phase 4.2: Load the dummy test extension.
    let ext_path = Path::new("tests/dummy_extension");
    let ext_info = webview.load_extension(ext_path).map_err(|e| {
        log::error!("Failed to load extension: {}", e);
        e
    })?;
    log::info!("{}", ext_info);

    // Reload the active page so the content script injects.
    webview.reload().map_err(|e| {
        log::error!("Failed to reload page: {}", e);
        e
    })?;
    log::info!("Page reloaded — content scripts should now be injected.");

    log::info!("Obsidian is up and running.");

    // Phase 4.1/4.2: start the UI event loop. Resize/move events are forwarded
    // to the WebView's set_bounds() so it stays in sync with the viewport.
    run_event_loop(
        &window,
        event_loop,
        &webview,
        |ext_path| {
            log::info!("Extension load requested for: {}", ext_path);
        },
    )
    .map_err(|e| {
        log::error!("Event loop failed: {}", e);
        e
    })?;

    Ok(())
}
