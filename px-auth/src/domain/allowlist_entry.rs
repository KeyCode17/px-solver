use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowlistEntry {
    pub domain: String,
    pub tos_reviewed: bool,
    pub justification: String,
    /// Optional handler routing hint (ADR-0021/0023). When set to
    /// `"cloudflare"`, the server's `RoutingDispatcher` routes this
    /// domain to the CF-bypass path (CamoufoxPool) instead of the
    /// default Chromium-only `perimeterx` handler. When absent, the
    /// domain inherits the default handler.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handler: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowlistEntryError {
    NotTosReviewed(String),
    EmptyJustification(String),
    EmptyDomain,
}

impl fmt::Display for AllowlistEntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotTosReviewed(d) => write!(f, "allowlist '{d}': tos_reviewed must be true"),
            Self::EmptyJustification(d) => write!(f, "allowlist '{d}': justification required"),
            Self::EmptyDomain => f.write_str("allowlist entry has empty domain"),
        }
    }
}

impl std::error::Error for AllowlistEntryError {}

impl AllowlistEntry {
    pub fn validate(&self) -> Result<(), AllowlistEntryError> {
        if self.domain.trim().is_empty() {
            return Err(AllowlistEntryError::EmptyDomain);
        }
        if !self.tos_reviewed {
            return Err(AllowlistEntryError::NotTosReviewed(self.domain.clone()));
        }
        if self.justification.trim().is_empty() {
            return Err(AllowlistEntryError::EmptyJustification(self.domain.clone()));
        }
        Ok(())
    }
}
