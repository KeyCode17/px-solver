# 0003. Hybrid solver: harvester + cache + native fallback

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** SOW §2, SOW-DEL-003, SOW-DEL-004, SOW-DEL-009

## Context

There are three viable strategies for obtaining a valid PX cookie bundle:

1. **Browser harvester** — run real Chromium, let PX `/init.js` execute, extract `_px3`.
2. **Native sensor generator** — reverse-engineer `/init.js`, build the sensor payload locally, POST `/xhr/b/s`, receive `_px3` directly.
3. **Third-party paid solver API** — outsource the problem to a vendor.

The harvester is reliable but heavy (Chromium per solve, 3–8 s, hundreds of MB RAM). The native generator is fast and light but brittle (PX rotates `/init.js` obfuscation frequently). The third-party API is out of scope for v1 (SOW §10).

## Decision

Adopt a **three-tier hybrid**:

1. **Tier 1 — TTL cookie cache** (`px-cache`). Serve cached cookies when a fresh bundle exists for the requested `(domain, appId, fingerprint key)`.
2. **Tier 2 — Browser harvester** (`px-harvester`). On cache miss, allocate a Chromium worker, harvest a fresh bundle, populate the cache.
3. **Tier 3 — Native sensor generator** (`px-native`). Reserved as a future fast path. For v1, ship as a stub crate with a `NativeSolver` trait and recorded fixtures so a future maintainer can drive TDD.

## Alternatives considered

- **Harvester only, no cache.** Rejected — cold-only solves waste Chromium and hammer the target.
- **Native only.** Rejected — too brittle for v1; requires deep RE work before any value is shipped.
- **Cache + harvester only, no native slot.** Rejected — would force a future architectural rewrite when native lands.

## Consequences

- **Positive:** ship value fast (Tier 1 + 2), with a clear non-blocking path to Tier 3; cache amortizes Chromium cost; tiers are independently testable.
- **Negative:** three crates to maintain; cache invalidation is a real source of bugs (mitigated by short TTL ≤ 10 min and explicit `(domain, appId, fingerprint key)` keying).
- **Follow-ups:** ADR-0010 records that Tier 3 is deferred but architecturally provisioned.
