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

px-server reads one more env var to decide which domains use the Camoufox path (see [ADR-0021](adr/0021-domain-based-handler-routing.md)):

- `PX_CAMOUFOX_DOMAINS` — comma-separated list of DNS suffixes that should route to `CloudflareHandler::with_harvester(CamoufoxPool)` instead of the default Chromium-only `PerimeterxHandler`. Matching is DNS-suffix, so `pedidosya.com.ar` covers `www.pedidosya.com.ar`. Unset or empty → no routing override; all targets go through Chromium.

```bash
PX_CAMOUFOX_DOMAINS=pedidosya.com.ar ./target/release/px-server
```

Startup logs `Camoufox routing enabled` at info level when the routing path activates, and `falling back to Chromium-only dispatcher` at warn level if `PX_CAMOUFOX_DOMAINS` is set but the Camoufox/geckodriver binaries cannot be validated.

### Licensing

Camoufox is MPL-2.0; geckodriver is MPL-2.0. Both are operator-installed and never bundled in px-solver releases (no distribution obligation triggered).

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
```

Any entry without `tos_reviewed: true` or with empty `justification` causes server startup to fail (see [`px-auth`](../px-auth/) `AllowlistEntry::validate`).

### `config/server.yaml` (optional)

Currently the server reads `PX_BIND`, `PX_KEYS`, `PX_ALLOWLIST` env vars. A YAML config file is reserved for future use.

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
