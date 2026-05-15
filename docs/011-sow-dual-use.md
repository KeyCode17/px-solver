# 011 — Dual-Use & Authorization Policy

PerimeterX bypass capability is dual-use. To bound misuse, this project enforces, at the code level:

- A per-domain allowlist that the operator must edit by hand to add new targets.
- An API key requirement; no anonymous solves.
- An audit log of every solve attempt.
- A refusal at startup to enable solving for any domain whose ToS the operator has not opted into via an explicit `allowlist.yaml` entry with a `tos_reviewed: true` flag and a free-text justification.

The operator commits to:

- Not using the solver for credential abuse, account takeover, or unauthorized access to private content.
- Not exposing the solver as a public/anonymous SaaS.
- Honoring `robots.txt` and rate-limiting downstream consumers built on top of the solver.

This section is binding for the lifetime of this SOW and may not be loosened without an explicit, dated addendum.
