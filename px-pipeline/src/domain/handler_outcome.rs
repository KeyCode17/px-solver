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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
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
            user_agent: None,
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
            user_agent: None,
        }
    }

    pub fn solved_with_ua(
        handler: impl Into<String>,
        cookies: CookieJarDelta,
        tokens: Vec<NamedToken>,
        metrics: HandlerMetrics,
        user_agent: impl Into<String>,
    ) -> Self {
        Self {
            handler: handler.into(),
            status: HandlerStatus::Solved,
            cookies,
            tokens,
            metrics,
            user_agent: Some(user_agent.into()),
        }
    }

    pub fn not_implemented(handler: impl Into<String>) -> Self {
        Self {
            handler: handler.into(),
            status: HandlerStatus::NotImplemented,
            cookies: CookieJarDelta::default(),
            tokens: Vec::new(),
            metrics: HandlerMetrics::default(),
            user_agent: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn solved_with_ua_sets_user_agent() {
        let oc = HandlerOutcome::solved_with_ua(
            "x",
            CookieJarDelta::default(),
            Vec::new(),
            HandlerMetrics::default(),
            "Mozilla/5.0",
        );
        assert_eq!(oc.status, HandlerStatus::Solved);
        assert_eq!(oc.user_agent.as_deref(), Some("Mozilla/5.0"));
    }

    #[test]
    fn solved_leaves_user_agent_none() {
        let oc = HandlerOutcome::solved(
            "x",
            CookieJarDelta::default(),
            Vec::new(),
            HandlerMetrics::default(),
        );
        assert!(oc.user_agent.is_none());
    }

    #[test]
    fn serde_skips_none_user_agent() {
        let oc = HandlerOutcome::solved(
            "x",
            CookieJarDelta::default(),
            Vec::new(),
            HandlerMetrics::default(),
        );
        let json = serde_json::to_string(&oc).expect("serialize");
        assert!(!json.contains("user_agent"));
    }
}
