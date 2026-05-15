use crate::fingerprint::Fingerprint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SolveRequest {
    pub url: String,
    pub proxy: Option<String>,
    pub fingerprint: Option<Fingerprint>,
}

impl SolveRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            proxy: None,
            fingerprint: None,
        }
    }

    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    pub fn with_fingerprint(mut self, fingerprint: Fingerprint) -> Self {
        self.fingerprint = Some(fingerprint);
        self
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn builder_assembles_request() {
        let r = SolveRequest::new("https://example.com").with_proxy("http://1.2.3.4:8080");
        assert_eq!(r.url, "https://example.com");
        assert_eq!(r.proxy.as_deref(), Some("http://1.2.3.4:8080"));
        assert!(r.fingerprint.is_none());
    }
}
