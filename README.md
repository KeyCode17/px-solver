# px-solver

A Rust-built solver service for PerimeterX (HUMAN Security) protection. Given a target URL on a per-domain allowlist, returns a valid `_px3` cookie bundle that a downstream authorized client can use to issue requests as if from a real browser.

> **Status:** v1.2.0 published to crates.io as the `pxsolver-*` family of crates. MVP gate hit at v1.0.0; v1.1.0 added a Camoufox-backed Cloudflare bypass path; v1.2.0 renamed the published library crates. See [GitHub Releases](https://github.com/KeyCode17/px-solver/releases) for the per-version notes.

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

From crates.io:

```bash
cargo install pxsolver-server pxsolver-cli
```

The 16 `pxsolver-*` library crates are also published individually for downstream Rust users; in source the workspace exposes them under the short `px-*` aliases.

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

## Documentation

| Doc | Purpose |
|---|---|
| [`docs/000-sow-index.md`](docs/000-sow-index.md) | Statement of Work index + deliverable traceability |
| [`docs/adr/README.md`](docs/adr/README.md) | Architecture Decision Records (23 ADRs as of 2026-05-17) |
| [`docs/phase/README.md`](docs/phase/README.md) | Phase plan (00–04 critical path + R research) |
| [`docs/deployment.md`](docs/deployment.md) | Fresh-Linux install, systemd, reverse proxy, key generation, allowlist editing |
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
