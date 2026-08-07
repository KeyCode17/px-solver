# pxsolver-errors

One `AppError` for the whole workspace, with its Axum `IntoResponse`.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-errors = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-errors`, but you `use px_errors::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_errors::AppError;

fn parse(url: &str) -> Result<(), AppError> {
    Err(AppError::BadRequest("Invalid url".into()))
}
```

`AppError` covers `BadRequest`, `Unauthorized`, `Forbidden`, `NotFound`, `Conflict`,
`ValidationError` and `InternalError`, each mapping to its status code and to a stable
machine-readable code (`bad_request`, `internal_error`, …) in the serialized `ErrorResponse`.

## Notes

The payload string **is** the user-facing message — it is returned verbatim, so it starts a
sentence and carries no internal detail. Diagnostics belong in the logs.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
