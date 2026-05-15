# 0017. Phase-aligned semantic versioning

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0011, ADR-0016, [`docs/phase/README.md`](../phase/README.md), [`docs/008-sow-acceptance.md`](../008-sow-acceptance.md)

## Context

`xtask bump` exposes `major | minor | patch` but the project needs an explicit policy for *when* each bump happens. Without policy, version numbers become noise (every commit becomes a minor) or stale (no bumps until release day). The natural cadence for `px-solver` is the **phase plan** ([`docs/phase/README.md`](../phase/README.md)): five critical-path phases (00 → 04) leading to the MVP gate, plus a parallel research track (Phase R).

## Decision

Adopt a phase-aligned semver policy, applied via `cargo xtask` invocations:

| Event | Bump | Resulting version (cumulative) |
|---|---|---|
| Repository initialized | — | `0.0.0` |
| Phase 00 — Bootstrap exit criteria pass | `minor` | `0.1.0` |
| Phase 01 — Foundation exit | `minor` | `0.2.0` |
| Phase 02 — Harvester exit | `minor` | `0.3.0` |
| Phase 03 — Server + Auth + Native stub exit | `minor` | `0.4.0` |
| **Phase 04 — CLI + Canary + Docs exit (MVP gate passes)** | **`major`** | **`1.0.0`** |
| Regular commit between phase exits | `patch` | `0.X.Y` → `0.X.(Y+1)` |
| Post-1.0.0 architectural change (e.g. native solver activates, ADR supersession) | `major` (manual) | `1.0.0` → `2.0.0` |
| Phase R sub-phase exit | no version bump | (research is off the critical path; see [ADR-0013](0013-re-methodology-and-scope.md)) |

### Mechanics

1. **Phase exits** use `cargo xtask phase <NN>` which:
   - For `NN ∈ {00, 01, 02, 03}` → calls `bump minor` with phase metadata.
   - For `NN == 04` → calls `bump major` with phase metadata (MVP gate).
   - For `NN == R*` → no-op (research bumps no version).
   - Commit message: `chore: bump to <X.Y.Z> (Phase <NN> — <name>)`.
   - Tag: annotated `v<X.Y.Z>`.
2. **Non-phase commits** that need a version bump (e.g. a hotfix on `main`) use `cargo xtask bump patch`.
3. **Post-1.0.0 major bumps** are explicit: `cargo xtask bump major` with a rationale in the commit body. No automation chooses these.

### Why this works

- The phase plan is already the project's truth source for "what's done." Tying versions to phase exits means the version number always answers "where in the plan are we?".
- The 0.x.x line means "pre-MVP"; reaching 1.0.0 means "MVP acceptance criteria from [`docs/008-sow-acceptance.md`](../008-sow-acceptance.md) all hold simultaneously."
- Patch bumps stay cheap. Most commits never call `xtask` at all; only when an actual release tag is wanted does a maintainer run `cargo xtask bump patch`.
- Phase R does not influence the production version — research outputs are off-critical-path artifacts.

## Alternatives considered

- **Calendar versioning (CalVer).** Clean dates but loses the link to project milestones.
- **Every merge = patch bump.** Pollutes git tags with hundreds of `v0.0.X` entries.
- **No version policy until 1.0.0.** Leaves the 0.x range unused; loses the "where in the plan" signal.
- **Phase exits all minor; MVP is just another minor bump.** The 1.0.0 milestone is the most meaningful boundary; using `major` for it makes the tag list immediately readable.

## Consequences

- **Positive:** version numbers communicate phase progress; tag list is short and meaningful; MVP arrival is unambiguous (`v1.0.0` exists ↔ MVP acceptance passed).
- **Negative:** phase exits and `xtask phase` invocations need to be coordinated with the actual phase-file `## Log` sign-off (operator discipline; not enforced by code).
- **Follow-ups:**
  - Each phase's exit criteria (`docs/phase/0X-*.md`) gain a line: *"Run `cargo xtask phase 0X` to record the version bump."*
  - `cargo xtask phase <NN>` is added to `xtask/src/main.rs` per this ADR.
  - Phase 04 specifically requires the MVP acceptance checklist (MVP-AC-1..7 in [`docs/008-sow-acceptance.md`](../008-sow-acceptance.md)) to pass **before** `xtask phase 04` is run; the xtask itself does not verify those criteria.
