# Obsidian — Phased Development Roadmap

## Current Status

| Phase | Milestone | Status |
|-------|-----------|--------|
| 0 | Workspace & architecture scaffolding | ✅ Complete |
| 1 | CEF Integration (`obsidian-core`) | ✅ Complete |
| 2 | Fluid UI overlay (`obsidian-ui`) | ✅ Complete |
| 2.5 | CEF ↔ UI Bridge | ✅ Complete |
| 3 | Live Engine Integration | ✅ Complete |
| 3.5 | UI Controls ↔ CEF Navigation | ✅ Complete |
| 3.7 | Viewport Constraints & Navigation Controls | 🚧 In progress |

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

## Phase 2.5 — CEF ↔ UI Bridge (`obsidian-core` ↔ `obsidian-ui`)

Goal: Attach the Chromium Embedded Framework browser to the Slint UI window
handle so CEF renders inside the `engine_viewport` viewport.

### Current work

* [x] `obsidian_core::create_browser(hwnd, url)` — attach CEF browser to the
    native window handle (HWND on Windows) returned by `obsidian-ui`
* [x] `obsidian-ui::init()` returns a raw window handle pointer
    (`*mut c_void`) for cross-crate CEF attachment
* [x] `obsidian-bin` wires: `core_init()` → `ui_init()` → `create_browser()`
    → event loop run

### Immediate next steps

* [ ] Wire Slint ↔ winit event loop integration so the window stays alive
    while CEF renders
* [ ] Implement CEF message loop pump on the UI thread
* [ ] Parent/clip CEF browser to the `engine_viewport` rectangle
* [ ] Implement off-screen rendering (OSR) → Slint texture composition
* [ ] Handle window resize → CEF resize notification

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

## Phase 3 — Live Engine Integration

Goal: Make CEF actually render inside the Slint UI viewport with proper
threading and resize handling.

### Immediate next steps

* [x] Pre-compiled CEF binaries downloaded via `scripts/download_cef.ps1`
    into `cef_binary/` (CEF 156.0.26 / Chromium 156.0.8078.88)
* [x] `cef_bindings` feature enabled with `multi_threaded_message_loop = true`
    so CEF doesn't block the Slint UI thread
* [x] `no_sandbox = true` set in CEF initialization settings
* [x] `obsidian_core::resize_browser(hwnd, width, height)` added to handle
    window resize events
* [x] Slint resize events wired to call `resize_browser()` with the
    `engine_viewport` dimensions

### Longer-term Phase 3 goals

* [ ] CEF message loop pump integrated with winit event loop
* [ ] Parent/clip CEF browser to the `engine_viewport` rectangle
* [ ] Off-screen rendering (OSR) → Slint texture composition
* [ ] Cookie / cache / profile management
* [ ] DevTools attachment and remote debugging

---

## Phase 3.5 — UI Controls ↔ CEF Navigation

Goal: Wire the Slint UI address bar and controls to control the live CEF
browser instance.

### Immediate next steps

* [x] `obsidian_core::navigate(hwnd, url)` — find CEF browser by HWND and call
    `get_main_frame().load_url(url)`
* [x] `obsidian-ui/src/ui/main.slint` — URL `LineEdit` with `accepted` signal
    triggering `request_navigation(string)` callback
* [x] Map Slint `request_navigation` callback to `obsidian_core::navigate()`
    with URL formatting (prepend `https://` if missing)
* [x] Ensure callback has access to raw HWND for cross-crate navigation

### Longer-term Phase 3.5 goals

* [x] Back / forward / reload buttons wired to CEF frame navigation
* [ ] Address bar updates on page load / navigation events
* [ ] Search engine fallback for non-URL queries
* [ ] Bookmark bar and bookmark management
* [ ] Tab-specific URL bars

---

## Phase 3.7 — Viewport Constraints & Navigation Controls

Goal: Constrain the CEF browser viewport to precisely match the Slint
`engine_viewport` rectangle coordinates, and wire Back / Forward / Reload
controls.

### Immediate next steps

* [x] `obsidian_core::resize_browser(hwnd, x, y, width, height)` — updated to
    accept absolute coordinates and call Win32 `SetWindowPos` on the CEF child
    window
* [x] `obsidian_core::go_back(hwnd)`, `go_forward(hwnd)`, `reload(hwnd)` —
    call respective CEF browser navigation methods
* [x] `obsidian-ui/src/ui/main.slint` — Back `[<]`, Forward `[>]`, Reload `[C]`
    buttons added to top bar with `request_back()`, `request_forward()`,
    `request_reload()` callbacks
* [x] `obsidian-ui/src/ui/main.slint` — `engine_viewport` exposes absolute
    `x`, `y`, `width`, `height` properties to Rust
* [x] Resize event handler fetches `engine_viewport` geometry and passes it to
    `obsidian_core::resize_browser()`
* [x] Slint navigation callbacks wired to `obsidian_core` functions with HWND
* [x] `windows-sys` crate added to `obsidian-core/Cargo.toml` with
    `Win32_UI_WindowsAndMessaging` and `Win32_Foundation` features

### Longer-term Phase 3.7 goals

* [ ] Parent/clip CEF browser to the `engine_viewport` rectangle
* [ ] Off-screen rendering (OSR) → Slint texture composition
* [ ] Address bar updates on page load / navigation events
* [ ] Search engine fallback for non-URL queries

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
