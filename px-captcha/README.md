# pxsolver-captcha

hCaptcha / reCAPTCHA handler — detection only, solve is a stub.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-captcha = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-captcha`, but you `use px_captcha::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## Status: stub

`CaptchaHandler` **detects** hCaptcha and reCAPTCHA (`hcaptcha.com/1/api.js`,
`recaptcha/api.js`, the `h-captcha` / `g-recaptcha` classes) and returns
`HandlerStatus::NotImplemented` from `solve`
([ADR-0015](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0015-v1-ships-pipeline-with-perimeterx-handler-only.md)).

```rust
use px_captcha::CaptchaHandler;
let handler = CaptchaHandler::new();
```

px-solver does not integrate a human-solving service, and this crate will not become one.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
