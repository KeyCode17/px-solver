# 004 — Data Governance & Ownership

| Data Asset | Owner | Access Level | Retention | Notes |
|---|---|---|---|---|
| Source code | KeyCode17 | Public on GitHub once published | Indefinite | License: TBD before publication (proposed: AGPL-3.0 to discourage closed-source resale) |
| API keys (hashed) | Operator | Local config only, file mode `0600` | Until rotated | Never logged in plaintext |
| Solved cookies (`_px3`, `_pxhd`, etc.) | Target site (logical) | In-memory cache only by default | TTL ≤ 10 min | Treated as transient secrets; redacted from audit log |
| Audit log records | Operator | Local disk, optional shipping to operator-owned syslog | 90 days default | Contains target domain, key id, outcome, latency — no cookie payload |
| Captured sensor fixtures (`px-native/tests/fixtures/`) | KeyCode17 | Public alongside repo | Indefinite | Sanitized before commit; no IP / no UA / no PII |
| Canary site response data | PedidosYa / Delivery Hero | Not stored beyond test assertion (status + size only) | None | Tests never persist response bodies |

There is no end-user PII processed by this system. Target-site response data is not retained.
