# px-solver

A Rust-built solver service for PerimeterX (HUMAN Security) protection. Given a target URL on a per-domain allowlist, returns a valid `_px3` cookie bundle that a downstream authorized client can use to issue requests as if from a real browser.

> **Status:** Pre-MVP. Planning artifacts only. No Rust code yet.

## What this is

- A REST API service backed by a stealth-patched Chromium pool (`chromiumoxide` + manual CDP patches).
- A **challenge pipeline** that routes each request to the correct vendor handler. v1 ships the **PerimeterX** handler; stubs exist for Cloudflare, Turnstile, hCaptcha/reCAPTCHA, and DataDome so they slot in cleanly in v2.
- An API-key + per-domain allowlist + audit log layer that bounds dual-use abuse.

## What this is **not**

- **Not** a native PerimeterX sensor generator. v1 defeats PX by **avoidance** (real Chromium passes the challenge legitimately), not by **defeat** (rebuilding the sensor payload from scratch). The codebase is structured so a native solver can be added; the reverse-engineering work itself is out of v1 scope. See [ADR-0010](docs/adr/0010-defer-native-sensor-generator.md) and [ADR-0013](docs/adr/0013-re-methodology-and-scope.md).
- **Not** an anonymous public solver. Every request requires an API key; every target requires an explicit allowlist entry with `tos_reviewed: true`. See [ADR-0007](docs/adr/0007-api-key-and-domain-allowlist-guardrails.md) and [`docs/011-sow-dual-use.md`](docs/011-sow-dual-use.md).
- **Not** a Cloudflare / Turnstile / DataDome / captcha solver in v1. The pipeline routes to those handlers when their protections are detected, but their `solve()` methods return `NotImplemented` until v2. Many passive edge layers (Fastly, CloudFront, Cloudflare silent scoring, Turnstile invisible mode) are **incidentally** defeated by running real Chromium with stealth — but no success rate is committed for them. See [ADR-0014](docs/adr/0014-challenge-pipeline-architecture.md) and [ADR-0015](docs/adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md).

## Documentation

| Doc | Purpose |
|---|---|
| [`docs/000-sow-index.md`](docs/000-sow-index.md) | Statement of Work index + deliverable traceability |
| [`docs/adr/README.md`](docs/adr/README.md) | Architecture Decision Records (15 ADRs as of 2026-05-16) |
| [`docs/phase/README.md`](docs/phase/README.md) | Phase plan (00–04 critical path + R research) |
| [`docs/standards/axum-best-practice.md`](docs/standards/axum-best-practice.md) | Coding standard (Clean Architecture, ≤200 LOC/file, no `unwrap`) |
| [`docs/standards/design-patterns.md`](docs/standards/design-patterns.md) | Canonical inventory of patterns + Rust idioms + anti-patterns |
| [`px-research/README.md`](px-research/README.md) | Research-track home (RE captures, deobfuscation notes) |
| [`xtask/`](xtask/) | Dev automation: `cargo xtask {bump,check-loc,release,canary,phase}` ([ADR-0016](docs/adr/0016-xtask-for-dev-automation.md)) |
| [`rust-toolchain.toml`](rust-toolchain.toml) | Pins Rust 1.95 with `rustfmt` + `clippy` |
| [`docs/adr/0017-phase-aligned-versioning.md`](docs/adr/0017-phase-aligned-versioning.md) | Versioning policy: phase 00–03 → minor bumps, phase 04 → 1.0.0 |

## License

To be selected before the first public push. Proposed: **AGPL-3.0-or-later**, to discourage closed-source resale as an anonymous SaaS.
