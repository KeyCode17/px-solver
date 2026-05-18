//! R5.8 — pedidosya end-to-end canary through the Cloudflare → Camoufox path.
//!
//! Gated behind `CI_CANARY_CF=1` (the env-var name keeps it disjoint from
//! the legacy `CI_CANARY=1` Chromium canary in `pedidosya.rs`). The test
//! drives a real Camoufox subprocess via geckodriver, so it requires both
//! binaries on the host and takes ~30s wall-clock. It also reaches out to
//! `https://www.pedidosya.com.ar/` over the public network; run only with
//! authorization per the dual-use policy.
//!
//! Assertions:
//! - HTTP 200 from `/v1/solve`
//! - response body contains `_px3` (PX session cookie surfaced through the
//!   Camoufox cf-bypass extractor)
//! - response body contains `cf_clearance` (CF Bot Management clearance
//!   cookie — the proof that the Camoufox path actually bypassed CF)

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{CountingAuditSink, body_string, hash};
use http::header::AUTHORIZATION;
use px_auth::{
    AllowlistEntry, ApiKeyRecord, CheckAllowlist, VerifyKey, YamlAllowlistStore, YamlKeyStore,
};
use px_cache::InMemoryCookieCache;
use px_camoufox::{CamoufoxConfig, CamoufoxPool};
use px_cloudflare::CloudflareHandler;
use px_harvester::{ChromiumoxidePool, Harvester, PoolConfig};
use px_perimeterx::PerimeterxHandler;
use px_pipeline::ChallengeHandler;
use px_server::application::routing::RoutingDispatcher;
use px_server::application::solve_endpoint::SolveDispatcher;
use px_server::{AppState, AppStateConfig, build_router};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn pedidosya_solve_via_camoufox_path() {
    if std::env::var("CI_CANARY_CF").ok().as_deref() != Some("1") {
        eprintln!("skipping CF canary: CI_CANARY_CF=1 not set");
        return;
    }

    let cfg = CamoufoxConfig::from_env();
    if let Err(e) = cfg.validate() {
        panic!("Camoufox config invalid: {e}");
    }

    // Default Chromium-only handler for non-CF hosts (won't be exercised here,
    // but the routing dispatcher needs a fallback).
    let chromium: Arc<dyn Harvester> = Arc::new(ChromiumoxidePool::new(PoolConfig {
        max_concurrent: 1,
        navigate_timeout: Duration::from_secs(30),
        headless: true,
    }));
    let perimeterx: Arc<dyn ChallengeHandler> =
        Arc::new(PerimeterxHandler::new(Arc::clone(&chromium)));

    let camoufox: Arc<dyn Harvester> = Arc::new(CamoufoxPool::new(cfg).expect("camoufox pool"));
    let cf_handler: Arc<dyn ChallengeHandler> =
        Arc::new(CloudflareHandler::with_harvester(camoufox));

    let dispatcher: Arc<dyn SolveDispatcher> = Arc::new(
        RoutingDispatcher::new(perimeterx).with_route("pedidosya.com.ar", Arc::clone(&cf_handler)),
    );

    // Auth + allowlist scaffolding mirrors common::build_state_with_dispatcher
    // but expands the allowlist to cover the apex and the www subdomain so
    // the request URL is accepted by check_allowlist (exact-match).
    let key_store = Arc::new(YamlKeyStore::from_records(vec![ApiKeyRecord {
        id: "key1".into(),
        argon2_hash: hash("topsecret"),
        note: None,
    }]));
    let allowlist_store = Arc::new(YamlAllowlistStore::from_entries(vec![
        AllowlistEntry {
            domain: "pedidosya.com.ar".into(),
            tos_reviewed: true,
            justification: "R5.8 canary".into(),
            handler: Some("cloudflare".into()),
        },
        AllowlistEntry {
            domain: "www.pedidosya.com.ar".into(),
            tos_reviewed: true,
            justification: "R5.8 canary".into(),
            handler: Some("cloudflare".into()),
        },
    ]));
    let audit = Arc::new(CountingAuditSink::default());

    let state = AppState::new(AppStateConfig {
        verify_key: Arc::new(VerifyKey::new(key_store)),
        check_allowlist: Arc::new(CheckAllowlist::new(allowlist_store)),
        dispatcher,
        fetch_dispatcher: Arc::new(
            px_server::application::fetch_endpoint::RoutingFetchDispatcher::new(None),
        ),
        cache: Arc::new(InMemoryCookieCache::new()),
        audit,
        build_sha: "canary-cf",
    });
    let app = build_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/solve")
        .header(AUTHORIZATION, "Bearer key1:topsecret")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"url":"https://www.pedidosya.com.ar/"}"#))
        .unwrap();

    let resp = tokio::time::timeout(Duration::from_secs(90), app.oneshot(req))
        .await
        .expect("solve did not return within 90s")
        .expect("router infallible");

    let status = resp.status();
    let body = body_string(resp).await;
    eprintln!("body: {body}");
    assert_eq!(status, StatusCode::OK, "non-200 status; body: {body}");
    assert!(
        body.contains("\"_px3\""),
        "expected _px3 in response cookies; body: {body}"
    );
    assert!(
        body.contains("\"cf_clearance\""),
        "expected cf_clearance in response cookies (Camoufox CF bypass); body: {body}"
    );
    assert!(
        body.contains("\"handler\":\"cloudflare\""),
        "expected handler=cloudflare, indicating RoutingDispatcher routed to CF handler; body: {body}"
    );
}
