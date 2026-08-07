# Deployment

Operator guide for running `px-server` on a fresh Linux host. Pairs with [ADR-0007](adr/0007-api-key-and-domain-allowlist-guardrails.md) (auth + allowlist + audit) and [011-sow-dual-use.md](011-sow-dual-use.md).

## Host prerequisites

- Linux x86_64 (other targets untested).
- Rust 1.95 toolchain (pinned via [`rust-toolchain.toml`](../rust-toolchain.toml)).
- Chromium or Google Chrome on `$PATH` (`/usr/bin/google-chrome` is found automatically).
- Optional (Cloudflare-fronted targets): Camoufox binary + `geckodriver` — see [Camoufox (optional)](#camoufox-optional) below.
- Optional: `cargo-audit`, `cargo-deny`, `taplo`, `yamllint`, `markdown-link-check` for lefthook checks.
- A dedicated unprivileged Linux user is recommended (avoid running Chrome as root).

## Install

```bash
git clone https://github.com/KeyCode17/px-solver.git
cd px-solver
cargo install lefthook
lefthook install
cargo build --release -p px-server -p px-cli
```

The release artifacts are at `target/release/px-server` and `target/release/px-cli`.

## Camoufox (optional)

Required only for targets fronted by Cloudflare Bot Management (e.g. `pedidosya.com.ar`). Chromium handles PX-direct targets fine; Camoufox is the deep-stealth path documented in [ADR-0020](adr/0020-adopt-camoufox-via-fantoccini-geckodriver.md).

**Important: runtime is pure Rust, not Python.** Camoufox is a patched-Firefox C++ binary. The `pip install camoufox` tool is a one-time installer that downloads the binary into `~/.cache/camoufox/`. After install, the runtime path is:

```
px-server (Rust)
  └─ px-camoufox::CamoufoxPool (Rust, Harvester impl)
       ├─ spawn `geckodriver` (Mozilla static binary)
       └─ fantoccini WebDriver client (Rust crate)
            └─ geckodriver launches Camoufox binary via `--binary`
```

No Python process is spawned per harvest. ADR-0001 (Rust-only runtime) holds.

### Install

```bash
# 1. Camoufox binary (one-time; needs Python only for download)
python3 -m venv /tmp/cf && source /tmp/cf/bin/activate
pip install camoufox
python -m camoufox fetch                  # downloads to ~/.cache/camoufox/camoufox
deactivate && rm -rf /tmp/cf              # Python venv discardable

# 2. geckodriver (static Mozilla binary)
curl -L https://github.com/mozilla/geckodriver/releases/latest/download/geckodriver-v0.36.0-linux64.tar.gz \
  | tar -xz -C ~/.local/bin
chmod +x ~/.local/bin/geckodriver
```

Alternative: download the Camoufox binary directly from <https://github.com/daijro/camoufox/releases> and place at `~/.cache/camoufox/camoufox` — no Python required.

### Configure

`px-camoufox` reads two env vars (defaults shown):

- `PX_CAMOUFOX_BIN` — default `$HOME/.cache/camoufox/camoufox`
- `PX_GECKODRIVER_BIN` — default `$HOME/.local/bin/geckodriver`

The set of domains routed through the Camoufox path is sourced from two places, unioned at startup:

1. **(preferred)** Allowlist entries with `handler: cloudflare` — see [ADR-0023](adr/0023-allowlist-handler-field-supersedes-env-csv.md).
2. **(deprecated, retained for v1.x compatibility)** `PX_CAMOUFOX_DOMAINS` env CSV — see [ADR-0021](adr/0021-domain-based-handler-routing.md). Slated for removal in v2.x.

Matching is DNS-suffix from either source, so `pedidosya.com.ar` covers `www.pedidosya.com.ar`. With neither source populated, all targets go through the Chromium-only path.

```bash
# Preferred: edit allowlist.yaml to add `handler: cloudflare` to the entry
./target/release/px-server

# Deprecated alternative (env var only):
PX_CAMOUFOX_DOMAINS=pedidosya.com.ar ./target/release/px-server
```

Startup logs `Camoufox routing enabled` at info level when the routing path activates, and `falling back to Chromium-only dispatcher` at warn level if the env var or YAML field request CF routing but the Camoufox/geckodriver binaries cannot be validated.

### Licensing

Camoufox is MPL-2.0; geckodriver is MPL-2.0. Both are operator-installed and never bundled in px-solver releases (no distribution obligation triggered).

### Performance budget

The Camoufox-routed solve path has a different latency profile than the cheap Chromium-only path because each harvest spawns a fresh geckodriver + Camoufox subprocess. Per [ADR-0022](adr/0022-readmit-pedidosya-to-canary-with-deep-stealth-budget.md), the acceptance budget for CF-fronted targets (handler `cloudflare`) is **median ≤15s, p95 ≤20s**; the original 6s budget continues to apply to the default Chromium-only handler. Empirical 100-sample soak on pedidosya: **100/100 success, median 10.27s, p95 10.99s, range 9.45-11.28s** — see [`docs/verification/2026-05-17-pedidosya-camoufox-soak-n100.md`](verification/2026-05-17-pedidosya-camoufox-soak-n100.md) (10-sample run preserved at [`-n10.md`](verification/2026-05-17-pedidosya-camoufox-soak-n10.md)).

The histogram metric `px_solve_ms_bucket{handler="..."}` at `/v1/metrics` lets you set per-handler alerts matching each budget.

## Configure

Create `config/` next to the binaries with three files:

### `config/keys.yaml`

```yaml
keys:
  - id: ops-key-1
    argon2_hash: "$argon2id$v=19$m=19456,t=2,p=1$..."  # cargo run -p px-cli -- keys generate --id ops-key-1
    note: operator-1
```

`px-cli keys generate --id <id>` prints the secret and the Argon2id hash. Store the hash only; share the secret out-of-band with the caller. The file should be mode `0600` and owned by the px-server user.

### `config/allowlist.yaml`

```yaml
entries:
  - domain: pedidosya.com.ar
    tos_reviewed: true
    justification: "Authorized research access — see internal doc <link>."
    handler: cloudflare    # optional, see ADR-0023
  - domain: www.havenwellwithin.com
    tos_reviewed: true
    justification: "Authorized PX research."
    # handler omitted → uses default Chromium-only path
```

Any entry without `tos_reviewed: true` or with empty `justification` causes server startup to fail (see [`px-auth`](../px-auth/) `AllowlistEntry::validate`).

The optional `handler:` field (added in v1.1.x per [ADR-0023](adr/0023-allowlist-handler-field-supersedes-env-csv.md)) routes the entry's domain through a non-default `ChallengeHandler`. The only currently-recognized value is `cloudflare`, which routes to the Camoufox-backed CF-bypass handler. Omit the field for PX-direct targets.

### `config/server.yaml` (optional)

Currently the server reads `PX_BIND`, `PX_KEYS`, `PX_ALLOWLIST` env vars. A YAML config file is reserved for future use.

## Egress proxies

There are **two** proxy mechanisms and they do different jobs. Pick by endpoint:

| | `/v1/solve` | `/v1/fetch` |
|---|---|---|
| How you assign it | `"proxy"` in the request body (or `px-cli solve --proxy`) | `PX_PROXIES` env var, operator-side |
| Chosen per | request | Camoufox session |
| Rotation | none — the solve uses exactly the proxy you named | round-robin across the list |
| Omitted → | server's own IP | server's own IP |

### `/v1/solve` — per-request proxy

`_px3` is bound to the IP that earned it, so a bundle is only usable from that same egress. Name the proxy you intend to send downstream traffic through:

```bash
curl -X POST http://127.0.0.1:8080/v1/solve \
  -H "Authorization: Bearer ops1:<secret>" \
  -H "content-type: application/json" \
  -d '{"url":"https://www.pedidosya.com.ar/","proxy":"socks5://127.0.0.1:9050"}'
```

Accepted schemes: `http`, `https`, `socks5`, `socks5h`. `"proxy":null` (or omitting the field) harvests from the server's own address.

The solve **never** falls back to the `PX_PROXIES` rotation — a bundle earned through an IP the caller cannot name would not be usable. The proxy is part of the cache key, so the same domain solved through two different proxies produces two entries and neither is served to the other.

Rust callers build the same request through the published crate:

```rust
use pxsolver_core::SolveRequest;

let req = SolveRequest::new("https://www.pedidosya.com.ar/")
    .with_proxy("socks5://127.0.0.1:9050");
```

### `/v1/fetch` — session rotation via `PX_PROXIES`

For sustained `/v1/fetch` traffic against rate-limiting WAFs (pedidosya's PerimeterX flags a single IP after ~30 fetches/min), set `PX_PROXIES` to a CSV list. Each persistent Camoufox session for a CF-routed domain is assigned one proxy round-robin at spawn:

```bash
PX_PROXIES="http://proxy1.example:8080,socks5://proxy2.example:1080" \
./target/release/px-server
```

Empty / unset → direct connection (no rotation). `/v1/fetch` has no per-request proxy field: a session is reused for its warm cookie jar, and switching its egress mid-life would mean respawning the browser.

**How many IPs you actually get.** A session takes a proxy when it is *spawned*, and keeps it until it ages out (300s TTL). Once a domain holds `PX_FETCH_MAX_PER_DOMAIN=N` sessions (default 2), further requests round-robin those existing sessions and no new proxy is drawn. Distinct egress IPs per domain is therefore:

```
min(PX_FETCH_MAX_PER_DOMAIN, len(PX_PROXIES))
```

Ten proxies with the default `N=2` gives a domain **two** IPs, not twenty. Raise `PX_FETCH_MAX_PER_DOMAIN` to use more of the list — each extra session is another live browser, so size it against RAM. The cursor is shared across domains, so proxy *k* is not reserved for any one target.

### Proxy credentials

**Browsers cannot authenticate to a proxy here.** geckodriver's W3C `proxy` capability has no credential field, and Chromium's `--proxy-server` ignores userinfo without a CDP `Fetch.authRequired` handler. `user:pass@` in a `PX_PROXIES` entry or in a `/v1/solve` proxy is **stripped**, and the server logs a warning naming the sanitized URL.

To use an authenticated upstream, front it with a local unauthenticated relay and point px-solver at the relay:

```bash
gost -L=socks5://127.0.0.1:1080 -F='http://user:pass@upstream.example:8080'
PX_PROXIES="socks5://127.0.0.1:1080" ./target/release/px-server
```

The one exception is the native sensor path (`PX_NATIVE_PROFILES`), which posts over `reqwest` and does support proxy authentication; a credentialed proxy works there and only there.

Tor as a quick test: install `tor`, let it bind `socks5://127.0.0.1:9050`, set `PX_PROXIES="socks5://127.0.0.1:9050"` or pass it as a per-request proxy. Many sites block Tor exit IPs; treat it as a fingerprint smoke-test rather than a production proxy.

## Run

```bash
PX_BIND=127.0.0.1:8080 \
PX_KEYS=config/keys.yaml \
PX_ALLOWLIST=config/allowlist.yaml \
./target/release/px-server
```

Binding to `127.0.0.1` is the default and the recommended posture. To expose remotely, front the server with nginx/Caddy doing TLS termination and a tight `proxy_pass`.

## Systemd unit (template)

```ini
[Unit]
Description=px-solver server
After=network.target

[Service]
User=px-solver
WorkingDirectory=/opt/px-solver
Environment=PX_BIND=127.0.0.1:8080
Environment=PX_KEYS=/opt/px-solver/config/keys.yaml
Environment=PX_ALLOWLIST=/opt/px-solver/config/allowlist.yaml
Environment=RUST_LOG=info
ExecStart=/opt/px-solver/bin/px-server
Restart=on-failure
RestartSec=5
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/px-solver/logs
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
```

## Smoke test

```bash
curl -sS -X POST http://127.0.0.1:8080/v1/solve \
  -H "Authorization: Bearer ops-key-1:<secret>" \
  -H "content-type: application/json" \
  -d '{"url":"https://www.pedidosya.com.ar/"}'
```

`/v1/health` returns build SHA + uptime; `/v1/metrics` returns Prometheus exposition.

## Logs and audit

- `tracing` logs go to stdout/journald. Filter with `RUST_LOG`.
- `AuditEvent` rows are written by the configured `AuditSink`. The default `StdoutAuditSink` emits via `tracing`; switch to `FileAuditSink` in the composition root (`px-server/src/main.rs`) for JSON-Lines file output.
- Cookie payloads are never written to the audit stream by construction (`AuditEvent` schema in `px-auth::domain::audit_event`).

## Upgrade

```bash
git pull
cargo build --release -p px-server
systemctl restart px-solver
```

Version is bumped by `cargo xtask phase NN` or `cargo xtask bump <kind>`; see [ADR-0017](adr/0017-phase-aligned-versioning.md).
