# Phase 02 — Harvester

- **Window:** 2026-05-27 → 2026-06-03
- **Status:** Not started
- **SOW-DEL covered:** SOW-DEL-003
- **Owner:** KeyCode17

## Goal

Make Chromium pass PX reliably. Tier-2 of the hybrid (see [ADR-0003](../adr/0003-hybrid-solver-architecture.md)). At the end of this phase, a single function returns a valid `PxCookieBundle` for any allowlisted PX-protected URL.

This phase can start in parallel with the tail of Phase 01 once the detector contract is frozen.

## Entry criteria

- [ ] Phase 00 marked **Done**.
- [ ] `PxDetection` and `PxCookieBundle` types from Phase 01 are stable (no breaking changes for the duration of this phase).
- [ ] System Chromium present at a known path; CI runner image has Chromium installed.

## Deliverable status

- [ ] SOW-DEL-003 — Browser harvester pool (`px-harvester`)
  - [ ] `px-harvester::domain::Harvester` trait with `harvest(&self, ctx: HarvestContext) -> Result<PxCookieBundle, AppError>`.
  - [ ] `ChromiumPool` worker pool backed by `chromiumoxide`, bounded by `tokio::sync::Semaphore`.
  - [ ] Per-worker lifecycle: spawn, warm-up, N solves, restart on N or on error.
  - [ ] Stealth bundle injected via `Page.addScriptToEvaluateOnNewDocument` (one patch per file under `infrastructure/stealth/`):
    - [ ] `navigator-webdriver.js`
    - [ ] `navigator-languages.js`
    - [ ] `navigator-plugins.js`
    - [ ] `chrome-runtime.js`
    - [ ] `webgl-vendor-renderer.js`
    - [ ] `canvas-noise.js`
    - [ ] `audio-noise.js`
    - [ ] `permissions-query.js`
  - [ ] Cookie extraction reads `_px3`, `_pxhd`, `_pxvid`, `_pxde` plus session-cookies the request needs.
  - [ ] `User-Agent` returned alongside cookies; must match what Chromium actually sent.

## Exit criteria

1. **Canary success rate:** ≥ 95% solve success across 50 consecutive solves of `https://www.pedidosya.com.ar/` from a clean residential IP.
2. **Latency:** median solve ≤ 6 s end-to-end (from `harvest()` call to returned bundle).
3. **No zombies:** after 100 solve cycles, `pgrep -c chrome` returns ≤ (configured pool size).
4. **No fingerprint regressions:** running [bot.sannysoft.com](https://bot.sannysoft.com) inside the harvester shows green on `WebDriver`, `WebDriver Advanced`, `Chrome (New)`, `Plugins Length`, `Languages`, `WebGL Vendor`, `WebGL Renderer` (manual one-off check, screenshot in `## Log`).
5. **No `unwrap()` / `expect()`** in non-test code (clippy + `scripts/check_forbidden.sh`).
6. Run `cargo xtask phase 02` → version becomes `0.3.0` per [ADR-0017](../adr/0017-phase-aligned-versioning.md).

## Risks

- **R-02-1:** PX bumps `/init.js` mid-phase and a stealth patch must be added. Mitigation: stealth patches are isolated files; adding one is < 1 hour.
- **R-02-2:** `chromiumoxide` event-loop backpressure under load. Mitigation: cap concurrent solves with the semaphore; benchmark before declaring phase done.
- **R-02-3:** IP reputation drift on test machine. Mitigation: record canary IP in `## Log`; reset to a known-clean IP before running the 50-solve gate.

## Log

_Append-only._
