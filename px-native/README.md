# pxsolver-native

Native `_px3` sensor synthesis — build the payload locally instead of driving a browser.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-native = "1.9"
```

> **The package name and the crate name differ.** You depend on `pxsolver-native`, but you `use px_native::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_native::{NativeSolver, SensorNativeSolver, SolveContext};
use px_native::profile::TenantProfile;

let solver = SensorNativeSolver::new(client, Arc::new(TenantProfile::load(path)?));
let ctx = SolveContext::new(url, app_id, fingerprint)
    .with_proxy(Some("socks5://127.0.0.1:9050".into()));
let bundle = solver.solve(&ctx).await?;
```

| Item | Purpose |
|---|---|
| `NativeSolver` | The port: context in, `PxCookieBundle` out |
| `SensorNativeSolver` | Builds the sensor payload, POSTs it, parses `Set-Cookie` |
| `cipher` | The `vP` encryption of the sensor batch |
| `events` | The `[{t, d}, …]` event grammar (`default_batch`) |
| `profile::TenantProfile` | Per-tenant app id, sensor path and fallbacks |
| `NativeFirstHandler` | Decorator: try native, fall back to a browser handler on failure |

## Notes

This path is **orders of magnitude cheaper** than a browser — no process spawn, one HTTP POST. Its
accuracy depends on `default_batch` matching the tenant's real event grammar, which is calibrated
against captures; see
[`docs/runbook-native-bypass.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/runbook-native-bypass.md)
and [ADR-0024](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0024-activate-native-px3-sensor-synthesis.md).

Unlike the browser paths, this one runs on `reqwest` and **does** support proxy authentication, so
`user:pass@` in the proxy URL works here.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
