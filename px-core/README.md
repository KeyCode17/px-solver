# pxsolver-core

Pure domain types shared across the px-solver crates — no I/O, no framework.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-core = "1.9"
```

> **The package name and the crate name differ.** You depend on `pxsolver-core`, but you `use px_core::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

| Type | Purpose |
|---|---|
| `SolveRequest` | A solve to perform: `url`, optional `proxy`, optional `fingerprint` |
| `PxCookieBundle` / `NamedCookie` / `CookieJarDelta` | The `_px3` bundle a solve returns, with expiry |
| `CacheKey` | Domain + app id + fingerprint/egress key |
| `PxAppId` | Validated PerimeterX app id (`PX` + 8 chars) |
| `Fingerprint` | Browser fingerprint the solver presents |
| `ApiKeyHash`, `NamedToken`, `PxDetection`, `SolveOutcome` | Supporting value types |

## Example

```rust
use px_core::SolveRequest;

// Name the egress you will send downstream traffic through: a _px3 bundle is
// bound to the IP that earned it.
let request = SolveRequest::new("https://example.com/")
    .with_proxy("socks5://127.0.0.1:9050");
```

Absent optionals are omitted from the wire rather than serialized as `null`.

## Notes

This crate has no dependency on any other px-solver crate — everything else depends on it.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
