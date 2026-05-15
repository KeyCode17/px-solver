# 008 — Acceptance Criteria

Each deliverable's acceptance criteria are stated under [`003-sow-deliverables.md`](003-sow-deliverables.md). Project-level acceptance — i.e. "the MVP is done" — requires all of the following to hold simultaneously over a single 24-hour window:

- **MVP-AC-1:** ≥ 95% solve success against the canary across 100 consecutive solves (`SOW-DEL-008`).
- **MVP-AC-2:** Median end-to-end `/v1/solve` latency ≤ 6 000 ms with the cache cold.
- **MVP-AC-3:** Cache hit ratio ≥ 70% across the same window at 1 RPS sustained.
- **MVP-AC-4:** Zero Chromium zombie processes after the 24-hour soak.
- **MVP-AC-5:** No clippy warnings, no `cargo audit` advisories at severity ≥ medium.
- **MVP-AC-6:** Audit log contains exactly one entry per solve attempt with no cookie payload present (verified by `grep -E 'px3|pxhd' audit.log` returning zero matches).
- **MVP-AC-7:** Threat model and dual-use policy documents signed off by the owner.
