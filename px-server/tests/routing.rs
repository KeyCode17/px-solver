#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use async_trait::async_trait;
use px_core::{CookieJarDelta, SolveRequest};
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerMetrics, HandlerOutcome, PageHtml, SolveAction};
use px_server::application::routing::{RoutingDispatcher, parse_camoufox_domains};
use px_server::application::solve_endpoint::SolveDispatcher;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
struct StaticHandler {
    name: &'static str,
    seen_proxy: Mutex<Option<String>>,
}

impl StaticHandler {
    fn new(name: &'static str) -> Arc<Self> {
        Arc::new(Self {
            name,
            seen_proxy: Mutex::new(None),
        })
    }
}

#[async_trait]
impl ChallengeHandler for StaticHandler {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn detects(&self, _page: &PageHtml) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn solve(&self, action: &SolveAction) -> Result<HandlerOutcome, AppError> {
        *self.seen_proxy.lock().await = action.proxy.clone();
        Ok(HandlerOutcome::solved_with_ua(
            self.name,
            CookieJarDelta::default(),
            Vec::new(),
            HandlerMetrics::default(),
            "ua",
        ))
    }
}

fn dispatcher_with_cf_route() -> RoutingDispatcher {
    RoutingDispatcher::new(StaticHandler::new("perimeterx"))
        .with_route("pedidosya.com.ar", StaticHandler::new("cloudflare"))
}

#[tokio::test]
async fn test_solve_unrouted_host_uses_default_handler() {
    let out = dispatcher_with_cf_route()
        .solve(SolveRequest::new("https://www.havenwellwithin.com/"))
        .await
        .expect("solve");
    assert_eq!(out.handler, "perimeterx");
}

#[tokio::test]
async fn test_solve_exact_host_uses_routed_handler() {
    let out = dispatcher_with_cf_route()
        .solve(SolveRequest::new("https://pedidosya.com.ar/"))
        .await
        .expect("solve");
    assert_eq!(out.handler, "cloudflare");
}

#[tokio::test]
async fn test_solve_subdomain_uses_routed_handler() {
    let out = dispatcher_with_cf_route()
        .solve(SolveRequest::new("https://www.pedidosya.com.ar/x"))
        .await
        .expect("solve");
    assert_eq!(out.handler, "cloudflare");
}

/// Regression: the cookie bundle's `user_agent` must carry the real
/// harvester UA (so cache-hit replies preserve it), not a placeholder.
#[tokio::test]
async fn test_solve_bundle_user_agent_matches_harvester() {
    let out = RoutingDispatcher::new(StaticHandler::new("perimeterx"))
        .solve(SolveRequest::new("https://example.com/"))
        .await
        .expect("solve");
    assert_eq!(out.user_agent, "ua");
    assert_eq!(out.bundle.user_agent, "ua");
}

/// Regression: the request's proxy has to reach the handler. It used to be
/// deserialized at the edge and dropped — no browser ever saw it.
#[tokio::test]
async fn test_solve_forwards_requested_proxy_to_handler() {
    let handler = StaticHandler::new("perimeterx");
    let dispatcher = RoutingDispatcher::new(Arc::clone(&handler) as Arc<dyn ChallengeHandler>);
    dispatcher
        .solve(SolveRequest::new("https://example.com/").with_proxy("socks5://127.0.0.1:9050"))
        .await
        .expect("solve");
    assert_eq!(
        handler.seen_proxy.lock().await.as_deref(),
        Some("socks5://127.0.0.1:9050")
    );
}

#[tokio::test]
async fn test_solve_without_proxy_forwards_none() {
    let handler = StaticHandler::new("perimeterx");
    let dispatcher = RoutingDispatcher::new(Arc::clone(&handler) as Arc<dyn ChallengeHandler>);
    dispatcher
        .solve(SolveRequest::new("https://example.com/"))
        .await
        .expect("solve");
    assert!(handler.seen_proxy.lock().await.is_none());
}

#[test]
fn test_parse_camoufox_domains_trims_and_lowercases() {
    let parsed = parse_camoufox_domains(Some(" Pedidosya.com.AR ,  ,foo.com "));
    assert_eq!(parsed, vec!["pedidosya.com.ar", "foo.com"]);
}

#[test]
fn test_parse_camoufox_domains_unset_is_empty() {
    assert!(parse_camoufox_domains(None).is_empty());
    assert!(parse_camoufox_domains(Some("")).is_empty());
}
