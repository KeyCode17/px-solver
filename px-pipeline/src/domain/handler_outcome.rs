use px_core::{CookieJarDelta, NamedToken};
use serde::{Deserialize, Serialize};

pub type HandlerName = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HandlerStatus {
    Skipped,
    Solved,
    NotImplemented,
    Failed,
}

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerOutcome {
    pub handler: String,
    pub status: HandlerStatus,
    pub cookies: CookieJarDelta,
    pub tokens: Vec<NamedToken>,
    pub metrics: HandlerMetrics,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct HandlerMetrics {
    pub detect_us: u64,
    pub solve_ms: u64,
    pub bytes_read: u64,
}

impl HandlerOutcome {
    pub fn skipped(handler: impl Into<String>) -> Self {
        Self {
            handler: handler.into(),
            status: HandlerStatus::Skipped,
            cookies: CookieJarDelta::default(),
            tokens: Vec::new(),
            metrics: HandlerMetrics::default(),
        }
    }

    pub fn solved(
        handler: impl Into<String>,
        cookies: CookieJarDelta,
        tokens: Vec<NamedToken>,
        metrics: HandlerMetrics,
    ) -> Self {
        Self {
            handler: handler.into(),
            status: HandlerStatus::Solved,
            cookies,
            tokens,
            metrics,
        }
    }

    pub fn not_implemented(handler: impl Into<String>) -> Self {
        Self {
            handler: handler.into(),
            status: HandlerStatus::NotImplemented,
            cookies: CookieJarDelta::default(),
            tokens: Vec::new(),
            metrics: HandlerMetrics::default(),
        }
    }
}
