# pxsolver-cloudflare

Cloudflare interstitial handler, backed by a Camoufox harvester.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-cloudflare = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-cloudflare`, but you `use px_cloudflare::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_cloudflare::CloudflareHandler;

let handler = CloudflareHandler::with_harvester(camoufox_pool);
```

Detects `cdn-cgi/challenge-platform`, `cf-mitigated` and `cf_clearance` markers, re-harvests the
URL through the supplied harvester, and returns `cf_clearance` / `__cf_bm` plus any PX cookies the
same fetch happens to set.

`extract_session_cookies` / `is_session_cookie` are exposed for reuse.

## Notes

Constructed without a harvester (`CloudflareHandler::new()`), `solve` returns `NotImplemented`
rather than pretending — detection still works. Pair it with
[`pxsolver-camoufox`](https://crates.io/crates/pxsolver-camoufox) for the bypass path
([ADR-0020](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0020-adopt-camoufox-via-fantoccini-geckodriver.md)).

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
