use obsidian_core::init as core_init;
use obsidian_ui::init as ui_init;

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

    // Phase 2: initialize the UI shell.
    ui_init().map_err(|e| {
        log::error!("Failed to initialize UI shell: {}", e);
        e
    })?;

    log::info!("Obsidian is up and running.");

    Ok(())
}
