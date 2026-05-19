# R3 — px-3 sensor grammar (eT15wiaE, 2026-05-20)

Follow-up to `r2-decoder-output-2026-05-20.md`. With ~1 460 string literals
inlined, the sensor emit-path is statically traceable.

## Encryption stack — the actual primitives

```
events[]                              ──► one JS object per signal
  │                                       { t: "PX12457", d: { … } }
  │ hY(events)                        ──► JSON-like serialisation
  │
  ▼ jw(json, iS)
  XOR (single-byte stream cipher)     ──► iS = 50  (0x32)         **line 1447**
  │
  ▼ hP / hQ
  Base64 (custom alphabet hI)         ──► payload string
  │
  ▼ vQ(salt, payload, offsets)
  VM splice                           ──► inserts substrings at
                                          offsets[i] − i − 1
  │
  ▼  url-encoded as `data=<…>&appID=…&tag=…&cu=…`
  XHR POST → `${origin}/<tenant>/xhr/b/s`
            (sendBeacon / img.src fallback)
```

That is the **entire** encryption layer. No AES, no HMAC at the
payload tier. Integrity is enforced separately by the `pxhc` cookie,
which is an MD5 over `<cu>:<appID>:<tag>` (see `hl/hm/hn/ho` round helpers
at lines 299-308). 

## Sensor builder

* `vP(events, ctx)` — top-level encryptor.  `px-research/deobf/.../main-pass2/deobfuscated.js:6050`
* `yA(events, ctx)` — high-level serializer that wraps `vP`.            `7701`
* `EC(t, n, …)` — XHR POST sender (`a.send(payload+counter)`)            `11151`
* `EE(t, n)` — `sendBeacon` fallback                                     `11212`
* `EF(t, n)` — `img.src` GET fallback (`/noCors?<payload>`)              `11221`

The endpoint resolves through the rotation in `Eh()`:
```js
Ea[le] = ["/api/v2/collector", "/b/s"];  // collector for telemetry, b/s for sensor
Ea[lf] = ["px-client.net", "px-cdn.net"];
Ea[lh] = ["/b/c"];                       // challenge submission
```

Tenant prefix is prepended at request build time:
`{scheme}//{appId-lowercase}.{px-client.net}/b/s`.

## Event shape

Each event the runtime collects is:
```js
{
  t: "PX12457",        // category tag — one of the 14 PX-codes + base64 aliases
  d: {
    "EwNgCVZlZDw=": …,  // visit context
    "dWFGKzACQxw=": …,  // xo()  — page-host info
    "HCgvIllLKRA=": …,  // xl()  — first-party origin
    "Dz98dUlZf0I=": …,  // xD — captcha state (when present)
    "WQUqDx9mKDU=": …,  // nl()  — visitor-id (vid)
    "XGhvYhkOb1g=": …,  // pk()  — pixel-counter hash
    "OARLTn1mTXQ=": …,  // te()  — captcha event sub-id (mobile-only)
    "Slp5EA85eis=": …,  // pq()  — request-context (referrer hash)
    "bHgfcikaG0M=": …,  // mb("_px3")  — cookie passthrough
    "YQ1SByRvUzU=": …,  // yt()  — challenge bookkeeping
    ...
  }
}
```

The base64-looking keys are **not** base64-decodable to plaintext; they are
short fixed identifiers that PerimeterX uses across tenants. They survive
across the JS runtime as opaque strings.

## Form params layer (output of `yA`)

`yA` returns an array that gets `"&"`-joined as the request body:
```
mN + h          // payload (= vP(events, ctx))
mO + appID
mP + tag
mQ + np()       // cu  (client uuid; UUIDv1 from md crypto, see line 2549)
mS + tagSession
mT + yz++       // request sequence (monotonic per page)
nb + yy
ng + iw
mR + v          // optional: vid
mU + cs         // optional: cookie checksum
mV + pc         // optional: page counter
mW + pf         // optional: parent-frame ref
mX + iO()       // optional: visitor-id
mY + nS
mZ + tm()       // optional: time-marker
na + pi()       // optional: page-info (when xo() is falsy)
nd + nU
nf + nV
ne + pj()       // optional: jitter token
```

The `m_/n_` prefixes are short literal strings like `"data="`, `"sid="`,
`"appId="`, etc., one per parameter. They expand from `gC(N)` table entries
and most are not yet promoted to literals in the pass-2 output (the
inlining only touched calls with literal numeric args).

## What this means for px-native

The bypass we deferred in ADR-0010 is now scoped. Native Rust sensor
synthesis needs to reproduce, at most:

1. **Crypto** — `xor_single_byte(buf, 50)` + base64 (alphabet `hI` =
   standard `A-Za-z0-9+/`, padded `=`).
2. **VM splice** — `vQ(salt, b64, offsets)` is a simple splice: insert
   `bf[i]` at `offsets[i] - i - 1` characters from the cursor. The
   `bf[]` and `offsets[]` arrays are produced by `vN(salt, len, secret)`,
   itself a deterministic PRNG seeded from `salt + secret`. Extracting
   `vN` is the remaining work — should fit in <200 LOC of Rust.
3. **Field collection** — populate the `d:{}` map per event with the
   pieces we can synthesise (timestamps, vid, cookie passthrough, fake
   mouse/scroll/click events with monotonic timestamps that match the
   `m^Vyj$~=…` event shapes).
4. **`pxhc` MD5** — straight RFC 1321 MD5 over `cu:appID:tag`. No key.
5. **Endpoint POST** — `${origin}/${appIdLower}/xhr/b/s` (or `/b/s`
   directly on px-cloud), one of `[POST|sendBeacon|img.src]`.

No HMAC key, no AES key, no per-tenant secret beyond the public appID
string. The whole obfuscation budget is spent on (a) making the JS
hard to read and (b) making sure the sensor cannot run outside a real
browser DOM. Layer (b) is what we still owe — and the work is
**collecting realistic input signals**, not reversing crypto.

## `vN` algorithm (un-flattened VM)

`vN(target, lenBound, secret)` produces the offsets array consumed by
`vQ`. The on-disk implementation is a switch/with/case state machine
(2 arms entered as `gJ(191, 33)`); after unwinding the dispatch:

```
function vN(target, lenBound, secret) {
  // Setup: derive aF from the per-call secret (one of t[-232] in vP).
  const aF = jw(hP(secret), vJ);     // vJ = 10
  // Arm 1: find aH = max product of charCodes inside aF.
  let aH = -1;
  for (let aI = 0; aI < target.length; aI++) {
    const aJ = Math.floor(aI / aF.length + 1);            // note: +1 inside floor()
    const aK = aI >= aF.length ? aI % aF.length : aI;
    const aL = aF.charCodeAt(aK) * aF.charCodeAt(aJ);
    if (aL > aH) aH = aL;
  }
  // Arm 2: build offsets, remap big ones into the safe window,
  //        dedupe by incrementing, sort.
  const aG = [];
  for (let aM = 0; aM < target.length; aM++) {
    const aN = Math.floor(aM / aF.length) + 1;            // note: +1 outside floor()
    const aO = aM % aF.length;
    let aP = aF.charCodeAt(aO) * aF.charCodeAt(aN);
    if (aP >= lenBound) aP = vM(aP, 0, aH, 0, lenBound - 1);
    while (aG.indexOf(aP) !== -1) aP += 1;
    aG.push(aP);
  }
  return aG.sort((t, n) => t - n);
}

// vM(t, n, e, r, g)  ==  linear remap from [n, e] onto [r, g]
function vM(t, n, e, r, g) {
  return Math.floor((t - n) / (e - n) * (g - r) + r);
}
```

`vL()` returns the secret feed:
```
function vL() {
  return jw(hP(pf() || gC(365)), vJ);  // gC(365) is a fallback string
}
```
i.e. `vL = jw(base64(pageFingerprint or fallback), 10)`. Same XOR family
as the payload cipher, but with key `vJ = 10` instead of `iS = 50`.

This is enough for the N1 sprint (ADR-0024) — port `vN`, `vM`, `vL`,
`jw`, `hP` to Rust, then `vQ` reduces to a deterministic splice driven
by the sorted offsets.

## Status update

| Phase | Status                                                                          |
| ----- | ------------------------------------------------------------------------------- |
| R0    | ✅ done                                                                          |
| R1    | ⏳ partial — static surface is enough to skip CDP for now.                       |
| R2    | ✅ done — 1 457 strings inlined.                                                 |
| R3    | ✅ encryption layer fully understood; field grammar partially mapped.            |
| R4    | ⏳ ADR draft is the next file (`r4-bypass-adr-2026-05-20.md`).                   |
