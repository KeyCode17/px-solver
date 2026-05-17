# 0021. Domain-based handler routing in the solve dispatcher

- **Date:** 2026-05-17
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** SOW §3 (deliverables), ADR-0014 (pipeline architecture), ADR-0015 (PX-only v1), ADR-0020 (Camoufox via fantoccini/geckodriver), R5.7

## Context

R5.7 introduces `CloudflareHandler::with_harvester` so the Cloudflare path
can re-harvest CF-fronted targets through `CamoufoxPool`. The remaining
question is **how the dispatcher picks between `PerimeterxHandler` (the
cheap Chromium path) and the Camoufox-backed `CloudflareHandler` per
inbound `/v1/solve` request**.

Forces:

- **No double-harvest tax.** A "detect first, then dispatch" loop would
  require either an extra cheap fetch (HEAD/GET) or running the cheap
  handler unconditionally before falling back. Both pay latency on every
  call, even for hosts we already know need Camoufox.
- **Camoufox is heavier.** ~24s wall-clock on pedidosya in current
  empirical runs (`px-camoufox/tests/diagnose.rs`), vs ~3-6s for the
  Chromium-only path. Using Camoufox by default on PX-direct targets
  would regress the AC-2 budget.
- **Operators already know.** When an operator adds a domain to
  `config/allowlist.yaml`, they know which protection vendor fronts it.
  Treating the dispatcher's handler choice as operator-config-driven
  matches reality and is auditable.
- **Predictability.** Heuristic detection (CDN header sniffing, HTML
  pattern matching) creates per-request branching that is hard to debug
  when a target's fronting changes. Config-driven routing keeps the
  dispatch decision out of the request hot path.

## Decision

Route `/v1/solve` to a `ChallengeHandler` keyed on the URL host, via
operator config. v1 reads `PX_CAMOUFOX_DOMAINS` (CSV) at startup;
domains listed there route to `CloudflareHandler::with_harvester(camoufox_pool)`,
everything else to the default `PerimeterxHandler`.

Implementation:

- `px-server::application::routing::RoutingDispatcher` — `SolveDispatcher`
  impl with a `BTreeMap<String, Arc<dyn ChallengeHandler>>` and a default
  fallback handler.
- Matching is DNS-suffix: `pedidosya.com.ar` covers `www.pedidosya.com.ar`,
  `m.pedidosya.com.ar`, etc.
- `main.rs` builds a `RoutingDispatcher` iff `PX_CAMOUFOX_DOMAINS` is
  non-empty AND `CamoufoxConfig::from_env().validate()` succeeds. When
  the env var is unset or the binaries are missing, the wiring is
  byte-identical to the pre-R5.7 `PxSolveDispatcher`-only path (with a
  warn-level log if the env var is set but binaries are absent).

## Alternatives considered

- **Heuristic detection in the request hot path** — cheap HTTP fetch
  to peek at `server: cloudflare` / `cf-ray` headers, then dispatch.
  Adds a network round-trip per request, breaks under cache (cached
  responses may not carry the headers), and conflates discovery with
  service. Rejected.
- **Run cheap path first, fall back on CF interstitial** — call
  `PerimeterxHandler`, detect `cdn-cgi/challenge-platform` in the result,
  retry through Camoufox. Doubles tail latency for CF-fronted targets
  and burns a wasted Chromium harvest. Rejected for v1; remains a
  possible "auto-discovery" mode for unknown targets in a future ADR.
- **Per-route field in `allowlist.yaml`** — e.g.
  `requires_camoufox: true` per entry. Cleaner than an env var but
  expands the persisted operator-facing schema and requires migration
  notes. Deferred to v2 once R5 stabilizes; the env-CSV form is
  forward-compatible since the YAML field would supersede it.
- **Pipeline-of-handlers via existing `Pipeline`** — `px-pipeline`
  already supports an ordered list of handlers with detect+solve. But
  `detects()` needs a `PageHtml`, which the dispatcher does not have
  without an extra fetch. The Pipeline shape remains useful for the
  detect-on-an-already-fetched-page flow (e.g. when a caller hands us
  HTML), and v1.0.0 leaves it in place; the dispatcher uses a simpler
  routing primitive for now.

## Consequences

- **Positive:**
  - No probe latency per request. Handler is chosen in O(routes) via
    case-insensitive DNS-suffix lookup.
  - Operators control which targets pay the Camoufox tax.
  - Backward-compatible: unset env var leaves wiring unchanged, so
    existing deployments and the v1.0.0 owner sign-off remain valid.
  - Cleanly composable with the future "per-route allowlist field"
    by swapping the source from env to YAML without changing the
    dispatcher shape.

- **Negative:**
  - The operator must list each CF-fronted domain explicitly. A target
    that silently moves behind Cloudflare will not be picked up until
    config is updated (and the cheap path will start failing, which
    surfaces in audit logs / metrics).
  - Two parallel cookie extractors (`px_perimeterx::extract_px_cookies`
    and `px_cloudflare::extract_session_cookies`) — the CF one is a
    superset and could be merged later, but kept separate for v1 to
    keep handler crates self-contained.

- **Follow-ups:**
  - R5.8: pedidosya integration test gated behind `CI_CANARY_CF=1`
    that exercises the routing path end-to-end against a recorded
    fixture.
  - R5.9: pedidosya soak via `xtask soak --url https://www.pedidosya.com.ar/`
    once `PX_CAMOUFOX_DOMAINS=www.pedidosya.com.ar,pedidosya.com.ar` is
    set; if AC-1 / AC-2 hold for 100 sequential requests, amend ADR-0018
    to re-add pedidosya to the canary set.
  - Future ADR: promote `PX_CAMOUFOX_DOMAINS` to a per-entry
    `handler: cloudflare` field in `allowlist.yaml` (env CSV becomes a
    deprecated fallback).
