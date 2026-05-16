# 003 — Deliverables & Milestones

All deliverables target a 4-week part-time MVP cadence starting **2026-05-19**. Dates are nominal; this is a self-paced effort.

## SOW-DEL-001: Cargo workspace scaffold

- **Description:** Multi-crate workspace conforming to the axum-best-practice layout (max 200 LOC/file, no inline comments, Clean Architecture per crate). 17 `px-*` crates plus `xtask/` for dev automation per [ADR-0016](adr/0016-xtask-for-dev-automation.md) = 18 workspace members. Includes Lefthook setup ([`lefthook.yml`](../lefthook.yml)) and the `scripts/check_loc.sh` / `scripts/check_forbidden.sh` / `scripts/check_commit_msg.sh` helpers per [ADR-0011](adr/0011-use-lefthook-for-git-hooks.md). `px-research/` exists at root as a plain directory (not a workspace member) per [ADR-0013](adr/0013-re-methodology-and-scope.md).
- **Depends on:** None
- **Target date:** 2026-05-23
- **Acceptance criteria:** `cargo check --workspace` passes; `cargo clippy --workspace -- -D warnings` clean; `rust-toolchain.toml` pins channel `1.95` with `rustfmt` + `clippy`; CI workflow runs fmt + clippy + test on push and matches the pre-push hook command set; `lefthook install` succeeds on a fresh clone and a deliberate violation (e.g. a 201-LOC file or a `feat` commit without colon) is rejected by the hook; `cargo xtask --help` lists `bump`, `check-loc`, `release`, `canary`, `phase`; `cargo xtask phase 00 --skip-gate` updates the workspace version to `0.1.0`, commits, and tags `v0.1.0` per [ADR-0017](adr/0017-phase-aligned-versioning.md).

## SOW-DEL-002: PX detection & domain model (`px-core`, `px-detector`)

- **Description:** Domain types (`PxAppId`, `PxCookieBundle`, `PxChallenge`, `SolveRequest`, `SolveOutcome`) plus a detector that, given a fetched HTML body, identifies whether PX is active and extracts the `appId`, init.js path, and collector path.
- **Depends on:** SOW-DEL-001
- **Target date:** 2026-05-27
- **Acceptance criteria:** Unit tests cover (a) the pedidosya.com.ar block-page fixture (appId `eT15wiaE`, block mode), (b) the hannaandersson.com block-page fixture (appId `feReLYy8`, captcha mode, `_pxFirstPartyEnabled=true`), (c) a non-PX fixture (must return `Detected::No`). 100% branch coverage on the detector. `_pxhd` parser conforms to [ADR-0012](adr/0012-pxhd-parser-and-synth.md).

## SOW-DEL-003a: Vendor-agnostic Chromium pool (`px-harvester`)

- **Description:** A `chromiumoxide`-driven worker pool with bounded concurrency (`tokio::sync::Semaphore`), per-worker lifecycle, and a vendor-agnostic stealth-patch suite injected via `Page.addScriptToEvaluateOnNewDocument` (navigator.webdriver=false, languages, plugins, `chrome.runtime`, WebGL vendor/renderer, canvas noise, permissions API). Exposes `harvest_page(url) -> (Page, html)` for handler crates. **Does not** know about PerimeterX or any other vendor.
- **Depends on:** SOW-DEL-002
- **Target date:** 2026-05-30
- **Acceptance criteria:** Pool spawns/restarts N Chromium workers without leaking processes after 100 cycles; stealth patches verified via [bot.sannysoft.com](https://bot.sannysoft.com) showing green on `WebDriver`, `Plugins Length`, `Languages`, `WebGL Vendor`, `WebGL Renderer`.

## SOW-DEL-003b: PerimeterX challenge handler (`px-perimeterx`)

- **Description:** Implements the `ChallengeHandler` trait from `px-pipeline`. `detects()` inspects HTML for `_pxAppId` / `/<appId>/init.js`. `solve()` drives `px-harvester` through the PX flow and extracts `_px3`, `_pxhd`, `_pxvid`, `_pxde`, plus the active User-Agent. Uses `_pxhd` parser from [ADR-0012](adr/0012-pxhd-parser-and-synth.md).
- **Depends on:** SOW-DEL-003a, SOW-DEL-013
- **Target date:** 2026-06-03
- **Acceptance criteria:** Per [ADR-0018](adr/0018-canary-scope-broaden-to-px-direct-targets.md), against any PX-direct allowlisted target (default `havenwellwithin.com`) from the operator's connection, ≥ 95% solve success across 50 consecutive solves; median solve latency ≤ 6 s. Pedidosya specifically is CF-fronted and is deferred to the R5 track per [ADR-0019](adr/0019-r5-track-deep-stealth-and-synthetic-profiles.md).

## SOW-DEL-004: Cookie cache layer (`px-cache`)

- **Description:** TTL cache keyed by `(domain, appId, fingerprint key)`. Default in-memory backend (DashMap with periodic GC); pluggable trait for a Redis backend. Records hit/miss/expiry metrics.
- **Depends on:** SOW-DEL-002
- **Target date:** 2026-06-05
- **Acceptance criteria:** Unit tests cover insert / hit / miss / TTL expiry / GC. Loom-style concurrency test for the DashMap backend (no contract violations under 8 parallel writers).

## SOW-DEL-005: REST API server (`px-server`)

- **Description:** Axum server exposing `POST /v1/solve`, `GET /v1/health`, `GET /v1/metrics` (Prometheus). Wires `px-detector`, `px-harvester`, `px-cache` via `Arc<dyn Trait>` dependency injection per the axum-best-practice rules. Handlers return `Result<_, AppError>`.
- **Depends on:** SOW-DEL-003, SOW-DEL-004
- **Target date:** 2026-06-10
- **Acceptance criteria:** OpenAPI document published at `/v1/openapi.json`; contract test asserts schema parity with the OpenAPI doc; integration test issues a real solve against the canary and uses the returned cookies to fetch `/v2/niles/partners/137390/menus?occasion=DELIVERY` and receive HTTP 200 with JSON body.

## SOW-DEL-006: Auth, allowlist, and audit log

- **Description:** API-key middleware (constant-time compare against hashed keys stored in config) plus a per-domain allowlist YAML file; each solve attempt is recorded (caller key id, target domain, outcome, latency, cache hit). Audit sink trait with file-backed default and pluggable stdout/JSON-line backend.
- **Depends on:** SOW-DEL-005
- **Target date:** 2026-06-12
- **Acceptance criteria:** Unit tests cover (a) unknown key → 401, (b) known key for non-allowlisted domain → 403, (c) known key + allowlisted domain → 200. Audit log fixture asserts redaction of cookie values.

## SOW-DEL-007: CLI (`px-cli`)

- **Description:** `clap`-based binary with subcommands `solve <url>`, `serve`, `detect <url>`, `keys generate`, `allowlist add|remove|list`.
- **Depends on:** SOW-DEL-006
- **Target date:** 2026-06-14
- **Acceptance criteria:** `cargo run -- solve https://www.pedidosya.com.ar/` returns a `PxCookieBundle` on stdout in JSON and exits 0; non-zero exit on detection failure.

## SOW-DEL-008: Canary integration test

- **Description:** `tests/pedidosya.rs` end-to-end test that spins up the server in-process, solves the canary, and validates that the returned cookies authenticate a second-stage `/home-page/v44/home/lazy_load` call. Gated behind a `CI_CANARY=1` env flag so it does not hammer the target in unit CI.
- **Depends on:** SOW-DEL-007
- **Target date:** 2026-06-15
- **Acceptance criteria:** Test passes 9 of 10 consecutive runs locally with `CI_CANARY=1` from a residential IP; failures emit a categorized diagnostic (network / PX block / cookie-shape mismatch).

## SOW-DEL-009: Native sensor generator stub (`px-native`)

- **Description:** Crate that defines the `NativeSolver` trait, error types, and a `not_implemented` default implementation. Includes a `tests/` folder with vectors captured from the harvester (sensor payloads + resulting `_px3`) so a future maintainer can drive TDD against them without re-collecting data.
- **Depends on:** SOW-DEL-003
- **Target date:** 2026-06-16
- **Acceptance criteria:** Public API documented in `docs/specs/px-native.md`; at least 5 captured `(sensor_payload, response_cookie)` vectors stored under `px-native/tests/fixtures/` with sanitization (no IP, no UA leak).

## SOW-DEL-013: Challenge pipeline (`px-pipeline`)

- **Description:** Defines the `ChallengeHandler` trait, the `HandlerOutcome` / `CookieJarDelta` / `NamedToken` shared types, and the pipeline orchestrator that walks handlers in `config/pipeline.yaml` order. Emits per-handler metrics. Implements [ADR-0014](adr/0014-challenge-pipeline-architecture.md).
- **Depends on:** SOW-DEL-001
- **Target date:** 2026-05-28
- **Acceptance criteria:** Unit tests cover (a) ordered execution stops at first matching handler when `stop_on_match: true`, (b) `solve()` on a handler whose `detects()` returned false produces `AppError::Conflict`, (c) metrics for skipped handlers record a `skipped` outcome, (d) pipeline YAML schema is validated at startup with a clear error for unknown handler names.

## SOW-DEL-014: Cloudflare handler stub (`px-cloudflare`)

- **Description:** Crate exists, listed in workspace. `detects()` recognizes Cloudflare challenge markers (`cdn-cgi/challenge-platform`, `cf-mitigated`, `cf_clearance` Set-Cookie). `solve()` returns `AppError::NotImplemented("cloudflare handler not enabled in v1")`. Feature-gated **off** by default. Per [ADR-0015](adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md).
- **Depends on:** SOW-DEL-013
- **Target date:** 2026-05-28
- **Acceptance criteria:** `px-cloudflare/tests/fixtures/` contains at least one sanitized CF Managed Challenge HTML capture; `detects()` returns `true` on the fixture and `false` on `havenwellwithin-non-px.html` (CF-fronted but no challenge fired).

## SOW-DEL-015: Turnstile handler stub (`px-turnstile`)

- **Description:** Crate exists. `detects()` recognizes the Turnstile widget (`https://challenges.cloudflare.com/turnstile/v0/api.js`, `cf-turnstile` div). `solve()` returns `AppError::NotImplemented`. Feature-gated off.
- **Depends on:** SOW-DEL-013
- **Target date:** 2026-05-28
- **Acceptance criteria:** Crate compiles; fixture present; trait impl matches `px-pipeline`'s signature; `cargo doc` renders without warnings.

## SOW-DEL-016: Captcha handler stub (`px-captcha`)

- **Description:** Crate exists. `detects()` recognizes hCaptcha (`h-captcha` class, `hcaptcha.com/1/api.js`) and reCAPTCHA (`g-recaptcha`, `recaptcha/api.js`). `solve()` returns `AppError::NotImplemented`. Feature-gated off. Forward-compatible with paid solver APIs (CapSolver / 2Captcha) as a v2 implementation choice.
- **Depends on:** SOW-DEL-013
- **Target date:** 2026-05-28
- **Acceptance criteria:** Crate compiles; fixture present for both hCaptcha and reCAPTCHA shapes; trait impl matches.

## SOW-DEL-017: DataDome handler stub (`px-datadome`)

- **Description:** Crate exists. `detects()` recognizes DataDome markers (`window.DD_OPTIONS`, `datadome` Set-Cookie, `datadome.co/captcha` script). `solve()` returns `AppError::NotImplemented`. Feature-gated off.
- **Depends on:** SOW-DEL-013
- **Target date:** 2026-05-28
- **Acceptance criteria:** Crate compiles; one sanitized DataDome block-page fixture present; trait impl matches.

## SOW-DEL-010: Operator documentation

- **Description:** `README.md`, `docs/deployment.md`, `docs/threat-model.md`, `docs/dual-use-policy.md`. Threat model identifies misuse vectors (credential stuffing, ToS-violating scraping, mass scraping) and the mitigations (API key, allowlist, rate limit, audit log).
- **Depends on:** SOW-DEL-006
- **Target date:** 2026-06-17
- **Acceptance criteria:** A second engineer can deploy the service from `docs/deployment.md` on a fresh Linux host without external help; threat-model document reviewed and signed off by owner.
