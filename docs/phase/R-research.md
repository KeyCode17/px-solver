# Phase R — Research (parallel, time-unbounded)

- **Status:** Not started
- **Track:** Parallel to Phases 00–04. **Not** on the v1 critical path.
- **Charter:** [ADR-0013](../adr/0013-re-methodology-and-scope.md)
- **Home:** [`px-research/`](../../px-research/) (repo-root directory; not a Cargo crate)
- **Owner:** KeyCode17

## Goal

Build, slowly and without delivery pressure, an understanding of PerimeterX's client-side challenge code that is deep enough to inform — and eventually replace — the harvester for high-volume use cases. Outputs feed `px-observer`, `px-mimic`, and (eventually) `px-native`.

## How this phase relates to the others

```
Phase 00 ─▶ Phase 01 ─▶ Phase 02 ─▶ Phase 03 ─▶ Phase 04 ─▶ MVP
                                                                  (critical path)

Phase R: ─────── continuous, parallel ───────▶ (no MVP impact)
```

Phase R may start as early as Phase 00 and never has a hard end date. It progresses opportunistically.

## Sub-phases

| Sub-phase | Goal | Output | Status |
|---|---|---|---|
| **R0 — Acquisition** | Fetch `/<appId>/init.js` for every allowlisted target and version-store it | `px-research/init-js/<appId>/<YYYY-MM-DD>-init.js` | Not started |
| **R1 — Surface mapping** | Catalog every browser API init.js touches | `px-research/notes/<appId>/r1-surface.md` | Not started |
| **R2 — Deobfuscation** | Recover readable JS from the minified + VM-obfuscated bundle | `px-research/notes/<appId>/r2-deobfuscated/` | Not started |
| **R3 — Sensor RE** | Understand the sensor payload schema, encryption, and HMAC | `px-research/sensor-maps/<appId>-r3.md` + `<appId>.schema.json` | Not started |
| **R4 — Bypass selection** | Pick the strategy (in-VM / native / patch / warehouse) | Next free ADR slot when ready | Not started |

## Entry criteria (per sub-phase)

| Sub-phase | Requires |
|---|---|
| R0 | At least one allowlisted PX target with `tos_reviewed: true` in `config/allowlist.yaml`. |
| R1 | R0 corpus exists for at least one target. |
| R2 | R0 corpus exists for ≥ 2 targets (so shared core can be isolated from per-tenant config). |
| R3 | R2 has produced readable JS for at least one target. |
| R4 | R3 has produced a complete sensor schema for at least one target. |

## Exit criteria

Phase R has no global exit criteria. Each sub-phase exits when its output exists and is reviewed by the owner. Sub-phases may be revisited as PX rotates its bundle (~monthly).

## Tracking

Progress lives in this file's `## Log` section, append-only, with one dated entry per material event:

```
## Log

- 2026-05-16 — Phase R defined. No captures yet.
```

## Risks

- **R-R-1:** Scope-creep. RE outputs look useful enough that R-track work starts blocking critical-path deliverables. **Mitigation:** ADR-0013 explicitly states R-track is off the critical path; any deliverable change requires an addendum to that ADR.
- **R-R-2:** Stale captures. PX rotates init.js ~monthly; old corpus loses value quickly. **Mitigation:** re-capture on a calendar cadence; never trust a > 60-day-old capture for decisions about live PX behavior.
- **R-R-3:** Legal posture drift. R-track captures expand the surface that needs ToS review. **Mitigation:** captures require the same allowlist entry as production solves (per ADR-0013).
- **R-R-4:** Tooling rot. `webcrack`/`humanify` evolve; old deobfuscation outputs may not be reproducible. **Mitigation:** pin tool versions in `px-research/notes/<appId>/r2-toolchain.md` per capture.

## Log

_Append-only._

- **2026-05-16** — Phase R defined per ADR-0013. `px-research/` scaffolded at repo root with init-js/, notes/, sensor-maps/, fingerprints/ subdirectories (no Cargo crate). No captures yet.
