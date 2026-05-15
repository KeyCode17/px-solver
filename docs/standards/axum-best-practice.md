# Axum Backend Best Practices

> Project-wide coding standard for `px-solver`. Referenced by [ADR-0006](../adr/0006-axum-clean-architecture-layout.md). Any deviation requires a new ADR.

## Core Principles

- Max 200 LOC per file.
- No code comments. Names and types document themselves; cross-file context lives in ADRs.
- No `unwrap()` / `expect()` in production code. Use `?` or return an `AppError` variant. Test code may use `expect` with a descriptive message.
- Clean Architecture: `domain` → `application` → `infrastructure`. Domain layer must compile without any infra dependency.
- Single responsibility per file.
- Reusability-first design: shared types in `px-types`, shared validation in `px-validation`, shared errors in `px-errors`.

## Project Structure (`px-` prefix)

```
px-server/              # Composition root, binary entry
px-auth/                # API-key + allowlist middleware
px-detector/            # PX detection (HTML, JS globals, block-page)
px-harvester/           # Vendor-agnostic chromiumoxide CDP pool + stealth core
px-pipeline/            # ChallengeHandler trait + orchestrator (ADR-0014)
px-perimeterx/          # PerimeterX handler (v1 marquee)
px-cloudflare/          # Cloudflare handler stub (v2 — ADR-0015)
px-turnstile/           # Turnstile handler stub (v2)
px-captcha/             # hCaptcha / reCAPTCHA gateway stub (v2)
px-datadome/            # DataDome handler stub (v2)
px-cache/               # Cookie cache (DashMap / Redis)
px-native/              # Native sensor generator (stub for v1)
px-core/                # Pure domain types shared across crates
px-errors/              # Centralized AppError + IntoResponse
px-types/               # Shared response shapes (SingleResponse, ListResponse, PaginationMeta)
px-validation/          # Validated extractor + validation framework
px-cli/                 # clap-based binary
xtask/                  # Dev automation (cargo xtask bump|check-loc|release|canary) — ADR-0016
```

Handler crates (`px-perimeterx`, `px-cloudflare`, `px-turnstile`, `px-captcha`, `px-datadome`) all depend on `px-pipeline` (trait) and `px-harvester` (shared Chromium pool). They never depend on each other.

`px-research/` lives at the repository root alongside the `px-*` crates, but it is **not** a Cargo crate. It is a plain directory of captures, notes, and field maps — no `Cargo.toml`, no Rust source, no participation in the workspace. See [ADR-0013](../adr/0013-re-methodology-and-scope.md) and [`px-research/README.md`](../../px-research/README.md). If research tooling ever matures into a reusable library, promote it to a regular `px-*` crate with its own ADR; do not retrofit `px-research/` into a crate.

`xtask/` is the **other** documented exception to the `px-` prefix. It is the universal Cargo convention for in-repo dev automation, invoked as `cargo xtask <subcommand>` (bump, check-loc, release, canary, phase). See [ADR-0016](../adr/0016-xtask-for-dev-automation.md). The crate is `publish = false`, inherits workspace metadata, and is exempt from the no-`println!` / no-`unwrap()` / 200-LOC rules below (it is a dev tool, not production code).

> Note: no `px-database/` or `px-migration/` in this project — there is no persistent DB. Audit log is append-only file-based, owned by `px-server`.

## Module Structure

Each feature crate follows the same Clean Architecture skeleton:

```
px-<feature>/src/
├── lib.rs
├── domain/
│   ├── mod.rs
│   ├── <entity>.rs        # Entity definition
│   └── repository.rs      # Repository trait (or "port")
├── application/
│   ├── mod.rs
│   └── <use_case>.rs      # One use case per file
└── infrastructure/
    ├── mod.rs
    ├── http/
    │   ├── handlers.rs
    │   ├── routes.rs
    │   └── dto.rs
    └── persistence/       # Or external/, browser/, etc., depending on crate
        └── <impl>.rs
```

## Error Handling

Centralized `AppError` in `px-errors`:

```rust
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    ValidationError(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg)        => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg)      => (StatusCode::BAD_REQUEST, msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg)    => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg)       => (StatusCode::FORBIDDEN, msg),
            AppError::Conflict(msg)        => (StatusCode::CONFLICT, msg),
            AppError::InternalError(msg)   => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(ErrorResponse { message })).into_response()
    }
}
```

px-solver maps PX-specific failures into this enum:

- `PxNotDetected` → `AppError::Conflict`
- `AllowlistDenied` → `AppError::Forbidden`
- `KeyInvalid` → `AppError::Unauthorized`
- `HarvesterTimeout` → `AppError::InternalError`
- `ChromiumUnavailable` → `AppError::InternalError`

## Repository / Port Pattern

Domain layer defines the trait:

```rust
#[async_trait]
pub trait CookieCache: Send + Sync {
    async fn get(&self, key: &CacheKey) -> Result<Option<PxCookieBundle>, AppError>;
    async fn put(&self, key: CacheKey, bundle: PxCookieBundle) -> Result<(), AppError>;
    async fn invalidate(&self, domain: &str) -> Result<(), AppError>;
}
```

Infrastructure implements it:

```rust
pub struct InMemoryCookieCache { /* ... */ }

#[async_trait]
impl CookieCache for InMemoryCookieCache {
    async fn get(&self, key: &CacheKey) -> Result<Option<PxCookieBundle>, AppError> { /* ... */ }
    async fn put(&self, key: CacheKey, bundle: PxCookieBundle) -> Result<(), AppError> { /* ... */ }
    async fn invalidate(&self, domain: &str) -> Result<(), AppError> { /* ... */ }
}
```

## Use Case Pattern

One file per use case. Use cases never reach across to other use cases — they compose ports.

```rust
pub struct SolvePx {
    detector: Arc<dyn PxDetector>,
    cache:    Arc<dyn CookieCache>,
    harvester: Arc<dyn Harvester>,
    audit:    Arc<dyn AuditSink>,
}

impl SolvePx {
    pub fn new(
        detector: Arc<dyn PxDetector>,
        cache: Arc<dyn CookieCache>,
        harvester: Arc<dyn Harvester>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self { detector, cache, harvester, audit }
    }

    pub async fn execute(&self, input: SolveInput) -> Result<PxCookieBundle, AppError> { /* ... */ }
}
```

## Handler Pattern

```rust
pub async fn solve_handler(
    Extension(use_case): Extension<Arc<SolvePx>>,
    Validated(payload): Validated<SolveRequest>,
) -> Result<Json<SingleResponse<SolveResponse>>, AppError> {
    let bundle = use_case.execute(payload.into()).await?;
    Ok(Json(SingleResponse::new(bundle.into(), "Solved")))
}
```

## Response Types (`px-types`)

```rust
pub struct SingleResponse<T> {
    pub data: T,
    pub message: String,
}

pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

pub struct PaginationMeta {
    pub page: u32,
    pub size: u32,
    pub total: u64,
}
```

## Rules (binding)

1. **Domain layer has no external dependencies.** Only `std`, `serde`, and other domain crates may be in its `Cargo.toml`.
2. **Repository / port traits live in `domain`. Implementations live in `infrastructure`.**
3. **One use case per file** in `application/`.
4. **DTOs live only in `infrastructure/http/dto.rs`.** Domain types never carry `serde` derives that leak transport concerns.
5. **Dependency injection uses `Arc<dyn Trait>`** at composition time.
6. **Handlers return `Result<_, AppError>`.** No panics, no `unwrap`.
7. **Input is validated via the `Validated<T>` extractor** from `px-validation`.
8. **No `println!` / `eprintln!`** outside `px-cli` and `xtask`. Use `tracing` macros.
9. **No `unwrap()` / `expect()` in production paths.** Tests and `xtask/` may use them.
10. **Files over 200 LOC must be split.** Enforced locally via Lefthook (`scripts/check_loc.sh`) per [ADR-0011](../adr/0011-use-lefthook-for-git-hooks.md); CI re-runs the same check as a backstop.
11. **`--no-verify` is forbidden.** Bypassing local hooks defeats the standard and pollutes history with `fix lint` commits.
