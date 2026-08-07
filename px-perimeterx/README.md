# pxsolver-perimeterx

The PerimeterX `ChallengeHandler` — px-solver's marquee path.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-perimeterx = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-perimeterx`, but you `use px_perimeterx::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_perimeterx::PerimeterxHandler;

let handler = PerimeterxHandler::new(harvester);
```

| Item | Purpose |
|---|---|
| `PerimeterxHandler` | Detects PX, harvests through a real browser, extracts the `_px*` bundle |
| `SolvePx` | The use case behind it, usable without the pipeline |
| `PxHd` / `PxHdUrl` / `Sid` | Parsed PX identifiers with their own error types |

## Notes

This handler defeats PX by **avoidance** — a real browser passes the challenge legitimately and
the resulting cookies are extracted. Native sensor synthesis lives in
[`pxsolver-native`](https://crates.io/crates/pxsolver-native).

The request's proxy is forwarded into the harvest, so the returned bundle is bound to the IP the
caller named.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
