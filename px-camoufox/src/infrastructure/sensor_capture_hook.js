// Monkey-patch hooks injected into a Camoufox session so we can read
// the plaintext sensor events the PX runtime is about to encrypt, plus
// the final wire payload it POSTs to /b/s.
//
// Stores into:
//   window.__pxCaptures — array of decoded `[{t, d}, …]` event batches
//   window.__pxXhr      — array of { url, body } for XHRs that hit /b/s
//
// The hook is idempotent — re-injecting is a no-op.
(function () {
  if (window.__pxHooked) return;
  window.__pxHooked = true;
  window.__pxCaptures = [];
  window.__pxXhr = [];

  var origStringify = JSON.stringify;
  JSON.stringify = function (value) {
    try {
      if (
        Array.isArray(value) &&
        value.length > 0 &&
        value[0] &&
        typeof value[0].t === 'string' &&
        value[0].d &&
        typeof value[0].d === 'object'
      ) {
        // Snapshot a copy so later mutation doesn't tamper with the record.
        window.__pxCaptures.push(JSON.parse(origStringify(value)));
      }
    } catch (_) {}
    return origStringify.apply(JSON, arguments);
  };

  var origOpen = XMLHttpRequest.prototype.open;
  var origSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.open = function (method, url) {
    this.__pxUrl = url;
    return origOpen.apply(this, arguments);
  };
  XMLHttpRequest.prototype.send = function (body) {
    try {
      if (this.__pxUrl && String(this.__pxUrl).indexOf('/b/s') !== -1) {
        window.__pxXhr.push({
          url: this.__pxUrl,
          body: typeof body === 'string' ? body : null,
        });
      }
    } catch (_) {}
    return origSend.apply(this, arguments);
  };
})();
