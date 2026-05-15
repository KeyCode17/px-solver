# 0007. API-key auth + per-domain allowlist + audit log

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** SOW §11, SOW-DEL-006

## Context

A PerimeterX bypass capability is dual-use. The maintainer explicitly does not want this service to become an anonymous SaaS or be reachable by callers who have not been vetted, and explicitly does not want it to be usable against arbitrary domains.

## Decision

The server enforces three independent controls, in order:

1. **API key (mandatory).** Every request carries an `Authorization: Bearer <key>` header. Keys are stored as Argon2id hashes in `config/keys.yaml` (file mode `0600`) and verified with `subtle::ConstantTimeEq`. Unknown key → `401 Unauthorized`.
2. **Per-domain allowlist (mandatory).** The target host of each solve request is checked against `config/allowlist.yaml`. Entries must include `tos_reviewed: true` and a free-text justification; the server refuses to start if any entry is missing these fields. Non-allowlisted domain → `403 Forbidden`.
3. **Audit log (mandatory).** Every solve attempt writes a structured record (key id, target domain, outcome, latency, cache hit) to a configured sink. Cookie payloads are never logged.

## Alternatives considered

- **Localhost-only, no auth.** Simpler but cannot be exposed even to a trusted internal network without breaking the threat model.
- **Open API + log only.** Trivially abusable; the operator becomes the public-anonymous SaaS the policy forbids.
- **Allowlist via regex.** Wider attack surface — a misplaced anchor allows unintended domains. Explicit string match is auditable.

## Consequences

- **Positive:** abuse is bounded by configuration the operator must edit by hand; misuse is traceable post-hoc.
- **Negative:** adding a new target requires an explicit two-line config change; "try anything" usage is impossible.
- **Follow-ups:** key rotation and allowlist editing are CLI subcommands (SOW-DEL-007). Audit sink is a trait — future S3 / syslog adapters are pluggable.
