# 0001. Record architecture decisions in ADRs

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** SOW-DEL-010

## Context

`px-solver` is a multi-crate Rust workspace built by a single maintainer. Decisions are easy to lose between commits and chat history. The project is also dual-use, so the reasoning behind security and authorization choices must be auditable by a future reviewer (operator-side or contributor).

## Decision

Record every non-trivial architectural decision as an MADR-lite file in `docs/adr/`. Each ADR is immutable; revisions happen by writing a new ADR that supersedes the old one and flipping the old one's status to `Superseded by ADR-NNNN`.

## Alternatives considered

- **Inline comments in code.** Rejected — invisible at planning time, drifts from intent, and violates the axum-best-practice "no inline comments" rule.
- **Single `DECISIONS.md` file.** Rejected — single file conflates history, becomes a merge hot spot, and resists per-decision review.
- **GitHub Discussions / issues only.** Rejected — not versioned with the code; lost on platform migration.

## Consequences

- **Positive:** decisions become reviewable artifacts; new ADRs require explicit context and consequences; supersession is explicit.
- **Negative:** small write overhead per decision; index needs to be kept current.
- **Follow-ups:** ADR-0002 through ADR-0010 capture the decisions made up to this date.
