#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{CountingAuditSink, build_state};
use http::header::AUTHORIZATION;
use px_server::build_router;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let app = build_router(build_state(Arc::new(CountingAuditSink::default())));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn solve_unauthorized_without_header() {
    let app = build_router(build_state(Arc::new(CountingAuditSink::default())));
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/solve")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"url":"https://pedidosya.com.ar/"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn solve_forbidden_for_non_allowlisted_domain() {
    let app = build_router(build_state(Arc::new(CountingAuditSink::default())));
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/solve")
                .header(AUTHORIZATION, "Bearer key1:topsecret")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"url":"https://random.example.com/"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn solve_succeeds_with_valid_key_and_allowlisted_domain() {
    let audit = Arc::new(CountingAuditSink::default());
    let app = build_router(build_state(audit.clone()));
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/solve")
                .header(AUTHORIZATION, "Bearer key1:topsecret")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"url":"https://pedidosya.com.ar/"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(audit.count.load(Ordering::Relaxed), 1);
}
