# OBSIDIAN

> **Unconquered speed. A Rust-native shell over the Chromium engine.**

```text
   _______      ________    ___   ____ ___   ____ _____ ____  
  / ____\ \    / /  _ \ \  / / |  / ___|_ _| / ___| ____|  _ \ 
 | |     \ \  / /| |_) \ \/ /| | | |    | || |  _|  _| | |_) |
 | |___   \ \/ / |  __/ >  < | | | |___ | || |_| | |___|  __/ 
  \____|   \__/  |_|   /_/\_\_|_|  \____|___\____|_____|_|    

  A high-performance, privacy-first web browser written in Rust
  wrapping the Chromium Embedded Framework (CEF).
```

---

## Overview

Obsidian is a high-performance, privacy-first web browser written in Rust
that wraps the Chromium Embedded Framework (CEF). Rather than shipping a
monolithic browser stack, Obsidian pairs a minimal Rust-native UI shell with
the proven rendering engine of Chromium, giving you:

* **Unconquered speed** — Chromium's rendering performance, driven by Rust.
* **Privacy-first** — a lean attack surface with no baked-in telemetry.
* **Modular architecture** — clean separation between the CEF bridge and the UI overlay.

---

## Architecture

Obsidian is built in two phases. Each phase lives in its own workspace crate.

### Phase 1 — CEF Bridge (`obsidian-core`)

The low-level bridge to the Chromium Embedded Framework. It owns:

* CEF C-API bindings and FFI glue.
* CEF framework initialization and shutdown.
* Browser / renderer / GPU subprocess lifecycle.
* Command-line argument parsing and forwarding to CEF.

```text
+---------------------------------------------+
|             obsidian-core                   |
|  CEF bindings  |  engine lifecycle  | FFI  |
+---------------------------------------------+
```

### Phase 2 — Rust UI Overlay (`obsidian-ui`)

The user-facing shell that composites CEF-rendered content. It owns:

* Native window creation and management.
* Browser chrome (address bar, tabs, bookmarks, navigation).
* Event routing between the shell and the engine core.
* Styling, theming, and accessibility.

```text
+---------------------------------------------+
|             obsidian-ui                     |
|   chrome  |  windowing  |  overlay layout   |
+---------------------------------------------+
```

### Binary entry point (`obsidian-bin`)

Ties core and UI together, initializes logging, and boots the engine and shell
in sequence:

```text
main.rs
  ├─ env_logger (logging)
  ├─ obsidian_core::init()   # Phase 1
  └─ obsidian_ui::init()     # Phase 2
```

---

## Crate layout

```text
Obsidian/
├── Cargo.toml                 # Workspace root
├── README.md
├── .gitignore
├── obsidian-core/             # CEF bridge + engine lifecycle (lib)
├── obsidian-ui/               # Rust UI shell (lib)
└── obsidian-bin/              # Main executable (bin)
```

---

## Build instructions

> **Important:** Pre-compiled CEF binaries will be required to build this project
> in the future. They are not checked into this repository. Once Phase 1
> FFI bindings are implemented, you will need to:
>
> 1. Download the pre-compiled CEF binary distribution matching your host
>    platform (Windows / Linux / macOS).
> 2. Point the build at the CEF include/lib directories via environment
>    variables or a `build.rs` script.
> 3. Run `cargo build --release`.

For now, this repository contains workspace scaffolding only:

```bash
# Clone the repository
git clone https://github.com/Invictus596/Obsidian.git
cd Obsidian

# (Once the Rust toolchain is installed)
cargo build
cargo run
```

Requires Rust 2021 edition toolchain.

---

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

---

```text
> System ready.
> Engine: nominal.
> Shell: nominal.
> Obsidian online.
```
