# Phase 03 — Server + Auth + Native stub

- **Window:** 2026-06-04 → 2026-06-12
- **Status:** Not started
- **SOW-DEL covered:** SOW-DEL-005, SOW-DEL-006, SOW-DEL-009
- **Owner:** KeyCode17

## Goal

Expose the solver as a guarded REST API. Wire the harvester and the cache behind the `SolvePx` use case. Reserve the architectural slot for the native sensor generator without implementing it.

## Entry criteria

- [ ] Phase 01 marked **Done**.
- [ ] Phase 02 marked **Done** (the harvester contract is stable).

## Deliverable status

- [ ] SOW-DEL-005 — REST API server (`px-server`)
  - [ ] `POST /v1/solve` consumes `SolveRequest`, returns `SingleResponse<SolveResponse>`.
  - [ ] `GET /v1/health` returns `200 OK` with build SHA + uptime.
  - [ ] `GET /v1/metrics` returns Prometheus exposition.
  - [ ] `GET /v1/openapi.json` returns the OpenAPI v3.1 doc.
  - [ ] Composition root in `px-server::infrastructure::bootstrap` wires `PxDetector`, `CookieCache`, `Harvester`, `ChallengePipeline` (with the configured handler order per `config/pipeline.yaml`), `NativeSolver`, `AuditSink` as `Arc<dyn Trait>`. Per [ADR-0014](../adr/0014-challenge-pipeline-architecture.md), the pipeline walks handlers in order; v1 ships `px-perimeterx` only — other handlers' `solve()` returns `AppError::NotImplemented` per [ADR-0015](../adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md).
  - [ ] Contract test asserts OpenAPI doc matches handler signatures (via `utoipa` or hand-rolled fixture).
- [ ] SOW-DEL-006 — Auth, allowlist, audit log (`px-auth`)
  - [ ] API-key middleware: Argon2id-hashed keys in `config/keys.yaml`, constant-time compare via `subtle::ConstantTimeEq`.
  - [ ] Allowlist middleware: rejects with `403 Forbidden` when target host is not in `config/allowlist.yaml` or its entry lacks `tos_reviewed: true`.
  - [ ] Audit sink trait `AuditSink::record(&AuditEvent)`; file-backed default writes one JSON-Lines record per attempt; cookie payloads redacted.
  - [ ] Server refuses to start if `allowlist.yaml` contains an entry without `tos_reviewed: true` + justification.
- [ ] SOW-DEL-009 — Native sensor generator stub (`px-native`)
  - [ ] `NativeSolver` trait defined in `px-native::domain`.
  - [ ] `NotImplementedNativeSolver` default returning `NativeError::Unsupported`.
  - [ ] At least 5 captured `(sensor_payload, response_cookie)` vectors stored under `px-native/tests/fixtures/`, sanitized (no IP, no UA, no PII).
  - [ ] Public API documented in `docs/specs/px-native.md`.

## Exit criteria

1. **Functional canary:** a real `/v1/solve` against `pedidosya.com.ar` returns cookies that authenticate a second-stage `/v2/niles/partners/137390/menus?occasion=DELIVERY` request (HTTP 200, JSON body).
2. **Auth matrix:** unit tests cover unknown-key → 401, known-key non-allowlisted → 403, known-key allowlisted → 200.
3. **Audit redaction:** `grep -E '_px3|_pxhd' audit.log` returns zero matches in a 100-solve fixture run.
4. **OpenAPI parity:** the contract test passes; `/v1/openapi.json` is byte-stable across a clean run.
5. **No native code path active by default:** `bootstrap.rs` selects `NotImplementedNativeSolver`; toggling via config returns `409 Conflict` from `/v1/solve`.
6. Run `cargo xtask phase 03` → version becomes `0.4.0` per [ADR-0017](../adr/0017-phase-aligned-versioning.md).

## Risks

- **R-03-1:** OpenAPI generator drift vs handlers. Mitigation: contract test is a hard gate; CI fails on drift.
- **R-03-2:** Argon2id key verification under load. Mitigation: cache `verified_key_id` per `Authorization` header value in-process for `≤ 60 s` to avoid re-hashing each request.
- **R-03-3:** Allowlist YAML parse errors cause silent skip. Mitigation: server refuses to start on parse error; covered by a test.

## Log

_Append-only._
