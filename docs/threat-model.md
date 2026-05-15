# Threat Model

`px-solver` is a dual-use capability: it produces valid `_px3` cookies for PerimeterX-protected sites. Anything that simplifies access to gated content is, by definition, a tool an attacker would also want to use. This document names the misuse vectors and the in-code mitigations that bound them.

Pairs with [`011-sow-dual-use.md`](011-sow-dual-use.md), [ADR-0007](adr/0007-api-key-and-domain-allowlist-guardrails.md), and [`docs/deployment.md`](deployment.md).

## In-scope adversaries

| Adversary | Goal | Plausibility |
|---|---|---|
| External caller, no credentials | Use the solver against any site | High |
| External caller, leaked API key | Use it against sites not on the allowlist | Medium |
| Internal operator | Misuse with valid credentials against in-scope targets | Low but possible |
| Insider with host access | Tamper with `config/keys.yaml` or `config/allowlist.yaml` | Low — local file ACLs apply |
| Target-site operator | Detect the solver and update PX detection | High — adversarial cadence is the bot-mitigation vendor's job |

Out of scope: nation-state attacker with arbitrary host compromise; supply-chain attacks on the Rust dependency graph (addressed indirectly by `cargo audit` / `cargo deny`).

## Misuse vectors and mitigations

### V1. Anonymous public solving
**Attack:** an attacker discovers the `/v1/solve` endpoint and uses it without credentials.
**Mitigation:** API key required (Bearer `<id>:<secret>`), verified by Argon2id with constant-time comparison. Unauthenticated requests → `401 Unauthorized`. Metric `px_auth_denied` increments. Default bind is `127.0.0.1`; remote exposure requires explicit reverse-proxy.

### V2. Solving against arbitrary domains
**Attack:** a caller with a valid API key tries to solve a target the operator did not authorize.
**Mitigation:** per-domain allowlist enforced after authentication. Domain extracted from the `url` field, compared against `config/allowlist.yaml`. Missing or unreviewed entry → `403 Forbidden`. Metric `px_allowlist_denied` increments. Allowlist entries require `tos_reviewed: true` and a non-empty justification; server refuses to start otherwise.

### V3. Credential stuffing / account takeover
**Attack:** the solver is pointed at login flows or auth endpoints to bulk-attempt credentials.
**Mitigation:** the solver returns cookies only; it does not store or replay account credentials. The dual-use policy ([`011-sow-dual-use.md`](011-sow-dual-use.md)) prohibits use in credential abuse. Operators are encouraged to keep API keys narrowly scoped per use case and to keep target sites in the allowlist that are clearly first-party-research targets.

### V4. ToS-violating mass scraping
**Attack:** a caller uses the solver to extract data at a volume the target site did not consent to.
**Mitigation:** the allowlist entry's `justification` field is intended to be a human-readable promise. The audit log captures every solve attempt with the caller key id, so post-hoc review is possible. No technical rate limit is enforced by `px-solver` itself in v1; operators may run a reverse proxy with rate limits in front.

### V5. Leaked API key
**Attack:** an operator's API key secret leaks (e.g. via env-var disclosure, copy-paste).
**Mitigation:** keys are stored as Argon2id hashes only (no plaintext on disk). Rotation is a config edit + `systemctl restart`. The audit log identifies which key was used for each solve.

### V6. Audit log poisoning
**Attack:** an insider tampers with the audit log to cover misuse.
**Mitigation:** `FileAuditSink` appends only, locked writer. Ship logs to an operator-owned syslog/SIEM for tamper-evident storage (out of scope to provide a sink for that here, but the `AuditSink` trait makes adding one trivial).

### V7. Resource exhaustion
**Attack:** a caller floods `/v1/solve` to exhaust Chromium worker pool.
**Mitigation:** `ChromiumoxidePool` is `tokio::sync::Semaphore`-bounded. Per-worker lifecycle limits zombie risk. No rate limit on the HTTP layer in v1; operators should add one in their reverse proxy.

### V8. Reverse-engineering the binary to recover secrets
**Attack:** the released binary is reverse-engineered to recover allowed domains or hashed keys.
**Mitigation:** the binary is `publish = false` and self-contained. Sensitive state (`keys.yaml`, `allowlist.yaml`) is loaded at runtime from disk and never embedded. Plaintext key secrets never live on the server; only their Argon2id hashes do.

### V9. Cookie reuse across callers
**Attack:** one caller's solve result is replayed by an unauthorized caller.
**Mitigation:** cookies are bound to the harvester's IP at PX server side (`_px3` is IP-scored by PX). A different caller's IP will trip a re-score by PX. For stronger isolation, run a separate server instance per caller key.

## Residual risk

- **Real-world solve verification is operator-side.** This codebase cannot certify the 95% solve gate; only an operator with a residential IP and real Chromium can. The release notes for v0.3.0 and later are explicit about this.
- **PX updates can break the solver overnight.** Stealth bundle is a moving target. The solver may return blocked pages instead of cookies until the bundle is updated.
- **Dependency chain risk.** Cargo dependencies (`chromiumoxide`, `argon2`, `axum`, etc.) ship with their own attack surface. Mitigated by `cargo audit` in CI but not eliminated.
