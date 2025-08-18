// decryptor.js
// Client-side AES decryption of hidden content with WebCrypto API
// Supports inline ciphertext (data-ciphertext + data-iv)
// OR external JSON (data-secret="/path/to.json")

/**
 * Convert a hex string to Uint8Array.
 */
function hexToBytes(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
        bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
    }
    return bytes;
}

/**
 * Import a raw AES key from hex string.
 */
async function importKey(hexKey) {
    const keyBytes = hexToBytes(hexKey);
    return crypto.subtle.importKey("raw", keyBytes, { name: "AES-CBC" }, false, [
        "decrypt",
    ]);
}

/**
 * Decrypt a ciphertext given hex IV + hex CT.
 */
async function decryptData(key, ivHex, ctHex) {
    const iv = hexToBytes(ivHex);
    const ct = hexToBytes(ctHex);
    const plainBuffer = await crypto.subtle.decrypt(
        { name: "AES-CBC", iv },
        key,
        ct
    );
    return new Uint8Array(plainBuffer);
}

/**
 * Load ciphertext from element.
 * Priority: external JSON if data-secret is present,
 * otherwise use inline data-ciphertext + data-iv.
 */
async function getCipherData(el) {
    if (el.dataset.secret) {
        const resp = await fetch(el.dataset.secret);
        const { iv, ct } = await resp.json();
        return { iv, ct };
    } else if (el.dataset.ciphertext && el.dataset.iv) {
        return {
            iv: el.dataset.iv,
            ct: el.dataset.ciphertext,
        };
    } else {
        throw new Error("No ciphertext/iv or secret JSON found for element");
    }
}

/**
 * Try to decrypt and reveal a text element.
 */
async function revealText(el, key) {
    try {
        const { iv, ct } = await getCipherData(el);
        const data = await decryptData(key, iv, ct);
        const text = new TextDecoder().decode(data);
        el.textContent = text;
        el.style.display = "";
    } catch (err) {
        console.warn("Text decryption failed:", err);
    }
}

/**
 * Try to decrypt and reveal a link element.
 */
async function revealLink(el, key) {
    try {
        const { iv, ct } = await getCipherData(el);
        const data = await decryptData(key, iv, ct);
        const href = new TextDecoder().decode(data);
        el.href = href;
        if (!el.textContent.trim() && el.dataset.keep === undefined) {
            el.textContent = "Secret Link";
        }
        el.style.display = "";
    } catch (err) {
        console.warn("Link decryption failed:", err);
    }
}

/**
 * Try to decrypt and reveal an image element.
 */
async function revealImage(el, key, mimeType = "image/png") {
    try {
        const { iv, ct } = await getCipherData(el);
        const data = await decryptData(key, iv, ct);
        const blob = new Blob([data], { type: mimeType });
        el.src = URL.createObjectURL(blob);
        el.style.display = "";
    } catch (err) {
        console.warn("Image decryption failed:", err);
    }
}

/**
 * Main entry point — check URL, decrypt and reveal items.
 *
 * config = {
 *   texts: ["id1", "id2"],
 *   links: ["id3"],
 *   images: [{ id: "img1", mimeType: "image/png" }]
 * }
 */
async function unlockSecrets(config) {
    const params = new URLSearchParams(window.location.search);
    const keyHex = params.get("key");
    if (!keyHex) return; // no key, fallback gracefully

    const key = await importKey(keyHex);

    for (const id of config.texts || []) {
        const el = document.getElementById(id);
        if (el) await revealText(el, key);
    }

    for (const id of config.links || []) {
        const el = document.getElementById(id);
        if (el) await revealLink(el, key);
    }

    for (const item of config.images || []) {
        const el = document.getElementById(item.id);
        if (el) await revealImage(el, key, item.mimeType);
    }
}

// Export if needed (for module use)
if (typeof module !== "undefined") {
    module.exports = { unlockSecrets };
}
