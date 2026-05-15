# 0013. Reverse-engineering methodology and scope

- **Date:** 2026-05-16
- **Status:** Accepted
- **Deciders:** KeyCode17
- **Related:** ADR-0003, ADR-0010, ADR-0014, [`docs/phase/R-research.md`](../phase/R-research.md), `px-research/`

## Context

The current solver architecture defeats PerimeterX by **avoidance** — drive a real Chromium with a stealth bundle, let PX's own `/init.js` execute, harvest the resulting `_px3` cookie. This shipped path does not require us to understand PX internals.

A second path — **defeat** — would build the sensor payload natively in Rust and POST it directly to the PX collector. This is faster (sub-500ms), lighter (no Chromium), and far more brittle. It also requires substantial reverse-engineering work that does not fit inside the v1 SOW timeline (see [ADR-0010](0010-defer-native-sensor-generator.md)).

We need to commit to a methodology for the RE work without committing to a delivery date for any RE-derived feature. The work must be allowed to make slow, parallel progress without blocking the harvester from shipping, and its outputs must feed into existing solver components (`px-observer`, `px-mimic`, eventual `px-native`) in a predictable shape.

## Decision

Codify a five-phase RE methodology, **R0 → R4**, scoped to a parallel research track that is explicitly **not on the v1 critical path**. The track's home is `px-research/` at the repository root — a plain directory of captures, notes, and field maps, **not** a Rust crate. It contains no source code and is invisible to Cargo / CI by default.

### Phase R0 — Acquisition

Fetch `/<appId>/init.js` for every allowlisted PX target, plus the matching `/captcha.js` when present. Store under `px-research/init-js/<appId>/<YYYY-MM-DD>-init.js`. Run a diff across tenants to confirm that the **core** of init.js is shared (only `appId` and per-tenant config differ).

**Output:** a versioned corpus of init.js files, one per (tenant, capture-date) pair.

### Phase R1 — Surface mapping

Run a captured init.js inside an instrumented browser (Puppeteer or our own `px-harvester` with extra CDP hooks). Log every `window` / `navigator` / `screen` / `document` property accessed, every `addEventListener` registration, every `XMLHttpRequest.open` / `fetch` call, every cookie write.

**Output:** `px-research/notes/<appId>/r1-surface.md` — a catalog of what init.js touches, without yet understanding *how*.

### Phase R2 — Deobfuscation

Apply automated deobfuscation:

- [`webcrack`](https://github.com/j4k0xb/webcrack) — string-array unpacking, control-flow deflattening, dead-code removal.
- [`humanify`](https://github.com/jehna/humanify) — LLM-assisted identifier renaming.
- Hand work — extract the VM opcode table and write a small interpreter dump so VM-protected functions become readable.

**Output:** `px-research/notes/<appId>/r2-deobfuscated/` — the recovered JS, organized by module.

### Phase R3 — Sensor RE

Find the function that builds the POST body to `/<appId>/xhr/b/s`. Trace inputs → outputs. Identify:

- The encryption layer (almost certainly AES-CBC, key derived from a server-issued challenge token + page state).
- The HMAC integrity check on the plaintext sensor JSON.
- The field grammar (PX uses numeric tags as keys: `{"PX1023": ua, "PX1029": "1920x1080", ...}`).
- Every signal source that ends up in the sensor object.

**Output:** `px-research/sensor-maps/<appId>-r3.md` and machine-readable schemas under `px-research/sensor-maps/<appId>.schema.json`.

### Phase R4 — Bypass selection

R3 output enables a choice between four bypass strategies, each with its own ADR when activated:

| Strategy | Trade-off |
|---|---|
| **In-VM execution** — keep PX's compiled VM, swap the environment | Cheapest; survives most PX updates. |
| **Native rewrite** — port the sensor builder to Rust | Fastest at runtime; rewrite required on every meaningful PX update. |
| **Patch-and-run** — monkey-patch detection branches in init.js | Fragile; useful for research only. |
| **Cookie warehouse** — solve once via real browser, pool `_px3` | Already in plan (`px-cache`); no R-track work needed. |

**Output:** an ADR selecting the strategy when (and only when) R3 has produced enough understanding to make the selection meaningful.

## Scope guarantees

- **Off the critical path.** No SOW deliverable, MVP acceptance criterion, or phase exit criterion depends on any R-phase outcome.
- **Not a crate.** `px-research/` is a plain repository-root directory; it has no `Cargo.toml`, no Rust source, and is invisible to the workspace. CI does not touch it by default.
- **Capture provenance.** Every artifact under `px-research/` is labeled with `(appId, capture-date, capture-method)` in its header. The sanitization rules in [`scripts/sanitize_capture.sh`](../../scripts/sanitize_capture.sh) apply to all committed captures.
- **Legal posture.** RE is performed on PX's client-side code, as served to the public web, on targets the operator has allowlisted in `config/allowlist.yaml`. No server-side intrusion is performed at any phase. The allowlist `tos_reviewed` requirement (ADR-0007) extends to R-track captures: targets without a reviewed allowlist entry are not subject to R0 acquisition.
- **No delivery commitment.** The maintainer commits to following the methodology *if* RE work is undertaken; the maintainer does not commit to undertaking it within any timeframe.

## Alternatives considered

- **No RE, ever.** Rejected — closes off the future native solver entirely; loses information value of even partial deobfuscation.
- **RE-first, ship after.** Rejected per [ADR-0010](0010-defer-native-sensor-generator.md) — risks never shipping.
- **RE as part of `px-harvester`.** Rejected — couples research lifecycle to the solver lifecycle; pollutes the production crate with research artifacts.

## Consequences

- **Positive:** RE has an official seat at the table; future maintainers know exactly what to capture and where to put it; the methodology is auditable; the v1 solver is unaffected.
- **Negative:** one more place to forget to update; tempting to scope-creep R-track work into Phase 02 once R1 produces useful surface maps. Mitigation: this ADR's "off the critical path" guarantee is binding until superseded.
- **Follow-ups:**
  - `px-research/README.md` documents the directory's role; no `Cargo.toml` exists at this location.
  - [`docs/phase/R-research.md`](../phase/R-research.md) tracks R-phase progress, time-unbounded.
  - When R3 produces a useful sensor schema, open the next free ADR slot (ADR-0016+) to record the bypass-strategy selection from R4 and move `px-native` from "stub" to "implementation" in a new SOW deliverable.
