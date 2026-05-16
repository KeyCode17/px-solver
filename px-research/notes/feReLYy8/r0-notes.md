---
appId: feReLYy8
captured: 2026-05-16
phase: R0 — Acquisition
target: hannaandersson.com
---

# R0 — Acquisition notes (feReLYy8 / hannaandersson.com)

## Files captured

| File | Vendor | Size | Source URL | Local path |
|---|---|---|---|---|
| `captcha.js` | HUMAN Security (PerimeterX) | 586 kB | `https://www.hannaandersson.com/feReLYy8/captcha/captcha.js?a=c&u=<UUID>&v=<UUID>&m=0&h=R0VU` | `init-js/feReLYy8/2026-05-16-captcha.js` |
| `main.min.js` | HUMAN Security (PerimeterX) | 228 kB | `https://client.px-cloud.net/PXfeReLYy8/main.min.js` | `init-js/feReLYy8/2026-05-16-main.min.js` |
| `auditor.js` | **unknown** (likely Akamai Bot Manager — see below) | 272 kB | `https://crcldu.com/bd/auditor.js?v=1778902560000` | `init-js/_off-host/2026-05-16-crcldu-auditor.js` |

## Surface observations (pre-R1)

### PerimeterX / HUMAN

- The first 200 bytes of both `captcha.js` and `main.min.js` carry an explicit
  HUMAN Security copyright header (`@license Copyright (C) 2012-2026 HUMAN Security, Inc`).
- `main.min.js` is the **per-tenant entry**: it begins
  `(function(){try{window._pxAppId = "PXfeReLYy8";...`. The tenant configuration
  is inlined at the top; the rest is presumably the shared core.
- `main.min.js` is **off-host** on `client.px-cloud.net`. The site's own
  `/feReLYy8/init.js` reverse-proxy path is documented in our existing
  fixtures, but in practice browser code is loaded from PX's CDN — the
  first-party path is for the cookie-bearing collector calls, not for the JS itself.
- `captcha.js` is the **active challenge bundle** loaded when block mode is
  captcha (as observed for hannaandersson — see `px-detector/tests/fixtures/hannaandersson-block.html`).
- The captcha URL carries two UUIDv1 query params (`u=` and `v=`) which match
  the `_pxhd` companion-UUID format documented in ADR-0012. After sanitization
  the URL appears in this corpus as `?a=c&u=<UUID>&v=<UUID>&m=0&h=R0VU` —
  the `h=R0VU` is base64 of `GET`.

### Off-host beacon (`crcldu.com`)

- `crcldu.com/bd/auditor.js` is loaded **on the same captcha page** as the
  HUMAN bundles. Path `/bd/` plus filename `auditor.js` plus hex-string
  obfuscation pattern are the classic Akamai Bot Manager shape, but no
  Akamai marker survives sanitization to confirm.
- No HUMAN copyright header. Very different code shape from the PX bundles.
- The fact that hannaandersson.com is loading **two different bot-defense
  layers** on the same page is operationally interesting: it explains why
  some sites that look "only PX" actually carry a second telemetry channel
  that our harvester will need to satisfy. Treat as candidate for
  Phase R surface mapping (R1) on a future pass.

## Open questions for R1

1. Does `main.min.js` from `client.px-cloud.net/PXfeReLYy8/` differ in any byte
   from `pedidosya.com.ar/eT15wiaE/init.js`? Hypothesis: only the embedded
   `_pxAppId` literal differs. **Requires:** capture pedidosya's main.min.js
   (or equivalent) and diff. *Not yet done.*
2. What is the VM opcode table in `main.min.js`'s sensor builder?
3. Where is the AES key for the sensor body derived? (Cross-reference between
   `main.min.js` and `captcha.js`.)
4. What does the `crcldu.com` auditor measure, and does failing to load it
   affect the PX score?

## Open questions for R3

(Out of scope until R1 + R2 land.)

- Sensor payload field grammar.
- HMAC chain.
- `_pxhd` server-salt derivation rule.

## Tool versions

| Tool | Version |
|---|---|
| `scripts/sanitize_capture.sh` | git commit `587c8d1`+ as of this capture |
| Operator user agent string sent | `Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36` |
| Referer sent | `https://www.hannaandersson.com/` |
