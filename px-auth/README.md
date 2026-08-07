# pxsolver-auth

API keys, per-domain allowlist and audit log — the guardrails around a dual-use tool.

Part of [**px-solver**](https://github.com/KeyCode17/px-solver) — a Rust solver service for
PerimeterX (HUMAN Security). Published standalone so you can depend on the one piece you need
instead of the whole workspace.

## Install

```toml
[dependencies]
pxsolver-auth = "1.9"
```

> **The package name and the crate name differ.** You depend on `pxsolver-auth`, but you `use px_auth::…`.
> The `pxsolver-` prefix exists to namespace the family on crates.io; the source-level name stays
> short.

## What's in it

```rust
use px_auth::{CheckAllowlist, VerifyKey, YamlKeyStore, YamlAllowlistStore};

verify_key.execute(id, secret).await?;      // argon2, constant-time
check_allowlist.execute(&domain).await?;    // explicit entry required
```

| Item | Purpose |
|---|---|
| `VerifyKey` / `KeyStore` / `ApiKeyRecord` | Argon2-hashed API keys, `id:secret` form |
| `CheckAllowlist` / `AllowlistStore` / `AllowlistEntry` | Per-domain allowlist, `tos_reviewed` enforced |
| `AuditSink` / `AuditEvent` / `AuditOutcome` | Append-only record of who solved what, when |
| `YamlKeyStore` / `YamlAllowlistStore` | File-backed stores |
| `FileAuditSink` / `StdoutAuditSink` | Audit destinations |

## Why this exists

px-solver is dual-use. Every request needs a key, every target needs an allowlist entry with
`tos_reviewed: true` and a non-empty justification, and the server **fails to start** if an entry
is missing either. See
[ADR-0007](https://github.com/KeyCode17/px-solver/blob/main/docs/adr/0007-api-key-and-domain-allowlist-guardrails.md).

## Dual use

px-solver is built for authorized testing against targets its operator controls or has permission
to test. The server enforces an API key and a per-domain allowlist requiring explicit
`tos_reviewed: true`. See
[`docs/dual-use-policy.md`](https://github.com/KeyCode17/px-solver/blob/main/docs/dual-use-policy.md).

## License

AGPL-3.0-or-later — chosen to discourage closed-source resale as an anonymous SaaS.
