# 009 — Change Control Process

Because this is a single-maintainer self-funded project, the change-control process is lightweight:

1. Material scope changes (adding/removing a deliverable, changing a success metric, changing the allowed target categories) are recorded as an addendum at the bottom of the relevant section file with a date, the rationale, and the new acceptance criteria.
2. Smaller adjustments (renaming a crate, updating a target date by ≤ 1 week) are tracked in the project changelog (`CHANGELOG.md`) only.
3. Any change that broadens the allowlist semantics (e.g. moving from explicit allowlist to a wildcard) requires updating [`011-sow-dual-use.md`](011-sow-dual-use.md) and the `docs/dual-use-policy.md` before code lands.
4. Changes are self-approved by the owner.
