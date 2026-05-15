# Phase 00 — Bootstrap

- **Window:** 2026-05-19 → 2026-05-23
- **Status:** Not started
- **SOW-DEL covered:** SOW-DEL-001
- **Owner:** KeyCode17

## Goal

Stand up a buildable, lintable, hookable Rust workspace. Nothing in this phase produces solver behavior — it produces the runway every later phase rolls on.

## Entry criteria

- [ ] SOW frozen at v1.0.0 (commit hash recorded).
- [ ] ADRs 0001–0011 merged.
- [ ] Lefthook config and helper scripts present (already in repo as of 2026-05-16).
- [ ] Local toolchain: stable Rust ≥ 1.84, Chromium present, `cargo install lefthook` available.

## Deliverable status

- [ ] SOW-DEL-001 — Cargo workspace scaffold
  - [ ] `Cargo.toml` workspace with the 17 `px-*` crates listed in [`docs/standards/axum-best-practice.md`](../standards/axum-best-practice.md) (the 11 original + 6 added per [ADR-0014](../adr/0014-challenge-pipeline-architecture.md): `px-pipeline`, `px-perimeterx`, `px-cloudflare`, `px-turnstile`, `px-captcha`, `px-datadome`) plus `xtask/` per [ADR-0016](../adr/0016-xtask-for-dev-automation.md). Total: 18 workspace members. `[workspace.package]` carries the project version (initialized to `0.0.0` per [ADR-0017](../adr/0017-phase-aligned-versioning.md)). `[workspace.dependencies]` declares `anyhow`, `clap` (with `derive` feature), `serde`, `serde_json`, and `walkdir` so `xtask/` compiles.
  - [ ] `rust-toolchain.toml` at repo root pins channel `1.95` with `rustfmt` and `clippy` components.
  - [ ] `cargo xtask --help` exits 0 and lists `bump`, `check-loc`, `release`, `canary`, `phase` subcommands.
  - [ ] At end of phase, run `cargo xtask phase 00` → version becomes `0.1.0` per [ADR-0017](../adr/0017-phase-aligned-versioning.md).
  - [ ] `cargo check --workspace` passes on an empty workspace.
  - [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - [ ] Workspace-level `[workspace.lints]` enables `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::dbg_macro`, `clippy::todo`, `clippy::unimplemented`.
  - [ ] `.github/workflows/ci.yml` runs fmt + clippy + test + `scripts/check_loc.sh` on push and PR.
  - [ ] `lefthook install` succeeds on a fresh clone.
  - [ ] A deliberate violation (201-LOC stub or `bad: subject` commit) is rejected by the matching hook.

## Exit criteria

1. CI green on `main` for one full run.
2. Owner has performed `lefthook install` and a deliberate-violation rejection test, both documented in `## Log`.
3. No clippy warnings, no `cargo audit` advisories at severity ≥ medium.

## Risks

- **R-00-1:** chromiumoxide locks the workspace MSRV higher than expected. Mitigation: capture actual MSRV in `Cargo.toml` and CI matrix before declaring phase done.
- **R-00-2:** workspace lints conflict with `chromiumoxide` macros generating non-trivial warnings. Mitigation: scope lints to project crates via `Cargo.toml` `lints.workspace` opt-in.

## Log

_Append-only._
