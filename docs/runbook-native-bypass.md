# Native PerimeterX bypass — operator runbook

Status: live validation pending operator execution (ADR-0024 v1.8.0 P4).

This is the end-to-end procedure for taking the native PX path from
"compiled and unit-tested" to "sustaining ≥40 req/min through a real
tenant". It assumes:

- A working AR (or other tenant-appropriate) residential proxy is
  available — set `PX_PROXIES=socks5://…` in your shell.
- Camoufox + geckodriver are installed and `CamoufoxConfig::from_env()`
  resolves them.
- You have the `eT15wiaE` (pedidosya) profile at
  `px-native/profiles/eT15wiaE.toml`.

## Step 1 — Capture ground truth

Run the XHR-hook capture against the live target. This drives a real
Firefox/Camoufox session through the proxy and records every
plaintext sensor event the runtime feeds into the encryptor:

```bash
CAPTURE_PX=1 \
  CAPTURE_URL=https://www.pedidosya.com.ar/ \
  CAPTURE_WAIT_MS=15000 \
  PX_PROXIES="$PX_PROXIES" \
  cargo test -q -p pxsolver-camoufox --test capture_sensor -- --ignored --nocapture
```

Output lands at `px-research/captures/eT15wiaE/<unix-ts>.json`:

- `plaintext_events` — every `[{t, d}, …]` batch JSON-stringified by
  the page (= the input to the cipher);
- `xhr_sends` — every `/b/s` request URL + body (= what hit the wire);
- `cookies` — the cookie jar at end of the wait window;
- `user_agent`.

Repeat the capture 3–5 times. Tag variation across captures helps
distinguish stable fields from per-session noise.

## Step 2 — Calibrate

For each capture, diff against `default_batch`:

```bash
px-cli calibrate px-research/captures/eT15wiaE/<unix-ts>.json
```

Output identifies:

- **Missing tags** the runtime emits but `default_batch` does not.
- **Extra tags** we emit but the runtime did not (those probably tank
  the trust score — drop them).
- **Per-tag missing keys** — base64-veiled field names we still need
  to populate.

Iterate `px-native/src/events/batch.rs` until the report shows no
missing tags or keys for the eT15wiaE tenant.

## Step 3 — Enable the native overlay

```bash
export PX_NATIVE_PROFILES="pedidosya.com.ar=px-native/profiles/eT15wiaE.toml"
cargo run -p px-server   # logs: "PX_NATIVE_PROFILES → native overlay enabled"
```

The dispatcher will try the native handler first for any solve
targeting `pedidosya.com.ar` and fall back to the existing Camoufox
path on error or non-solved status.

## Step 4 — Throughput soak

```bash
NATIVE_SOAK=1 \
  NATIVE_SOAK_URL=https://www.pedidosya.com.ar/ \
  NATIVE_SOAK_N=80 \
  NATIVE_SOAK_CONCURRENCY=8 \
  NATIVE_SOAK_TARGET_RPM=40 \
  NATIVE_SOAK_PROFILE=px-native/profiles/eT15wiaE.toml \
  cargo test -q -p pxsolver-native --test throughput_soak -- --ignored --nocapture
```

The soak runs `SensorNativeSolver::solve` 80× through your live proxy
and asserts ≥40 req/min sustained throughput. Output:

```
=== NATIVE_SOAK ===
  n: 80
  ok: <count>
  err: <count>
  success_rate: <%>
  elapsed: <duration>
  rpm: <req/min>
  p50: <ms>
  p95: <ms>
```

If the assertion fails on `success_rate`, calibration in step 2
needs another iteration. If it fails on `rpm` only, the cipher
correctness is fine but the proxy/concurrency setup needs tuning.

## Step 5 — Promote the soak

Once a green soak run sustains ≥40 req/min for at least three
consecutive runs across different proxies / time-of-day, freeze
the profile and document the result in
`docs/verification/<date>-pedidosya-native-soak.md`. Open the
follow-up ADR proposing the profile schema lock + tenant
expansion plan (other PX tenants).
