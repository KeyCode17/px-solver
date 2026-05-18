//! Port trait for executing an arbitrary HTTP request inside a
//! handler-owned browser session. Built so the px-server `/v1/fetch`
//! endpoint can proxy a single request through Camoufox / Chromium
//! and bypass the JA3/UA mismatch that breaks bundle replay from
//! downstream Rust HTTP clients (rustls, libcurl, etc).
//!
//! Lives next to `ChallengeHandler` because both ports are "things a
//! browser-backed handler can do". Implementations live in their
//! existing handler crates (`px-camoufox`, `px-harvester`).

use async_trait::async_trait;
use px_errors::AppError;
use std::collections::HashMap;

/// What to fetch. `method` is upper-case HTTP verb; missing → `GET`.
#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub url: String,
    pub method: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout_ms: u64,
}

impl FetchRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: None,
            headers: HashMap::new(),
            body: None,
            timeout_ms: 15_000,
        }
    }

    pub fn method(&self) -> &str {
        self.method.as_deref().unwrap_or("GET")
    }
}

/// One response, captured verbatim from the browser-side `fetch`.
#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u64,
}

#[async_trait]
pub trait Fetcher: Send + Sync {
    /// Returns the response of running `req` from inside the
    /// browser-managed session. Implementations must NOT cache the
    /// result; this is a pass-through.
    async fn fetch(&self, req: FetchRequest) -> Result<FetchResponse, AppError>;
}
