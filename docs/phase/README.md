# Phase Plan — `px-solver`

Phases break the SOW into time-ordered, gate-checked slices. Each phase has explicit **entry criteria**, **exit criteria**, **deliverables (SOW-DEL-XXX)**, and **risks**. A phase is "done" only when its exit criteria pass.

## Timeline

```
Week 0  Week 1  Week 2  Week 3  Week 4
 ┌──┐
 │00│  Bootstrap
 └──┘
        ┌──────┐
        │  01  │  Foundation
        └──────┘
                ┌────────┐
                │   02   │  Harvester
                └────────┘
                          ┌────────┐
                          │   03   │  Server + Auth + Native stub
                          └────────┘
                                    ┌──────┐
                                    │  04  │  CLI + Canary + Docs
                                    └──────┘
                                                ▲
                                                │ MVP gate (008-sow-acceptance MVP-AC-1..7)
```

Anchored to start date **2026-05-19**; cadence is part-time, dates nominal (see [`docs/009-sow-change-control.md`](../009-sow-change-control.md)).

## Index

| # | Phase | Window | SOW-DEL covered | File |
|---|-------|--------|-----------------|------|
| 00 | Bootstrap | 2026-05-19 → 2026-05-23 | SOW-DEL-001 | [`00-bootstrap.md`](00-bootstrap.md) |
| 01 | Foundation | 2026-05-25 → 2026-06-05 | SOW-DEL-002, SOW-DEL-004 | [`01-foundation.md`](01-foundation.md) |
| 02 | Harvester | 2026-05-27 → 2026-06-03 | SOW-DEL-003 | [`02-harvester.md`](02-harvester.md) |
| 03 | Server + Auth + Native stub | 2026-06-04 → 2026-06-12 | SOW-DEL-005, SOW-DEL-006, SOW-DEL-009 | [`03-server-auth-native.md`](03-server-auth-native.md) |
| 04 | CLI + Canary + Docs | 2026-06-13 → 2026-06-17 | SOW-DEL-007, SOW-DEL-008, SOW-DEL-010 | [`04-cli-canary-docs.md`](04-cli-canary-docs.md) |

## Parallel research track

Alongside the critical-path phases above runs **Phase R — Research**, a time-unbounded reverse-engineering track. It has its own sub-phases (R0–R4), its own home in [`px-research/`](../../px-research/) at the repo root (not a Cargo crate), and is **explicitly not on the v1 critical path** per [ADR-0013](../adr/0013-re-methodology-and-scope.md). It may begin as early as Phase 00 and never has a hard end date.

| # | Phase | Window | Charter | File |
|---|-------|--------|---------|------|
| R | Research | continuous, parallel | [ADR-0013](../adr/0013-re-methodology-and-scope.md) | [`R-research.md`](R-research.md) |

## Phase status legend

- **Not started** — entry criteria not yet evaluated.
- **In progress** — entry criteria met, deliverables open.
- **Exit pending** — all deliverables merged, exit criteria evaluation in flight.
- **Done** — exit criteria signed off by the owner.
- **Blocked** — explicit `BLOCKED:` note in the phase file, with the blocker and the owner.

Status updates land in the phase file's `## Log` section, dated, append-only.

## How to advance a phase

1. Confirm **entry criteria** are met. Add a dated entry to `## Log`.
2. Open the SOW deliverables in the phase as separate branches / PRs.
3. As each deliverable's acceptance criteria pass, check it off in the phase's `## Deliverable status` section.
4. When all deliverables are checked, evaluate **exit criteria**.
5. Owner signs off in `## Log` with the date and a short note.
6. Next phase moves from **Not started** to **In progress**.
