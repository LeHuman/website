function getKey() {
    const params = new URLSearchParams(window.location.search);
    // return params.get('key') || sessionStorage.getItem('secretKey') || null;
    return params.get('key'); // no sessionStorage fallback
}

// function persistKey(key) {
//     try { sessionStorage.setItem('secretKey', key); } catch (_) { /* ignore */ }
// }

function isSkippableHref(rawHref) {
    if (!rawHref) return true;
    const lower = rawHref.trim().toLowerCase();
    return (
        lower.startsWith('#') ||
        lower.startsWith('mailto:') ||
        lower.startsWith('tel:') ||
        lower.startsWith('javascript:') ||
        lower.startsWith('data:') ||
        lower.startsWith('//') // protocol-relative external
    );
}

function rewriteLinksWithKey(key) {
    const anchors = document.querySelectorAll('a[href]');
    anchors.forEach(a => {
        const rawHref = a.getAttribute('href');
        if (isSkippableHref(rawHref)) return;

        let url;
        try {
            // a.href is absolute (resolved against <base> or document.baseURI)
            url = new URL(a.href, document.baseURI);
        } catch (_) {
            return; // malformed or unsupported URL
        }

        // Skip externals
        if (url.origin !== window.location.origin) return;

        // Set/override key while preserving other params & hash
        url.searchParams.set('key', key);
        a.href = url.toString();
    });
}

document.addEventListener('DOMContentLoaded', () => {
    const params = new URLSearchParams(window.location.search);
    const key = getKey();
    if (!key) return;

    // If key came from URL, persist it so subpages can unlock even without ?key
    // if (params.get('key')) persistKey(key);

    // Rewrite internal links to propagate key forward
    rewriteLinksWithKey(key);

    // Optional: keep the current page URL clean (remove ?key from address bar after persisting)
    // history.replaceState(null, '', window.location.pathname + window.location.hash);
});
