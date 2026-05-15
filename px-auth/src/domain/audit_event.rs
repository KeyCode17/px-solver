use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AuditOutcome {
    Solved,
    CacheHit,
    AuthDenied,
    AllowlistDenied,
    HandlerFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp_unix: u64,
    pub key_id: String,
    pub target_domain: String,
    pub outcome: AuditOutcome,
    pub latency_ms: u64,
    pub handler: Option<String>,
}

impl AuditEvent {
    pub fn redacted_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
