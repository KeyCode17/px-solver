# 0022. Re-admit pedidosya to the canary set with a deep-stealth latency budget

- **Date:** 2026-05-17
- **Status:** Accepted (amends ADR-0018)
- **Deciders:** KeyCode17
- **Related:** ADR-0018 (canary scope), ADR-0020 (Camoufox), ADR-0021 (routing), R5.7/R5.8/R5.9

## Context

ADR-0018 moved `pedidosya.com.ar` out of the canary set for v1.0.0 because
the Chromium-only path could not bypass Cloudflare Bot Management and
the Turnstile widget was never even rendered. R5.7 wired
`CloudflareHandler::with_harvester(CamoufoxPool)` end-to-end and R5.8
proved a single end-to-end /v1/solve succeeded.

R5.9 ran the open question: does the Camoufox path hold up under
sustained sequential load on pedidosya, and at what latency cost?

Evidence (see [`docs/verification/2026-05-17-pedidosya-camoufox-soak.md`](../verification/2026-05-17-pedidosya-camoufox-soak.md)):

| Metric          | Value         |
|-----------------|---------------|
| N               | 10            |
| Success ratio   | 1.0000 (10/10)|
| Median solve_ms | 10,576        |
| p95 solve_ms    | 11,096        |
| Min ms          | 10,154        |
| Max ms          | 11,096        |
| Cookies / harvest | 6 (cf_clearance, __cf_bm, _pxhd, _pxvid, _px3, pxcts) |

The two acceptance criteria that gate the canary:

- **AC-1 (≥95% solve)** — observed 100%. Passes the production threshold.
- **AC-2 (median solve_ms ≤6000)** — observed 10,576ms. **Fails** the
  production threshold by ~76%. The latency is structural: each harvest
  spawns a fresh geckodriver subprocess, launches a patched-Firefox
  binary, navigates, waits for CF to issue clearance, and tears down.
  Pre-warming a long-lived browser would defeat the per-request
  fingerprint randomization that gets cf_clearance issued in the first
  place; bringing the median under 6s would require either parallel
  pools (more memory, more zombies risk) or a process-reuse strategy
  (more detection risk).

## Decision

Re-admit `pedidosya.com.ar` (and the `www` subdomain) to the canary set
with a **deep-stealth latency budget**: AC-2 is relaxed to **median ≤15s,
p95 ≤20s** for any allowlisted target whose `handler` resolves to
`cloudflare` (i.e. CF-fronted targets routed via Camoufox per
[ADR-0021](0021-domain-based-handler-routing.md)). AC-1 (≥95% success)
remains unchanged.

The split budgets reflect a real product split: the cheap Chromium path
serves PX-direct tenants under the original 6s SLA; the deep-stealth
path serves the CF-fronted targets that would otherwise be uncovered,
at a higher but bounded latency. Both budgets continue to enforce zero
browser zombies (AC-4) and the existing cache-hit ratio AC-3 (cache
amortizes the per-harvest cost for repeat callers).

## Alternatives considered

- **Keep pedidosya in R5 backlog** — defer re-admission until median
  drops to 6s. Rejected: blocks a real customer use case for an
  optimization that may not be reachable without breaking the bypass.
- **Lift AC-2 entirely for canary** — too permissive; removes the
  signal that catches regressions when a target falls back to a slow
  retry path. Rejected.
- **Per-target AC-2 in `allowlist.yaml`** — cleanest long-term, but
  expands operator-facing config schema. Deferred; the handler-name
  derived split covers v1.

## Consequences

- **Positive:**
  - pedidosya is back on the canary list with empirical evidence that
    the Camoufox path is reliable (10/10 with sub-second variance) and
    bounded in latency (max 11.1s in 10-sample soak).
  - Future targets that need Camoufox inherit the relaxed budget by
    virtue of their routing entry, without per-target config knobs.
  - Owner sign-off for v1.0.0 is unaffected — that document was
    explicit about the AC-1 gap for pedidosya, and R5.7-R5.9 close
    exactly that gap.

- **Negative:**
  - Two AC-2 budgets means dashboards/alerts must split by handler.
    Not currently a problem (metrics distinguishes handler) but a
    follow-up.
  - The 10-sample soak is small. Larger soak (N ≥100 over a working
    week) is desirable before promoting pedidosya from canary to a
    documented SLA target.

- **Follow-ups:**
  - Larger pedidosya soak (N=100+) and append-evidence under
    `docs/verification/`. Until that lands, treat pedidosya canary
    status as "provisional".
  - Metrics: expose `px_solve_ms_by_handler{handler="cloudflare"}`
    histogram so the deep-stealth budget can be alerted on separately.
  - Operator docs ([`deployment.md`](../deployment.md)) updated to
    state the two-tier AC-2.
