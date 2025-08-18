// decryptor.js
// Client-side AES decryption for text, links, images, PDFs, and generic downloads.
// Supports inline (data-ciphertext + data-iv) OR external JSON (data-secret="/path.json").
// JSON format: { "iv": "<hex>", "ct": "<hex>" }

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
        return { iv, ct };
    } else if (el.dataset.ciphertext && el.dataset.iv) {
        return { iv: el.dataset.iv, ct: el.dataset.ciphertext };
    }
    throw new Error("No ciphertext/iv or data-secret on element");
}

// --- Reveal helpers ---

async function revealText(el, key) {
    try {
        const { iv, ct } = await getCipherData(el);
        const data = await decryptData(key, iv, ct);
        el.textContent = new TextDecoder().decode(data);
        el.style.display = "";
    } catch (e) { console.warn("Text decrypt failed:", e); }
}

async function revealLink(el, key) {
    try {
        const { iv, ct } = await getCipherData(el);
        const data = await decryptData(key, iv, ct);
        const href = new TextDecoder().decode(data);
        el.href = href;
        if (!el.textContent.trim() && el.dataset.keep === undefined) el.textContent = "Secret Link";
        el.style.display = "";
    } catch (e) { console.warn("Link decrypt failed:", e); }
}

async function revealImage(el, key, mimeType = "image/png") {
    try {
        const { iv, ct } = await getCipherData(el);
        const data = await decryptData(key, iv, ct);
        const blob = new Blob([data], { type: mimeType });
        el.src = URL.createObjectURL(blob);
        el.style.display = "";
    } catch (e) { console.warn("Image decrypt failed:", e); }
}

// View PDF inline via <iframe> or <object>
async function revealPdf(el, key) {
    try {
        const { iv, ct } = await getCipherData(el);
        const data = await decryptData(key, iv, ct);
        const blob = new Blob([data], { type: "application/pdf" });
        const url = URL.createObjectURL(blob);

        // Works for <iframe>, <object>, or <embed>
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
        const { iv, ct } = await getCipherData(el);
        const data = await decryptData(key, iv, ct);
        const blob = new Blob([data], { type: mimeType });
        const url = URL.createObjectURL(blob);

        el.href = url;
        el.download = filename;
        if (!el.textContent.trim() && el.dataset.keep === undefined) el.textContent = `Download ${filename}`;
        el.style.display = "";
    } catch (e) { console.warn("Download decrypt failed:", e); }
}

// --- Entry point ---
// Reads key from URL (?key=...) first, then optional sessionStorage fallback if you use it elsewhere.
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
