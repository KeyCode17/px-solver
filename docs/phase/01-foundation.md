# Phase 01 — Foundation

- **Window:** 2026-05-25 → 2026-06-05
- **Status:** Not started
- **SOW-DEL covered:** SOW-DEL-002, SOW-DEL-004
- **Owner:** KeyCode17

## Goal

Build the parts that have **no Chromium dependency**: the domain model + PX detector + cookie cache. Everything in this phase is unit-testable in milliseconds and forms the seam every later crate depends on.

## Entry criteria

- [ ] Phase 00 marked **Done**.
- [ ] `px-core`, `px-detector`, `px-cache`, `px-errors`, `px-types` crates exist as stubs from Phase 00.

## Deliverable status

- [ ] SOW-DEL-002 — PX detection & domain model (`px-core`, `px-detector`)
  - [ ] `px-core::domain` defines `PxAppId(String)`, `PxCookieBundle`, `PxChallenge`, `SolveRequest`, `SolveOutcome`, `CacheKey { domain, app_id, fingerprint_key }`.
  - [ ] `px-detector::application::detect_px` returns `PxDetection { app_id, init_js_path, collector_path, mode }`.
  - [ ] Fixtures (already captured + sanitized as of 2026-05-16):
    - `px-detector/tests/fixtures/pedidosya-block.html` (HTTP 403, no captcha, appId `eT15wiaE`).
    - `px-detector/tests/fixtures/hannaandersson-block.html` (HTTP 307, captcha mode, appId `feReLYy8`, full `_px*` global suite).
    - `px-detector/tests/fixtures/non-px-control.html` (synthetic negative).
    - `px-detector/tests/fixtures/havenwellwithin-non-px.html` (real-world Cloudflare + Salesforce Commerce Cloud negative — must return `Detected::No`).
  - [ ] `_pxhd` parser per [ADR-0012](../adr/0012-pxhd-parser-and-synth.md): accept `^[0-9a-f]{64}:<UUIDv1>$`, reject malformed.
  - [ ] 100% branch coverage on the detector (verified with `cargo tarpaulin --packages px-detector`).
- [ ] SOW-DEL-004 — Cookie cache layer (`px-cache`)
  - [ ] `px-cache::domain::CookieCache` trait with `get`, `put`, `invalidate`, `metrics_snapshot`.
  - [ ] `InMemoryCookieCache` (DashMap + tokio GC task at 60 s).
  - [ ] `RedisCookieCache` behind `--features redis` (compile-only smoke test for v1; full integration deferred).
  - [ ] Loom-style concurrency test for the in-memory backend (8 parallel writers, no panics, contract preserved).

## Exit criteria

1. `cargo test -p px-core -p px-detector -p px-cache` passes on all three OS targets in CI (linux-x86_64 only for v1).
2. Detector returns the correct `PxDetection` on the pedidosya fixture without any network IO in tests.
3. Cache loom test runs clean.
4. No public API on `px-core::domain` carries `serde` derives (axum-best-practice rule #4).
5. Run `cargo xtask phase 01` → version becomes `0.2.0` per [ADR-0017](../adr/0017-phase-aligned-versioning.md).

## Risks

- **R-01-1:** detector becomes a leaky abstraction (carries Chromium-specific assumptions). Mitigation: detector input is an `&str` of HTML; no `chromiumoxide` types cross the crate boundary.
- **R-01-2:** loom tests are slow and flaky. Mitigation: cap iteration count, run loom only in `--release` CI lane.

## Log

_Append-only._
