# 002 — Scope & Boundaries

## In scope

- Detecting PX/HUMAN presence on a target page and extracting its `appId`.
- Running a pool of headless Chromium instances (via `chromiumoxide`) with manual CDP stealth patches (navigator.webdriver, plugins, languages, `chrome.runtime`, canvas/WebGL noise) to obtain a valid `_px3` cookie and its companions.
- Caching cookies per `(domain, appId, fingerprint key)` with TTL and metrics.
- Exposing a REST API (`/solve`, `/health`, `/metrics`) backed by Axum.
- Enforcing API-key authentication and a per-domain allowlist; recording every solve to an audit log.
- A CLI for local solves and operator diagnostics.
- A stub crate (`px-native`) carrying types, interfaces, and tests for a future native sensor-payload generator, so that path is not blocked architecturally.
- An integration test harness whose canary target is `pedidosya.com.ar`.
- Operator-facing documentation: README, deployment guide, threat model, dual-use policy.

## Out of scope (for v1 active solving)

- Any scraping logic beyond what is needed to validate that returned cookies work (e.g. one read of `/v2/niles/partners/{id}/menus` on the canary).
- A full reverse-engineering of PX's `/init.js` for v1 — only an architectural slot is reserved.
- **Active solving** of non-PX challenges in v1. Cloudflare Managed Challenge, Turnstile interactive widget, hCaptcha / reCAPTCHA, DataDome widget, and Akamai Bot Manager are **not implemented** in v1 — they ship as scaffolded handler stubs only (see [ADR-0014](adr/0014-challenge-pipeline-architecture.md) and [ADR-0015](adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md)).
- Proxy procurement and rotation logic — the service accepts a proxy URL per request but does not source proxies.
- Hosted SaaS offering, multi-tenant billing, or any commercial distribution.
- Use cases that involve unauthorized access, credential abuse, or violation of target ToS without an explicit operator-side allowlist entry.

### Passive defeat is not "out of scope"

Because the harvester drives a real Chromium with a stealth bundle, **passive edge layers are typically defeated incidentally** without a dedicated handler. This includes:

- Fastly / CloudFront / Cloudflare silent scoring (no JS challenge fired).
- Cloudflare JS Challenge (legacy `cdn-cgi/challenge-platform`) — usually passes.
- Turnstile **invisible** mode — usually passes.
- Akamai passive WAF / DDoS layers.

v1 does not commit to any success rate against these layers; they are best-effort byproducts of running real Chromium. Anything that requires an **interactive** challenge or **captcha** does not pass without a dedicated handler implementation (deferred to v2).

## Scope addendum — 2026-05-16

The original "out of scope: Cloudflare Turnstile, DataDome, Akamai BMP" wording was softened to reflect the pipeline architecture decision in [ADR-0014](adr/0014-challenge-pipeline-architecture.md): v1 ships the pipeline scaffold and stub crates for these vendors so v2 implementations slot in without architectural rewrites. The actual *implementation* of those handlers remains out of v1 scope.

## System boundaries

```
client ──HTTPS──▶ px-server (Axum)
                       │
                       ▼
                  px-pipeline ──┐
                                ├─ px-cloudflare (stub, v2)
                                ├─ px-turnstile  (stub, v2)
                                ├─ px-perimeterx (v1, marquee)
                                ├─ px-datadome   (stub, v2)
                                └─ px-captcha    (stub, v2)
                       │
            ┌──────────┴──────────┐
            ▼                     ▼
       px-cache             px-harvester
            │              (Chromium pool, shared infra)
            │                     │
            │                     ▼
            │              Chromium (CDP) ──▶ target site
            ▼
        DashMap / Redis
```

`px-pipeline` walks handlers in configured order. Each handler decides via `detects(page)` whether it applies; `solve(page)` runs only when matched. `px-harvester` is the vendor-agnostic Chromium pool; every handler shares it.
