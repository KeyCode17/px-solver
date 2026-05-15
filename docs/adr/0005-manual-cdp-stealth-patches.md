# 0005. Manual CDP stealth patches over pre-patched browser

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0004, SOW-DEL-003

## Context

A vanilla headless Chromium is trivially detectable by PX's `/init.js` (navigator.webdriver, plugins list, languages, missing `chrome.runtime`, default WebGL vendor `Google Inc.`, deterministic canvas hash). The harvester must blend in.

## Decision

Inject a hand-written stealth patch bundle on every page via `Page.addScriptToEvaluateOnNewDocument` before any navigation. The bundle covers, at minimum:

- `navigator.webdriver` → `false`
- `navigator.languages` → realistic locale list
- `navigator.plugins` and `navigator.mimeTypes` → realistic non-empty arrays
- `window.chrome.runtime` → present
- WebGL `UNMASKED_VENDOR_WEBGL` / `UNMASKED_RENDERER_WEBGL` → realistic GPU pair
- Canvas `toDataURL`/`getImageData` → noise injected at sub-pixel scale
- Audio API → noise on `AnalyserNode.getFloatFrequencyData`
- `permissions.query({name: 'notifications'})` → matches `Notification.permission`

## Alternatives considered

- **Camoufox / patched browser binary.** Better fingerprint out of the box, but harder to package (custom binary per OS), slower to update, and Camoufox is Firefox-based which raises the questions about PX's Chromium-shaped expectations.
- **Skip stealth initially.** Reject — fails the canary in single-digit seconds and pollutes the IP reputation.
- **Use a Rust port of puppeteer-extra-stealth.** None exists at production quality; would require porting effort comparable to writing the bundle ourselves.

## Consequences

- **Positive:** packaging stays simple (system Chromium); patches are auditable; can extend per-target on demand.
- **Negative:** patch maintenance burden; PX upgrades may surface new detection vectors that the bundle must catch up to.
- **Follow-ups:** the stealth bundle lives in `px-harvester/src/infrastructure/stealth/`. Each patch is its own file (axum-best-practice 200-LOC rule).
