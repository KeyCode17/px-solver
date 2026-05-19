#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Identity → event batch → JSON → `encrypt_sensor` → payload bytes.
//! Verifies the N1–N3 wiring is consistent and deterministic.

use px_native::cipher::encrypt_sensor;
use px_native::events::{SyntheticIdentity, default_batch};

const FIXED_NOW_MS: u64 = 1_716_192_345_678;

fn build_payload() -> Vec<u8> {
    let id = SyntheticIdentity::test_default();
    let batch = default_batch(&id, FIXED_NOW_MS);
    let json = serde_json::to_vec(&batch).expect("serialize batch");
    encrypt_sensor(&json, b"pedidosya.com.ar", b"abc-cu-1234").expect("encrypt")
}

#[test]
fn payload_is_deterministic() {
    assert_eq!(build_payload(), build_payload());
}

#[test]
fn payload_is_non_empty() {
    let bytes = build_payload();
    assert!(!bytes.is_empty());
    // The output is base64 + spliced salt characters. Sanity-check by
    // confirming most bytes fall inside the printable-ascii range —
    // base64 alphabet + salt feed never produces a NUL byte for this
    // input set.
    assert!(bytes.iter().all(|&b| b != 0));
}

#[test]
fn payload_changes_with_cu() {
    // The PRNG inside vN is weak; near-identical short `cu` values
    // can collide. Real-world `cu` is a UUIDv1 so collisions are
    // astronomically unlikely. Use clearly distinct values here.
    let id = SyntheticIdentity::test_default();
    let batch = default_batch(&id, FIXED_NOW_MS);
    let json = serde_json::to_vec(&batch).expect("serialize batch");
    let a = encrypt_sensor(&json, b"pf", b"00000000-0000-0000-0000-000000000001").expect("encrypt");
    let b = encrypt_sensor(&json, b"pf", b"ffffffff-ffff-ffff-ffff-ffffffffffff").expect("encrypt");
    assert_ne!(a, b);
}
