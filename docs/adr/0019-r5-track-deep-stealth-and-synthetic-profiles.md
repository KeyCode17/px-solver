# 0019. R5 research track: deep stealth + synthetic profiles for CF-fronted PX

- **Date:** 2026-05-16
- **Status:** Closed 2026-05-17 — R5.1/R5.2/R5.5/R5.6 superseded by [ADR-0020](0020-adopt-camoufox-via-fantoccini-geckodriver.md); R5.3/R5.4 deferred (Camoufox subsumes for current targets); R5.7/R5.8/R5.9 delivered. See [`docs/retro/r5-retrospective.md`](../retro/r5-retrospective.md).
- **Deciders:** KeyCode17
- **Related:** ADR-0006, ADR-0013, ADR-0017, ADR-0018, ADR-0020, ADR-0021, ADR-0022, ADR-0023

## Context

ADR-0018 reclassified `pedidosya.com.ar` from MVP canary to a
post-v1.0.0 stretch goal because the site is fronted by Cloudflare
Bot Management. Live probing on 2026-05-16 confirmed that CF refuses
to even render a Turnstile widget for our chromiumoxide-driven
browser regardless of headless mode, indicating the CF risk score
is driven by browser-fingerprint / behavioral signals, not by
TLS/HTTP/2 fingerprinting (which a real Chromium passes by default).

Other CF-fronted PX targets exist in the wild (Argentinian e-commerce,
several US delivery / SaaS dashboards). Treating each as its own
ad-hoc bypass effort is not sustainable. The project needs a
documented, named research track for the engineering required to
reliably defeat CF Bot Management upstream of PX, without requiring
residential proxies.

## Decision

Open an explicit R5 research track named **"deep stealth + synthetic
profiles"**. The scope below is committed to as a single coherent
work-stream, but each item ships as its own PR/commit cluster per
ADR-0017's R-track no-version-bump rule.

### R5 scope

| Sub-phase | Deliverable | Notes |
|-----------|-------------|-------|
| **R5.1** | Launch-flag stealth in `ChromiumoxidePool` | `--disable-blink-features=AutomationControlled`, drop `--enable-automation`, set realistic `User-Agent`, viewport, locale. |
| **R5.2** | Stealth bundle v2 JS patches | Beyond the current 8: permissions API hardening, WebGL renderer/vendor realism, MimeType array, plugin descriptions, audio context noise refinement, `Notification.permission`, `chrome.app`, `chrome.csi`, more `navigator` keys (`hardwareConcurrency`, `deviceMemory`, `connection.effectiveType`). |
| **R5.3** | Human-like mouse paths | Bezier-curve mouse-move helper that drives `Input.dispatchMouseEvent` along plausible trajectories with timing distributions matching real users, used for any synthetic click (Turnstile, native buttons). |
| **R5.4** | `px-profiles` crate | Synthetic Chrome `--user-data-dir` generator: History/Cookies SQLite seeding, Preferences/Local State JSON, locale + timezone per-target geo template, age 30-180 days. Operator can opt in per harvest. |
| **R5.5** | Chromium binary patcher | Read `/usr/bin/chromium-browser` (or system Chromium), rewrite `cdc_*` markers and `HeadlessChrome` strings, write a patched copy to a cache dir, launch from there. Per-Chromium-version handling. |
| **R5.6** | Cross-origin iframe interaction | Robust Turnstile widget locator (poll iframe attributes + bounding box). CDP click-through with R5.3 mouse paths. Wait for `cf_clearance` cookie before declaring success. |
| **R5.7** | px-cloudflare upgrade | Move the stub crate to a real implementation that recognises CF interstitials, dispatches R5.6 click flow, surfaces `cf_clearance` to the pipeline. |
| **R5.8** | CF-fronted PX integration test | Reinstate `pedidosya.com.ar` as a gated test target. Pass = both `cf_clearance` and `_px3` returned. |

### Out of scope for R5

- Residential proxy rotation (operator policy).
- Defeating CF interactive challenges that exceed Turnstile checkbox
  (image-grid puzzles, hCaptcha fallback). If CF promotes the
  challenge level, the run logs the escalation and aborts.
- Defeating PX bot-detection that runs at strictness levels beyond
  the standard `_px3` / sensor / challenge cycle handled by
  `px-perimeterx` + Phase 02 work.

## Alternatives considered

- **Skip R5, accept that CF-fronted PX is unreachable** — pros: no
  engineering cost. Cons: leaves a large class of real targets
  permanently unreachable, breaks the "generic PX bypass" promise on
  any CF customer. Rejected.
- **Wrap an existing tool (nodriver via Python sidecar, etc.)** —
  pros: faster to a working bypass. Cons: pulls Python into the
  runtime, complicates packaging, and the project is explicitly
  Rust-only per ADR-0001. Rejected.
- **Move to a different browser driver (playwright-rust,
  rust-headless-chrome)** — pros: some have more stealth defaults.
  Cons: each has its own CDP type-version drift problem (this
  session debugged exactly that on chromiumoxide 0.7 → 0.9), and
  rewriting the harvester is more work than building patches on
  top of chromiumoxide. Rejected for v1.0.0; revisit if R5 stalls.

## Consequences

- **Positive:**
  - CF-fronted PX targets become reachable without paid proxy
    infrastructure.
  - The synthetic-profile crate is reusable for future operator-
    isolation features (one profile per API key, per geographic
    region, etc.) entirely outside the CF context.
  - Mouse-path simulation and stealth patches harden the harvester
    against future PX detection escalation too.
- **Negative:**
  - R5 is multi-week engineering. It does not unblock v1.0.0; it
    extends post-v1.0.0 capability.
  - Each R5 sub-phase has its own arms-race exposure. CF updates
    Bot Management ~monthly; sub-phases may need refresh cycles.
  - Patched-Chromium copies on disk add storage overhead + a
    per-Chromium-version maintenance burden.
- **Follow-ups:**
  - Each R5 sub-phase commits land as `feat(...)` or `feat(R5.x):...`
    under the R-track per ADR-0017 (no version bump until a deliverable
    completes a phase milestone).
  - New crate `px-profiles` added to workspace at R5.4.
  - `px-cloudflare` upgraded from stub at R5.7.
  - Re-run pedidosya soak after R5 completes; if it passes, ADR-0018
    can be amended to add pedidosya back to the canary set.
