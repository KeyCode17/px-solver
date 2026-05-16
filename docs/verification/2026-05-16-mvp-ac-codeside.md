# Code-side MVP-AC verification — 2026-05-16

Verifies the items in [`docs/008-sow-acceptance.md`](../008-sow-acceptance.md)
that do **not** require operator-side runtime soak. The remaining
items (MVP-AC-1, MVP-AC-2, MVP-AC-3, MVP-AC-4, MVP-AC-7) require a
clean residential IP, the 24-hour canary window, and an owner sign-off,
and are explicitly out of scope for this code-side pass.

## MVP-AC-5 — clippy + audit clean

- **Command:** `cargo clippy --workspace --all-targets --all-features --
  -D warnings -D clippy::unwrap_used -D clippy::expect_used
  -D clippy::panic -D clippy::dbg_macro -D clippy::todo
  -D clippy::unimplemented`
- **Result:** zero warnings, all crates compile.
- **Command:** `cargo audit`
- **Result:** 1090 advisories loaded, 256 dependencies scanned, zero
  vulnerabilities. Exit code 0.
- **Verdict:** ✅ PASS.

## MVP-AC-6 — audit log carries no cookie payload

The SOW specifies `grep -E 'px3|pxhd' audit.log` returns zero matches.
That is a runtime check, but the property holds *by construction*:

- `AuditEvent` (in [`px-auth/src/domain/audit_event.rs`](../../px-auth/src/domain/audit_event.rs))
  declares only the fields `timestamp_unix`, `key_id`, `target_domain`,
  `outcome`, `latency_ms`, `handler: Option<String>`. None of them
  carry cookie data.
- `AuditEvent::redacted_json` serdes the struct verbatim — only the
  declared fields land on disk.
- The only sinks are `StdoutAuditSink` (tracing::info on the JSON line)
  and `FileAuditSink` (append-write of the same JSON line).
- Workspace-wide search confirms the only `tracing::*` callsites that
  emit at runtime are `px-server/src/main.rs:49` (bind announcement)
  and `px-auth/src/infrastructure/audit_sink.rs:66` (the redacted JSON).
  Neither carries cookie material.
- `px-cli/src/main.rs` uses `println!` for operator commands (key/allowlist
  management). That output never enters audit.log.

The handler name on a cache-hit path is the literal `"cache"`; on a
solver path it is the dispatcher's `HandlerOutcome::handler`
(e.g. `"perimeterx"`). Neither leaks cookie content.

- **Verdict:** ✅ PASS (structural).
- Operator should still run `grep -E '_px3|_pxhd' audit.log` after the
  AC-1 soak as a belt-and-braces check.

## Out of scope (operator-side)

| AC | Why operator-side |
|----|-------------------|
| MVP-AC-1 | Needs 100 real solves against pedidosya.com.ar from a clean residential IP. Sandbox/CI IPs are PX-blocked. |
| MVP-AC-2 | Latency requires real wall-clock against the live target. |
| MVP-AC-3 | Hit-ratio measurement piggybacks on AC-1 — `/v1/metrics` now exposes `px_cache_hit_ratio` (wired in `cb5d0df`). |
| MVP-AC-4 | 24-hour soak with `pgrep chrome` post-run. |
| MVP-AC-7 | Owner sign-off on `docs/threat-model.md` and `docs/dual-use-policy.md`. |

After the operator confirms AC-1 through AC-4 and AC-7, run
`cargo xtask phase 04` to bump to `1.0.0` per ADR-0017.

## Reproducibility

```bash
cargo clippy --workspace --all-targets --all-features -- \
  -D warnings -D clippy::unwrap_used -D clippy::expect_used \
  -D clippy::panic -D clippy::dbg_macro -D clippy::todo \
  -D clippy::unimplemented
cargo audit
grep -rnE 'tracing::|println!|eprintln!|print!' --include='*.rs' \
  px-server px-auth px-pipeline px-perimeterx px-harvester px-cache \
  px-detector px-core
```
