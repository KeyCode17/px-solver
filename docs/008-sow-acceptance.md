# 008 — Acceptance Criteria

Each deliverable's acceptance criteria are stated under [`003-sow-deliverables.md`](003-sow-deliverables.md). Project-level acceptance — i.e. "the MVP is done" — requires all of the following to hold simultaneously over a single 24-hour window:

- **MVP-AC-1:** ≥ 95% solve success across 100 consecutive solves against the active canary target (`SOW-DEL-008`). Per [ADR-0018](adr/0018-canary-scope-broaden-to-px-direct-targets.md), the canary is "any PX-direct allowlisted target with `tos_reviewed: true`"; the default in `cargo xtask soak` is `havenwellwithin.com`. Pedidosya's CF-fronted variant is deferred to R5.
- **MVP-AC-2:** Median end-to-end `/v1/solve` latency ≤ 6 000 ms with the cache cold.
- **MVP-AC-3:** Cache hit ratio ≥ 70% across the same window at 1 RPS sustained.
- **MVP-AC-4:** Zero Chromium zombie processes after the 24-hour soak.
- **MVP-AC-5:** No clippy warnings, no `cargo audit` advisories at severity ≥ medium.
- **MVP-AC-6:** Audit log contains exactly one entry per solve attempt with no cookie payload present (verified by `grep -E 'px3|pxhd' audit.log` returning zero matches).
- **MVP-AC-7:** Threat model and dual-use policy documents signed off by the owner.

## How to produce evidence

MVP-AC-1..4 are collected by a single soak run (operator-side, runs from a
clean residential IP):

```bash
# Terminal 1: start the server
PX_KEYS=config/keys.yaml PX_ALLOWLIST=config/allowlist.yaml \
  cargo run -p px-server

# Terminal 2: run the soak (writes evidence to docs/verification/<date>-soak.md)
PX_SOAK_KEY=<key-id>:<secret> cargo xtask soak --duration 24h --rps 1
```

`scripts/soak.sh` (invoked by the xtask subcommand) records per-request
timings, samples `/v1/metrics` every minute, runs a `pgrep` zombie check
on exit, and emits a markdown evidence file with PASS/FAIL verdicts for
AC-1 through AC-4.

MVP-AC-5 + AC-6 are code-side; see
[`docs/verification/2026-05-16-mvp-ac-codeside.md`](verification/2026-05-16-mvp-ac-codeside.md).

MVP-AC-7 is owner-side; see
[`docs/verification/owner-signoff-template.md`](verification/owner-signoff-template.md).

After all seven items hold, run `cargo xtask phase 04` per
[ADR-0017](adr/0017-phase-aligned-versioning.md).
