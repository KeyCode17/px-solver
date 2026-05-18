//! `/v1/fetch` application layer.
//!
//! Mirrors `SolveDispatcher` in shape: routes inbound URL to one of N
//! configured `Fetcher`s by domain, executes the request inside that
//! handler's browser session, returns the captured response.
//!
//! The dispatcher does no caching — every fetch spawns its own
//! browser session. That's expensive (~13s warm-up for Camoufox) but
//! keeps the v1 API honest: callers know each request is fresh.

use crate::application::solve_endpoint::domain_from_url;
use async_trait::async_trait;
use px_errors::AppError;
use px_pipeline::{FetchRequest, FetchResponse, Fetcher};
use std::collections::BTreeMap;
use std::sync::Arc;

#[async_trait]
pub trait FetchDispatcher: Send + Sync {
    /// Resolve the fetcher for `req.url` and run the request through it.
    /// Returns `(handler_name, response)` so the API layer can echo
    /// which handler executed.
    async fn dispatch(&self, req: FetchRequest) -> Result<(String, FetchResponse), AppError>;
}

#[derive(Clone)]
pub struct RoutingFetchDispatcher {
    default: Option<Arc<dyn Fetcher>>,
    routes: BTreeMap<String, (String, Arc<dyn Fetcher>)>,
}

impl RoutingFetchDispatcher {
    pub fn new(default: Option<Arc<dyn Fetcher>>) -> Self {
        Self {
            default,
            routes: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_route(
        mut self,
        domain: impl Into<String>,
        name: impl Into<String>,
        fetcher: Arc<dyn Fetcher>,
    ) -> Self {
        self.routes
            .insert(domain.into().to_lowercase(), (name.into(), fetcher));
        self
    }

    fn resolve(&self, host: &str) -> Option<&(String, Arc<dyn Fetcher>)> {
        let host = host.to_lowercase();
        for (domain, route) in &self.routes {
            if host == *domain || host.ends_with(&format!(".{domain}")) {
                return Some(route);
            }
        }
        None
    }
}

#[async_trait]
impl FetchDispatcher for RoutingFetchDispatcher {
    async fn dispatch(&self, req: FetchRequest) -> Result<(String, FetchResponse), AppError> {
        let host = domain_from_url(&req.url)?;
        if let Some((name, fetcher)) = self.resolve(&host) {
            let resp = fetcher.fetch(req).await?;
            return Ok((name.clone(), resp));
        }
        if let Some(default) = &self.default {
            let resp = default.fetch(req).await?;
            return Ok(("default".into(), resp));
        }
        Err(AppError::Conflict(format!(
            "no fetcher configured for host {host}"
        )))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct StaticFetcher {
        tag: &'static str,
    }

    #[async_trait]
    impl Fetcher for StaticFetcher {
        async fn fetch(&self, _req: FetchRequest) -> Result<FetchResponse, AppError> {
            Ok(FetchResponse {
                status: 200,
                headers: HashMap::new(),
                body: self.tag.to_string(),
                duration_ms: 0,
            })
        }
    }

    #[tokio::test]
    async fn routes_match_by_dns_suffix() {
        let d = RoutingFetchDispatcher::new(None).with_route(
            "pedidosya.com.ar",
            "cloudflare",
            Arc::new(StaticFetcher { tag: "cf" }),
        );
        let (name, resp) = d
            .dispatch(FetchRequest::new("https://www.pedidosya.com.ar/x"))
            .await
            .expect("dispatch");
        assert_eq!(name, "cloudflare");
        assert_eq!(resp.body, "cf");
    }

    #[tokio::test]
    async fn returns_conflict_when_no_route_and_no_default() {
        let d = RoutingFetchDispatcher::new(None);
        let err = d
            .dispatch(FetchRequest::new("https://example.com/"))
            .await
            .expect_err("should fail");
        assert!(matches!(err, AppError::Conflict(_)));
    }
}
