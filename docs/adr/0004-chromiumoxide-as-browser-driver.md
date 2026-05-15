# 0004. chromiumoxide as browser driver

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0002, ADR-0003, SOW-DEL-003

## Context

Tier 2 (browser harvester) needs to drive Chromium from inside an async Rust service. Requirements: speak CDP (not WebDriver), share a Tokio runtime with the rest of the server, allow per-context stealth patches, expose cookies post-load.

## Decision

Use [`chromiumoxide`](https://crates.io/crates/chromiumoxide) as the CDP client. Spawn one `Browser` per worker, isolate per-solve state via `Page` and an `incognito` `BrowserContext`.

## Alternatives considered

- **`headless_chrome`.** Sync API on top of CDP; fewer ergonomic primitives for async pools.
- **`thirtyfour` / WebDriver.** Requires running an external `chromedriver`; sets `navigator.webdriver = true` by default, which PX detects trivially.
- **Spawning external Playwright/Node workers.** Maximum stealth plugin reuse but adds a second runtime, IPC, and dependency tree the maintainer wants to avoid.

## Consequences

- **Positive:** native async; direct access to CDP for `Page.addScriptToEvaluateOnNewDocument` (see ADR-0005); no `navigator.webdriver` taint by default; small dependency tree.
- **Negative:** smaller stealth-plugin ecosystem than puppeteer-extra; canvas/WebGL FP randomization must be implemented by hand; less battle-tested at very high concurrency than puppeteer.
- **Follow-ups:** ADR-0005 details the stealth patch suite.
