# pxsolver-turnstile

Cloudflare Turnstile handler — detection only, solve is a stub.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-turnstile = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-turnstile`, but you `use px_turnstile::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## Status: stub

`TurnstileHandler` **detects** Turnstile (`challenges.cloudflare.com/turnstile`, `cf-turnstile`)
and returns `HandlerStatus::NotImplemented` from `solve`. It exists so the pipeline routes
correctly today and the real implementation slots in without a signature change
([ADR-0015](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md)).

```rust
use px_turnstile::TurnstileHandler;
let handler = TurnstileHandler::new();
```

Turnstile's invisible mode is often **incidentally** passed by running a real stealth browser, but
no success rate is committed for it.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
