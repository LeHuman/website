// decryptor.js
// Client-side AES decryption for text, links, images, PDFs, and generic downloads.
// Supports inline (data-ciphertext + data-iv) OR external JSON (data-secret="/path.json").
// JSON format: { "iv": "<hex>", "ct": "<hex>" }

// Initialize secretCache from sessionStorage or create a new Map
const secretCache = new Map(
    JSON.parse(sessionStorage.getItem("secretCache") || "[]")
);

// Function to save the cache back to sessionStorage
function saveSecretCache() {
    sessionStorage.setItem("secretCache", JSON.stringify(Array.from(secretCache.entries())));
}

function hexToBytes(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
        bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
    }
    return bytes;
}

async function importKey(hexKey) {
    const keyBytes = hexToBytes(hexKey);
    return crypto.subtle.importKey("raw", keyBytes, { name: "AES-CBC" }, false, ["decrypt"]);
}

async function decryptData(key, ivHex, ctHex) {
    const iv = hexToBytes(ivHex);
    const ct = hexToBytes(ctHex);
    const plainBuffer = await crypto.subtle.decrypt({ name: "AES-CBC", iv }, key, ct);
    return new Uint8Array(plainBuffer);
}

// Load ciphertext either from external JSON or inline data-attrs.
async function getCipherData(el) {
    if (el.dataset.secret) {
        const resp = await fetch(el.dataset.secret, { credentials: "same-origin" });
        if (!resp.ok) throw new Error(`Failed to fetch secret: ${resp.status}`);
        const { iv, ct } = await resp.json();
        if (!iv || !ct) throw new Error("Secret JSON missing iv/ct");
        return { iv, ct, cacheKey: `url:${el.dataset.secret}` };
    } else if (el.dataset.ciphertext && el.dataset.iv) {
        const iv = el.dataset.iv;
        const ct = el.dataset.ciphertext;
        return { iv, ct, cacheKey: `inline:${iv}:${ct.substring(0, 32)}` }; // partial ct to limit key length
    }
    throw new Error("No ciphertext/iv or data-secret on element");
}

// --- Reveal helpers ---

async function revealText(el, key) {
    try {
        const { iv, ct, cacheKey } = await getCipherData(el);

        if (secretCache.has(cacheKey)) {
            el.textContent = secretCache.get(cacheKey);
        } else {
            const data = await decryptData(key, iv, ct);
            const text = new TextDecoder().decode(data);
            secretCache.set(cacheKey, text);
            saveSecretCache(); // Save to sessionStorage
            el.textContent = text;
        }
        el.style.display = "";
    } catch (e) {
        console.warn("Text decrypt failed:", e);
    }
}

async function revealLink(el, key) {
    try {
        const { iv, ct, cacheKey } = await getCipherData(el);

        let href;
        if (secretCache.has(cacheKey)) {
            href = secretCache.get(cacheKey);
        } else {
            const data = await decryptData(key, iv, ct);
            href = new TextDecoder().decode(data);
            secretCache.set(cacheKey, href);
            saveSecretCache(); // Save to sessionStorage
        }

        el.href = href;
        if (!el.textContent.trim() && el.dataset.keep === undefined) el.textContent = "Secret Link";
        el.style.display = "";
    } catch (e) {
        console.warn("Link decrypt failed:", e);
    }
}

async function revealImage(el, key, mimeType = "image/png") {
    try {
        const { iv, ct, cacheKey } = await getCipherData(el);

        let url;
        if (secretCache.has(cacheKey)) {
            url = secretCache.get(cacheKey);
        } else {
            const data = await decryptData(key, iv, ct);
            const blob = new Blob([data], { type: mimeType });
            url = URL.createObjectURL(blob);
            secretCache.set(cacheKey, url);
            saveSecretCache(); // Save to sessionStorage
        }

        el.src = url;
        el.style.display = "";
    } catch (e) { console.warn("Image decrypt failed:", e); }
}

// View PDF inline via <iframe> or <object>
async function revealPdf(el, key) {
    try {
        const { iv, ct, cacheKey } = await getCipherData(el);

        let url;
        if (secretCache.has(cacheKey)) {
            url = secretCache.get(cacheKey);
        } else {
            const data = await decryptData(key, iv, ct);
            const blob = new Blob([data], { type: "application/pdf" });
            url = URL.createObjectURL(blob);
            secretCache.set(cacheKey, url);
            saveSecretCache(); // Save to sessionStorage
        }

        if ("src" in el) {
            el.src = url;
        } else if (el.tagName.toLowerCase() === "object") {
            el.setAttribute("data", url);
            el.setAttribute("type", "application/pdf");
        }
        el.style.display = "";
    } catch (e) { console.warn("PDF decrypt failed:", e); }
}

// Create a download link for any binary (including PDFs)
async function revealDownload(el, key, opts = {}) {
    const { filename = "download.bin", mimeType = "application/octet-stream" } = opts;
    try {
        const { iv, ct, cacheKey } = await getCipherData(el);

        let url;
        if (secretCache.has(cacheKey)) {
            url = secretCache.get(cacheKey);
        } else {
            const data = await decryptData(key, iv, ct);
            const blob = new Blob([data], { type: mimeType });
            url = URL.createObjectURL(blob);
            secretCache.set(cacheKey, url);
            saveSecretCache(); // Save to sessionStorage
        }

        el.href = url;
        el.download = filename;
        if (!el.textContent.trim() && el.dataset.keep === undefined) el.textContent = `Download ${filename}`;
        el.style.display = "";
    } catch (e) { console.warn("Download decrypt failed:", e); }
}

// --- Entry point ---
async function unlockSecrets(config) {
    const params = new URLSearchParams(window.location.search);
    const keyHex = params.get("key");
    if (!keyHex) return; // No key => graceful fallback

    const key = await importKey(keyHex);

    for (const id of (config.texts || [])) {
        const el = document.getElementById(id);
        if (el) await revealText(el, key);
    }

    for (const id of (config.links || [])) {
        const el = document.getElementById(id);
        if (el) await revealLink(el, key);
    }

    for (const item of (config.images || [])) {
        const el = document.getElementById(item.id);
        if (el) await revealImage(el, key, item.mimeType || "image/png");
    }

    for (const item of (config.pdfs || [])) {
        const el = document.getElementById(item.id);
        if (el) await revealPdf(el, key);
    }

    for (const item of (config.downloads || [])) {
        const el = document.getElementById(item.id);
        if (el) await revealDownload(el, key, { filename: item.filename, mimeType: item.mimeType });
    }
}

// Optional CommonJS export
if (typeof module !== "undefined") {
    module.exports = { unlockSecrets };
}
