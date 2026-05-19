# R0 → R2 — pedidosya.com.ar PX tenant `eT15wiaE`, 2026-05-19

## Provenance
- App ID: `PXeT15wiaE` (script path uses `eT15wiaE`, runtime `_pxAppId` carries the `PX` prefix).
- Capture method: Camoufox webdriver session via px-server `/v1/fetch`
  with `PX_DIAGNOSE_LAND_DIRECT=1`, routed through AR residential
  proxy `socks5://us.novproxy.io:1000` (region-AR, t-5 sticky session).
- Capture date: 2026-05-19.
- Operator IP: redacted (residential AR via novproxy gateway).

## Captures
| Path | Size | Purpose |
|---|---|---|
| `init-js/eT15wiaE/2026-05-19-init.js` | 226 130 B | Main sensor / challenge JS served at `/eT15wiaE/init.js`. |
| `init-js/eT15wiaE/captcha/2026-05-19-captcha.js` | 586 549 B | Captcha widget JS served at `/eT15wiaE/captcha/captcha.js`. |
| `deobf/eT15wiaE/2026-05-19-init.webcrack.js` | 423 327 B | Pass-1 deobfuscation (webcrack `transpile, unminify`; 12 333 changes; string-array deobf was a no-op because PX uses VM-style protection, not the standard string-array pattern). |

## Surface observations (R1, hand-skim)

The de-obfuscated `init.webcrack.js` opens with:

```js
(function () {
  try {
    window._pxAppId = "PXeT15wiaE";
    function gB(t) {
      var n = "" + (t || "");
      var c = "RgbH8zv3:#<yhG17YJs}TM_\"x~49+mI;`jAedPp]C)F=ouQ@E?,KX!nq6$iZDlcSW0aBwkU.V(2^*fLrtN&[|O%5{/>".indexOf(n[a]);
      ...
```

- `gB(t)` is the canonical string decoder, base-91 over the custom
  alphabet shown above. Every readable string literal in the file is
  wrapped in `gB("…")`. Decoding `gB` calls at static-analysis time is
  the **next R2 sub-step** — webcrack doesn't recognize this pattern,
  needs a custom transform.
- A second alphabet appears in the body
  (`"50}.!w*?x@~62Nv_VKP|G%=4jot9#m:q;8i$,)kW1LE<F\`rDM&QH[lIahZ]S>JCzYuBcARXe+g/7^Un{pbd3Tyf(Os"`)
  — likely a per-tenant rotation, or a different decoder used by the
  VM section. Catalogue all alphabets before R3.
- The 226 KB main + 586 KB captcha bundle together is on the larger
  end for PX-3 tenants we've seen (12Ew76qT / feReLYy8 captures were
  ~190 KB combined); pedidosya has the bigger sensor surface, likely
  because the captcha JS includes the press-and-hold widget logic.

## Status board

| Phase | Status |
|---|---|
| R0 — Acquisition | ✅ done 2026-05-19. |
| R1 — Surface mapping | ✅ done 2026-05-20 — static surface from `r2-decoder-output-2026-05-20.md` is sufficient; CDP trace deferred to N3 (event-collector calibration) where it serves a clearer purpose. |
| R2 — Deobfuscation | ✅ done 2026-05-20 — 1 457 strings inlined via custom `gB(t)` transform; pass-2 output at `deobf/eT15wiaE/2026-05-20-init.gb-decoded.v2.js`. See `r2-decoder-output-2026-05-20.md`. |
| R3 — Sensor RE | ✅ done 2026-05-20 — cipher = XOR(0x32) + base64 + `vQ`-splice VM; no AES, no HMAC. See `r3-sensor-grammar-2026-05-20.md`. |
| R4 — Bypass selection | ✅ done 2026-05-20 — ADR-0024 promotes ADR-0010 to native synthesis. |

## Hard-ban observation (relevant to bypass strategy)

Direct webdriver navigation to `/v2/niles/partners/<id>/menus?occasion=DELIVERY`
on this tenant returns a 6072-byte response containing
`<title>Access to this page has been denied</title>` and
`class="px-captcha-error-*"` markup — i.e. the PX **error/denied**
state, **not** a solvable press-and-hold widget. Iframes to
`js.px-cloud.net` and `crcldu.com/bd/sync.html` are present but
`display:none`. This means:

- A click-the-widget bypass is **not applicable** for this tenant at
  the niles endpoint — there is no widget to click. The denial page
  is served directly when PX's risk score crosses a hard threshold.
- The native sensor / cookie-synthesis path (ADR-0010) is the
  remaining native route. The `_px3` cookie issued during a clean
  homepage harvest does not carry enough trust to access niles; we
  need to either lift the risk score behaviourally (R3 → know what
  signals PX wants) or synthesise a higher-trust `_px3` directly.

## Next concrete RE steps (in order)

1. Write a webcrack post-processor that statically decodes every
   `gB("…")` call. Estimated 1-2 days work in JavaScript.
2. Re-run pass 2 → expect 30-50 % of strings to become readable
   (event names, prop names, URL paths).
3. CDP-hooked browser run (R1 proper). Log everything `init.js`
   accesses during a successful homepage solve.
4. Cross-reference the property-access log with the deobfuscated code
   to identify the sensor builder function. Tag it.
5. R3 — trace inputs → encrypted payload → POST to `/eT15wiaE/xhr/b/s`.

Estimated calendar effort: 4-8 weeks at half-time. Aligns with the
"3-6 months full-time" rough estimate in ADR-0013.
