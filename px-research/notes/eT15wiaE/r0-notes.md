---
appId: eT15wiaE
captured: 2026-05-16
phase: R0 — Acquisition
target: pedidosya.com.ar
---

# R0 — Acquisition notes (eT15wiaE / pedidosya.com.ar)

## Files captured

| File | Vendor | Size | Source URL | Local path |
|---|---|---|---|---|
| `main.min.js` | HUMAN Security | 230 kB | `https://client.px-cloud.net/PXeT15wiaE/main.min.js` | `init-js/eT15wiaE/2026-05-16-main.min.js` |
| `personalization2.js` | HUMAN Security / PedidosYa shim | 4.5 kB | `https://live.pystatic.com/pxassets/personalization2.js` | `init-js/eT15wiaE/2026-05-16-personalization2.js` |
| `captcha.js` | not captured (403 — single-use session token expired) | n/a | `<site>/eT15wiaE/captcha/captcha.js?a=c&u=<UUID>&v=<UUID>&m=0&...` | — |
| `auditor.js` | unknown / off-host | 272 kB | `https://crcldu.com/bd/auditor.js?v=1778905440000` | `init-js/_off-host/...` (shared) |

## Notable

- **First-party captcha host** (`<site>/eT15wiaE/captcha/`), not the hosted
  CDN form haven uses.
- **Captcha session URL is single-use.** Replaying the captured captcha.js
  URL returned 403 with a PX block page. The 586 kB byte-identical bundle
  already lives under `init-js/feReLYy8/` and `init-js/12Ew76qT/`.
- `personalization2.js` is a small PedidosYa-side UI shim that touches
  `window._pxUuid`, `window._pxSelectedLocale`, and `window._pxTranslation`.
  Confirms operators can localize the block-page UI without modifying PX.
- Additional off-host calls observed (not captured because they require
  active session state): `tzm.px-cloud.net/ns?c=<UUID>` (tag/event tracker)
  and `js.px-cloud.net/?t=<token>&v=<UUID>` (one-time challenge token).
  Endpoints recorded in `sensor-maps/2026-05-16-collector-payloads.md`.

## Collector POST captured

See `sensor-maps/2026-05-16-ped-collector-post.txt`.

Pedidosya's POST carries an additional `hid=<base64>:<base64>` field absent
from haven's, suggesting a tenant-configurable extra ID. The decoded `sid`
watermark: 1778905440180 ms (2026-05-16 04:24:00 UTC).
