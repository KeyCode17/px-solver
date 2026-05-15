use crate::cookie_bundle::PxCookieBundle;
use serde::{Deserialize, Serialize};

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SolveOutcome {
    pub bundle: PxCookieBundle,
    pub handler: String,
    pub latency_ms: u64,
    pub cache_hit: bool,
}

impl SolveOutcome {
    pub fn new(
        bundle: PxCookieBundle,
        handler: impl Into<String>,
        latency_ms: u64,
        cache_hit: bool,
    ) -> Self {
        Self {
            bundle,
            handler: handler.into(),
            latency_ms,
            cache_hit,
        }
    }
}
