//! Domain-based handler routing.
//!
//! `RoutingDispatcher` picks one of N pre-configured `ChallengeHandler`s
//! based on the URL host of the inbound solve request. Routes are matched
//! via DNS-suffix semantics so `pedidosya.com.ar` covers
//! `www.pedidosya.com.ar`, `m.pedidosya.com.ar`, etc.
//!
//! See ADR-0021. Routes come from operator config (env CSV in v1, allowlist
//! field in v2) — never from heuristics, so picking a handler is O(routes)
//! per request with no network probes.

use crate::application::solve_endpoint::{SolveDispatcher, SolveOutput, domain_from_url};
use async_trait::async_trait;
use px_core::PxCookieBundle;
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerStatus, PageHtml};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct RoutingDispatcher {
    default: Arc<dyn ChallengeHandler>,
    routes: BTreeMap<String, Arc<dyn ChallengeHandler>>,
}

impl RoutingDispatcher {
    pub fn new(default: Arc<dyn ChallengeHandler>) -> Self {
        Self {
            default,
            routes: BTreeMap::new(),
        }
    }

    /// Register a handler for `domain` and any DNS-subdomains of it.
    /// Domains are normalized to lowercase. Returns `self` for chaining.
    #[must_use]
    pub fn with_route(
        mut self,
        domain: impl Into<String>,
        handler: Arc<dyn ChallengeHandler>,
    ) -> Self {
        self.routes.insert(domain.into().to_lowercase(), handler);
        self
    }

    /// Look up the handler matched by `host`. Matching is DNS-suffix:
    /// `pedidosya.com.ar` matches host `www.pedidosya.com.ar`.
    fn resolve(&self, host: &str) -> &Arc<dyn ChallengeHandler> {
        let host = host.to_lowercase();
        for (domain, handler) in &self.routes {
            if host == *domain || host.ends_with(&format!(".{domain}")) {
                return handler;
            }
        }
        &self.default
    }
}

#[async_trait]
impl SolveDispatcher for RoutingDispatcher {
    async fn solve(&self, url: &str) -> Result<SolveOutput, AppError> {
        let host = domain_from_url(url)?;
        let handler = self.resolve(&host);
        let page = PageHtml::new(url, "");
        let outcome = handler.solve(&page).await?;
        if !matches!(outcome.status, HandlerStatus::Solved) {
            return Err(AppError::Conflict(format!(
                "{} returned status {:?}",
                handler.name(),
                outcome.status
            )));
        }
        let bundle = PxCookieBundle::new(
            outcome.cookies.set.clone(),
            "px-harvester",
            SystemTime::now(),
            Duration::from_secs(600),
        );
        Ok(SolveOutput {
            bundle,
            user_agent: outcome.user_agent.unwrap_or_default(),
            solve_ms: outcome.metrics.solve_ms,
            cache_hit: false,
            handler: outcome.handler,
        })
    }
}

/// Parse `PX_CAMOUFOX_DOMAINS` CSV (comma-separated, whitespace trimmed,
/// empty entries dropped, lowercased). Returns empty vec when unset.
pub fn parse_camoufox_domains(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use px_core::CookieJarDelta;
    use px_pipeline::{HandlerMetrics, HandlerOutcome};

    struct StaticHandler {
        name: &'static str,
    }

    #[async_trait]
    impl ChallengeHandler for StaticHandler {
        fn name(&self) -> &'static str {
            self.name
        }
        async fn detects(&self, _page: &PageHtml) -> Result<bool, AppError> {
            Ok(true)
        }
        async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
            Ok(HandlerOutcome::solved_with_ua(
                self.name,
                CookieJarDelta::default(),
                Vec::new(),
                HandlerMetrics::default(),
                "ua",
            ))
        }
    }

    #[tokio::test]
    async fn default_handler_used_when_no_match() {
        let d = RoutingDispatcher::new(Arc::new(StaticHandler { name: "perimeterx" })).with_route(
            "pedidosya.com.ar",
            Arc::new(StaticHandler { name: "cloudflare" }),
        );
        let out = d
            .solve("https://www.havenwellwithin.com/")
            .await
            .expect("solve");
        assert_eq!(out.handler, "perimeterx");
    }

    #[tokio::test]
    async fn exact_host_routes_to_match() {
        let d = RoutingDispatcher::new(Arc::new(StaticHandler { name: "perimeterx" })).with_route(
            "pedidosya.com.ar",
            Arc::new(StaticHandler { name: "cloudflare" }),
        );
        let out = d.solve("https://pedidosya.com.ar/").await.expect("solve");
        assert_eq!(out.handler, "cloudflare");
    }

    #[tokio::test]
    async fn subdomain_routes_to_match() {
        let d = RoutingDispatcher::new(Arc::new(StaticHandler { name: "perimeterx" })).with_route(
            "pedidosya.com.ar",
            Arc::new(StaticHandler { name: "cloudflare" }),
        );
        let out = d
            .solve("https://www.pedidosya.com.ar/x")
            .await
            .expect("solve");
        assert_eq!(out.handler, "cloudflare");
    }

    #[test]
    fn parse_csv_trims_and_lowercases() {
        let r = parse_camoufox_domains(Some(" Pedidosya.com.AR ,  ,foo.com "));
        assert_eq!(r, vec!["pedidosya.com.ar", "foo.com"]);
    }

    #[test]
    fn parse_csv_empty_unset() {
        assert!(parse_camoufox_domains(None).is_empty());
        assert!(parse_camoufox_domains(Some("")).is_empty());
    }
}
