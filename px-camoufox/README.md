# pxsolver-camoufox

Camoufox (patched Firefox) harvester and fetcher, driven over geckodriver.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-camoufox = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-camoufox`, but you `use px_camoufox::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_camoufox::{CamoufoxConfig, CamoufoxPool};

let pool = CamoufoxPool::new(CamoufoxConfig::from_env())?;
```

| Item | Purpose |
|---|---|
| `CamoufoxPool` | Implements both `Harvester` and `Fetcher` over geckodriver + Camoufox |
| `CamoufoxConfig` | Binary paths, locale, headless, timeouts, concurrency — `from_env` |
| `capture_sensor` | XHR-hook capture of live PX sensor payloads, for calibration |

Warm `PersistentSession`s are kept per domain for `/v1/fetch`, so repeat requests reuse a browser
that already holds a coherent cookie jar.

## Egress rotation

Set `PX_PROXIES` to a CSV list and each session takes one round-robin **at spawn**, holding it for
the session's 300s TTL. Distinct egress IPs per domain is therefore
`min(PX_FETCH_MAX_PER_DOMAIN, len(PX_PROXIES))` — not the product.

Per-request harvests use the proxy named on the `HarvestRequest`; they do not draw from the
rotation, because a bundle bound to an IP the caller cannot name is not usable.

## Requires operator-installed binaries

Camoufox and geckodriver are **not** vendored — install them and point `CamoufoxConfig` at them
([ADR-0020](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0020-adopt-camoufox-via-fantoccini-geckodriver.md)).

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
