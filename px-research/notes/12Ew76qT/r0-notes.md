---
appId: 12Ew76qT
captured: 2026-05-16
phase: R0 — Acquisition
target: havenwellwithin.com
---

# R0 — Acquisition notes (12Ew76qT / havenwellwithin.com)

## Status correction

Earlier recon (commit 0099e666 area) classified havenwellwithin.com as
NOT PerimeterX based on an unchallenged GET that returned HTTP 200 with
no PX globals. This operator-DevTools capture shows the **site IS PX
protected**; PX simply did not fire on that initial GET. Detector test
renamed to `havenwellwithin_unchallenged_response_returns_no` to reflect
the actual semantics (commit ec76613).

## Files captured

| File | Vendor | Size | Source URL | Local path |
|---|---|---|---|---|
| `main.min.js` | PerimeterX (LEGACY copyright '(C) 2014-2026 PerimeterX, Inc') | 261 kB | `https://client.px-cloud.net/PX12Ew76qT/main.min.js` | `init-js/12Ew76qT/2026-05-16-main.min.js` |
| `captcha.js` | HUMAN Security | 586 kB | `https://captcha.px-cdn.net/PX12Ew76qT/captcha.js?a=c&u=<UUID>&v=<UUID>&m=0` | `init-js/12Ew76qT/2026-05-16-captcha.js` |
| `auditor.js` | unknown / off-host (likely Akamai BMP) | 272 kB | `https://crcldu.com/bd/auditor.js?v=1778902920000` | `init-js/_off-host/2026-05-16-crcldu-auditor.js` (shared) |

## Notable

- **Hosted captcha mode.** Unlike hannaandersson and pedidosya which serve
  `captcha.js` from a first-party path under `<site>/<appId>/captcha/`,
  haven loads it from `captcha.px-cdn.net`. `PxMode::Hosted` confirmed.
- **Legacy PerimeterX copyright.** haven's `main.min.js` carries the
  pre-rebrand `(C) 2014-2026 PerimeterX, Inc` header; hanna and ped's
  builds carry the post-rebrand `(C) 2012-2026 HUMAN Security, Inc`.
  haven appears to be on an older build chain.
- **captcha.js is byte-identical to hannaandersson's** (md5
  `d5185dc31771cd80104622ef844dbcc9`). Captcha bundle is shared core
  with no per-tenant variation.

## Collector POST captured

See `sensor-maps/2026-05-16-haven-collector-post.txt` and
`sensor-maps/2026-05-16-collector-payloads.md`.

The decoded `sid` watermark: 1778902922699 ms (2026-05-16 03:42:02 UTC).
