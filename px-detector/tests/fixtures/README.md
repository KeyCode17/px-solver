# px-detector test fixtures

Captures of real PerimeterX-protected pages, plus a negative control. Used by the unit tests in `px-detector/tests/`.

## Files

| File | Source | Captured | Block status | Block style | Notes |
|---|---|---|---|---|---|
| `pedidosya-block.html` | `curl https://www.pedidosya.com.ar/home-page/v44/home/lazy_load?...` | 2026-05-16 | HTTP 403 | Block page (no captcha) | Standard PX block — `PX2`/`PXe6.../PXJ6...` rotating tokens visible. appId `eT15wiaE`. |
| `hannaandersson-block.html` | `curl https://www.hannaandersson.com/` | 2026-05-16 | HTTP 307 → 406 | Captcha (`<meta description="px-captcha">`) | Full `_px*` global suite exposed in HTML. appId `feReLYy8`. First-party mode confirmed via `_pxFirstPartyEnabled = true`. |
| `non-px-control.html` | Hand-written | 2026-05-16 | n/a | n/a | Negative case (synthetic) — detector must return `Detected::No`. |
| `havenwellwithin-non-px.html` | `curl https://havenwellwithin.com/` (first 8 KB) | 2026-05-16 | HTTP 200 | n/a | Negative case (real-world). Cloudflare + Salesforce Commerce Cloud, no PX layer. Guards against detector false positives on legitimate commerce HTML. |

## Sanitization

All captures are run through [`scripts/sanitize_capture.sh`](../../../../scripts/sanitize_capture.sh) before being committed:

- UUIDv1/v4 → `<UUID>`
- Lowercase hex strings ≥ 32 chars → `<HEX>`
- IPv4 addresses → `<IPV4>`
- `Bearer <token>` / `Authorization: <token>` → `<TOKEN>`
- JWT-shaped strings (`eyJ...`) → `<JWT>`

The PX tokens (`PX2`, `PXe6...`, `PXJ6...`, etc.) are intentionally **not** sanitized — they are server-issued per-request markers that contain no caller-identifying information and are required to exercise the block-page recognizer.

## Re-capturing

```bash
# Pedidosya block page (no PX cookies → 403)
curl -sS -A 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36' \
  https://www.pedidosya.com.ar/home-page/v44/home/lazy_load?country_id=3 \
  | scripts/sanitize_capture.sh /dev/stdin px-detector/tests/fixtures/pedidosya-block.html

# Hannaandersson block page (initial visit → 307)
curl -sS -L -A 'Mozilla/5.0 ... Chrome/124.0.0.0 Safari/537.36' \
  https://www.hannaandersson.com/ \
  | scripts/sanitize_capture.sh /dev/stdin px-detector/tests/fixtures/hannaandersson-block.html
```

## Detector expectations

For each fixture the detector must return:

| Fixture | `app_id` | `init_js_path` | `mode` | `block_class` |
|---|---|---|---|---|
| `pedidosya-block.html` | `eT15wiaE` | `/eT15wiaE/init.js` | `ReverseProxy` | `Block` |
| `hannaandersson-block.html` | `feReLYy8` | `/feReLYy8/init.js` | `ReverseProxy` | `Captcha` |
| `non-px-control.html` | — | — | — | `Detected::No` |
| `havenwellwithin-non-px.html` | — | — | — | `Detected::No` |
