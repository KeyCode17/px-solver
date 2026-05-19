# R2 — gB(t) decoder + sensor surface map (eT15wiaE, 2026-05-20)

Follow-up to `r0-r2-pedidosya-2026-05-19.md`. The webcrack pass-1 output left
**1457 obfuscated string lookups** behind `gC(N)` / `gF(N)` indirections.
This step writes a static decoder for the `gB(t)` base-91-ish algorithm and
applies it to every literal-indexed call site.

## Decoder

`px-research/tools/decoder/gb_decode.js` is a Node port of the in-page
`gB(t)`. Two alphabets observed in the capture:

* **Outer** — used by the top-level memoizer (`gC`/`gF`):
  ```
  RgbH8zv3:#<yhG17YJs}TM_"x~49+mI;`jAedPp]C)F=ouQ@E?,KX!nq6$iZDlcSW0aBwkU.V(2^*fLrtN&[|O%5{/>
  ```
* **VM** — used by a second-level decoder inside the embedded VM:
  ```
  50}.!w*?x@~62Nv_VKP|G%=4jot9#m:q;8i$,)kW1LE<F`rDM&QH[lIahZ]S>JCzYuBcARXe+g/7^Un{pbd3Tyf(Os
  ```

The transform (`tools/decoder/transform.js`) walks the Babel AST and rewrites
every `gB("…")` / `gC(N)` / `gF(N)` call with the decoded literal whenever
the argument is a string literal or numeric index into `gE`.

## Pipeline

```
1. Capture     -> init-js/eT15wiaE/2026-05-19-init.js                                (raw)
2. webcrack #1 -> deobf/eT15wiaE/2026-05-19-init.webcrack.js                         (12 333 changes)
3. gB pass #1  -> deobf/eT15wiaE/2026-05-20-init.gb-decoded.js                       (92 inlined)
4. webcrack #2 -> deobf/eT15wiaE/main-pass2/deobfuscated.js                          (9 392 changes)
5. gB pass #2  -> deobf/eT15wiaE/2026-05-20-init.gb-decoded.v2.js                    (1 457 inlined)
```

## What surfaced

### PerimeterX cookies + telemetry endpoints

```js
var gW = ["/init.js", "/main.min.js"];                              // self-fetch markers
var gX = "https://collector-a.px-cloud.net/api/v2/collector/clientError?r=";
var gY = "pxhc";    // cookie name (HMAC/integrity)
var gZ = "pxjsc";   // cookie name (JS client sentinel)
var hb = "pxc";     // cookie name (PX client)
```

### Bot-defense sister domains (base64-veiled in source)

| Encoded                                       | Decoded                              |
| --------------------------------------------- | ------------------------------------ |
| `aHR0cHM6Ly9qcy5weC1jbG91ZC5uZXQ=`            | `https://js.px-cloud.net`            |
| `aHR0cHM6Ly9jcmNsZHUuY29t`                    | `https://crcldu.com`                 |
| `aHR0cHM6Ly9jcmNsZHUuY29tL2JkL3N5bmMuaHRtbA==`| `https://crcldu.com/bd/sync.html`    |
| `aHR0cHM6Ly9jbGllbnQud3JhLWFwaS5uZXQ=`        | `https://client.wra-api.net`         |
| `cHgtY2RuLm5ldA==` / `cHgtY2xpZW50Lm5ldA==`   | `px-cdn.net` / `px-client.net`       |

### Sensor event channels

```
"PX561"   "PX11353" "PX11745" "PX11859" "PX11978" "PX11994"
"PX12002" "PX12013" "PX12132" "PX12457" "PX12616" "PX12617"
"PX12635" "PX12733"   // plus  "AN08"  "LN10"  "LN2"  "ZW49"
```

These are event/error type tags, not JSON keys — they tag entries inside
the encrypted payload that POSTs to `/eT15wiaE/xhr/b/s`.

### Behavioural surface

```
mousedown, mouseup, mouseout, click, scroll, keydown, touchstart, message,
load, onloadend, securitypolicyviolation, xhrSuccess, xhrResponse, xhrFailure
```

### Fingerprint surface

`hardwareConcurrency`, `addressSpace`, `webgl`, `webkit`, `mozSetImageElement`,
~140 enumerated font names, ~30 `MediaSource.isTypeSupported(...)` codec
probes (video/mp4 avc1 variants, video/ogg+theora/dirac, video/x-matroska,
video/3gpp, …).

### Webdriver-detection probes (base64-veiled)

| Encoded                                   | Decoded                       |
| ----------------------------------------- | ----------------------------- |
| `bmF2aWdhdG9yLndlYmRyaXZlcg==`            | `navigator.webdriver`         |
| `c2VsZW5pdW0tZXZhbHVhdGU=`                | `selenium-evaluate`           |
| `d2ViZHJpdmVyLWV2YWx1YXRl`                | `webdriver-evaluate`          |
| `JGNocm9tZV9hc3luY1NjcmlwdEluZm8=`        | `$chrome_asyncScriptInfo`     |
| `aXNOb2RlUmVhY2hhYmxlXw==`                | `isNodeReachable_` (PhantomJS)|

These resolve to the legacy detection vectors Camoufox already patches —
useful as a list of patches to verify against.

### CAPTCHA hooks

```
handleCaptcha · pxCaptchaUIEvents · px-captcha · triggerPxAutoAbrCaptchaDemo
pxHandleAutoABR · recaptcha-token
```

The presence of `pxHandleAutoABR` + `triggerPxAutoAbrCaptchaDemo` confirms
the eT15wiaE tenant is on Auto-ABR (the press-and-hold UI) — matching what
we see in `PX_DIAGNOSE_LAND_DIRECT=1` dumps on the menus endpoint.

### Bit-mix primitives at lines 299-308 of the decoded source

```
function hl(t, n) { ... }  // 32-bit ADD with carry
function hm(t, n) { ... }  // ROL
function hn(t, n, e, r, g, o) { return hl(hm(hl(hl(n, t), hl(r, o)), g), e); }
function ho(t, n, e, r, g, o, i) { ... }
```

This is the **MD5 inner loop** (F/G/H/I round helpers). The sensor uses MD5
for at least one integrity field — likely the `pxhc` cookie or a row in the
encrypted payload. A SHA-256 path and an AES-CBC path are also expected to
appear in the next pass; the alphabet `ABCDEFGHIJKLMNOPQRSTUVWXYZab…0123456789+/=`
is present, confirming a base64 encoder is also inlined.

## Status update (was R0=done, R2=partial)

| Phase | Status                                                                 |
| ----- | ---------------------------------------------------------------------- |
| R0    | ✅ done (2026-05-19)                                                    |
| R1    | ⏳ partial — surface mapped statically; CDP-hooked dynamic trace next.  |
| R2    | ✅ pass 1+2 done (1 457 strings decoded). VM alphabet decoder ready for |
|       |    the inner VM stage once we extract it.                              |
| R3    | ⏳ not started — needs MD5/AES walk to identify sensor builder fn.      |
| R4    | ⏳ blocked on R3.                                                       |

## Next concrete actions

1. Tag the sensor-builder function. With strings inlined we can grep for
   `pxhc`, `pxjsc`, `PX12457`, `/eT15wiaE/xhr/b/s` → that locates the
   POST-emit site, walk backwards from there.
2. Extract the inner VM (the `gN`/`gO`-scoped runtime that uses
   `ALPHABET_VM`) into its own file, then run the v2 decoder on it.
3. Hook `XMLHttpRequest.prototype.send` in Camoufox to dump the
   pre-encrypted sensor payload — gives ground truth for R3.
