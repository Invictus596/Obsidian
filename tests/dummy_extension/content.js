// Obsidian Test Ext content script
console.log("[Obsidian Test Ext] Content script injected!");

(async function() {
    try {
        // Test chrome.storage.local.set
        await chrome.storage.local.set({ testKey: "hello from extension", count: 42 });
        console.log("[Obsidian Test Ext] storage.local.set succeeded");

        // Test chrome.storage.local.get
        const result = await chrome.storage.local.get("testKey");
        console.log("[Obsidian Test Ext] storage.local.get result:", result);

        // Test chrome.runtime.sendMessage
        await chrome.runtime.sendMessage({ type: "ping" });
        console.log("[Obsidian Test Ext] runtime.sendMessage succeeded");

        console.log("[Obsidian Test Ext] All chrome.* API tests passed!");
    } catch (err) {
        console.error("[Obsidian Test Ext] Error:", err);
    }
})();
