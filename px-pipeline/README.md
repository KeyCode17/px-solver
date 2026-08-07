# pxsolver-pipeline

The `ChallengeHandler` port and the ordered pipeline that routes a page to the right vendor handler.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-pipeline = "1"
```

> **The package name and the crate name differ.** You depend on `pxsolver-pipeline`, but you `use px_pipeline::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_pipeline::{ChallengeHandler, PageHtml, Pipeline, SolveAction};

let pipeline = Pipeline::new(vec![perimeterx, cloudflare]);
let action = SolveAction::new(PageHtml::new(url, html))
    .with_proxy(Some("socks5://127.0.0.1:9050".into()));
let outcomes = pipeline.run(&action).await?;
```

| Item | Purpose |
|---|---|
| `ChallengeHandler` | `detects(&PageHtml)` + `solve(&SolveAction)` — one impl per protection vendor |
| `SolveAction` | What to solve: the page plus the egress proxy for this request |
| `Pipeline` | Runs handlers in order, stopping at the first solve (configurable) |
| `HandlerOutcome` / `HandlerStatus` | Solved / Skipped / NotImplemented, with metrics |
| `Fetcher` | Port for running one arbitrary HTTP request inside a handler-owned browser session |

## Notes

`detects` sees only the page — detection is content-only. `solve` additionally receives the
egress, because that is a property of the request rather than of the page.

## v1.9.0 is source-breaking

`ChallengeHandler::solve` and `SolveDispatcher::solve` take a `SolveAction` / `SolveRequest`
instead of `&PageHtml` / `&str`, so the per-request egress proxy can reach the browser:

```rust
// before
async fn solve(&self, page: &PageHtml) -> Result<HandlerOutcome, AppError>
// after — action.page is the old argument, action.proxy the requested egress
async fn solve(&self, action: &SolveAction) -> Result<HandlerOutcome, AppError>
```

Rationale: [ADR-0025](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0025-egress-proxy-propagation-contract.md).

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
