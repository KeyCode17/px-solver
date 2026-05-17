# Architecture Decision Records

Format: [MADR](https://adr.github.io/madr/) lite. One file per decision, never edit history — supersede via a new ADR.

## Filename convention

`NNNN-kebab-case-title.md` where `NNNN` is a 4-digit zero-padded monotonic counter.

## Status lifecycle

`Proposed` → `Accepted` → `Deprecated` | `Superseded by ADR-NNNN`

## Index

| ID | Title | Status | Date |
|----|-------|--------|------|
| [0001](0001-record-architecture-decisions-in-adrs.md) | Record architecture decisions in ADRs | Accepted | 2026-05-16 |
| [0002](0002-use-rust-2024-for-px-solver.md) | Use Rust (edition 2024) for px-solver | Accepted | 2026-05-16 |
| [0003](0003-hybrid-solver-architecture.md) | Hybrid solver: harvester + cache + native fallback | Accepted | 2026-05-16 |
| [0004](0004-chromiumoxide-as-browser-driver.md) | chromiumoxide as browser driver | Accepted | 2026-05-16 |
| [0005](0005-manual-cdp-stealth-patches.md) | Manual CDP stealth patches over pre-patched browser | Accepted | 2026-05-16 |
| [0006](0006-axum-clean-architecture-layout.md) | Axum + Clean Architecture per axum-best-practice | Accepted | 2026-05-16 |
| [0007](0007-api-key-and-domain-allowlist-guardrails.md) | API-key auth + per-domain allowlist + audit log | Accepted | 2026-05-16 |
| [0008](0008-dashmap-default-redis-optional-cache.md) | DashMap default, Redis optional cookie cache | Accepted | 2026-05-16 |
| [0009](0009-runtime-px-appid-detection.md) | Generic runtime PX appId detection | Accepted | 2026-05-16 |
| [0010](0010-defer-native-sensor-generator.md) | Defer native sensor generator; ship stub crate | Accepted | 2026-05-16 |
| [0011](0011-use-lefthook-for-git-hooks.md) | Use Lefthook for git hooks (pre-commit, pre-push, commit-msg) | Accepted | 2026-05-16 |
| [0012](0012-pxhd-parser-and-synth.md) | `_pxhd` cookie parser + partial UUIDv1 synthesis | Accepted | 2026-05-16 |
| [0013](0013-re-methodology-and-scope.md) | Reverse-engineering methodology and scope (R0–R4, off critical path) | Accepted | 2026-05-16 |
| [0014](0014-challenge-pipeline-architecture.md) | Challenge pipeline architecture (pluggable handlers) | Accepted | 2026-05-16 |
| [0015](0015-v1-ships-pipeline-with-perimeterx-handler-only.md) | v1 ships pipeline scaffold + PerimeterX handler; CF/Turnstile/DataDome/Captcha stubbed | Accepted | 2026-05-16 |
| [0016](0016-xtask-for-dev-automation.md) | Use `xtask` for dev automation (bump, check-loc, release, canary) | Accepted | 2026-05-16 |
| [0017](0017-phase-aligned-versioning.md) | Phase-aligned semver: minor per phase 00–03, major at Phase 04 → 1.0.0 | Accepted | 2026-05-16 |
| [0018](0018-canary-scope-broaden-to-px-direct-targets.md) | Canary scope broadens to any PX-direct allowlisted target for v1.0.0; pedidosya → R5 | Accepted | 2026-05-16 |
| [0019](0019-r5-track-deep-stealth-and-synthetic-profiles.md) | Open R5 research track: stealth flags, stealth bundle v2, mouse paths, px-profiles, binary patcher, px-cloudflare upgrade | Accepted | 2026-05-16 |
| [0020](0020-adopt-camoufox-via-fantoccini-geckodriver.md) | Adopt Camoufox (patched-Firefox) via fantoccini+geckodriver; subsumes R5.1/R5.2/R5.5; operator-installed binaries | Accepted | 2026-05-16 |
| [0021](0021-domain-based-handler-routing.md) | Domain-based handler routing in the solve dispatcher (PX_CAMOUFOX_DOMAINS env CSV → CloudflareHandler) | Accepted | 2026-05-17 |
| [0022](0022-readmit-pedidosya-to-canary-with-deep-stealth-budget.md) | Re-admit pedidosya to canary with relaxed AC-2 budget (15s median / 20s p95) for CF-fronted targets; amends ADR-0018 | Accepted | 2026-05-17 |
| [0023](0023-allowlist-handler-field-supersedes-env-csv.md) | `handler:` field in allowlist.yaml supersedes `PX_CAMOUFOX_DOMAINS` env CSV; env retained as deprecated fallback through v1.x | Accepted | 2026-05-17 |

## Template

See [`_template.md`](_template.md).
