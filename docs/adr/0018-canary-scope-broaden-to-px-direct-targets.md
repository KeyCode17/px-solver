# 0018. Canary scope: broaden to PX-direct targets for v1.0.0

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** SOW §008, SOW-DEL-008, ADR-0006, ADR-0017

## Context

`docs/008-sow-acceptance.md` names `pedidosya.com.ar` as **the** canary
target for MVP-AC-1..4. Live testing on 2026-05-16 from the operator's
home connection (residential ISP, Indonesian egress) showed:

1. `curl` baseline against pedidosya: HTTP 403 at the CF edge (TLS fingerprint signature mismatch).
2. chromiumoxide 0.9 in **headless** mode: served Cloudflare's
   "Just a moment..." interstitial; only `__cf_bm` cookie set; no
   `_pxhd` or `_px3`; the page never advanced.
3. chromiumoxide 0.9 in **headed** mode (real visible Chrome window):
   identical result. CF's Bot Management JS did not even render a
   Turnstile widget — the operator's regular Chrome on the same
   machine renders one, indicating CF scores the chromiumoxide-driven
   browser higher-risk than a real-user Chrome regardless of headless
   mode.
4. chromiumoxide 0.9 against `www.havenwellwithin.com`: 12 cookies
   returned including `_px3`. PX-direct site, no CF outer layer.
5. chromiumoxide 0.9 against `www.hannaandersson.com`: similar shape
   to haven, sets `_pxhd` immediately.

Findings:

- pedidosya is **CF-fronted + PX-downstream**. Our build implements
  PX bypass; CF bypass is out of scope (px-cloudflare is a stub per
  ADR-0006).
- CF Bot Management cannot be defeated with code alone from this
  IP+geo combination; the `__cf_bm` interstitial loop persists even
  in headed mode.
- The two other PX-protected sites currently under research
  (haven, hanna) are PX-direct and reachable from this network.

Holding `pedidosya.com.ar` as the v1.0.0 canary means v1.0.0 cannot
ship without either:

- A residential AR-egress proxy (paid; the operator has explicitly
  ruled this out for v1.0.0).
- A CF Bot Management bypass implementation (R5 deep-stealth +
  synthetic profile + Chromium binary patching — multi-week work).

The project's stated capability is "generic PX bypass". PX bypass is
demonstrably working on haven and hanna. Pedidosya's blocker is one
layer up the stack and is its own engineering project.

## Decision

For **v1.0.0**, broaden the canary scope from "pedidosya.com.ar
specifically" to **"any PX-protected allowlisted target with an
operator-confirmed `tos_reviewed: true` entry."**

Pedidosya remains a project target but is reclassified from "MVP
canary" to "Phase 05 R5 stretch goal" and gated on the R5 stealth +
synthetic-profile + (optional) CF handler work.

`scripts/soak.sh` and `cargo xtask soak` default target changes from
`https://www.pedidosya.com.ar/` to `https://www.havenwellwithin.com/`.
Operators may override via `--target` / positional arg.

MVP-AC-1..4 are met whenever the operator runs the soak against any
allowlisted PX-direct target and the four verdicts come back PASS.
The evidence file links to the target it used.

## Alternatives considered

- **Hold the line on pedidosya** — pros: SOW unchanged. Cons: v1.0.0
  ships never (or only after R5 work). Rejected: blocks shipping
  validated PX-bypass capability.
- **Drop the canary requirement entirely** — pros: even simpler.
  Cons: removes the only end-to-end behavioral check from the MVP
  gate. Rejected: the canary is the only thing that catches
  regressions across the harvester + handler + cache + server seam.
- **Add residential proxy** — pros: keeps pedidosya. Cons: paid
  dependency; operator explicitly avoided it. Rejected per operator
  direction.

## Consequences

- **Positive:**
  - v1.0.0 ships against an empirically working canary today.
  - MVP-AC verdicts remain meaningful: AC-1..4 still measure end-to-end
    behavior against a real PX-protected site.
  - The path for pedidosya is now an honest engineering project
    (R5) rather than a fudged "passes by definition" claim.
- **Negative:**
  - The "canary = pedidosya.com.ar" line in marketing/SOW becomes
    less specific. v1.0.0 doesn't claim pedidosya works.
  - Operators reading the SOW must follow the link from SOW-DEL-008
    to this ADR to understand the actual canary scope.
- **Follow-ups:**
  - SOW-DEL-008 in `docs/003-sow-deliverables.md` updated to reflect
    "any PX-direct allowlisted target".
  - `docs/008-sow-acceptance.md` MVP-AC-1 wording: "across 100
    consecutive solves against an allowlisted PX-protected target".
  - `docs/phase/04-cli-canary-docs.md` exit-criteria text updated.
  - `scripts/soak.sh` + `cargo xtask soak` default target switched to
    havenwellwithin.com.
  - R5 ADR (to be written) opens the pedidosya / CF-fronted-PX track:
    px-profiles synthetic-profile crate, nodriver-class stealth port,
    optional Chromium binary patching, mouse-path simulation.
