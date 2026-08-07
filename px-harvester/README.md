# pxsolver-harvester

The vendor-agnostic `Harvester` port plus a stealth-patched Chromium pool.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-harvester = "1.9"
```

> **The package name and the crate name differ.** You depend on `pxsolver-harvester`, but you `use px_harvester::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_harvester::{ChromiumoxidePool, HarvestRequest, Harvester, PoolConfig};

let pool = ChromiumoxidePool::new(PoolConfig::default());
let result = pool
    .harvest(HarvestRequest::new("https://example.com/")
        .with_proxy(Some("socks5://127.0.0.1:9050".into())))
    .await?;
```

| Item | Purpose |
|---|---|
| `Harvester` | The port: drive a real browser at a URL, return HTML + UA + cookies |
| `ChromiumoxidePool` | `chromiumoxide` implementation with a concurrency semaphore and CDP stealth patches |
| `HarvestRequest` / `HarvestResult` | What to harvest and what came back |
| `StealthBundle` | The injected `evaluate_on_new_document` payload |
| `strip_credentials` | Drops `user:pass@` from a proxy URL, with a warning |

## Proxy credentials do not work in a browser

geckodriver's W3C `proxy` capability has no credential field, and Chromium's `--proxy-server`
ignores userinfo without a CDP `Fetch.authRequired` handler. Rather than fail silently,
`strip_credentials` removes them and logs the sanitized URL. Front an authenticated upstream with
a local unauthenticated relay (gost, 3proxy).

`socks5h://` is normalized to `socks5://` for Chromium, which has no such scheme and would
otherwise ignore the spec and go direct.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
