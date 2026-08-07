# pxsolver-validation

A `Validated<T>` Axum extractor plus the `Validate` trait behind it.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-validation = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-validation`, but you `use px_validation::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_validation::{Validate, Validated};

impl Validate for CreateThing {
    fn validate(&self) -> Result<(), AppError> { /* … */ Ok(()) }
}

async fn handler(Validated(body): Validated<CreateThing>) -> impl IntoResponse { /* … */ }
```

Deserialization and validation both happen at the edge, so handlers receive a value that is
already known good.

## Notes

Validation failures surface as `AppError::ValidationError` → HTTP 400 with the standard error
envelope.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
