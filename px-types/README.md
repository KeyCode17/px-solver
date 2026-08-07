# pxsolver-types

The HTTP response envelope every px-solver route returns.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-types = "1.9"
```

> **The package name and the crate name differ.** You depend on `pxsolver-types`, but you `use px_types::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_types::{SingleResponse, ListResponse, PaginationMeta};

// { "data": …, "status": "solved" }
let body = SingleResponse::new(payload, "solved");
```

| Type | Shape |
|---|---|
| `SingleResponse<T>` | `{ data, status }` |
| `ListResponse<T>` | `{ data, meta, status }` |
| `PaginationMeta` | `{ page, per_page, total }` |

## Notes

Wire keys are snake_case. Every route returns one of these — there are no bare payloads.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
