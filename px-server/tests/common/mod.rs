#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic, dead_code)]

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::Request;
use http::header::AUTHORIZATION;
use px_auth::{
    AllowlistEntry, ApiKeyRecord, AuditEvent, AuditSink, CheckAllowlist, VerifyKey,
    YamlAllowlistStore, YamlKeyStore,
};
use px_cache::InMemoryCookieCache;
use px_core::NamedCookie;
use px_errors::AppError;
use px_server::application::solve_endpoint::{SolveDispatcher, SolveOutput};
use px_server::{AppState, AppStateConfig};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

pub fn hash(secret: &str) -> String {
    let salt = SaltString::encode_b64(b"px-solver-test-salt-fixed").expect("salt");
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .expect("hash")
        .to_string()
}

#[derive(Default)]
pub struct FakeDispatcher {
    pub calls: AtomicUsize,
}

#[async_trait]
impl SolveDispatcher for FakeDispatcher {
    async fn solve(&self, _url: &str) -> Result<SolveOutput, AppError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
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
pub struct CountingAuditSink {
    pub count: AtomicUsize,
}

#[async_trait]
impl AuditSink for CountingAuditSink {
    async fn record(&self, _event: &AuditEvent) -> Result<(), AppError> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

pub fn build_state(audit: Arc<CountingAuditSink>) -> AppState {
    build_state_with_dispatcher(audit, Arc::new(FakeDispatcher::default()))
}

pub fn build_state_with_dispatcher(
    audit: Arc<CountingAuditSink>,
    dispatcher: Arc<dyn SolveDispatcher>,
) -> AppState {
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
        dispatcher,
        cache: Arc::new(InMemoryCookieCache::new()),
        audit,
        build_sha: "test",
    })
}

pub fn solve_request(body: &'static str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/solve")
        .header(AUTHORIZATION, "Bearer key1:topsecret")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

pub async fn body_string(resp: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}
