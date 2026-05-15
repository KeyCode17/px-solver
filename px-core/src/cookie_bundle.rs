use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PxCookieBundle {
    pub cookies: Vec<NamedCookie>,
    pub user_agent: String,
    pub captured_at: u64,
    pub expires_at: u64,
}

impl PxCookieBundle {
    pub fn new(
        cookies: Vec<NamedCookie>,
        user_agent: impl Into<String>,
        captured_at: SystemTime,
        ttl: Duration,
    ) -> Self {
        let captured_secs = captured_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let expires_secs = captured_secs.saturating_add(ttl.as_secs());
        Self {
            cookies,
            user_agent: user_agent.into(),
            captured_at: captured_secs,
            expires_at: expires_secs,
        }
    }

    pub fn is_expired_at(&self, now: SystemTime) -> bool {
        let now_secs = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX);
        now_secs >= self.expires_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NamedCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CookieJarDelta {
    pub set: Vec<NamedCookie>,
    pub removed: Vec<String>,
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn make_bundle(ttl_secs: u64) -> PxCookieBundle {
        PxCookieBundle::new(
            vec![NamedCookie {
                name: "_px3".into(),
                value: "abc".into(),
                domain: "pedidosya.com.ar".into(),
                path: "/".into(),
            }],
            "UA",
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000),
            Duration::from_secs(ttl_secs),
        )
    }

    #[test]
    fn expires_at_is_capture_plus_ttl() {
        let b = make_bundle(600);
        assert_eq!(b.expires_at - b.captured_at, 600);
    }

    #[test]
    fn is_expired_at_after_ttl() {
        let b = make_bundle(600);
        let after = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_700);
        assert!(b.is_expired_at(after));
    }

    #[test]
    fn is_not_expired_within_ttl() {
        let b = make_bundle(600);
        let within = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_300);
        assert!(!b.is_expired_at(within));
    }
}
