# 0009. Generic runtime PX appId detection

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0003, SOW-DEL-002

## Context

PerimeterX is deployed as a first-party reverse proxy on the customer's own domain. Each customer gets a randomized `appId` (e.g. PedidosYa's `eT15wiaE`) and serves init JS from `/<appId>/init.js`. Hardcoding `eT15wiaE` would make the solver useless for any other target.

## Decision

`px-detector` extracts the `appId` and challenge paths at runtime by inspecting:

1. `window._pxAppId` — set by the init script as `PX<appId>`.
2. Script tags with `src` matching `/<token>/init.js` or `/<token>/captcha.js` where `<token>` matches `[A-Za-z0-9]{6,12}`.
3. The 403 block-page HTML, which contains the same paths and a `PX2` marker token.

The result is a `PxDetection { app_id, init_js_path, collector_path, mode: ReverseProxy | Hosted }` value that the harvester consumes.

## Alternatives considered

- **Hardcode per-target.** Trivially correct but breaks the "generic solver" goal in SOW §2.
- **External config of `appId` per target.** Operator burden; also unnecessary because the value is reliably extractable.
- **HTTP-only detection (no JS execution).** Sufficient for the appId itself, but cannot validate that init.js actually loaded — keeping the JS-side check is cheap once Chromium is already running.

## Consequences

- **Positive:** the solver works against any PX-protected site without code changes; per-target work is reduced to an allowlist entry.
- **Negative:** detection logic must be maintained as PX changes its sentinel globals (history shows `_pxAppId` has been stable for 2+ years).
- **Follow-ups:** the detector exposes `Detected::No` for non-PX pages so the server can return `409 Conflict` instead of attempting a futile solve.
