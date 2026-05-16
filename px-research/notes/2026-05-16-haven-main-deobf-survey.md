# R2 first pass — haven `main.min.js` deobfuscation survey

**Tenant:** PX12Ew76qT (havenwellwithin.com)
**Tool:** webcrack 2.x (`npx webcrack --force`)
**Input:** `px-research/init-js/12Ew76qT/2026-05-16-main.min.js` (256 KB minified)
**Output:** `px-research/deobf/12Ew76qT/main/deobfuscated.js` (557 KB, 17,135 lines)

## Webcrack stats

- `prepare` pass: 1,765 changes
- `deobfuscate`: 260 changes
  - String Array `vu`, length 64
  - String Array Rotate: yes
  - String Array Decoders: `Lu`
  - inline-object-props: 63 changes
  - inline-decoder-wrappers: 106 changes
  - inline-decoded-strings: 88 strings inlined
- `transpile, unminify`: 12,010 changes
- `self-defending / debug-protection / jsx`: 0 changes (no anti-debug wrapper detected)
- `merge-object-assignments / evaluate-globals`: 0 changes
- **Module unpack:** none — webcrack did not detect a bundler split, so the
  output is a single readable file rather than per-module sources. Likely a
  custom packer or a pre-bundled rollup output without webpack/browserify
  signatures.

## What is now readable

- ~675 function declarations are recoverable in the unwrapped file
- Cookie names: `_px3`, `_px2`, `_pxhd`, `_pxcdi`, `_pxmvid`, `_pxAppId`
- Hardcoded tenant string: `"https://collector-PX12Ew76qT.px-cloud.net"`
  at line 8373 — confirms collector hostname is baked into the bundle for
  this tenant rather than computed at runtime.
- Endpoint paths constant: `["/api/v2/collector", "/b/s"]` at line 8402
- Error reporter path: `collector-a.px-cloud.net/api/v2/collector/clientError`
  (line 44, line 17135) — second host for error pings, distinct from the
  per-tenant collector.
- Captcha URL builders:
  - Hosted form: `captcha.px-cdn.net/${appId}/captcha.js?...` (line 16665)
  - Cloud form: `captcha.px-cloud.net/${appId}/captcha.js?a=c&u=${vid}&v=${sid}&m=0` (line 16706)
- Sensor field literals visible in source: `appId`, `tag`, `vid`, `sid`,
  `uuid`, `pc`, `en`, `seq`, `ft`, `bi`, `cs`, `ci`, `pxhd`, `cts`, `hid`,
  plus lowercase `&app_id=` and `&ti=` variants suggesting two distinct
  request shapes.

## XHR call sites (collector POST candidates)

Line numbers in `deobfuscated.js`:

| Line   | Context |
|--------|---------|
| 1144   | clientError ping (error reporter) |
| 1230   | second clientError variant |
| 5276   | (unidentified) |
| 8505   | likely main collector — near the per-tenant URL builder |
| 8580   | follow-up XHR |
| 8681   | `h.send(v)` — body-bearing POST, candidate for sensor batch |
| 9415   | `sendBeacon(n, r)` fallback |
| 11982  | `c.send(r.pn)` — postData via `pn` field |
| 16002  | XHR with HEADERS_RECEIVED check |
| 16338  | `sendBeacon` fallback |

Per `2026-05-16-collector-payloads.md` the sensor POST hits
`/b/g?payload=...&appId=...&tag=...&uuid=...`. The most likely emitter is
the 8505/8681 cluster sitting next to the `collector-${tenant}` host
constant.

## Anti-RE signals

- String Array rotate was active (rotation reversed by webcrack).
- Heavy base64 noise in the global string table — many entries decoded by
  a custom decoder `yo(...)` still seen at line ~14948, suggesting a second
  encoding layer per-call instead of upfront array rotation. Those `yo()`
  calls produce the property keys for runtime feature detection
  (XMLHttpRequestEventTarget, etc.).
- No `self-defending` block, no debug-protection traps — the bundle is
  reasonable to step through in DevTools after this pass.

## Next R2 / R3 moves

1. **Locate the sensor payload assembler.** Bisect around line 8505–8681:
   find the function that concatenates `payload=` + URL-encoded body and
   reads from `_pxAppId`, `vid`, `sid`, `pc`. Document its inputs as the
   sensor schema entry point.
2. **Decode the `bi=` field.** The recon capture has
   `bi=YjIXcjcFUFMkZndXe24rUWMbBTVzN1hXbi8CP1lzW3hgHVFIJnNaTQxpZQljHBwrYQhdTEo6VBELO0NtIVMcCi4vG18gOCRfdxoHKiR+CwFufAZSXnNYeDZDVBlpYB4=` — base64 of an
   XOR/rolling-key envelope. Find the encoder in the deobf'd source by
   tracing the `"bi"` key access.
3. **Crack the second-layer `yo()` decoder.** `yo(...)` shows up in the
   feature-detection map. If we can decode those at build time we eliminate
   the remaining indirection in the property-access paths.
4. **HMAC key for `cs`.** Recon shows `cs` is 64-hex SHA-256-HMAC of the
   payload. Search the deobf source for `subtle` / `crypto.subtle` /
   `hmac` / `SHA-256` to find the key derivation. This unlocks the native
   solver path more than anything else.
5. **Diff against ped (PXeT15wiaE).** Run webcrack on the pedidosya bundle
   and diff string-array sizes + line counts to confirm the bundles share
   a core (we already proved `captcha.js` is byte-identical via md5).

## Reproducibility

```bash
npx webcrack@latest --force \
  -o px-research/deobf/12Ew76qT/main \
  px-research/init-js/12Ew76qT/2026-05-16-main.min.js
```
