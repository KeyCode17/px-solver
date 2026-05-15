# 001 — Executive Summary

`px-solver` is a Rust service that, given a target URL, returns a valid PerimeterX (HUMAN Security) cookie bundle so that downstream authorized clients can issue requests as if from a real browser. It is built as a **hybrid solver**: a Chromium-based **browser harvester** is the primary engine, fronted by a **TTL cookie cache**, with a **native sensor-payload generator** reserved as a future drop-in fast path. The service is generic — it detects the PerimeterX `appId` at runtime — and ships with `pedidosya.com.ar` as a canary integration target.

The project is owned and operated by a single maintainer for personal research, authorized scraping, and security education. Access is gated by an API key plus a per-domain allowlist enforced server-side, with full audit logging.
