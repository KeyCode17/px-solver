# 0008. DashMap default, Redis optional cookie cache

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0003, SOW-DEL-004

## Context

The cookie cache (Tier 1 of the hybrid) needs:

- Sub-millisecond reads on the hot path.
- TTL with periodic GC (PX cookies expire in ~10 minutes).
- Safe concurrent use by many request handlers.
- Optional shared state across processes for future horizontal scaling.

## Decision

Define a `CookieCache` trait in `px-cache::domain`. Ship two implementations:

- **`InMemoryCookieCache`** (default) — backed by `dashmap::DashMap` with a `tokio::time` GC task. Zero external dependencies.
- **`RedisCookieCache`** (feature-gated, `--features redis`) — backed by `fred`. Used when more than one solver instance shares cookies.

The composition root in `px-server` picks one based on `config/server.yaml`.

## Alternatives considered

- **Single Redis-only backend.** Adds a hard dependency on Redis even for solo deployments; loses the sub-ms hot path.
- **`moka::sync::Cache`.** Good TTL and eviction story, but more deps; DashMap is sufficient at MVP scale.
- **In-process LRU only.** No TTL semantics; would need to be wrapped anyway.

## Consequences

- **Positive:** default deployment is single-binary; horizontal scaling is a config flip, not a rewrite.
- **Negative:** two backends to maintain; Redis backend needs serialization (decided: `bincode` for `PxCookieBundle`).
- **Follow-ups:** the trait must expose `get`, `put`, `invalidate(domain)`, `metrics_snapshot`. GC interval defaults to 60 s and is configurable.
