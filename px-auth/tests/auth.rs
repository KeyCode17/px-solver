#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use px_auth::{
    AllowlistEntry, ApiKeyRecord, AuditEvent, AuditOutcome, AuditSink, CheckAllowlist,
    FileAuditSink, VerifyKey, YamlAllowlistStore, YamlKeyStore,
};
use std::sync::Arc;

fn hash(secret: &str) -> String {
    let salt = SaltString::encode_b64(b"px-solver-test-salt-fixed").expect("salt");
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .expect("hash")
        .to_string()
}

#[tokio::test]
async fn verify_key_accepts_correct_secret() {
    let record = ApiKeyRecord {
        id: "key1".into(),
        argon2_hash: hash("topsecret"),
        note: None,
    };
    let store = Arc::new(YamlKeyStore::from_records(vec![record.clone()]));
    let verifier = VerifyKey::new(store);
    let got = verifier.execute("key1", "topsecret").await.expect("verify");
    assert_eq!(got.id, "key1");
}

#[tokio::test]
async fn verify_key_rejects_wrong_secret() {
    let record = ApiKeyRecord {
        id: "key1".into(),
        argon2_hash: hash("right"),
        note: None,
    };
    let store = Arc::new(YamlKeyStore::from_records(vec![record]));
    let verifier = VerifyKey::new(store);
    assert!(verifier.execute("key1", "wrong").await.is_err());
}

#[tokio::test]
async fn verify_key_rejects_unknown_key() {
    let store = Arc::new(YamlKeyStore::from_records(vec![]));
    let verifier = VerifyKey::new(store);
    assert!(verifier.execute("nope", "anything").await.is_err());
}

#[tokio::test]
async fn allowlist_admits_reviewed_entry() {
    let store = Arc::new(YamlAllowlistStore::from_entries(vec![AllowlistEntry {
        domain: "pedidosya.com.ar".into(),
        tos_reviewed: true,
        justification: "research".into(),
    }]));
    let check = CheckAllowlist::new(store);
    let entry = check.execute("pedidosya.com.ar").await.expect("allow");
    assert_eq!(entry.domain, "pedidosya.com.ar");
}

#[tokio::test]
async fn allowlist_rejects_unreviewed_entry() {
    let store = Arc::new(YamlAllowlistStore::from_entries(vec![AllowlistEntry {
        domain: "x.com".into(),
        tos_reviewed: false,
        justification: "test".into(),
    }]));
    let check = CheckAllowlist::new(store);
    assert!(check.execute("x.com").await.is_err());
}

#[tokio::test]
async fn allowlist_rejects_unknown_domain() {
    let store = Arc::new(YamlAllowlistStore::from_entries(vec![]));
    let check = CheckAllowlist::new(store);
    assert!(check.execute("missing.com").await.is_err());
}

#[tokio::test]
async fn file_audit_sink_writes_redacted_jsonl() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("audit.log");
    let sink = FileAuditSink::new(&path);
    let event = AuditEvent {
        timestamp_unix: 1_700_000_000,
        key_id: "key1".into(),
        target_domain: "pedidosya.com.ar".into(),
        outcome: AuditOutcome::Solved,
        latency_ms: 1234,
        handler: Some("perimeterx".into()),
    };
    sink.record(&event).await.expect("record");
    let body = std::fs::read_to_string(&path).expect("read");
    assert!(body.contains("\"target_domain\":\"pedidosya.com.ar\""));
    assert!(!body.contains("_px3"));
    assert!(!body.contains("_pxhd"));
}
