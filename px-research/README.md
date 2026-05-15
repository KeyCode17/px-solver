# `px-research/`

Reverse-engineering workspace for the `px-solver` project. **Not** part of the v1 deliverable. Despite the `px-` prefix, this is **not** a Cargo crate — it is a plain directory chosen to sit alongside the `px-*` crates at the repo root for naming consistency.

See [ADR-0013](../docs/adr/0013-re-methodology-and-scope.md) for the methodology and scope guarantees, and [`docs/phase/R-research.md`](../docs/phase/R-research.md) for the live phase tracker.

## What lives here

| Path | Contents |
|---|---|
| [`init-js/<appId>/`](init-js/) | Versioned captures of `/<appId>/init.js` from real PX-protected targets, one file per `(appId, capture-date)` pair. |
| [`notes/<appId>/`](notes/) | Per-target notes — R1 surface maps, R2 deobfuscated output, scratch findings. |
| [`sensor-maps/`](sensor-maps/) | R3 outputs — sensor field grammars and machine-readable schemas. |
| [`fingerprints/`](fingerprints/) | Captured browser fingerprint vectors for the `px-mimic` corpus. |

## Capture provenance

Every artifact must include a header (Markdown frontmatter for `.md`, top-comment for `.js`, JSON object for `.json`) with:

```yaml
appId: <appId>
captured: YYYY-MM-DD
method: <how it was obtained — curl / chromiumoxide / manual>
sanitized_with: scripts/sanitize_capture.sh@<commit>
source_allowlist_entry: <path to the allowlist.yaml line that authorized this capture>
```

## Sanitization

All committed captures must pass through [`scripts/sanitize_capture.sh`](../scripts/sanitize_capture.sh) before staging:

- UUIDs → `<UUID>`
- 32+ char lowercase hex → `<HEX>`
- IPv4 → `<IPV4>`
- Bearer/Authorization tokens → `<TOKEN>`
- JWT-shaped strings → `<JWT>`

PX server-issued markers (`PX2`, `PXe6...`, `PXJ6...`, etc.) are intentionally **not** sanitized — they are per-request server tokens carrying no caller information.

## Build status

This directory is **not** a Cargo crate and does not participate in the workspace. It contains no Rust source. Nothing under `px-research/` is built, tested, or linked by CI by default. Add ad-hoc tooling (e.g. a deobfuscation harness, a sensor decoder) as needed; if such tooling matures into a reusable library, promote it to its own `px-*` crate (with a `Cargo.toml`) and write an ADR for the promotion.

## Legal posture

R-track captures are subject to the same per-domain allowlist (`config/allowlist.yaml`) as production solves, per [ADR-0007](../docs/adr/0007-api-key-and-domain-allowlist-guardrails.md) and [ADR-0013](../docs/adr/0013-re-methodology-and-scope.md). Targets without a reviewed allowlist entry are not subject to R0 acquisition.
