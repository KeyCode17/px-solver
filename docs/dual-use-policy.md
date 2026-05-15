# Dual-Use Policy

`px-solver` produces valid PerimeterX cookies. That capability is **dual-use**: useful for authorized research, defensive testing, and legitimate scraping; misusable for credential abuse, ToS-violating scraping, account takeover, or mass exfiltration.

This policy is binding on every operator of this codebase. It pairs with [`011-sow-dual-use.md`](011-sow-dual-use.md), [`docs/threat-model.md`](threat-model.md), and [ADR-0007](adr/0007-api-key-and-domain-allowlist-guardrails.md).

## Operator commitments

By running this codebase, the operator commits to:

1. **No anonymous solving.** The server must always run with `config/keys.yaml` populated and the API-key middleware active. The codebase enforces this at startup.
2. **Allowlist hygiene.** Every entry in `config/allowlist.yaml` must include:
   - `tos_reviewed: true` — the operator has personally read the target site's Terms of Service and believes the planned use is consistent with them.
   - A non-empty `justification` describing the authorized use case (research, defensive test, internal monitoring, etc.).
   Adding a target to the allowlist is a deliberate act, not a default.
3. **No credential abuse.** The solver is not pointed at login forms, auth endpoints, or any flow that would enable account takeover or credential stuffing.
4. **No public SaaS.** The server is not exposed as an anonymous or pay-per-call public service. Operators may share access with named individuals via API keys, but not with the open internet.
5. **Respect rate constraints.** The solver does not include built-in rate limits; the operator commits to running a reverse-proxy / external limiter when serving downstream consumers, and to respecting target-site `robots.txt` for any consumer of the cookies.
6. **Audit log review.** The operator reviews `AuditEvent` logs at least monthly (or in response to incidents). Anomalies (sudden volume spikes, calls to allowlist entries from unexpected keys, repeated `HandlerFailed` outcomes) are investigated.
7. **No deception of target sites.** The operator does not misrepresent the source of solved requests to the target site beyond what is inherent in stealth fingerprinting (i.e. does not impersonate a specific real user account).

## Prohibited uses

- Credential stuffing, brute-force login, account takeover.
- Mass scraping at a volume the target site did not consent to, even when the target is allowlisted.
- Reselling solver capacity to anonymous or third-party consumers.
- Bypassing access controls on private/authenticated content (logged-in pages, paywalled content).
- Using the solver against sites whose ToS the operator has not personally reviewed.
- Any use described as in-scope by [`docs/threat-model.md`](threat-model.md) under "Misuse vectors."

## Non-binding guidance

- Prefer using a paid `cookies-as-a-service` vendor (CapSolver, ScrapFly, Bright Data) over self-hosting if the volume justifies it; those vendors have dedicated abuse teams and incident response.
- Discuss with the target site's security team before running automated traffic at any meaningful volume.
- Keep operator-side instances ephemeral. A long-running production solver instance is a stationary target for both the target site's bot-mitigation team and external attackers.

## Disclaimer

`px-solver` is research software released under AGPL-3.0-or-later. It carries no warranty. The author is not responsible for misuse. By running it, the operator acknowledges they have read this policy and the threat model, and accepts the legal and operational responsibility for use under their control.
