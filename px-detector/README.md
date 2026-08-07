# pxsolver-detector

PerimeterX detection from HTML, JS globals and block-page markers.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-detector = "1.9"
```

> **The package name and the crate name differ.** You depend on `pxsolver-detector`, but you `use px_detector::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_detector::{Detected, Detector, RegexDetector};

match RegexDetector::new().detect(&html) {
    Detected::Yes(app_id) => { /* PX present, app id recovered when exposed */ }
    Detected::No => { /* no PX markers */ }
}
```

`RegexDetector` looks for the captcha/challenge scaffolding, `_px*` cookie names, the
`window._pxAppId` family of globals, and the block-page signatures — returning the app id when
the page exposes one.

## Notes

Detection is content-only and does no I/O, so it is cheap enough to run on every fetched page.

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
