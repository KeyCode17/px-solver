# Design Patterns — `px-solver`

The canonical inventory of patterns the codebase uses, why each one is here, and which crate it lives in. Reviewed alongside [`docs/standards/axum-best-practice.md`](axum-best-practice.md) and the ADRs.

## A. Architectural patterns

| Pattern | Role | Where |
|---|---|---|
| Clean Architecture (Hexagonal-flavored) | 3 layers per crate: `domain` → `application` → `infrastructure`. Domain compiles with no infra deps. | Every `px-*` crate; standard per [ADR-0006](../adr/0006-axum-clean-architecture-layout.md). |
| Composition Root | All `Arc<dyn Trait>` wiring happens in one place. | `px-server/src/infrastructure/bootstrap.rs` |
| Ports & Adapters | Domain defines traits (ports); infrastructure provides impls (adapters). | `CookieCache` (port) ↔ `InMemoryCookieCache` / `RedisCookieCache` (adapters); analogous for `AuditSink`, `KeyStore`, `AllowlistStore`, `NativeSolver`. |
| Domain-Driven Design — Value Objects | Newtypes carry meaning + invariants. | `PxAppId`, `ApiKeyHash`, `CacheKey`, `PxCookieBundle`, `Fingerprint`. |

## B. GoF patterns

| Pattern | Crate / type | Purpose |
|---|---|---|
| Strategy | `ChallengeHandler` trait + N impls | Each protection vendor is a swappable strategy in `px-pipeline`. |
| Chain of Responsibility | `px-pipeline::Pipeline::run()` walks handlers in configured order, each decides whether to handle. | Layered protections (CF → Turnstile → PX → DataDome → Captcha). |
| Repository | `CookieCache`, `AuditSink`, `KeyStore`, `AllowlistStore` | Domain depends on the trait; storage backend swaps freely. |
| Adapter | `InMemoryCookieCache` / `RedisCookieCache`, `FileAuditSink` / `StdoutAuditSink` | Match concrete tech to the port. |
| Object Pool | `px-harvester::ChromiumPool` | Reuse expensive Chromium instances under a `Semaphore`. |
| Façade | `SolvePx` use case (`px-perimeterx::application::solve_px`) | One method (`execute`) hides the dance of detector → cache → pipeline → audit. |
| Builder | `SolveRequestBuilder`, `HandlerOutcomeBuilder` | Multi-field assembly with validation. |
| Null Object | `NotImplementedNativeSolver`, `NotImplemented{Cf,Turnstile,Captcha,DataDome}Handler` | Architectural seat without implementation; honest "not enabled in v1." Per [ADR-0015](../adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md). |
| Decorator | Stealth patches wrap a `Page` before navigation; tracing/metrics middleware wraps handlers. | Cross-cutting concerns without polluting core. |
| Template Method | `ChallengeHandler` defines the `detects → solve` flow shape; impls fill the body. | Same outer pattern across all vendor handlers. |
| Observer | `px-observer` taps every solve event from `px-harvester`. | Pattern mining without coupling. |
| Iterator | `Pipeline::iter_handlers()` yields handlers in config order. | Standard Rust idiom. |

## C. Rust-specific idioms

| Pattern | Where | Why |
|---|---|---|
| Newtype | `PxAppId(String)`, `ApiKeyHash([u8; 32])`, `CacheKey { ... }` | Compile-time prevention of mixing same-typed values; per-type `PartialEq` / `Hash`. |
| Typestate | `Page<Unloaded>` → `Page<Loaded>` → `Page<ChallengeDetected>` → `Page<Solved>` | Misuse is a type error, not a runtime panic. |
| Sealed Trait | `ChallengeHandler` is sealed via a private supertrait | External crates can't add handlers without going through `px-pipeline`. |
| `Result` + `AppError` | Every fallible API returns `Result<_, AppError>` | One error type per crate; one mapping to HTTP at the edge (`px-errors`). |
| `?` propagation, no `unwrap` | Enforced by `clippy::unwrap_used` in lefthook | Per axum-best-practice rule #9. |
| `Arc<dyn Trait + Send + Sync>` DI | All services in the composition root | Shareable across tokio tasks. |
| `#[non_exhaustive]` on public enums | `AppError`, `BlockClass`, `HandlerOutcome` variants | Adding a variant is non-breaking. |
| Feature gates | `--features redis`, `--features cloudflare`, `--features turnstile`, `--features captcha`, `--features datadome` | v1 stubs link only when explicitly enabled. |
| `async_trait` | All ports that are async | Required for `Send + Sync + 'static` trait objects in current Rust. |
| `Cow<'a, str>` | Detection results may return borrowed strings from input HTML | Zero-copy when possible. |
| `#[must_use]` on critical structs | `SolveOutcome`, `HandlerOutcome`, `CookieJarDelta` | Forces caller to acknowledge results. |

## D. Concurrency patterns

| Pattern | Where |
|---|---|
| Semaphore-bounded pool | `px-harvester::ChromiumPool` caps concurrent Chromium workers. |
| Actor (single-task ownership) | Each Chromium worker is owned by exactly one tokio task; messaged via `mpsc`. |
| Read-write sharing via DashMap | `InMemoryCookieCache` — many readers, few writers, sharded internally. |
| Cooperative cancellation | Every async path takes `tokio::time::timeout` budgets; no detached tasks. |
| GC tick | Cache GC via `tokio::time::interval(Duration::from_secs(60))`. |

## E. Observability patterns

| Pattern | Where |
|---|---|
| Structured logging | `tracing` everywhere; no `println!` outside `px-cli` (lefthook-enforced). |
| Span propagation | `#[tracing::instrument]` on use cases; spans carry `request_id`, `target_domain`, `handler` fields. |
| Metrics ledger | `HandlerMetrics` emitted by every handler regardless of outcome; aggregated to Prometheus by `px-server`. |
| Audit event | Separate stream from logs; one append-only JSON-Lines record per solve attempt. Cookie payloads redacted (per [ADR-0007](../adr/0007-api-key-and-domain-allowlist-guardrails.md)). |

## F. Anti-patterns we explicitly do **not** use

| Avoided | Why |
|---|---|
| `unwrap()` / `expect()` in production | Hides errors; lefthook blocks via `clippy::unwrap_used`. |
| `panic!` for control flow | Same — use `AppError::InternalError`. |
| Inline comments | Names and types document; rationale lives in ADRs. Per axum-best-practice rule #2. |
| God services | Each use case is one file with one public `execute()`. |
| Singletons | `Arc<dyn Trait>` in composition root; no statics for behavior. |
| `lazy_static!` / `once_cell` for behavior | Configuration via constructor injection only. |
| Hidden global config | Every behavioral knob is on the use case struct or its config DTO. |
| Macros where a function would do | Macros only for `tracing` spans, derive macros, and the `Validated<T>` extractor. |
| Trait inheritance (deep hierarchies) | Flat trait surface; composition over inheritance. |
| `Arc<Mutex<HashMap<_,_>>>` | Use `DashMap` or message-passing instead. |

## Pattern-to-ADR map

| Pattern | Documented in |
|---|---|
| Clean Architecture | [ADR-0006](../adr/0006-axum-clean-architecture-layout.md) |
| Chain of Responsibility / Strategy (handlers) | [ADR-0014](../adr/0014-challenge-pipeline-architecture.md) |
| Null Object (stub handlers + native) | [ADR-0010](../adr/0010-defer-native-sensor-generator.md), [ADR-0015](../adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md) |
| Repository + Adapter (cache) | [ADR-0008](../adr/0008-dashmap-default-redis-optional-cache.md) |
| Object Pool + Decorator (Chromium + stealth) | [ADR-0004](../adr/0004-chromiumoxide-as-browser-driver.md), [ADR-0005](../adr/0005-manual-cdp-stealth-patches.md) |
| API-key + allowlist + audit | [ADR-0007](../adr/0007-api-key-and-domain-allowlist-guardrails.md) |
| Runtime appId detection (Template Method input) | [ADR-0009](../adr/0009-runtime-px-appid-detection.md) |
| `_pxhd` parser + UUIDv1 synth (Value Object + Builder) | [ADR-0012](../adr/0012-pxhd-parser-and-synth.md) |
