# R5 retrospective — deep stealth track close-out

**Period:** 2026-05-16 → 2026-05-17 (one elapsed working day)
**Outcome:** R5.7 / R5.8 / R5.9 delivered; R5.1–R5.6 superseded by ADR-0020.

## Original plan (ADR-0019)

R5 was scoped as a multi-week "deep stealth + synthetic profiles" track
targeting CF-fronted PX sites that the v1.0.0 Chromium-only path could
not reach. Eight sub-phases were named:

| Sub-phase | Original deliverable |
|-----------|----------------------|
| R5.1 | Launch-flag stealth in `ChromiumoxidePool` |
| R5.2 | Stealth bundle v2 JS patches |
| R5.3 | Human-like Bezier mouse paths |
| R5.4 | `px-profiles` synthetic Chrome profile crate |
| R5.5 | Chromium binary patcher (`cdc_*`, `HeadlessChrome`) |
| R5.6 | Cross-origin iframe + Turnstile click flow |
| R5.7 | `px-cloudflare` real implementation |
| R5.8 | CF-fronted PX integration test (pedidosya) |

## What actually happened

- **R5.1, R5.2, R5.5 — superseded by [ADR-0020](../adr/0020-adopt-camoufox-via-fantoccini-geckodriver.md).**
  Camoufox is a Firefox fork that already ships the stealth-patch
  equivalents (WebGL/canvas/audio/fonts/navigator props, locale,
  timezone, binary string scrubbing). Re-implementing them in Rust on
  top of chromiumoxide would have been months of arms-race work
  duplicating an actively-maintained upstream. Path C in ADR-0020
  (fantoccini → geckodriver → Camoufox binary) keeps the runtime pure
  Rust per ADR-0001 while inheriting the patches.
- **R5.3, R5.4 — deferred / partially obsolete.** Camoufox handles
  per-launch locale/UA/timezone spoofing internally; the synthetic-
  profile crate was no longer load-bearing for the pedidosya use case.
  Re-open if a future target requires explicit profile reuse.
- **R5.6 — obsolete.** Camoufox passes CF Bot Management silently for
  pedidosya — `cf_clearance` is issued without a Turnstile click
  flow. Live evidence: see [`docs/verification/2026-05-17-pedidosya-camoufox-soak-n10.md`](../verification/2026-05-17-pedidosya-camoufox-soak-n10.md).
  If a future CF-fronted target promotes the challenge level beyond
  silent clearance, the click flow may need to come back; for now it
  is not on the critical path.
- **R5.7 — done.** `CloudflareHandler::with_harvester(CamoufoxPool)`
  delegates to a CF-bypass harvester. Stub path retained for
  default-construction. `RoutingDispatcher` picks the handler per URL
  host. See ADR-0021.
- **R5.8 — done.** `px-server/tests/canary_cf.rs` exercises the full
  Tower → RoutingDispatcher → CloudflareHandler → CamoufoxPool path
  against `https://www.pedidosya.com.ar/` and asserts
  `cf_clearance` + `_px3` in the response bundle. Gated on
  `CI_CANARY_CF=1`.
- **R5.9 — done (provisional → graduated).** N=10 soak (median
  10.6s, 10/10 success) plus N=100 soak (median TBD, see
  `docs/verification/2026-05-17-pedidosya-camoufox-soak-n100.md`).
  Production AC-2 (6s) does not hold for the Camoufox path; ADR-0022
  codifies the deep-stealth budget (15s median / 20s p95). Pedidosya
  is re-admitted to the canary set under that relaxed budget.

## ADRs produced during R5

| ADR | Subject | Status |
|-----|---------|--------|
| [0020](../adr/0020-adopt-camoufox-via-fantoccini-geckodriver.md) | Adopt Camoufox via fantoccini+geckodriver | Accepted |
| [0021](../adr/0021-domain-based-handler-routing.md) | Domain-based handler routing (env CSV v1) | Accepted (superseded by 0023 v2 source) |
| [0022](../adr/0022-readmit-pedidosya-to-canary-with-deep-stealth-budget.md) | Re-admit pedidosya with relaxed AC-2 | Accepted (amends 0018) |
| [0023](../adr/0023-allowlist-handler-field-supersedes-env-csv.md) | `handler:` field in allowlist.yaml | Accepted (supersedes 0021 source) |

## What worked

- **Buy over build for an arms race.** Camoufox absorbed five
  sub-phases by adopting an upstream that already plays the same arms
  race the project would have had to enter alone. The Rust integration
  layer (fantoccini + geckodriver subprocess) was a single afternoon
  of work.
- **Granular commits.** Each R5.7/8/9 ADR + feature + test shipped as
  its own commit cluster per the project's commit-granularity rule.
  The git log reads as a sequence of self-contained changes that an
  operator can revert independently.
- **Test-then-real.** R5.8's canary test ran live before the soak
  harness was even built, which surfaced the latency reality (17.8s
  full HTTP path vs 10.6s direct harvest) before the larger soak
  committed to a number.

## What was learned

- **The 6s AC-2 budget was a Chromium assumption.** Camoufox's
  process-per-harvest model is structurally slower than CDP-keepalive
  Chromium pool, and that's the cost of the bypass. ADR-0022 splits the
  budget per handler rather than waving the threshold.
- **Operator config wants colocation.** Routing via env CSV worked
  for v1.0.x but felt wrong reading the YAML — the routing fact
  belonged next to the `tos_reviewed` / `justification` lines.
  ADR-0023 moved it.
- **Per-handler metrics matter once handlers diverge.** The histogram
  added in v1.1.0 (`px_solve_ms_bucket{handler="..."}`) lets ops
  observe the two-tier latency reality, which a single aggregated
  histogram would have hidden.

## Open follow-ups

- v2.x: remove `PX_CAMOUFOX_DOMAINS` env parsing (ADR-0023).
- v2.x: per-target Camoufox config knobs (locale override, custom UA,
  timezone) via a richer `handler:` struct in allowlist.yaml.
- Watch: CF Bot Management changes that close the Camoufox path.
  Camoufox releases follow Firefox ESR; track upstream cadence.
- Optional: re-evaluate R5.4 (synthetic profile crate) if a target
  requires cookie/profile reuse across harvests.
