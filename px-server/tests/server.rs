#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http::header::AUTHORIZATION;
use px_auth::{
    AllowlistEntry, ApiKeyRecord, AuditEvent, AuditSink, CheckAllowlist, VerifyKey,
    YamlAllowlistStore, YamlKeyStore,
};
use px_core::NamedCookie;
use px_errors::AppError;
use px_server::application::solve_endpoint::{SolveDispatcher, SolveOutput};
use px_server::{AppState, AppStateConfig, build_router};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};
use tower::ServiceExt;

fn hash(secret: &str) -> String {
    let salt = SaltString::encode_b64(b"px-solver-test-salt-fixed").expect("salt");
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .expect("hash")
        .to_string()
}

struct FakeDispatcher;

#[async_trait]
impl SolveDispatcher for FakeDispatcher {
    async fn solve(&self, _url: &str) -> Result<SolveOutput, AppError> {
        Ok(SolveOutput {
            bundle: px_core::PxCookieBundle::new(
                vec![NamedCookie {
                    name: "_px3".into(),
                    value: "token".into(),
                    domain: "pedidosya.com.ar".into(),
                    path: "/".into(),
                }],
                "TestUA",
                SystemTime::now(),
                Duration::from_secs(600),
            ),
            user_agent: "TestUA".into(),
            solve_ms: 1,
            cache_hit: false,
            handler: "perimeterx".into(),
        })
    }
}

#[derive(Default)]
struct CountingAuditSink {
    count: AtomicUsize,
}

#[async_trait]
impl AuditSink for CountingAuditSink {
    async fn record(&self, _event: &AuditEvent) -> Result<(), AppError> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

fn build_state(audit: Arc<CountingAuditSink>) -> AppState {
    let key_store = Arc::new(YamlKeyStore::from_records(vec![ApiKeyRecord {
        id: "key1".into(),
        argon2_hash: hash("topsecret"),
        note: None,
    }]));
    let allowlist_store = Arc::new(YamlAllowlistStore::from_entries(vec![AllowlistEntry {
        domain: "pedidosya.com.ar".into(),
        tos_reviewed: true,
        justification: "test".into(),
    }]));
    AppState::new(AppStateConfig {
        verify_key: Arc::new(VerifyKey::new(key_store)),
        check_allowlist: Arc::new(CheckAllowlist::new(allowlist_store)),
        dispatcher: Arc::new(FakeDispatcher),
        audit,
        build_sha: "test",
    })
}

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
