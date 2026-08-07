# pxsolver-datadome

DataDome handler — detection only, solve is a stub.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-datadome = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-datadome`, but you `use px_datadome::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## Status: stub

`DataDomeHandler` **detects** DataDome (`DD_OPTIONS`, `datadome.co/captcha`, `ddg_datadome`) and
returns `HandlerStatus::NotImplemented` from `solve`
([ADR-0015](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md)).

```rust
use px_datadome::DataDomeHandler;
let handler = DataDomeHandler::new();
```

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
