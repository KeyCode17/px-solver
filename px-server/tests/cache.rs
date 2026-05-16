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
