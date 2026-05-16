# Phase 04 — CLI + Canary + Docs

- **Window:** 2026-06-13 → 2026-06-17
- **Status:** Not started
- **SOW-DEL covered:** SOW-DEL-007, SOW-DEL-008, SOW-DEL-010
- **Owner:** KeyCode17

## Goal

Ship the operator experience: a CLI, an end-to-end canary test, and the documentation required for the dual-use policy to be enforceable. Close the MVP gate.

## Entry criteria

- [ ] Phase 03 marked **Done**.
- [ ] `/v1/solve` succeeds end-to-end against the canary in a one-off manual run.

## Deliverable status

- [ ] SOW-DEL-007 — CLI (`px-cli`)
  - [ ] `px-cli solve <url>` — solves and prints `PxCookieBundle` as JSON on stdout. Exit 0 on success; non-zero with categorized stderr diagnostic on failure.
  - [ ] `px-cli serve` — runs the embedded server using `config/server.yaml`.
  - [ ] `px-cli detect <url>` — fetches the URL, runs the detector, prints `PxDetection`.
  - [ ] `px-cli keys generate` — emits a new key + its Argon2id hash; never persists the plaintext.
  - [ ] `px-cli allowlist {add,remove,list}` — manipulates `config/allowlist.yaml` with schema validation.
- [ ] SOW-DEL-008 — Canary integration test
  - [ ] `px-server/tests/pedidosya.rs` spins the server in-process, solves the canary, validates the cookies on `/home-page/v44/home/lazy_load`.
  - [ ] Gated by `CI_CANARY=1`; defaults to `#[ignore]` in CI.
  - [ ] On failure emits a categorized diagnostic: `network`, `px-block`, `cookie-shape-mismatch`, `harvester-timeout`.
- [ ] SOW-DEL-010 — Operator documentation
  - [ ] `README.md` — what / why / install / first-solve / threat-model link.
  - [ ] `docs/deployment.md` — fresh-Linux install, systemd unit, reverse proxy, key generation, allowlist editing.
  - [ ] `docs/threat-model.md` — misuse vectors (credential stuffing, ToS-violating scraping, mass scraping) and mitigations (API key, allowlist, audit log, rate limit).
  - [ ] `docs/dual-use-policy.md` — operator commitments per [`docs/011-sow-dual-use.md`](../011-sow-dual-use.md), signable.

## Exit criteria (= MVP gate)

All of [`docs/008-sow-acceptance.md`](../008-sow-acceptance.md) MVP-AC-1..7 hold simultaneously over a single 24-hour window:

1. ≥ 95% solve success on 100 consecutive canary solves.
2. p50 `/v1/solve` ≤ 6 000 ms cold.
3. Cache hit ratio ≥ 70% at 1 RPS sustained.
4. Zero Chromium zombies after 24-hour soak.
5. No clippy warnings; no `cargo audit` advisories ≥ medium.
6. Audit log: one record/solve, zero cookie payloads (`grep -E '_px3|_pxhd' audit.log` = 0).
7. Threat model + dual-use policy signed off by owner.

### Producing the evidence

Items 1–4 (the soak block) are gathered with one command:

```bash
PX_SOAK_KEY=<key-id>:<secret> cargo xtask soak --duration 24h --rps 1
```

That writes `docs/verification/<date>-soak.md` with PASS/FAIL per AC.

Item 5 is replayed by `cargo clippy ... && cargo audit` (see
[`docs/verification/2026-05-16-mvp-ac-codeside.md`](../verification/2026-05-16-mvp-ac-codeside.md)).
Item 6 is structurally guaranteed by `AuditEvent` having no cookie fields,
documented in the same file.

Item 7: copy `docs/verification/owner-signoff-template.md`, fill in name +
date + the two doc commit SHAs, commit.

After all 7 MVP-AC items are confirmed, run `cargo xtask phase 04` → version becomes `1.0.0` (major bump from `0.4.0`) per [ADR-0017](../adr/0017-phase-aligned-versioning.md). `xtask` does **not** verify the MVP acceptance checklist; that verification is the operator's responsibility before invoking the phase bump.

## Risks

- **R-04-1:** CLI scope creep (proxy commands, custom fingerprints). Mitigation: subcommand list is closed; new subcommands require an ADR.
- **R-04-2:** 24-hour soak fails on memory growth. Mitigation: Phase 02 worker-restart policy must include `restart_after_n_solves` and `restart_after_max_resident_kb`.
- **R-04-3:** Doc rot between SOW and `docs/deployment.md`. Mitigation: deployment doc is reviewed by walking it on a fresh VM before sign-off.

## Log

_Append-only._
