#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod common;

use axum::http::StatusCode;
use common::{
    CountingAuditSink, FakeDispatcher, body_string, build_state_with_dispatcher, solve_request,
};
use px_server::build_router;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tower::ServiceExt;

#[tokio::test]
async fn second_solve_for_same_domain_hits_cache() {
    let audit = Arc::new(CountingAuditSink::default());
    let dispatcher = Arc::new(FakeDispatcher::default());
    let state = build_state_with_dispatcher(audit.clone(), dispatcher.clone());
    let app = build_router(state);

    let r1 = app
        .clone()
        .oneshot(solve_request(r#"{"url":"https://pedidosya.com.ar/"}"#))
        .await
        .unwrap();
    assert_eq!(r1.status(), StatusCode::OK);
    let b1 = body_string(r1).await;
    assert!(b1.contains("\"cache_hit\":false"), "first body: {b1}");

    let r2 = app
        .oneshot(solve_request(r#"{"url":"https://pedidosya.com.ar/cart"}"#))
        .await
        .unwrap();
    assert_eq!(r2.status(), StatusCode::OK);
    let b2 = body_string(r2).await;
    assert!(b2.contains("\"cache_hit\":true"), "second body: {b2}");
    assert!(b2.contains("\"handler\":\"cache\""), "second body: {b2}");

    assert_eq!(dispatcher.calls.load(Ordering::Relaxed), 1);
    assert_eq!(audit.count.load(Ordering::Relaxed), 2);
}

/// The egress is part of a bundle's identity: PX binds `_px3` to the IP
/// that earned it, so two solves of one domain through different proxies
/// must not share a cache entry.
#[tokio::test]
async fn solves_through_different_proxies_do_not_share_a_cache_entry() {
    let audit = Arc::new(CountingAuditSink::default());
    let dispatcher = Arc::new(FakeDispatcher::default());
    let state = build_state_with_dispatcher(audit.clone(), dispatcher.clone());
    let app = build_router(state);

    let first = app
        .clone()
        .oneshot(solve_request(
            r#"{"url":"https://pedidosya.com.ar/","proxy":"http://a.example:8080"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert!(body_string(first).await.contains("\"cache_hit\":false"));
    assert_eq!(
        dispatcher.last_proxy.lock().await.as_deref(),
        Some("http://a.example:8080")
    );

    let second = app
        .clone()
        .oneshot(solve_request(
            r#"{"url":"https://pedidosya.com.ar/","proxy":"http://b.example:8080"}"#,
        ))
        .await
        .unwrap();
    assert!(body_string(second).await.contains("\"cache_hit\":false"));

    let repeat = app
        .oneshot(solve_request(
            r#"{"url":"https://pedidosya.com.ar/","proxy":"http://a.example:8080"}"#,
        ))
        .await
        .unwrap();
    assert!(body_string(repeat).await.contains("\"cache_hit\":true"));

    assert_eq!(dispatcher.calls.load(Ordering::Relaxed), 2);
}

/// A direct solve must not be served a bundle harvested through a proxy.
#[tokio::test]
async fn a_proxied_bundle_is_not_replayed_for_a_direct_solve() {
    let audit = Arc::new(CountingAuditSink::default());
    let dispatcher = Arc::new(FakeDispatcher::default());
    let state = build_state_with_dispatcher(audit.clone(), dispatcher.clone());
    let app = build_router(state);

    let proxied = app
        .clone()
        .oneshot(solve_request(
            r#"{"url":"https://pedidosya.com.ar/","proxy":"http://a.example:8080"}"#,
        ))
        .await
        .unwrap();
    assert!(body_string(proxied).await.contains("\"cache_hit\":false"));

    let direct = app
        .oneshot(solve_request(
            r#"{"url":"https://pedidosya.com.ar/","proxy":null}"#,
        ))
        .await
        .unwrap();
    assert!(body_string(direct).await.contains("\"cache_hit\":false"));
    assert!(dispatcher.last_proxy.lock().await.is_none());
    assert_eq!(dispatcher.calls.load(Ordering::Relaxed), 2);
}
