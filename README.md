# px-solver

A Rust-built solver service for PerimeterX (HUMAN Security) protection. Given a target URL on a per-domain allowlist, returns a valid `_px3` cookie bundle that a downstream authorized client can use to issue requests as if from a real browser.

> **Status:** v1.10.0, published to crates.io as the `pxsolver-*` family of crates. MVP gate hit at v1.0.0; v1.1.0 added a Camoufox-backed Cloudflare bypass path; v1.2.0 renamed the published library crates; v1.8.0 activated native `_px3` sensor synthesis ([ADR-0024](docs/adr/0024-activate-native-px3-sensor-synthesis.md)); v1.9.0 made the per-request egress proxy real end to end ([ADR-0025](docs/adr/0025-egress-proxy-propagation-contract.md)); v1.10.0 carries per-crate documentation only — no code change from v1.9.0. See [GitHub Releases](https://github.com/KeyCode17/px-solver/releases) for the per-version notes.
>
> ⚠️ **v1.9.0 is source-breaking despite the minor bump.** `ChallengeHandler::solve` and `SolveDispatcher::solve` take a `SolveAction` / `SolveRequest` instead of `&PageHtml` / `&str`. If you implement `ChallengeHandler` outside this workspace, add the parameter when upgrading; the rest of the published surface is unchanged.

## What this is

- A REST API service backed by a stealth-patched Chromium pool (`chromiumoxide` + manual CDP patches) and, for Cloudflare-fronted targets, a Camoufox/geckodriver pool ([ADR-0020](docs/adr/0020-adopt-camoufox-via-fantoccini-geckodriver.md)).
- A **challenge pipeline** that routes each request to the correct vendor handler. v1 ships the **PerimeterX** handler and a Camoufox-backed **Cloudflare** handler; stubs exist for Turnstile, hCaptcha/reCAPTCHA, and DataDome so they slot in cleanly in v2.
- An API-key + per-domain allowlist + audit log layer that bounds dual-use abuse ([ADR-0007](docs/adr/0007-api-key-and-domain-allowlist-guardrails.md)).

## What this is **not**

- **Not** a native PerimeterX sensor generator. v1 defeats PX by **avoidance** (real Chromium passes the challenge legitimately), not by **defeat** (rebuilding the sensor payload from scratch). The codebase is structured so a native solver can be added; the reverse-engineering work itself is out of v1 scope. See [ADR-0010](docs/adr/0010-defer-native-sensor-generator.md) and [ADR-0013](docs/adr/0013-re-methodology-and-scope.md).
- **Not** an anonymous public solver. Every request requires an API key; every target requires an explicit allowlist entry with `tos_reviewed: true`. See [ADR-0007](docs/adr/0007-api-key-and-domain-allowlist-guardrails.md) and [`docs/011-sow-dual-use.md`](docs/011-sow-dual-use.md).
- **Not** a Turnstile / DataDome / captcha solver in v1. The pipeline routes to those handlers when their protections are detected, but their `solve()` methods return `NotImplemented` until v2. Many passive edge layers (Fastly, CloudFront, Cloudflare silent scoring, Turnstile invisible mode) are **incidentally** defeated by running real Chromium or Camoufox with stealth — but no success rate is committed for them. See [ADR-0014](docs/adr/0014-challenge-pipeline-architecture.md) and [ADR-0015](docs/adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md).

## Install

From source:

```bash
git clone https://github.com/KeyCode17/px-solver
cd px-solver
cargo build --release
```

That builds both binaries — `px-server` and `px-cli`. They are **not** on crates.io (`publish = false`); source is the only way to get them.

From crates.io, as libraries:

```toml
[dependencies]
pxsolver-core     = "1"   # domain types: SolveRequest, PxCookieBundle, CacheKey
pxsolver-pipeline = "1"   # ChallengeHandler, SolveAction, Pipeline
pxsolver-harvester = "1"  # Harvester port + stealth Chromium pool
```

All 16 `pxsolver-*` library crates are published individually, so you can depend on one piece
without the workspace. Each has its own README on crates.io.

> **Package name ≠ crate name.** You depend on `pxsolver-core` but you `use px_core::…`. The
> `pxsolver-` prefix namespaces the family on crates.io; the source-level name stays short.

| Crate | Purpose |
|---|---|
| [`pxsolver-core`](https://crates.io/crates/pxsolver-core) | Pure domain types, no I/O |
| [`pxsolver-types`](https://crates.io/crates/pxsolver-types) · [`pxsolver-errors`](https://crates.io/crates/pxsolver-errors) · [`pxsolver-validation`](https://crates.io/crates/pxsolver-validation) | Response envelope, `AppError`, `Validated<T>` |
| [`pxsolver-pipeline`](https://crates.io/crates/pxsolver-pipeline) | `ChallengeHandler` port + ordered pipeline |
| [`pxsolver-detector`](https://crates.io/crates/pxsolver-detector) | PerimeterX detection from HTML/JS |
| [`pxsolver-harvester`](https://crates.io/crates/pxsolver-harvester) · [`pxsolver-camoufox`](https://crates.io/crates/pxsolver-camoufox) | Chromium pool · Camoufox/geckodriver pool |
| [`pxsolver-perimeterx`](https://crates.io/crates/pxsolver-perimeterx) · [`pxsolver-cloudflare`](https://crates.io/crates/pxsolver-cloudflare) | Shipping handlers |
| [`pxsolver-turnstile`](https://crates.io/crates/pxsolver-turnstile) · [`pxsolver-captcha`](https://crates.io/crates/pxsolver-captcha) · [`pxsolver-datadome`](https://crates.io/crates/pxsolver-datadome) | Detection-only stubs |
| [`pxsolver-native`](https://crates.io/crates/pxsolver-native) | Native `_px3` sensor synthesis |
| [`pxsolver-auth`](https://crates.io/crates/pxsolver-auth) · [`pxsolver-cache`](https://crates.io/crates/pxsolver-cache) | API keys + allowlist + audit · cookie cache |

## Quickstart

1. Generate a key and add an allowlist entry:

   ```bash
   cargo run -p px-cli -- keys generate --id ops1 --note "first operator"
   # paste the printed id + argon2_hash into config/keys.yaml

   cargo run -p px-cli -- allowlist add \
     --domain pedidosya.com.ar \
     --justification "internal price observability"
   ```

2. Start the server:

   ```bash
   PX_BIND=127.0.0.1:8080 \
   PX_KEYS=config/keys.yaml \
   PX_ALLOWLIST=config/allowlist.yaml \
   cargo run -p px-server
   ```

3. Solve a target:

   ```bash
   curl -X POST http://127.0.0.1:8080/v1/solve \
     -H "Authorization: Bearer ops1:<secret>" \
     -H "content-type: application/json" \
     -d '{"url":"https://www.pedidosya.com.ar/","proxy":null}'
   ```

   Response shape:

   ```json
   {
     "data": {
       "user_agent": "Mozilla/5.0 ...",
       "solve_ms": 10273,
       "cache_hit": false,
       "handler": "perimeterx",
       "cookies": [{"name":"_px3","value":"...","domain":".pedidosya.com.ar","path":"/"}],
       "expires_at": 1747500000
     },
     "status": "solved"
   }
   ```

For systemd, reverse proxy, and key rotation workflows see [`docs/deployment.md`](docs/deployment.md).

## Proxies

Two mechanisms, one per endpoint. They are **not** interchangeable, and neither one falls back to the other.

| | `/v1/solve` | `/v1/fetch` |
|---|---|---|
| Assigned by | `"proxy"` in the request body, or `px-cli solve --proxy` | `PX_PROXIES` env var (CSV), operator-side |
| Chosen per | request | Camoufox session, at spawn |
| Rotation | none — the solve uses exactly the proxy you named | round-robin across the list |
| Omitted | server's own IP | server's own IP |

```bash
# /v1/solve — name the egress you will send downstream traffic through
-d '{"url":"https://www.pedidosya.com.ar/","proxy":"socks5://127.0.0.1:9050"}'

# /v1/fetch — operator-side rotation across warm sessions
PX_PROXIES="http://p1.example:8080,socks5://p2.example:1080" ./target/release/px-server
```

A `_px3` bundle is bound to the IP that earned it, so **use the same proxy downstream that you named on the solve**. The proxy is part of the cache key: the same domain solved through two proxies yields two entries and neither is served to the other.

Three things that surprise people:

- **`PX_PROXIES` does nothing for `/v1/solve`.** Rotation there would hand you a bundle bound to an IP you cannot know.
- **Browser proxies cannot authenticate.** geckodriver's W3C `proxy` capability has no credential field and Chromium ignores userinfo without a CDP `Fetch.authRequired` handler, so `user:pass@` is stripped with a warning. Front an authenticated upstream with a local relay (gost, 3proxy). The native sensor path goes over `reqwest` and *does* accept credentials.
- **Distinct egress IPs per domain on `/v1/fetch` is `min(PX_FETCH_MAX_PER_DOMAIN, len(PX_PROXIES))`**, not the product — a session takes its proxy at spawn and keeps it for the 300s TTL.

Full reference: [Egress proxies](docs/deployment.md#egress-proxies) · rationale: [ADR-0025](docs/adr/0025-egress-proxy-propagation-contract.md).

## Documentation

| Doc | Purpose |
|---|---|
| [`docs/000-sow-index.md`](docs/000-sow-index.md) | Statement of Work index + deliverable traceability |
| [`docs/adr/README.md`](docs/adr/README.md) | Architecture Decision Records (23 ADRs as of 2026-05-17) |
| [`docs/phase/README.md`](docs/phase/README.md) | Phase plan (00–04 critical path + R research) |
| [`docs/deployment.md`](docs/deployment.md) | Fresh-Linux install, systemd, reverse proxy, key generation, allowlist editing, egress proxies |
| [`docs/threat-model.md`](docs/threat-model.md) | Misuse vectors + mitigations |
| [`docs/dual-use-policy.md`](docs/dual-use-policy.md) | Operator commitments per [`docs/011-sow-dual-use.md`](docs/011-sow-dual-use.md) |
| [`docs/standards/axum-best-practice.md`](docs/standards/axum-best-practice.md) | Coding standard (Clean Architecture, ≤200 LOC/file, no `unwrap`) |
| [`docs/standards/design-patterns.md`](docs/standards/design-patterns.md) | Canonical inventory of patterns + Rust idioms + anti-patterns |
| [`px-research/README.md`](px-research/README.md) | Research-track home (RE captures, deobfuscation notes) |
| [`xtask/`](xtask/) | Dev automation: `cargo xtask {bump,check-loc,release,canary,phase}` ([ADR-0016](docs/adr/0016-xtask-for-dev-automation.md)) |
| [`rust-toolchain.toml`](rust-toolchain.toml) | Pins Rust 1.95 with `rustfmt` + `clippy` |
| [`docs/adr/0017-phase-aligned-versioning.md`](docs/adr/0017-phase-aligned-versioning.md) | Versioning policy: phase 00–03 → minor bumps, phase 04 → 1.0.0 |

## License

**AGPL-3.0-or-later** — chosen to discourage closed-source resale as an anonymous SaaS. The license is declared in the workspace `Cargo.toml`; a top-level `LICENSE` file is tracked as a follow-up.
