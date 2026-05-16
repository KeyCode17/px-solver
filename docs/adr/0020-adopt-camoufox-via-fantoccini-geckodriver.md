# 0020. Adopt Camoufox via fantoccini + geckodriver (Path C)

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0001 (Rust-only), ADR-0006, ADR-0018, ADR-0019

## Context

ADR-0019 opened R5 with eight sub-phases (stealth launch flags,
stealth bundle v2, mouse paths, synthetic profile, Chromium binary
patcher, iframe interaction, px-cloudflare upgrade, pedidosya test).
Each is a real engineering project with an arms-race tail against
Cloudflare Bot Management.

[Camoufox](https://github.com/daijro/camoufox) is a community-maintained
Firefox fork that patches Firefox at the C++ source level to inject
anti-detection across WebGL, canvas, audio, fonts, navigator
properties, locale, timezone, and OS spoofing. It is designed
specifically to defeat CF, DataDome, and Akamai bot management and
is tested against those targets in production by its maintainer.

Adopting Camoufox subsumes the bulk of R5:

- R5.1 (launch flags) — handled at engine level.
- R5.2 (stealth bundle v2) — patched into Firefox source.
- R5.5 (Chromium binary patcher) — Camoufox **is** the patched
  browser; nothing to write.
- R5.3 (mouse paths) — Camoufox has built-in humanization for
  navigator events; we still drive synthetic clicks where required.
- R5.4 (synthetic profile) — we wrap Camoufox's fingerprint config
  rather than building our own profile generator.

What remains for the project to write is:

1. A pure-Rust client that drives Camoufox.
2. A `Harvester` implementation that drops into `px-pipeline` as a
   peer of `ChromiumoxidePool`.
3. Routing logic so CF-fronted sites use Camoufox and PX-direct
   sites continue using chromiumoxide (lighter, faster, no Firefox
   download).

Three candidate Rust integration paths were considered in chat:

- **Path A — Python sidecar.** Spawn `python -m camoufox ...` from
  Rust; speak JSON-RPC. Fastest to a working bypass, but pulls
  Python into the runtime, which directly violates ADR-0001
  (Rust-only).
- **Path B — Direct binary + WebDriver BiDi.** Talk WebDriver BiDi
  natively from Rust against the Camoufox binary. Pure Rust, no
  geckodriver, no Python. But the BiDi protocol client alone is a
  multi-day project, and we re-derive what the Python wrapper does
  for fingerprint config injection.
- **Path C — `fantoccini` + geckodriver fronting Camoufox.**
  geckodriver speaks standard W3C WebDriver to fantoccini in Rust;
  `--binary <camoufox-path>` makes geckodriver launch the Camoufox
  patched Firefox instead of stock Firefox. Camoufox-specific
  fingerprint config flows in via Firefox `prefs` (no Python).

## Decision

**Adopt Camoufox via Path C.** Build a new `px-camoufox` crate that:

1. Locates a Camoufox binary at `$PX_CAMOUFOX_BIN` (default
   `~/.cache/camoufox/camoufox`), and `geckodriver` at
   `$PX_GECKODRIVER_BIN` (default `geckodriver` from `$PATH`).
2. Spawns `geckodriver --port <free-port> --binary <camoufox>`
   as a managed subprocess per `CamoufoxPool` instance.
3. Talks to geckodriver via `fantoccini` (W3C WebDriver) from Rust.
4. Implements `px_harvester::Harvester` so `px-pipeline` can use it
   in place of `ChromiumoxidePool`.
5. Injects Camoufox fingerprint configuration via Firefox `prefs`
   passed in the WebDriver capabilities map (locale, timezone,
   geo, screen, useragent — replicating the most important pieces
   of the Python wrapper's behavior; advanced auto-fingerprint
   selection deferred).

Binary distribution model: **operator-installed, never bundled**.
The project does not redistribute Camoufox or geckodriver binaries
in releases. Operators install both separately (the Python
`camoufox` package handles the download into `~/.cache/camoufox`
exactly as the operator already has it set up; geckodriver is a
static binary from Mozilla's GitHub releases). `px-camoufox`
returns a clear startup error if either binary is missing, with
links to the install instructions.

## Alternatives considered

- **Path A (Python sidecar).** Rejected: ADR-0001 prohibits Python
  in the runtime path; this is non-negotiable for v1.x.
- **Path B (direct BiDi).** Rejected for v1.x: writing a usable BiDi
  client in Rust is more work than C, and the practical benefit
  (no geckodriver in the runtime) is small. Revisit if geckodriver
  becomes a maintenance burden.
- **Stay on chromiumoxide + hand-port nodriver stealth.** Rejected:
  multi-week stealth engineering with monthly maintenance cycles
  against CF updates, much of which Camoufox already shipped.
- **Use playwright-rust.** Rejected: same engine choice problem
  (Chromium) and the Playwright protocol shifts faster than W3C
  WebDriver, costing more maintenance.

## Consequences

- **Positive:**
  - Eliminates R5.1, R5.2, R5.5 from the engineering backlog. The
    stealth work is upstream; we consume it.
  - CF-fronted PX targets (pedidosya, others) become reachable
    without a Python runtime dep and without paid proxies.
  - The hybrid pipeline (chromiumoxide for PX-direct, Camoufox for
    CF-fronted) is honest about tradeoffs: chromiumoxide is faster
    and lighter for the easy case, Camoufox carries the
    heavyweight stealth for the hard case.
- **Negative:**
  - Adds a new operator install step (Camoufox binary ~200 MB; the
    Python `camoufox` package's auto-download is the path of least
    friction even though we don't use the Python code at runtime).
  - geckodriver becomes a runtime dependency for the Camoufox path.
  - We don't get Camoufox's full Python-side fingerprint-config
    auto-selection — we re-implement what we need in Rust over time.
  - License: Camoufox is MPL-2.0 (the Firefox source patches).
    px-solver itself is AGPL-3.0. The two licenses are compatible
    for the "operator installs both separately" distribution model
    we adopt; the project never redistributes Camoufox-derived
    binaries, so AGPL §13 does not pull MPL code into AGPL scope.
- **Follow-ups:**
  - `px-camoufox` crate added to workspace.
  - `px-cloudflare` (currently a stub from ADR-0006) is promoted in
    a subsequent R5 commit to delegate to `CamoufoxPool` when CF
    interstitials are detected by the pipeline.
  - Routing: a future ADR may codify "detect CF first → Camoufox;
    else → chromiumoxide". For now px-pipeline gets both harvesters
    as concrete options and the operator picks per allowlist
    entry.
  - Update `docs/deployment.md` with the geckodriver + Camoufox
    install steps once the integration lands.
