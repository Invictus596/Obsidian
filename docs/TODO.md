# Obsidian — Phased Development Roadmap

## Current Status

| Phase | Milestone | Status |
|-------|-----------|--------|
| 0 | Workspace & architecture scaffolding | ✅ Complete |
| 1 | CEF Integration (`obsidian-core`) | ✅ Complete |
| 2 | Fluid UI overlay (`obsidian-ui`) | 🚧 In progress |

---

## Phase 0 — Workspace & Architecture Scaffolding (Complete)

* [x] Initialize Cargo workspace with `obsidian-core`, `obsidian-ui`, `obsidian-bin`
* [x] Dummy `obsidian_core::init()` and `obsidian_ui::init()` entry points
* [x] Binary entry point that initializes logging and boots core + UI
* [x] Comprehensive `.gitignore` for Rust + pre-compiled C++ binaries
* [x] Cyberpunk/terminal README with architecture breakdown
* [x] Git repository initialized, committed, and pushed to `origin/main`

---

## Phase 1 — CEF Integration (`obsidian-core`)

Goal: Establish a safe Rust bridge over the Chromium Embedded Framework (CEF)
C-API. Owns CEF binary discovery, FFI bindings, framework init/shutdown, and
subprocess lifecycle.

### Immediate next steps

* [ ] Add `cef` crate dependency to `obsidian-core/Cargo.toml`
* [ ] Create `obsidian-core/build.rs`:
  * [ ] Locate CEF pre-compiled binaries via `CEF_ROOT` env var or default
    `cef_binary/` folder
  * [ ] Emit `cargo:rustc-link-search` for `cef_binary/Release`
  * [ ] Emit `cargo:rustc-link-lib` for `libcef` / `cef`
  * [ ] Emit `cargo:include` for CEF headers
  * [ ] Fail gracefully with a clear message when CEF binaries are absent
* [ ] Add `scripts/download_cef.ps1` to fetch Windows x64 CEF binaries from the
  Spotify CEF tracker and extract into `cef_binary/`
* [ ] Implement safe `cef_initialize` wrapper in `obsidian-core/src/lib.rs`
* [ ] Implement `cef_shutdown` wrapper
* [ ] Define `CefSettings` struct and `CefMainArgs` handling
* [ ] Wire subprocess spawn (browser / renderer / GPU)
* [ ] Update `obsidian_bin` to pass command-line args into `core_init`

### Longer-term Phase 1 goals

* [ ] CEF message loop integration with a Rust event loop
* [ ] Off-screen rendering (OSR) support for UI composition
* [ ] Cookie / cache / profile management
* [ ] DevTools attachment and remote debugging
* [ ] `cef` crate version pinning and vendoring strategy

---

## Phase 2 — Fluid UI Overlay (`obsidian-ui`)

Goal: An expressive, fluid, borderless UI shell inspired by Arc and Helium,
compositing CEF-rendered content behind native Rust window chrome.

Current focus: **borderless, compact-mode UI shell** with Arc-style vertical
sidebar, compact top bar (URL input + window controls), and an
`engine_viewport` rectangle where CEF will eventually render.

### Immediate next steps

* [x] Add `winit` and `slint` dependencies to `obsidian-ui/Cargo.toml`
* [x] Create `obsidian-ui/src/ui/main.slint` Slint UI template:
    * [x] Left-aligned vertical sidebar (Arc-style, dark cyberpunk aesthetic)
    * [x] Top horizontal compact mode title bar (dummy URL input + window controls)
    * [x] Large empty rectangle as `engine_viewport` for CEF rendering
* [x] Update `obsidian-ui/src/lib.rs` to spawn a winit borderless window and render the Slint UI inside it
* [x] Expose `engine_viewport_handle()` returning the native window handle (HWND on Windows)
* [ ] Wire Slint ↔ winit event loop integration
* [ ] Implement window frame controls (close / minimize / maximize / drag)
* [ ] Implement translucent background and drop-shadow styling
* [ ] Implement vertical tab strip with tab switching
* [ ] Implement CEF off-screen → UI texture composition pipeline

### Longer-term Phase 2 goals

* [ ] Address bar / omnibox with live suggestions
* [ ] Tab management: open / close / pin / drag-to-reorder
* [ ] Back / forward / reload / navigation controls
* [ ] Bookmarks bar and bookmark manager
* [ ] Fluid transitions (tab switches, page loads, modal overlays)
* [ ] Theme engine (dark / light / accent color)
* [ ] Accessibility and keyboard navigation
* [ ] Multi-window / split-view support

---

## Build & Runtime Requirements

Pre-compiled CEF binaries are **required** to compile Phase 1. They are not
checked into this repository.

1. Run `scripts/download_cef.ps1` to fetch Windows x64 CEF binaries into
   `cef_binary/`.
2. Ensure `CEF_ROOT` points to the extraction folder, or accept the default
   `cef_binary/` location.
3. `cargo build` / `cargo run` — `build.rs` will emit the correct link/search
   paths for the CEF Release binaries.

---

## License

Dual-licensed under MIT or Apache-2.0.
