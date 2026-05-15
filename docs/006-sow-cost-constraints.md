# 006 — API / Compute Cost Constraints

| Category | Provider | Monthly Estimate | Cap |
|---|---|---|---|
| Compute (Chromium pool) | Self-hosted Linux | $5 – $40 (depending on VPS sizing) | $50 |
| Storage (audit log) | Local disk | < 1 GB / month | 5 GB |
| Outbound bandwidth | VPS provider | < 20 GB / month at MVP load | 100 GB |
| Optional Redis | Self-hosted | $0 | $20 |
| Third-party APIs | None | $0 | $0 |

Primary cost driver: Chromium memory footprint. A 4-worker pool is expected to need ~1.5 GB RAM steady-state.
