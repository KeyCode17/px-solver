# 0024. Activate native px-3 sensor synthesis; promote ADR-0010

- **Date:** 2026-05-20
- **Status:** Proposed
- **Deciders:** KeyCode17
- **Related:** ADR-0010 (deferred), ADR-0013 (R-track), ADR-0019 (R5),
  px-research/notes/eT15wiaE/r3-sensor-grammar-2026-05-20.md

## Context

R0–R3 of the R-track are complete for the pedidosya tenant (`PXeT15wiaE`):

* the `gB(t)` base-91 string decoder is statically reversed, inlining
  1 457 obfuscated literals;
* the sensor builder (`yA → vP → jw → hP → vQ`) is identified;
* the cipher stack is **XOR-single-byte + base64 + string-splice VM** —
  no AES, no HMAC at the payload tier, no per-tenant secret beyond the
  public appID.

ADR-0010 deferred the native generator on the assumption that PX uses
"HMAC signing" and "dozens of environment fingerprints" — both of which
turned out to be either non-existent (HMAC) or already enumerable from
the static analysis (fingerprint vector list is finite and fixed for
this tenant).

The current pipeline relies on **Camoufox + Tier-2 harvester** for every
solve. Cold-start cost is ~13 s, warm reuse drops it to ~3-5 s but caps
fleet throughput at ≤2 req/min per session even with the AR residential
proxy. The user-defined target is ≥40 req/min through `/v1/fetch`,
which Camoufox-only cannot reach without a 20-session pool.

The strategic constraint from the user is unchanged: **no third-party
CAPTCHA services**. Native synthesis is the only remaining vector.

## Decision

Move `px-native` from `Unsupported` stub to a **production solver path
for the eT15wiaE tenant**, in five sprints:

1. **N1 — `vN` extraction.** Statically extract the offsets-array PRNG
   so we can splice/un-splice a `vQ`-flavoured payload in Rust without
   running JS. ~150 LOC + test fixtures from R0 capture.
2. **N2 — Cipher core.** `xor_single_byte`, `b64_encode`, `vq_splice`
   as a 100 % `no_std`-ish module under `px-native/src/cipher/`. Round
   trip against 50+ captured `(plaintext, encrypted)` pairs collected
   via XHR hook in Camoufox.
3. **N3 — Event collector.** Synthesise realistic event sequences
   (mouse/scroll/click with monotonic timestamps, viewport dimensions
   tied to the existing `SyntheticUserPool`, navigation timing) and
   feed them to the cipher. Reproduce a working `_px3` cookie response
   end-to-end for the homepage. **Target: 95 % solve rate, p50 ≤500 ms.**
4. **N4 — Endpoint orchestration.** Implement `NativeSolver` impl: POST
   `${origin}/${appIdLower}/xhr/b/s`, parse the `Set-Cookie` reply,
   surface the bundle through the existing `SolveOutcome` adapter.
   Wire it into the composition root as a domain-routed Tier 2.5.
5. **N5 — Tenant generalisation.** Extract the appID-dependent parts
   (alphabet, `iS` XOR key, sensor endpoint path) into a TOML profile
   so adding a new tenant is a config change + capture, not a code
   change. Profile the eT15wiaE values into
   `px-native/profiles/eT15wiaE.toml`.

The harvester remains in the pipeline as a fallback. Routing prefers
native, falls back to Camoufox on `NativeError::Mismatch`.

## Alternatives considered

- **Stay Camoufox-only, scale session pool.** Rejected — capped at
  ~2 req/min per session × 20 sessions = 40 req/min just barely
  meets the target, and IP cost scales linearly with session count.
  Maintenance burden is browser/geckodriver upgrades plus Camoufox
  patches, not lower than native.
- **Native rewrite with full JS-VM emulation (quickjs/boa).** Rejected
  — pulls in a 1-2 MB runtime dependency and a 100 ms+ JIT warm-up,
  defeating the latency goal. Static synthesis is sufficient since the
  cipher is XOR+base64.
- **Hybrid: run the obfuscated JS in a stripped Deno isolate.**
  Rejected — duplicates the third-party-tooling exposure the user
  ruled out (Deno permissions sandbox is a third-party trust surface,
  and the isolate cannot satisfy `navigator.webdriver === false`
  without patches we'd have to maintain anyway).
- **Cookie warehouse (pre-harvest N×1000 cookies, hand out from a
  queue).** Rejected — fails the "real-time scrape" workflow when a
  cookie expires mid-session and pegs IP usage during the harvest
  burst. Useful as a **batch fallback for catalog-only passes**
  (vendors-only mode) — captured as a separate ADR follow-up, not the
  primary path.

## Consequences

- **Positive:**
  * P50 solve latency target moves from 3-5 s to <500 ms.
  * Throughput ceiling moves from ~40 req/min (with 20 sessions) to
    network-bound (effectively 200+ req/min per proxy).
  * Removes the Camoufox single-point-of-failure (Firefox CVEs,
    geckodriver upgrades, AR proxy availability).
  * Reusable for other PX tenants once the profile abstraction lands
    in N5.
- **Negative:**
  * 4-5 sprints of focused work before the harvester can be demoted.
  * The implementation is brittle against PX rotations of the cipher
    parameters (alphabet, XOR key, splice algorithm). Rotation
    cadence observed in the wild is ~quarterly per tenant; we
    re-capture and re-profile each time.
  * `vQ` VM extraction may surface unexpected complexity. **Hard kill
    criterion:** if N1 spills past 400 LOC of Rust or 5 sprint-days,
    we re-open the ADR with the cookie-warehouse fallback as the
    primary path.
- **Follow-ups:**
  * **ADR-0025** to record the TOML profile schema once N5 lands.
  * **ADR-0026** for the cookie-warehouse mode used by
    `--vendors-only` batch scrapes in pedidosya-scrapper.
  * **SOW-DEL-014** to scope test fixtures: capture 200 successful
    sensor POSTs through Camoufox with XHR-hooked plaintext +
    response, store under `px-native/tests/fixtures/eT15wiaE/`.
