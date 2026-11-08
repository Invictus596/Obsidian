//! `obsidian-core/build.rs`
//!
//! Build script for the Obsidian CEF bridge. It locates the pre-compiled CEF
//! binaries and emits the cargo directives needed to link against them.
//!
//! CEF binary discovery order:
//!   1. The `CEF_ROOT` environment variable (if set).
//!   2. The default `cef_binary/` folder at the workspace root.
//!
//! Expected CEF folder layout (Windows x64):
//!   cef_binary/
//!   ├── Release/
//!   │   ├── libcef.dll
//!   │   ├── libcef.lib
//!   │   └── ...
//!   └── Include/
//!       └── cef_*.h
//!
//! Link directives are only emitted when the `cef-bindings` feature is
//! enabled, so `cargo build` works without pre-compiled CEF binaries.

use std::env;
use std::path::PathBuf;

fn main() {
    let cef_bindings_enabled = env::var("CARGO_FEATURE_CEF_BINDINGS").is_ok();

    // Locate the CEF root directory.
    let cef_root = env::var("CEF_ROOT")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("cef_binary"));

    println!("cargo:rerun-if-env-changed=CEF_ROOT");

    if !cef_root.is_dir() {
        if cef_bindings_enabled {
            panic!(
                "CEF binaries not found at {:?}. \
                 Set the CEF_ROOT environment variable or extract pre-compiled \
                 CEF binaries into a `cef_binary/` folder at the workspace root.\n\
                 See scripts/download_cef.ps1 to fetch Windows x64 CEF binaries.",
                cef_root
            );
        } else {
            eprintln!(
                "cargo:warning=CEF binaries not found at {:?}. \
                 The `cef-bindings` feature is disabled, so Obsidian will \
                 build with a dummy CEF placeholder. Enable `cef-bindings` \
                 and provide CEF binaries for real CEF integration.",
                cef_root
            );
        }
        return;
    }

    // Only emit link directives when CEF bindings are actually enabled.
    if !cef_bindings_enabled {
        return;
    }

    // Tell the linker where to find CEF libraries.
    let release_dir = cef_root.join("Release");
    if release_dir.is_dir() {
        println!("cargo:rustc-link-search={}", release_dir.display());
    } else {
        println!(
            "cargo:warning=CEF Release folder not found at {:?}",
            release_dir
        );
    }

    // Link against the CEF library.
    // On MSVC the import library is typically `libcef.lib`.
    println!("cargo:rustc-link-lib=dylib=libcef");

    // Tell the compiler where the CEF headers live (useful for any `include!`
    // or `cc`-based FFI generation in the future).
    let include_dir = cef_root.join("Include");
    if include_dir.is_dir() {
        println!("cargo:include={}", include_dir.display());
    }

    // Propagate the CEF root to any downstream crates.
    println!("cargo:rustc-env=CEF_ROOT={}", cef_root.display());

    println!(
        "cargo:warning=Obsidian CEF bridge linked against binaries at {:?}",
        cef_root
    );
}
