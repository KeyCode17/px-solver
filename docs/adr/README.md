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

## Template

See [`_template.md`](_template.md).
