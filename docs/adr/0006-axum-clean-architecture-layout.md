# 0006. Axum + Clean Architecture per axum-best-practice

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0002, ADR-0014, SOW-DEL-001, SOW-DEL-005, [`docs/standards/axum-best-practice.md`](../standards/axum-best-practice.md)

> **Update 2026-05-16 ([ADR-0014](0014-challenge-pipeline-architecture.md)):** The crate list grew by 6 crates (`px-pipeline`, `px-perimeterx`, `px-cloudflare`, `px-turnstile`, `px-captcha`, `px-datadome`). `px-harvester` is now vendor-agnostic; PX-specific logic moved into `px-perimeterx`. See the standards doc and skill for the canonical list.

## Context

The HTTP surface is small (`/v1/solve`, `/v1/health`, `/v1/metrics`) but must be:

- testable in isolation (no real Chromium in unit tests),
- gated by an API key and a per-domain allowlist,
- structured so that the native solver (ADR-0010) can later be swapped in behind the same trait.

The maintainer has imposed a project-wide architectural standard, [`axum-best-practice`](../standards/axum-best-practice.md), also wired as a Claude project skill at `.claude/skills/axum-best-practice/SKILL.md`: Clean Architecture per module (`domain` / `application` / `infrastructure`), max 200 LOC per file, no inline comments, centralized `AppError`, repository pattern via `Arc<dyn Trait>`, one use case per file.

## Decision

Adopt the axum-best-practice layout verbatim. Per crate:

```
px-<crate>/src/
├── lib.rs
├── domain/         # entities + trait definitions, no external deps
├── application/    # use cases, one per file
└── infrastructure/ # http/, persistence/, external/
```

`AppError` lives in `px-errors` and implements `IntoResponse` for axum. Handlers return `Result<Json<SingleResponse<T>>, AppError>`. Dependencies are injected as `Arc<dyn Trait>` from a `px-server` composition root.

## Alternatives considered

- **Flat single-crate layout.** Faster to start, but conflates use cases and breaks the 200-LOC rule.
- **Hexagonal split with separate adapter crates.** More crates, more compile time, marginal architectural gain at this size.
- **No `AppError`, return `anyhow::Error`.** Easier ergonomics but loses HTTP status mapping and structured error responses.

## Consequences

- **Positive:** trait boundaries make the harvester and the native solver interchangeable (ADR-0003); unit tests target use cases without spawning a server; consistent error responses.
- **Negative:** more boilerplate per feature; new contributors must learn the layout.
- **Follow-ups:** new use cases land as `<crate>/src/application/<verb_noun>.rs`; new HTTP routes land as `<crate>/src/infrastructure/http/handlers/<noun>.rs`.
