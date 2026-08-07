# 0025. Egress proxy propagation: per-request for solve, session rotation for fetch

- **Date:** 2026-08-07
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0014 (challenge pipeline), ADR-0020 (Camoufox), ADR-0021 (handler routing),
  ADR-0024 (native sensor), [`docs/deployment.md`](../deployment.md)

## Context

Four proxy surfaces shipped through v1.8.0 and only one reached a browser:

| Surface | Status before this ADR |
|---|---|
| `PX_PROXIES` → `ProxyPool` → `SessionPool` | worked, `/v1/fetch` sessions only |
| `POST /v1/solve` body `"proxy"` | deserialized into `SolveRequestDto`, never read |
| `px-cli solve --proxy` | sent in the body, dropped server-side |
| `pxsolver_core::SolveRequest::with_proxy()` | published builder with zero consumers |
| `HarvestRequest.proxy` | honoured by `CamoufoxPool`, ignored by `ChromiumoxidePool` |

The cause was structural, not an oversight at one call site: `SolveDispatcher::solve(&self, url: &str)`
and `ChallengeHandler::solve(&self, page: &PageHtml)` had no parameter a proxy could travel in, and
both browser handlers built `HarvestRequest::new(url)`, which defaults `proxy` to `None`. Two
implementations of one `Harvester` trait also disagreed on whether the field meant anything.

Downstream Rust users were the worst affected: `SolveRequest::with_proxy()` is the obvious API to
reach for, is published on crates.io, and did nothing.

## Decision

**1. The solve path carries a per-request proxy end to end.** `ChallengeHandler::solve` takes a
`SolveAction { page, proxy }` — a domain action struct built at the infrastructure edge and passed
inward. `SolveDispatcher::solve` takes `px_core::SolveRequest`, so the published builder is the type
the HTTP edge maps into. Both browser handlers forward it via `HarvestRequest::with_proxy`, and
`ChromiumoxidePool` gained the `--proxy-server` argument it never had.

**2. The native sensor path honours it too.** `SolveContext` carries the proxy and
`SensorNativeSolver` posts through a per-proxy `reqwest::Client`, cached in `ProxyClients` so the
pooled TLS connection survives repeat solves.

**3. Solving never falls back to the `PX_PROXIES` rotation.** `_px3` is bound to the IP that earned
it. A caller who did not name an egress cannot route downstream traffic through a rotating one, so a
bundle harvested that way would not be usable. Rotation stays where it is useful: long-lived
`/v1/fetch` Camoufox sessions, which own their cookie jar and consume the bundle themselves.

**4. The egress is part of the cache key.** `sentinel_cache_key(domain, proxy)` hashes the proxy into
`fp_key`; a direct solve keeps `0`, so pre-existing keys still resolve. Without this, a bundle earned
through proxy A would be replayed to a caller who asked for proxy B.

**5. Proxy credentials are stripped for browser paths, with a warning.** geckodriver's W3C `proxy`
capability has no credential field and Chromium's `--proxy-server` ignores userinfo without a CDP
`Fetch.authRequired` handler. `strip_credentials` (px-harvester) removes `user:pass@` and logs the
sanitized URL. Authenticated upstreams are used through a local unauthenticated relay. The native
path is exempt: `reqwest` implements proxy auth.

## Consequences

- **Breaking:** `ChallengeHandler::solve` and `SolveDispatcher::solve` change signature; every
  handler crate (`px-perimeterx`, `px-cloudflare`, `px-native`, and the `px-turnstile` /
  `px-captcha` / `px-datadome` stubs) is updated in the same change. Per ADR-0017 this is a
  post-1.0.0 architectural change → manual `major` bump.
- `docs/deployment.md` gains an "Egress proxies" section correcting the old
  `N × len(proxies)` rotation claim: a session takes its proxy at spawn and keeps it until the 300s
  TTL, so distinct egress IPs per domain is `min(PX_FETCH_MAX_PER_DOMAIN, len(PX_PROXIES))`.
- `/v1/fetch` deliberately gains **no** per-request proxy field: switching a warm session's egress
  mid-life means respawning the browser and discarding the cookie jar it exists to hold.

## Alternatives considered

- **Delete the dead surfaces.** Smaller diff, but removes a published `pxsolver-core` API and two
  documented operator-facing options — a bigger break for downstream users than wiring them up.
- **Rotate on solve as well, and report the chosen egress in the response.** Needs the resolved
  proxy threaded back through `HarvestResult` → `HandlerOutcome` → `SolveOutput` → DTO. Deferred:
  no operator has asked for solve-side rotation, and per-request assignment already covers it.
- **Per-request proxy on `/v1/fetch`, keying sessions by `(domain, proxy)`.** Rejected for now; it
  multiplies live browsers per domain and complicates TTL eviction for a case the env list covers.
