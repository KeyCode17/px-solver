# 0002. Use Rust (edition 2024) for px-solver

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** SOW §1, SOW-DEL-001

## Context

The solver is a long-running network service holding a worker pool of Chromium instances. It must be memory-safe under concurrency, have predictable resource usage, and run unattended on a small VPS. The maintainer also wants the codebase to act as a learning vehicle for systems-grade Rust.

## Decision

Implement the entire workspace in Rust (edition 2024), with `tokio` as the async runtime and `axum` as the HTTP framework.

## Alternatives considered

- **Python (FastAPI + Playwright).** Easiest scraping ecosystem, but GIL + Playwright subprocess overhead make the worker pool less efficient, and dependency surface for a long-running service is larger.
- **Node.js (Fastify + Playwright).** Best Playwright support and easiest stealth plugins, but the maintainer wants Rust; runtime memory is higher than Rust at idle.
- **Go (chromedp / rod).** Lower memory than Node, comparable to Rust, but the maintainer prefers Rust's stricter type system for the dual-use guardrails (newtypes for `PxAppId`, `ApiKeyHash`, etc.).

## Consequences

- **Positive:** memory safety; small steady-state footprint; strong typing for security-sensitive types; first-party `chromiumoxide` CDP client.
- **Negative:** smaller stealth-plugin ecosystem than Node — stealth patches must be written by hand (see ADR-0005); slower iteration speed compared to Python.
- **Follow-ups:** ADR-0004 picks the browser driver; ADR-0005 picks the stealth approach; ADR-0006 picks the HTTP layout.
