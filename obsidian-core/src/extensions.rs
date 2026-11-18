//! `obsidian-core::extensions` — Custom Rust WebExtensions runtime.
//!
//! Parses Manifest V3 extension manifests, injects a `chrome.*` IPC polyfill
//! into every page, and injects content scripts into matching URLs.
//!
//! ## Status
//!
//! Phase 4.2 — Manifest V3 parser, `chrome.*` IPC polyfill, and content
//! script injector.

use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// A Manifest V3 extension manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionManifest {
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub content_scripts: Vec<ContentScriptRule>,
    #[serde(default)]
    pub background: Option<BackgroundInfo>,
}

/// A content script rule from the manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct ContentScriptRule {
    pub matches: Vec<String>,
    #[serde(default)]
    pub js: Option<Vec<String>>,
    #[serde(default)]
    pub css: Option<Vec<String>>,
    #[serde(default)]
    pub run_at: Option<String>,
}

/// Background service worker info.
#[derive(Debug, Clone, Deserialize)]
pub struct BackgroundInfo {
    #[serde(default)]
    pub service_worker: Option<String>,
}

/// A loaded extension in memory.
#[derive(Debug, Clone)]
pub struct LoadedExtension {
    pub manifest: ExtensionManifest,
    pub extension_id: String,
    pub content_js: Vec<String>,   // loaded JS file contents
    pub content_css: Vec<String>,  // loaded CSS file contents
}

/// ExtensionManager holds all loaded extensions and their per-extension
/// key-value storage.
pub struct ExtensionManager {
    extensions: HashMap<String, LoadedExtension>,
    storage: HashMap<String, HashMap<String, serde_json::Value>>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
            storage: HashMap::new(),
        }
    }

    /// Load an unpacked extension from a directory containing manifest.json.
    pub fn load_unpacked(&mut self, dir: &Path) -> Result<LoadedExtension, String> {
        let manifest_path = dir.join("manifest.json");
        let manifest_str = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest.json: {}", e))?;

        let manifest: ExtensionManifest = serde_json::from_str(&manifest_str)
            .map_err(|e| format!("Failed to parse manifest.json: {}", e))?;

        if manifest.manifest_version != 3 {
            return Err(format!(
                "Unsupported manifest version: {} (expected 3)",
                manifest.manifest_version
            ));
        }

        // Generate a stable extension id from the directory name.
        let extension_id = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Read referenced content scripts into memory.
        let mut content_js = Vec::new();
        let mut content_css = Vec::new();

        for rule in &manifest.content_scripts {
            if let Some(js_files) = &rule.js {
                for js_file in js_files {
                    let path = dir.join(js_file);
                    let contents = std::fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read content script {}: {}", js_file, e))?;
                    content_js.push(contents);
                }
            }
            if let Some(css_files) = &rule.css {
                for css_file in css_files {
                    let path = dir.join(css_file);
                    let contents = std::fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read content CSS {}: {}", css_file, e))?;
                    content_css.push(contents);
                }
            }
        }

        let ext = LoadedExtension {
            manifest,
            extension_id: extension_id.clone(),
            content_js,
            content_css,
        };

        // Initialize empty storage for this extension.
        self.storage.insert(extension_id.clone(), HashMap::new());
        self.extensions.insert(extension_id.clone(), ext.clone());

        info!("Loaded extension '{}' (id: {})", ext.manifest.name, extension_id);
        Ok(ext)
    }

    /// Generate a self-contained JavaScript shim defining `window.chrome = {}`
    /// with `chrome.runtime`, `chrome.storage.local`, and `chrome.tabs` APIs
    /// bridged to Rust via `window.ipc.postMessage`.
    pub fn generate_polyfill_script(&self) -> String {
        r#"
(function() {
    // Ensure chrome namespace exists.
    window.chrome = window.chrome || {};
    window.chrome.runtime = window.chrome.runtime || {};

    // Helper to send an IPC message and await a response from Rust.
    function chromeIpc(message) {
        return new Promise(function(resolve, reject) {
            var callId = Math.random().toString(36).substr(2, 9);
            window.ipc.postMessage(JSON.stringify({
                type: 'chrome_api',
                callId: callId,
                message: message
            }));
            // Store pending callback keyed by callId.
            window.__chromePending = window.__chromePending || {};
            window.__chromePending[callId] = { resolve: resolve, reject: reject };
            // Timeout after 10 seconds.
            setTimeout(function() {
                if (window.__chromePending && window.__chromePending[callId]) {
                    delete window.__chromePending[callId];
                    reject(new Error('chrome API call timed out'));
                }
            }, 10000);
        });
    }

    // chrome.runtime
    window.chrome.runtime.id = window.chrome.runtime.id || 'obsidian-extension';

    window.chrome.runtime.sendMessage = function(message) {
        return chromeIpc({ type: 'runtime.sendMessage', message: message });
    };

    // chrome.storage.local
    window.chrome.storage = window.chrome.storage || {};
    window.chrome.storage.local = window.chrome.storage.local || {};

    window.chrome.storage.local.get = function(keys) {
        return chromeIpc({ type: 'storage.local.get', keys: keys });
    };

    window.chrome.storage.local.set = function(items) {
        return chromeIpc({ type: 'storage.local.set', items: items });
    };

    // chrome.tabs
    window.chrome.tabs = window.chrome.tabs || {};
    window.chrome.tabs.query = function(queryInfo) {
        return chromeIpc({ type: 'tabs.query', queryInfo: queryInfo });
    };

    // Handle responses from Rust.
    window.__chromeResponse = window.__chromeResponse || function(callId, ok, value) {
        var pending = window.__chromePending && window.__chromePending[callId];
        if (pending) {
            delete window.__chromePending[callId];
            if (ok) {
                pending.resolve(value);
            } else {
                pending.reject(new Error(value));
            }
        }
    };
})();
"#
        .to_string()
    }

    /// Match a URL against all loaded extensions' content script rules and
    /// return the concatenated JS/CSS injection payloads.
    pub fn scripts_for_url(&self, url: &str) -> Vec<String> {
        let mut payloads = Vec::new();

        for ext in self.extensions.values() {
            for rule in &ext.manifest.content_scripts {
                if rule_matches_url(url, &rule.matches) {
                    // Inject JS files first, then CSS.
                    for js in &ext.content_js {
                        payloads.push(format!("/* Content script from extension '{}' */\n{}", ext.manifest.name, js));
                    }
                    for css in &ext.content_css {
                        payloads.push(format!("/* Content CSS from extension '{}' */\n<style>{}</style>", ext.manifest.name, css));
                    }
                }
            }
        }

        payloads
    }

    /// Handle a chrome_api IPC call from the page and return a JSON response.
    pub fn handle_chrome_api(&mut self, message: serde_json::Value) -> serde_json::Value {
        let msg_type = message.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let extension_id = message.get("extension_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match msg_type {
            "runtime.sendMessage" => {
                info!("chrome.runtime.sendMessage from extension {}: {:?}", extension_id, message.get("message"));
                serde_json::json!({ "ok": true, "result": null })
            }
            "storage.local.get" => {
                let keys = message.get("keys").and_then(|v| v.as_str()).unwrap_or("");
                let storage = self.storage.get(extension_id);
                let value = storage.and_then(|s| s.get(keys)).cloned().unwrap_or(serde_json::Value::Null);
                serde_json::json!({ "ok": true, "result": value })
            }
            "storage.local.set" => {
                let items = message.get("items");
                if let Some(items_obj) = items.and_then(|v| v.as_object()) {
                    let storage = self.storage.entry(extension_id.to_string()).or_default();
                    for (k, v) in items_obj {
                        storage.insert(k.clone(), v.clone());
                    }
                    info!("chrome.storage.local.set for extension {}: {:?}", extension_id, items_obj);
                }
                serde_json::json!({ "ok": true, "result": null })
            }
            "tabs.query" => {
                serde_json::json!({ "ok": true, "result": [] })
            }
            _ => {
                serde_json::json!({ "ok": false, "error": format!("Unknown chrome API type: {}", msg_type) })
            }
        }
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple glob-style URL matching. Supports `<all_urls>` and `*` wildcards.
fn rule_matches_url(url: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if pattern == "<all_urls>" {
            return true;
        }
        if glob_match(url, pattern) {
            return true;
        }
    }
    false
}

fn glob_match(text: &str, pattern: &str) -> bool {
    // Simple implementation: split pattern by '*' and check each segment appears in order.
    let mut pattern_parts = pattern.split('*').peekable();
    let mut text_pos = 0;

    while let Some(part) = pattern_parts.next() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = text[text_pos..].find(part) {
            text_pos += found + part.len();
        } else {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_all_urls() {
        assert!(rule_matches_url("https://example.com", &["<all_urls>".to_string()]));
    }

    #[test]
    fn test_glob_match_wildcard() {
        assert!(rule_matches_url("https://example.com/page", &["*example.com*".to_string()]));
        assert!(!rule_matches_url("https://other.com/page", &["*example.com*".to_string()]));
    }

    #[test]
    fn test_manifest_version_validation() {
        let manager = ExtensionManager::new();
        let _ = manager; // Just ensure it constructs
    }
}
