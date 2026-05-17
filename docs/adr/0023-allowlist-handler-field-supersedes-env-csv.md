# 0023. `handler:` field in allowlist.yaml supersedes `PX_CAMOUFOX_DOMAINS`

- **Date:** 2026-05-17
- **Status:** Accepted (supersedes the env-only routing source from [ADR-0021](0021-domain-based-handler-routing.md); env CSV retained as a deprecated fallback through v1.x)
- **Deciders:** KeyCode17
- **Related:** ADR-0007 (allowlist semantics), ADR-0021 (domain-based routing)

## Context

ADR-0021 introduced `RoutingDispatcher` and a `PX_CAMOUFOX_DOMAINS` CSV
env var as the v1 source of truth for which targets route through the
Camoufox-backed Cloudflare handler. That ADR explicitly anticipated a
v2 move to a per-entry `allowlist.yaml` field once R5 stabilized:

> Future ADR: promote `PX_CAMOUFOX_DOMAINS` to a per-entry
> `handler: cloudflare` field in `allowlist.yaml` (env CSV becomes a
> deprecated fallback).

R5.9 has now produced live evidence (`docs/verification/2026-05-17-pedidosya-camoufox-soak-n10.md`)
that the Camoufox path is reliable for pedidosya, so the operator
config schema should reflect that this is a normal product capability,
not an env-flag experiment.

Operator-experience problems with the env-only source:

- Two files to keep in sync — `allowlist.yaml` says **what** is allowed,
  the env var says **how** it gets solved.
- No comment / justification next to the routing choice.
- Env var is invisible to anyone reading the config tree (auditor,
  on-call engineer, future operator).
- Adding a new CF-fronted target requires both a YAML edit and a
  process restart with a new env var.

## Decision

Add an optional `handler: <string>` field to each `AllowlistEntry`. When
set to `"cloudflare"`, the server's `RoutingDispatcher` routes that
domain to the CF-bypass path (CloudflareHandler backed by CamoufoxPool).
When absent, the domain inherits the default handler (currently
`PerimeterxHandler` for the cheap Chromium path).

`PX_CAMOUFOX_DOMAINS` continues to work and is unioned with the YAML
sources, so existing v1.0.1 deployments can roll forward to v1.1.x
without touching their YAML files. The env var is marked deprecated in
`docs/deployment.md` and slated for removal in v2.x.

Resolution order in [`px-server::main::resolve_cf_domains`](../../px-server/src/main.rs):

1. Read `allowlist.yaml` via `AllowlistStore::list()`.
2. Collect domains whose `handler.as_deref() == Some("cloudflare")`.
3. Union with the CSV from `PX_CAMOUFOX_DOMAINS`.
4. Lowercase + dedupe.

Serde defaults make the new field forward-compatible with existing YAML
files (no migration required for non-CF targets).

## Alternatives considered

- **Keep env-only** — simpler, no schema churn. Rejected: see context
  for operator-experience problems.
- **Reload allowlist at runtime** — full SIGHUP-driven reload that
  re-wires the routing table without a process restart. Out of scope
  here; would also require pool lifecycle changes. Deferred.
- **Inline handler config under the entry** (full struct rather than a
  string) — supports future per-target options like budget overrides
  and per-target Camoufox configs. Considered overkill for v1.1 — a
  plain string keeps YAML readable. Revisit when a second knob exists.

## Consequences

- **Positive:**
  - One file describes both authorization and routing for each
    target.
  - Adding a CF-fronted target is a single `allowlist.yaml` edit, then
    a process restart.
  - Auditors see the routing choice next to the `tos_reviewed` /
    `justification` they're already reviewing.
  - `AllowlistEntry` deserialization is forward-compatible: omitted
    `handler` field deserializes to `None` (`#[serde(default)]`).

- **Negative:**
  - One more public field on `AllowlistEntry`. Public API surface
    grows; the existing serde struct now has a 4th field. Anyone
    constructing `AllowlistEntry` manually (tests, scripts, the CLI's
    `allowlist add`) must add `handler: None` or `handler: Some(...)`.
    Done in this change.
  - Two sources for the same routing fact during the v1.1.x deprecation
    window. Documented and bounded — env source goes away in v2.

- **Follow-ups:**
  - v2.x: remove `PX_CAMOUFOX_DOMAINS` parsing and the union in
    `resolve_cf_domains`. Update deployment.md to drop the env var.
  - `px-cli allowlist add` could grow a `--handler cloudflare` flag.
    Trivial follow-up — currently the operator must edit YAML by hand
    to set the handler, which is acceptable since marking a target as
    CF-fronted is a deliberate decision worth a manual touch.
