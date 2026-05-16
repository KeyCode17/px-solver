# R2 cross-tenant diff — haven vs ped main bundles

**Captured:** 2026-05-16
**Tool:** webcrack 2.x
**Inputs:**

- `px-research/init-js/12Ew76qT/2026-05-16-main.min.js` (haven)
- `px-research/init-js/eT15wiaE/2026-05-16-main.min.js` (ped)

**Outputs:**

- `px-research/deobf/12Ew76qT/main/deobfuscated.js`
- `px-research/deobf/eT15wiaE/main/deobfuscated.js`

## Structural diff

| Metric                       | haven (12Ew76qT) | ped (eT15wiaE) |
|------------------------------|------------------|----------------|
| Deobf line count             | 17,135           | 13,772         |
| Function declarations        | 675              | 973            |
| `.send()` / `sendBeacon()`   | 10               | 8              |
| `prepare` pass changes       | 1,765            | 1,561          |
| `deobfuscate` pass changes   | 260              | 0              |
| String Array detected        | yes (64 entries, rotate) | **no**  |
| Decoder fn                   | `Lu`             | none (per-call accessor instead) |
| Inlined-decoder-wrapper hits | 106              | 0              |
| Inlined-decoded-strings hits | 88               | 0              |
| `transpile, unminify` changes| 12,010           | 12,635         |
| `self-defending`             | none             | none           |
| `debug-protection`           | none             | none           |

The **headline finding**: the two tenants ship the **same logical PX
core** but with **different obfuscation strategies** layered on top.

- Haven uses the classic `obfuscator.io` string-array + rotate + decoder
  triplet, which webcrack fully unwraps.
- Ped does **not** use that pattern. Its string indirection survives as
  per-call accessors like `hR(328)`, `t("us3TztL5yN/e39TO09vWyQ")`, etc.,
  pulling from a runtime table that webcrack cannot resolve statically.

That gap is why ped's deobf still reads as ~50% obfuscated despite being
formally "unminified".

## Same logical core — confirmed landmarks

Both bundles share:

| Concept              | Where (haven / ped)             | Notes |
|----------------------|---------------------------------|-------|
| Collector primary    | line 8373 `bp()` / line 11307 `Ft()` | Tenant baked in: `collector-PX<APPID>.px-cloud.net` |
| Endpoint paths       | `/api/v2/collector`, `/b/s`, `/b/c` | Identical literal table |
| CDN host families    | `px-cdn.net`, `pxchk.net`, `px-client.net` | Identical |
| Asset paths          | `/assets/js/bundle`, `/res/uc`  | Identical |
| Captcha builder      | line 16706 / line 13225         | `captcha.px-cloud.net/${appId}/captcha.js?a=c&u=${vid}&v=${sid}&m=0` |
| Hosted captcha       | line 16665                      | `captcha.px-cdn.net/${appId}/captcha.js` |
| Cookie name table    | line 9217 / similar             | `_px3`, `_px2`, `_pxhd`, `_pxcdi`, `_pxmvid`, `_pxAppId` |
| `_px*` window globals| line 16627 / similar            | `_pxUuid`, `_pxAppId`, `_pxHostUrl`, `_pxJsClientSrc`, `_pxFirstPartyEnabled`, `_pxRootUrl`, `_pxVid`, `_pxmvid` |
| Error reporter       | line 17135 / similar            | `collector-a.px-cloud.net/api/v2/collector/clientError` (cross-tenant shared host) |

The captcha core `captcha.js` is already byte-identical across the two
tenants (md5 `d5185dc31771cd80104622ef844dbcc9`); this diff confirms the
*main* bundle is also logically identical with just per-tenant
obfuscation layered on top.

## Per-tenant differences worth recording

### 1. Ped has a per-tenant XHR path

```js
// ped, line 11307
var n = ["https://collector-PXeT15wiaE.px-cloud.net", "/eT15wiaE/xhr"];
```

Haven does not show this `/eT15wiaE/xhr`-style path. The ped bundle
appears to support a fallback that POSTs to the tenant-prefixed XHR
endpoint, likely a back-compat route. Worth probing live to see if it
accepts the same sensor body shape.

### 2. Ped contains pure-JS SHA-256 + HMAC exports

Lines 1389–1453 of the ped deobf contain the full SHA-256 K constant
table (`var E = new Array(1116352408, 1899447441, ...);`) and an IIFE
exporting `hex_hmac`, `b64_hmac`, `any_hmac`. The entire block is
gated by:

```js
if ("HiEvlNZ" in ia) { (function () { /* SHA-256 + HMAC */ })(); }
```

`ia` is the bundle-local equivalent of `window`. `"HiEvlNZ"` is not a
public PX field — most likely a runtime feature-detection switch that
PX engineers can toggle by setting that property on the page. The IIFE
ends with `console.log(t)`, suggesting it is currently used for
diagnostic/canary purposes rather than the live `cs=` field.

Haven does not contain these K constants. So either:

- a) the live `cs=` is computed by a different code path in both
  bundles, and the SHA-256 IIFE is dead/debug-only (ped), OR
- b) the K constants in haven are reachable only via the residual
  `t("...")` accessor table that webcrack could not resolve.

Web Crypto (`crypto.subtle.sign`) is **not** used in either bundle —
only `crypto.getRandomValues` for nonces. So the live HMAC, if any,
must be pure JS.

### 3. CDN HEAD probe + cache headers

Both bundles probe response headers `active-cdn`, `x-served-by`,
`cache-control`, `x-px-cs-source` (haven line 8426–8429). The
`x-px-cs-source` header is interesting — `cs-source` literally maps to
the `cs=` query param. If this header advertises the source the
collector used to derive `cs`, it would be the cleanest way to confirm
the cs algorithm during a live capture.

## Collector POST plumbing

Haven's `Tf(r, t)` at line 11935 is the POST executor:

```js
function Tf(r, t) {
  // builds an XMLHttpRequest with f.open("POST", r.g, true)
  // sets headers from r.On
  // sends r.pn as body
}
```

- `r.g`  — request URL (already built upstream)
- `r.On` — header dictionary
- `r.pn` — pre-serialized body string

The descriptor `r` is built by `Vf(r)` at line 12017, which is the
real schema-aware assembler. `Vf` reads from the obfuscated
state tables and is the next bisect target if we want to derive the
sensor schema in full.

## Next R3 moves (priority order)

1. **Crack the per-call accessor table in ped.** Trace `hR(int)` and
   the inline `t("base64")` calls back to the table they read from.
   That table is what webcrack could not resolve. Once cracked, ped's
   deobf becomes fully readable and the haven cross-check is trivial.
2. **Live-capture `x-px-cs-source` from the canary** — confirm the
   server-side cs algorithm and short-circuit the JS reverse.
3. **Bisect `Vf()` in haven** (line 12017) to extract the field-name
   table for `payload`. Cross-reference each field against the recon
   capture in `2026-05-16-collector-payloads.md`.
4. **Decode `bi=` envelope.** The recon `bi=` value is base64 over a
   rolling-XOR'd binary payload. Likely emitted by a closer-up function
   to `Vf()`.
5. **Test the `/eT15wiaE/xhr` ped fallback** — does it accept the same
   sensor body? If yes, that simplifies the URL composition for the
   native solver path.

## Reproducibility

```bash
npx webcrack@latest --force -o px-research/deobf/12Ew76qT/main \
  px-research/init-js/12Ew76qT/2026-05-16-main.min.js
npx webcrack@latest --force -o px-research/deobf/eT15wiaE/main \
  px-research/init-js/eT15wiaE/2026-05-16-main.min.js
```
