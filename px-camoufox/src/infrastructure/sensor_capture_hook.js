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
  window.__pxJoinCalls = 0;
  window.__pxJoinSamples = []; // first 10 join outputs for debugging

  // hY (the PX serialiser) builds JSON manually via Array.prototype.join,
  // so JSON.stringify is never called for sensor batches. Hook join too.
  var origJoin = Array.prototype.join;
  Array.prototype.join = function (sep) {
    var result = origJoin.apply(this, arguments);
    try {
      window.__pxJoinCalls += 1;
      if (typeof result === 'string' && result.length > 24) {
        // Capture anything that smells like a sensor batch.
        var firstTwo = result.substring(0, 2);
        if (
          (firstTwo === '[{' || firstTwo === '{"') &&
          result.indexOf('"t":') !== -1
        ) {
          window.__pxCaptures.push(result);
        }
        // Debug: sample first 10 join outputs >24 chars to see what
        // the page actually produces.
        if (window.__pxJoinSamples.length < 10 && result.length < 200) {
          window.__pxJoinSamples.push(result);
        }
      }
    } catch (_) {}
    return result;
  };

  var origStringify = JSON.stringify;
  window.__pxAllStringify = [];
  JSON.stringify = function (value) {
    try {
      // Tight filter: arrays of `{t, d}` events (post-N3 expected shape).
      if (
        Array.isArray(value) &&
        value.length > 0 &&
        value[0] &&
        typeof value[0].t === 'string' &&
        value[0].d &&
        typeof value[0].d === 'object'
      ) {
        window.__pxCaptures.push(origStringify(value));
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && value[0] !== null) {
        // Loose filter: any array of objects (may catch `hY` if it
        // wraps each event in a different shape than {t, d}).
        try {
          var snap = origStringify(value);
          if (snap && snap.length < 16384) {
            window.__pxAllStringify.push(snap);
          }
        } catch (_) {}
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
      if (this.__pxUrl && /\/b\/[sc]|\/api\/v2\/collector|\/eT15wiaE/.test(String(this.__pxUrl))) {
        window.__pxXhr.push({
          channel: 'xhr',
          url: this.__pxUrl,
          body: typeof body === 'string' ? body : null,
        });
      }
    } catch (_) {}
    return origSend.apply(this, arguments);
  };

  // sendBeacon path — primary sensor channel on most PX tenants.
  if (navigator && typeof navigator.sendBeacon === 'function') {
    var origBeacon = navigator.sendBeacon.bind(navigator);
    navigator.sendBeacon = function (url, data) {
      try {
        if (/\/b\/[sc]|\/api\/v2\/collector|px-cloud|\/eT15wiaE/.test(String(url))) {
          var entry = { channel: 'beacon', url: String(url), body: null };
          if (typeof data === 'string') {
            entry.body = data;
            window.__pxXhr.push(entry);
          } else if (data instanceof Blob) {
            entry.body = '__blob_pending_' + data.size + 'b';
            window.__pxXhr.push(entry);
            // Read blob text asynchronously; capture into a separate
            // bucket since the beacon call returns sync.
            data
              .text()
              .then(function (text) {
                window.__pxXhr.push({
                  channel: 'beacon-blob',
                  url: String(url),
                  body: text,
                });
              })
              .catch(function () {});
          } else {
            window.__pxXhr.push(entry);
          }
        }
      } catch (_) {}
      return origBeacon(url, data);
    };
  }

  // fetch() path — rarer but possible for some PX wrappers.
  if (typeof window.fetch === 'function') {
    var origFetch = window.fetch;
    window.fetch = function (input, init) {
      try {
        var url = typeof input === 'string' ? input : (input && input.url) || '';
        if (/\/b\/[sc]|\/api\/v2\/collector|\/eT15wiaE/.test(url)) {
          var body = init && init.body;
          window.__pxXhr.push({
            channel: 'fetch',
            url: url,
            body: typeof body === 'string' ? body : null,
          });
        }
      } catch (_) {}
      return origFetch.apply(this, arguments);
    };
  }
})();
