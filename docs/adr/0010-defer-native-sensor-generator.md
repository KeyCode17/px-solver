# 0010. Defer native sensor generator; ship stub crate

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0003, SOW-DEL-009

## Context

A native sensor-payload generator would let the solver skip Chromium entirely: parse the obfuscated `/init.js`, generate the sensor payload, POST `/xhr/b/s`, get `_px3`. This is the fast path used by commercial PX solvers and would drop solve latency from 3–8 s to under 500 ms.

It is also the most expensive deliverable in the project: deobfuscating modern PX init JS (~2–3 MB minified, with VM-style control flow), replicating the HMAC signing of the sensor payload, and matching the dozens of environment fingerprints PX checks. PX rotates this code frequently, so any implementation accumulates maintenance cost.

## Decision

Defer the native generator. For v1, ship `px-native` as a **stub** that:

- Defines a `NativeSolver` trait (`async fn solve(&self, ctx: &SolveContext) -> Result<PxCookieBundle, NativeError>`).
- Provides a `NotImplemented` default implementation returning `NativeError::Unsupported`.
- Carries a `tests/fixtures/` directory of `(sensor_payload, response_cookie)` vectors captured by the harvester so that a future maintainer can do TDD without re-collecting data.

The composition root never selects `NativeSolver` for v1; the harvester is the only Tier 2.

## Alternatives considered

- **Build it now.** Rejected — multiplies the project timeline by 3–5×, and most of the value is delivered by the harvester alone.
- **Omit the stub entirely.** Rejected — leaves no architectural seam, forcing a rewrite when native lands.
- **Buy a third-party solver API.** Out of scope per SOW §10; reconsider as a future ADR if priorities change.

## Consequences

- **Positive:** ship v1 on schedule; future maintainer has a clean TDD target; no premature complexity in the harvester crate.
- **Negative:** v1 cannot meet sub-500-ms cold solve latency; high-volume use is gated on this work.
- **Follow-ups:** when revisiting, open ADR-NNNN to record the decision to activate native and the specific PX init.js fingerprint targeted.
