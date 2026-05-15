# 0014. Challenge pipeline architecture

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0003, ADR-0006, ADR-0010, ADR-0015, SOW §2.2

## Context

Recon of three real targets (`pedidosya.com.ar`, `hannaandersson.com`, `havenwellwithin.com`) confirmed that production bot protection is rarely deployed in isolation. Even within our small sample, half the surface area carries an edge layer (Fastly, Cloudflare) in front of, or instead of, PerimeterX. In the wild, common stacks are:

```
Caller → CDN/edge (CF, Fastly, Akamai, CloudFront) → origin protection (PX, DataDome, Akamai BMP, Imperva) → site session (JWT)
```

A solver that only handles PerimeterX **and breaks the moment a Cloudflare Managed Challenge fires** is barely useful for real targets. At the same time, broadening scope to "solve everything" defeats the SOW. We need an architecture that:

1. Keeps v1 deliverables tight (PX handler only).
2. Does not require a rewrite when CF/Turnstile/DataDome/captcha handlers are added later.
3. Stays honest: a stealth Chromium pipeline *incidentally* defeats most passive edge layers without a dedicated handler.

## Decision

Replace the implicit one-shot solver model with an explicit **ChallengePipeline**: an ordered list of pluggable `ChallengeHandler` impls. Each handler answers "is my protection active on this page?" and, if so, "solve it." The pipeline walks handlers in configured order; each handler is independent and lives in its own crate.

```
SolveRequest
     │
     ▼
┌──────────────────────────────────────────────────────────┐
│  ChallengePipeline (px-pipeline)                         │
│                                                          │
│  for handler in config.order:                            │
│      if handler.detects(&page).await? { handler.solve() }│
└──────────────────────────────────────────────────────────┘
     │
     ▼
┌────────────┬────────────┬────────────┬────────────┬────────────┐
│ Cloudflare │ Turnstile  │ Perimeter  │ DataDome   │ Captcha    │
│ px-cloud   │ px-turn    │ px-pmx     │ px-datad   │ px-captcha │
│ flare      │ stile      │            │ ome        │            │
└────────────┴────────────┴────────────┴────────────┴────────────┘
                          │
                          ▼
                  PxCookieBundle + Other (cf_clearance, turnstile_token, …)
```

### The trait

```rust
#[async_trait]
pub trait ChallengeHandler: Send + Sync {
    fn name(&self) -> &'static str;
    async fn detects(&self, page: &Page, html: &str) -> Result<bool, AppError>;
    async fn solve(&self, page: &mut Page) -> Result<HandlerOutcome, AppError>;
}

pub struct HandlerOutcome {
    pub handler: &'static str,
    pub cookies: CookieJarDelta,
    pub tokens: Vec<NamedToken>,
    pub metrics: HandlerMetrics,
}
```

### Pipeline order is config-driven

`config/pipeline.yaml`:

```yaml
order:
  - cloudflare        # outermost edge
  - turnstile         # often co-deployed with cf
  - perimeterx        # primary marquee handler
  - datadome          # mutually exclusive with px on most sites; safe in same list
  - captcha           # last-resort fallback when any of the above bounce to captcha
```

Order may be overridden per target via the allowlist entry.

### Crate layout (additive)

```
px-pipeline/           NEW   Orchestrator + ChallengeHandler trait + HandlerOutcome
px-harvester/          KEPT  Vendor-agnostic Chromium pool (CDP, stealth core)
px-perimeterx/         NEW   PX handler (was the PX-specific half of px-harvester)
px-cloudflare/         NEW   CF handler (stub in v1, see ADR-0015)
px-turnstile/          NEW   Turnstile handler (stub in v1)
px-captcha/            NEW   hCaptcha / reCAPTCHA gateway (stub in v1)
px-datadome/           NEW   DataDome handler (stub in v1)
```

All handler crates depend on `px-pipeline` (for the trait) and `px-harvester` (for the shared Chromium pool). None depend on each other.

### Out-of-band invariants

- A handler MUST NOT mutate cookies belonging to a sibling handler's namespace. (`px-perimeterx` may only touch `_px*`; `px-cloudflare` may only touch `cf_*`; etc.)
- A handler MUST be idempotent across `detects() == false` — calling `solve()` when not detected is a programming error and returns `AppError::Conflict`.
- A handler MUST emit its `HandlerMetrics` regardless of outcome, so the pipeline can report which layer cost what.

## Alternatives considered

- **Monolithic PX-only solver.** Simpler today; needs full rewrite when CF/Turnstile lands.
- **Hard-coded pipeline order inside `px-server`.** No trait, no plugin surface. Marginally simpler but loses crate-level isolation and feature-gating.
- **Strategy pattern with a single `Solver` enum.** Couples all handlers into one crate; violates the axum-best-practice single-responsibility rule.
- **External orchestration (Temporal / pipeline workflow engine).** Overkill for a single-process solver.

## Consequences

- **Positive:** v2 handlers add zero changes to existing code; clear ownership boundaries per vendor; per-handler metrics make it obvious which layer dominates latency or failure; the solver becomes a "challenge router," not just a PX-solver.
- **Negative:** more crates to maintain (4 stub crates land in v1 that do not solve anything yet); the abstraction has a tiny runtime cost (~µs per handler `detects()` check) — negligible.
- **Follow-ups:**
  - [ADR-0015](0015-v1-ships-pipeline-with-perimeterx-handler-only.md) records that v1 ships **only the PX handler implementation**; the other four crates are stubs.
  - SOW §2.2 softens "out of scope" wording (see [`002-sow-scope.md`](../002-sow-scope.md) addendum 2026-05-16).
  - [ADR-0006](0006-axum-clean-architecture-layout.md) crate list updated.
  - axum-best-practice standard updated with the new crate names.
