# xtask

Dev automation for px-solver, run as `cargo xtask <cmd>`.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver). `publish = false` — this is a
build-tool member, never a dependency.

## Commands

| Command | What it does |
|---|---|
| `cargo xtask bump <major\|minor\|patch>` | Rewrites `[workspace.package] version`, re-pins the internal `pxsolver-*` deps, runs the gate, commits and tags |
| `cargo xtask phase <NN>` | Phase-aligned bump per [ADR-0017](../docs/adr/0017-phase-aligned-versioning.md) — minor for 00–03, major at 04, no-op for `R*` |
| `cargo xtask check-loc [--max N]` | The 200-LOC-per-file rule |
| `cargo xtask release [--remote]` | Pushes `HEAD` + tag, creates the GitHub release |
| `cargo xtask canary` | Live canary solve against an allowlisted target (gated on `CI_CANARY=1`) |
| `cargo xtask soak` | Throughput soak against a running server |

## Why `bump` re-pins

Every crate carries `version.workspace = true`, so all 16 published crates move in lockstep. The
`[workspace.dependencies]` entries pin those same crates by version for `cargo publish`, and if
they are left behind, a **major** bump stops resolving: `1.9.0` still satisfies `^1.4.0`, `2.0.0`
does not. `bump` rewrites both, so the drift cannot recur.

## Release flow

`bump` and `release` only push the tag. `.github/workflows/release.yml` takes it from there: a
GitHub release is created on any `v*.*.*` tag, while the crates.io publish is gated on the tagged
commit being on `main` **and** the bump being minor/major/initial. Patch tags never publish.

Note that `release.yml` rewrites the GitHub release body from the commit log, so hand-written
release notes must be applied **after** the workflow finishes.

## License

AGPL-3.0-or-later.
