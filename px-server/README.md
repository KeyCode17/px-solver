# px-server

Composition root + Axum HTTP API — the px-solver service binary.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver).

## Not on crates.io

This crate is `publish = false`. Only the 16 `pxsolver-*` **library** crates are published; the
binaries are built from source:

```bash
git clone https://github.com/KeyCode17/px-solver
cd px-solver
cargo build --release -p px-server
```

## Run

```bash
PX_BIND=127.0.0.1:8080 \
PX_KEYS=config/keys.yaml \
PX_ALLOWLIST=config/allowlist.yaml \
./target/release/px-server
```

Binding to `127.0.0.1` is the default and the recommended posture; front it with nginx/Caddy for
TLS. Full install, systemd unit and key rotation:
[`docs/deployment.md`](../docs/deployment.md).

## Endpoints

| Route | Purpose |
|---|---|
| `POST /v1/solve` | Solve a target, return the `_px3` cookie bundle |
| `POST /v1/fetch` | Run one HTTP request inside a warm browser session |
| `GET /health` | Status, build sha, uptime |
| `GET /metrics` | Solve counters, latency histogram, auth/allowlist denials |

Every request needs `Authorization: Bearer <id>:<secret>`, and every target needs an allowlist
entry with `tos_reviewed: true`.

## Proxies

`/v1/solve` takes a per-request `"proxy"`; `/v1/fetch` rotates the operator's `PX_PROXIES` list
across warm sessions. They are separate mechanisms and neither falls back to the other — a `_px3`
bundle is bound to the IP that earned it, so a rotating egress the caller cannot name would be
useless. The proxy is part of the cache key.

See [Egress proxies](../docs/deployment.md#egress-proxies) and
[ADR-0025](../docs/adr/0025-egress-proxy-propagation-contract.md).

## Layout

Hexagonal: `application/` holds the dispatcher ports and use cases, `infrastructure/http/` the
handlers, DTOs and router, `infrastructure/bootstrap/` the composition root that wires state,
handlers and the native overlay.

## License

AGPL-3.0-or-later.
