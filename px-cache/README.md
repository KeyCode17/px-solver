# pxsolver-cache

The cookie-bundle cache port, with a DashMap-backed in-memory implementation.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-cache = "1.9"
```

> **The package name and the crate name differ.** You depend on `pxsolver-cache`, but you `use px_cache::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_cache::{CookieCache, InMemoryCookieCache};

let cache = InMemoryCookieCache::default();
if let Some(bundle) = cache.get(&key).await? {
    // still within the bundle's own expiry
}
```

| Item | Purpose |
|---|---|
| `CookieCache` | The port: `get` / `put`, async, object-safe |
| `InMemoryCookieCache` | Default implementation over `DashMap`, evicting on bundle expiry |
| `CacheMetrics` | Hit / miss / eviction counters |

## Notes

Entries expire with the bundle itself rather than on a separate timer — a cached `_px3` that has
outlived its own `expires_at` is never served.

The key (`px_core::CacheKey`) includes the egress, so bundles earned through different proxies do
not collide. See [ADR-0025](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0025-egress-proxy-propagation-contract.md).

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
