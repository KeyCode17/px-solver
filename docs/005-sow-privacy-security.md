# 005 — Data Privacy & Security

- API keys are stored as Argon2id hashes in a YAML config file (`config/keys.yaml`), file mode `0600`, owner-only readable.
- Constant-time comparison (`subtle::ConstantTimeEq`) for key verification.
- The per-domain allowlist is the primary abuse control; any domain not listed is rejected with `403 Forbidden` before a browser worker is allocated.
- Audit logs redact cookie values and request/response bodies; only metadata is recorded.
- The server binds to `127.0.0.1` by default; remote exposure requires explicit `bind` config plus TLS termination by a reverse proxy (nginx/Caddy).
- Chromium workers run with `--no-sandbox` disabled where the host supports it; a dedicated unprivileged user is recommended (documented in `docs/deployment.md`).
- Dependency auditing via `cargo audit` and `cargo deny` in CI.
- No telemetry leaves the host unless the operator explicitly configures a Prometheus scrape or remote syslog target.
