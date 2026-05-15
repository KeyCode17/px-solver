# 0016. Use `xtask` for dev automation

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0006, ADR-0011, SOW-DEL-001, [`docs/standards/axum-best-practice.md`](../standards/axum-best-practice.md)

## Context

The project already has a polyglot toolchain for developer ergonomics: shell scripts in `scripts/` (LOC check, forbidden patterns, commit-msg, sanitize), `lefthook.yml` for git hooks, and (when scaffolded) `cargo fmt` / `cargo clippy` / `cargo test`. As the workspace grows, two cross-cutting tasks need a Rust home so they can read `Cargo.toml`, walk the workspace tree, and shell out reliably:

1. **Version bumping** of the workspace `[workspace.package].version` plus the matching git commit + tag.
2. **Release automation** (push tag + commit to origin), and **canary** (run the `CI_CANARY=1` integration test).

A sister project (`rust-ai-surfer`) already uses the standard Cargo [`xtask` pattern](https://github.com/matklad/cargo-xtask) for exactly this. Adopting the same pattern keeps tooling muscle memory consistent across the maintainer's projects.

## Decision

Add an `xtask` crate at the **repository root** (alongside the `px-*` crates) implementing dev automation as `cargo xtask <subcommand>`. The crate is `publish = false`, inherits workspace metadata, and is exempt from the standard production rules (`println!`/`eprintln!` allowed; it is a dev tool, not a runtime binary).

### Subcommands (v1)

| Command | Behavior |
|---|---|
| `cargo xtask bump <major\|minor\|patch> [phase] [--skip-gate]` | Bumps the workspace version in root `Cargo.toml`. Unless `--skip-gate`, runs `cargo build --workspace --all-targets` and `cargo test --workspace --no-fail-fast` before committing. Commits with `chore: bump to X.Y.Z` (or `chore: bump to X.Y.Z (Phase <phase>)` if a phase tag is given) and creates an annotated git tag `vX.Y.Z`. |
| `cargo xtask check-loc [max=200]` | Rust-native equivalent of `scripts/check_loc.sh`. Walks the tree, skips `target/`, `tests/`, `examples/`, `xtask/`, `px-research/`, fails if any `.rs` file exceeds the limit. |
| `cargo xtask release [--remote <name>]` | Pushes `HEAD` and the current `v<version>` tag to `<remote>` (default `origin`). Read-only with respect to the working tree; does not bump. |
| `cargo xtask canary` | Runs `cargo test -p px-server --test pedidosya -- --nocapture` with `CI_CANARY=1`. The canary is gated behind this env flag in source per SOW-DEL-008 to keep unit CI from hammering the target. |

### Naming exception

This is the **second** documented exception to the `px-` crate prefix:

| Crate | Type | Reason |
|---|---|---|
| `px-research/` | Plain directory, no `Cargo.toml` | Data only — captures, notes, field maps. Not a Cargo crate; named `px-` for visual consistency. |
| `xtask/` | Cargo crate, `publish = false` | Universal Cargo convention — `cargo xtask <cmd>` is recognized across the Rust ecosystem. Renaming to `px-xtask` would require a cargo alias and lose the convention. |

### Production-rule exemptions for `xtask/`

The axum-best-practice standard normally forbids `println!`, `eprintln!`, and `unwrap()`/`expect()` in production code. `xtask/` is a dev tool, not production:

- `println!` and `eprintln!` are allowed (it is a CLI; humans read stdout/stderr).
- `unwrap()` / `expect()` are allowed with judgement — the tool runs locally and should fail loudly on operator mistakes.
- The 200-LOC-per-file rule is **waived** for `xtask/`. The crate is small enough to stay readable as a single file; splitting it into modules adds ceremony without value. If the binary grows past ~400 lines, revisit with a new ADR.

`scripts/check_forbidden.sh`, `scripts/check_loc.sh`, and `scripts/check_loc_workspace.sh` are amended to exempt `xtask/` from the `println!` / `unwrap()` / 200-LOC checks.

## Alternatives considered

- **No xtask; keep all dev automation in `scripts/*.sh`.** Works for the LOC check, but `bump` needs to parse and rewrite `Cargo.toml` reliably, which is fragile in shell. Rust gives proper parsing + the same Cargo idioms the rest of the project uses.
- **Use `cargo-release` or `cargo-smart-release` instead of a hand-rolled bump.** Mature tools but add an external install step, and our bump policy is simple enough (workspace-level only) that a 30-line function suffices.
- **Make `xtask` a `px-*` crate (e.g. `px-xtask`).** Loses the universal `cargo xtask` invocation; would need a `.cargo/config.toml` alias. Convention is more valuable than prefix purity here, and the exception is small and well-documented.
- **Put dev-automation into `px-cli`.** Conflates operator-facing CLI (`px-cli solve <url>`) with developer-facing tooling (`cargo xtask bump`); blurs the no-`println!` rule.

## Consequences

- **Positive:** consistent automation surface across maintainer's projects; cargo-native invocation (no extra installs); easy to extend with new subcommands.
- **Negative:** a duplicate of `scripts/check_loc.sh` exists in Rust form. Mitigation: lefthook calls the shell script (faster, no cargo build needed); CI may call `cargo xtask check-loc` for cargo-only environments; behavior is identical.
- **Follow-ups:**
  - **SOW-DEL-001** amended to include `xtask/` in the workspace scaffold; the workspace `Cargo.toml` must declare `anyhow`, `clap` (with `derive`), `serde`, `serde_json`, and `walkdir` in `[workspace.dependencies]`.
  - **`scripts/check_forbidden.sh`** amended to recognize `xtask/` as a `println!`-allowed path.
  - **axum-best-practice standard** updated with the second prefix exception.
  - **Phase 00** entry criteria add `cargo xtask --help` returning successfully.
