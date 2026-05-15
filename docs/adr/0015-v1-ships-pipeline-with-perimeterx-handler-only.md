# 0015. v1 ships the pipeline scaffold with the PerimeterX handler only

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0014, ADR-0010, SOW §2.2, [`docs/008-sow-acceptance.md`](../008-sow-acceptance.md)

## Context

[ADR-0014](0014-challenge-pipeline-architecture.md) commits to a pluggable pipeline of `ChallengeHandler` implementations. The pipeline architecture by itself does nothing — it only routes to handlers. Shipping handlers for *every* common protection stack (Cloudflare, Turnstile, PerimeterX, DataDome, hCaptcha/reCAPTCHA) inside the v1 MVP window violates the SOW timeline (Phase 02–04 in [`docs/phase/README.md`](../phase/README.md)) and risks delivering nothing.

At the same time, leaving the non-PX crates out entirely creates merge friction later, because handler implementations need shared types (`HandlerOutcome`, `CookieJarDelta`, `NamedToken`) and shared fixtures (block-page HTML, network captures) that should land alongside the trait.

## Decision

v1 ships:

1. **`px-pipeline`** — full implementation. Trait, orchestrator, ordering, metrics, error mapping.
2. **`px-harvester`** — full implementation. Vendor-agnostic Chromium pool + stealth core (used by every handler).
3. **`px-perimeterx`** — full implementation. The marquee handler. Detector + cookie extraction + harvester glue.
4. **`px-cloudflare`, `px-turnstile`, `px-captcha`, `px-datadome`** — **scaffolded stubs**:
   - `Cargo.toml` exists, listed in workspace.
   - `domain/mod.rs` defines the public types used by future implementations.
   - `application/<handler>.rs` defines the `ChallengeHandler` impl, but `solve()` returns `AppError::NotImplemented("<vendor> handler not enabled in v1")`.
   - `tests/fixtures/` contains at least one captured block-page HTML per vendor so a future implementer can do TDD on `detects()` without re-collecting evidence.
   - Each stub is feature-gated **off by default** (`features = ["cloudflare", "turnstile", "captcha", "datadome"]` opt-in flags on `px-server`). The default build does not link them.

## Alternatives considered

- **Ship all five handlers fully implemented in v1.** Rejected — at least doubles the timeline; no operator value before PX itself works.
- **Ship only the PX handler crate, no stubs.** Rejected — every later handler PR must redefine shared types and re-do scaffolding work; merge friction high.
- **Ship feature flags but no stub `solve()` methods.** Rejected — the trait wouldn't compile without a placeholder impl, and "not implemented" is itself useful documentation of intent.

## Consequences

- **Positive:** v1 ships a working PX solver on the SOW timeline; v2 adds CF/Turnstile/DataDome/captcha implementations by *replacing* a `NotImplemented` body, not by inventing new types or rewiring composition; per-vendor block-page fixtures land early so the surface is documented even before the solver works.
- **Negative:** the workspace ships four "do nothing" crates. Mitigation: clearly labeled in `README.md` + each crate's `lib.rs` rustdoc says "stub — see ADR-0015".
- **Follow-ups:**
  - SOW deliverable list adds **SOW-DEL-013 (`px-pipeline`)**, **SOW-DEL-014 (`px-cloudflare` stub)**, **SOW-DEL-015 (`px-turnstile` stub)**, **SOW-DEL-016 (`px-captcha` stub)**, **SOW-DEL-017 (`px-datadome` stub)**.
  - SOW-DEL-003 splits: `px-harvester` (Chromium pool only) + `px-perimeterx` (PX-specific stealth + cookie extraction).
  - Phase 00 scaffold list grows by 5 crates.
  - Phase 03 composition root reads `config/pipeline.yaml` instead of hard-wiring the PX handler.
  - When a future ADR activates one of the stubs, that ADR amends this one's status to `Superseded by ADR-NNNN`.
