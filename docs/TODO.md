# Obsidian — Phased Development Roadmap

## Current Status

| Phase | Milestone | Status |
|-------|-----------|--------|
| 0 | Workspace & architecture scaffolding | ✅ Complete |
| 1 | CEF Integration (`obsidian-core`) | ✅ Superseded — replaced by wry pivot |
| 2 | Fluid UI overlay (`obsidian-ui`) | ✅ Superseded — ultra-compact UI |
| 2.5 | CEF ↔ UI Bridge | ✅ Superseded — wry WebView handles embedding |
| 3 | Live Engine Integration | ✅ Superseded — wry WebView natively embeds |
| 3.5 | UI Controls ↔ CEF Navigation | ✅ Superseded — wry IPC replaces CEF bridge |
| 3.7 | Viewport Constraints & Navigation Controls | ✅ Superseded — wry `set_bounds()` replaces SetWindowPos |
| 3.9 | Window Dragging & Developer Tools | ✅ Superseded — wry DevTools via `eval()` / native popup |
| 4.1 | wry OS-native WebView + Ultra-compact Slint UI | ✅ Complete |
| 4.2 | Custom Rust WebExtensions Runtime | 🚧 In progress |

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

## Phase 3.9 — Window Dragging & Developer Tools

Goal: Enable window dragging on the borderless compact title bar and expose
Chromium Developer Tools for debugging rendered content.

### Immediate next steps

* [x] `obsidian-ui/src/ui/main.slint` — compact top bar wrapped in `TouchArea`
    with `window_drag_requested()` callback triggered on pointer press
* [x] `obsidian-ui/src/ui/main.slint` — `[D]` DevTools button added with
    `request_devtools()` callback (F12 listener as alternative)
* [x] `obsidian_ui::setup_slint_ui()` maps `window_drag_requested` to
    `window.drag_window()` on the winit `Window`
* [x] `obsidian_ui::setup_slint_ui()` maps `request_devtools` to
    `obsidian_core::show_dev_tools(hwnd)`
* [x] `obsidian_core::show_dev_tools(hwnd)` — finds active `BrowserHost`,
    configures an empty `WindowInfo` (spawns DevTools in native OS popup),
    calls `browser_host.show_dev_tools()`

### Longer-term Phase 3.9 goals

* [ ] Window drag constraints (snap to screen edges)
* [ ] DevTools docking / positioning control
* [ ] Keyboard shortcuts (F12, Ctrl+Shift+I) for DevTools toggle
* [ ] Custom DevTools theme integration

---

## Phase 4 — Extension Loader Architecture

Status: ✅ Superseded — replaced by wry + Custom Rust WebExtensions runtime.

---

## Phase 4.1 — wry OS-native WebView (~10MB core) + Ultra-compact Slint UI

Goal: Pivot `obsidian-core` away from heavy CEF binaries (~100MB+) to OS-native
WebViews via the `wry` crate (~10MB target binary), paired with a custom Rust
WebExtensions runtime. Upgrade `obsidian-ui` to an ultra-minimal single-row
compact layout.

### Immediate next steps

* [x] Remove `obsidian-core/build.rs` and `scripts/download_cef.ps1`
* [x] Replace `cef` + `windows-sys` dependencies with `wry = "0.47"`, `serde`, `serde_json`
* [x] Refactor `obsidian-core/src/lib.rs` — `BrowserEngine` wrapping `wry::WebView`:
    * [x] `BrowserEngine::new(&window, x, y, width, height)` — `WebViewBuilder::new_as_child()`
    * [x] `navigate(&self, url: &str)` — `webview.load_url(url)`
    * [x] `resize(&self, x, y, width, height)` — `webview.set_bounds()`
    * [x] `eval(&self, js: &str)` — back/forward/reload via JS
    * [x] `with_initialization_script` — `mouseover` on `<a>` → `window.ipc.postMessage(href)`
* [x] `obsidian-ui/src/ui/main.slint` — ultra-minimal compact layout:
    * [x] Single thin top chrome bar (36px height)
    *   [x] Custom window controls, sidebar toggle, domain-only URL pill (expands on click)
    *   [x] `<`, `>`, `↻`, `[Ext]` buttons
    * [x] Collapsible left vertical sidebar (`sidebar_open`, 0px ↔ 220px)
    * [x] Floating dark pill at bottom-left of `engine_viewport` bound to `hovered_url`
* [x] `obsidian-ui/src/lib.rs` + `obsidian-bin/src/main.rs` — wire winit window, Slint UI, `wry::WebView` together; update WebView bounds on resize / sidebar toggle

### Longer-term Phase 4.1 goals

* [ ] Custom Rust WebExtensions runtime (manifest parsing, permission checks)
* [ ] Extension management UI (enable/disable, uninstall)
* [ ] Extension icon toolbar integration
* [ ] Extension storage / messaging API support
* [ ] Floating link preview pill rendering from IPC messages
* [ ] Off-screen rendering (OSR) → Slint texture composition
* [ ] Multi-window / split-view support

---

## Phase 4.2 — Custom Rust WebExtensions Runtime

Goal: Build a Manifest V3 extension runtime inside `obsidian-core` that parses
extension manifests, injects a `chrome.*` IPC polyfill into every page, and
injects content scripts into matching URLs.

### Immediate next steps

* [x] `obsidian-core/src/extensions.rs` — new module with:
    * [x] `ExtensionManifest` serde struct (Manifest V3 fields: manifest_version,
        name, version, description, permissions, content_scripts, background)
    * [x] `ContentScriptRule` struct (matches, js, css, run_at)
    * [x] `ExtensionManager` struct (HashMap<String, LoadedExtension>,
        per-extension storage, load_unpacked, generate_polyfill_script,
        scripts_for_url)
* [x] `obsidian-core/src/lib.rs` — declare `pub mod extensions;` and update
    `BrowserEngine`:
    * [x] Hold `Arc<Mutex<ExtensionManager>>`
    * [x] IPC handler distinguishes `link_hover` vs `chrome_api` messages
    * [x] Inject polyfill script via `with_initialization_script`
    * [x] `pub fn load_extension(&self, path: &Path) -> Result<String, String>`
* [x] `tests/dummy_extension/` — real Manifest V3 test extension:
    * [x] `manifest.json` (manifest_version 3, name, version, content_scripts
        matching `<all_urls>`)
    * [x] `content.js` — tests `chrome.storage.local.set` / `get` and logs result
* [x] `obsidian-ui` — `[Ext]` button loads `tests/dummy_extension` via
    `BrowserEngine::load_extension`, reloads page, logs extension name

### Longer-term Phase 4.2 goals

* [ ] Extension management UI (enable/disable, uninstall)
* [ ] Extension permission prompts and policy
* [ ] Extension icon toolbar integration
* [ ] Extension storage persistence to disk
* [ ] Multi-extension isolation / per-extension storage keyspaces

---

## Build & Runtime Requirements

---

## License

Dual-licensed under MIT or Apache-2.0.
