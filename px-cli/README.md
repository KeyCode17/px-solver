# px-cli

Operator CLI for px-solver — key generation, allowlist editing, solving, and sensor calibration.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver).

## Not on crates.io

This crate is `publish = false`. Only the 16 `pxsolver-*` **library** crates are published; build
the CLI from source:

```bash
git clone https://github.com/KeyCode17/px-solver
cd px-solver
cargo build --release -p px-cli
```

## Commands

```bash
# generate an API key — paste the printed id + argon2_hash into config/keys.yaml
px-cli keys generate --id ops1 --note "first operator"

# add an allowlist entry (justification is mandatory)
px-cli allowlist add --domain example.com --justification "internal price observability"

# solve a target through a running px-server
px-cli solve https://example.com/ \
  --api-key ops1:<secret> \
  --proxy socks5://127.0.0.1:9050

# diff a live capture against the native sensor's default_batch
px-cli calibrate px-research/captures/<tenant>/<ts>.json
```

`--server` defaults to `http://127.0.0.1:8080` and reads `PX_SERVER_URL`; `--api-key` reads
`PX_API_KEY`.

## `--proxy`

The egress the **solve** harvests through — `scheme://host:port` for `http`, `https`, `socks5` or
`socks5h`. The returned bundle is bound to that IP, so send downstream requests through the same
proxy. Omit it to harvest from the server's own address.

This is not `PX_PROXIES`, which is an operator-side rotation for `/v1/fetch` sessions only. See
[Egress proxies](../docs/deployment.md#egress-proxies).

Browser engines cannot authenticate to a proxy, so `user:pass@` is stripped with a warning —
front an authenticated upstream with a local relay (gost, 3proxy).

## License

AGPL-3.0-or-later.
