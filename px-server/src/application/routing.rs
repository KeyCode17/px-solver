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
use px_core::{PxCookieBundle, SolveRequest};
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerStatus, PageHtml, SolveAction};
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

    /// Exact-key route lookup. Overlays wrap the result in a
    /// decorator and re-register it.
    pub fn handler_for(&self, domain: &str) -> Option<&Arc<dyn ChallengeHandler>> {
        self.routes.get(&domain.to_lowercase())
    }
    pub fn default_handler(&self) -> &Arc<dyn ChallengeHandler> {
        &self.default
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
    async fn solve(&self, req: SolveRequest) -> Result<SolveOutput, AppError> {
        let host = domain_from_url(&req.url)?;
        let handler = self.resolve(&host);
        let action = SolveAction::new(PageHtml::new(&req.url, "")).with_proxy(req.proxy);
        let outcome = handler.solve(&action).await?;
        if !matches!(outcome.status, HandlerStatus::Solved) {
            return Err(AppError::Conflict(format!(
                "{} returned status {:?}",
                handler.name(),
                outcome.status
            )));
        }
        let user_agent = outcome.user_agent.clone().unwrap_or_default();
        let bundle = PxCookieBundle::new(
            outcome.cookies.set.clone(),
            user_agent.clone(),
            SystemTime::now(),
            Duration::from_secs(600),
        );
        Ok(SolveOutput {
            bundle,
            user_agent,
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
