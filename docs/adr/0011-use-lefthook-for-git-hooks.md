# 0011. Use Lefthook for git hooks

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0006, SOW-DEL-001, [`docs/standards/axum-best-practice.md`](../standards/axum-best-practice.md)

## Context

The axum-best-practice standard imposes rules (max 200 LOC/file, no `unwrap()`, no `println!` outside `px-cli`, conventional commit messages) that are cheap to enforce locally and expensive to chase later. CI catches them eventually, but a failed CI round-trip wastes maintainer time and pollutes git history with `fix lint` commits.

A pre-commit / pre-push hook manager keeps the standard honest at the workstation. Requirements:

- Cross-platform (Linux primary, optional macOS).
- Single config file checked into the repo.
- Parallel command execution to keep `git commit` snappy.
- Plays nicely with a Rust workspace (cargo fmt, cargo clippy, cargo test).
- No Python or Node dependency for the hook runner itself.

## Decision

Adopt [Lefthook](https://github.com/evilmartians/lefthook) as the project's git-hook manager. Config lives at `lefthook.yml` at the repo root. Helper scripts live under `scripts/`.

Hooks:

- **pre-commit** (parallel) — `cargo fmt --check`, `cargo clippy` with `clippy::unwrap_used` / `expect_used` / `panic` / `dbg_macro` / `todo` / `unimplemented` denied, `scripts/check_loc.sh` (200 LOC), `scripts/check_forbidden.sh` (`println!` / `eprintln!` / `dbg!` outside `px-cli`, `unwrap()` / `expect()` outside test code), optional `taplo` and `yamllint` if installed.
- **commit-msg** — `scripts/check_commit_msg.sh` enforces Conventional Commits (`feat:`, `fix:`, etc.) and a 72-char subject limit.
- **pre-push** (sequential) — `cargo fmt --all -- --check` (full workspace), `cargo clippy --workspace --all-targets --all-features -- -D warnings` (and the same denial set as pre-commit), `scripts/check_loc_workspace.sh` (workspace-wide LOC sweep), `cargo test --workspace --all-features --no-fail-fast`, optional `cargo audit`, optional `cargo deny check`. Canary integration test is intentionally **not** in pre-push; it is gated behind `CI_CANARY=1` to avoid hammering the target on every push (`cargo xtask canary` runs it on demand).

## Alternatives considered

- **`pre-commit` (Python).** Mature ecosystem but adds a Python toolchain dependency to a pure-Rust project. Hook latency higher.
- **`cargo-husky`.** Rust-native, no extra dep, but tied to a `[package]` and works less well for a workspace with many crates; lifecycle (install/uninstall) is opaque.
- **Hand-rolled `.git/hooks/*.sh`.** Zero deps but not versioned with the repo, no parallelism, no per-glob filtering.
- **Husky (Node).** Adds a Node toolchain for a Rust project; not justified.

## Consequences

- **Positive:** the axum-best-practice rules are enforced before code lands; commit-message hygiene is automatic; CI is a backstop, not the first line of defense; parallel pre-commit keeps the loop fast.
- **Negative:** contributors must run `lefthook install` once after cloning. CI must independently re-run the same checks because hooks can be bypassed with `--no-verify` (forbidden by the standard but technically possible).
- **Follow-ups:**
  - `SOW-DEL-001` updated to include `lefthook install` in the bootstrap.
  - CI workflow (`.github/workflows/ci.yml`) re-runs `cargo fmt --check`, `cargo clippy`, `cargo test`, and `scripts/check_loc.sh` against the full tree.
  - Operators are warned in `docs/standards/axum-best-practice.md` that `--no-verify` is forbidden; ADR-0006 rule set unchanged.
