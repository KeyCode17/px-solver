use crate::fingerprint::Fingerprint;
use serde::{Deserialize, Serialize};

/// Absent optionals are omitted from the wire, not sent as `null`: this
/// type is what a client serializes, and a `"fingerprint": null` the
/// server has no field for reads as a promise the API does not keep.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SolveRequest {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

    /// Route this solve through one egress proxy, so the returned bundle is
    /// bound to an IP the caller can reuse. `scheme://host:port`, where
    /// scheme is `http`, `https`, `socks5` or `socks5h`.
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// [`Self::with_proxy`] for an already-optional value, so an edge that
    /// deserializes a nullable `proxy` field forwards it without branching.
    #[must_use]
    pub fn with_proxy_opt(mut self, proxy: Option<String>) -> Self {
        self.proxy = proxy;
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
    fn absent_optionals_stay_off_the_wire() {
        let json = serde_json::to_string(&SolveRequest::new("https://example.com"))
            .expect("serialize request");
        assert_eq!(json, r#"{"url":"https://example.com"}"#);
    }

    #[test]
    fn a_named_proxy_is_serialized() {
        let json = serde_json::to_string(
            &SolveRequest::new("https://example.com").with_proxy("socks5://127.0.0.1:9050"),
        )
        .expect("serialize request");
        assert!(
            json.contains(r#""proxy":"socks5://127.0.0.1:9050""#),
            "{json}"
        );
    }

    #[test]
    fn builder_assembles_request() {
        let r = SolveRequest::new("https://example.com").with_proxy("http://1.2.3.4:8080");
        assert_eq!(r.url, "https://example.com");
        assert_eq!(r.proxy.as_deref(), Some("http://1.2.3.4:8080"));
        assert!(r.fingerprint.is_none());
    }
}
