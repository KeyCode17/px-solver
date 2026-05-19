#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Round-trip the cipher primitives against fixtures emitted by the
//! JS reference (`px-research/tools/decoder/emit_fixtures.js`).

use std::fs;
use std::path::Path;

use px_native::cipher::{h_p, jw, v_l, v_m, v_n, v_q};
use serde::Deserialize;

const ROOT: &str = "tests/fixtures/cipher";

fn load<T: for<'de> Deserialize<'de>>(name: &str) -> Vec<T> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(ROOT)
        .join(format!("{name}.json"));
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[derive(Deserialize)]
struct JwVec {
    input: Vec<u8>,
    key: u8,
    expected: Vec<u8>,
}

#[test]
fn jw_matches_reference() {
    for v in load::<JwVec>("jw") {
        let got = jw(&v.input, v.key);
        assert_eq!(got, v.expected, "jw mismatch on key={}", v.key);
    }
}

#[derive(Deserialize)]
struct HpVec {
    input: String,
    expected: String,
}

#[test]
fn h_p_matches_reference() {
    for v in load::<HpVec>("hP") {
        assert_eq!(
            h_p(v.input.as_bytes()),
            v.expected,
            "hP mismatch on {:?}",
            v.input
        );
    }
}

#[derive(Deserialize)]
struct VmVec {
    t: i64,
    n: i64,
    e: i64,
    r: i64,
    g: i64,
    expected: i64,
}

#[test]
fn v_m_matches_reference() {
    for v in load::<VmVec>("vM") {
        assert_eq!(v_m(v.t, v.n, v.e, v.r, v.g), v.expected);
    }
}

#[derive(Deserialize)]
struct VlVec {
    pf: String,
    expected: Vec<u8>,
}

#[test]
fn v_l_matches_reference() {
    for v in load::<VlVec>("vL") {
        assert_eq!(v_l(v.pf.as_bytes()), v.expected);
    }
}

#[derive(Deserialize)]
struct VnVec {
    target: String,
    #[serde(rename = "lenBound")]
    len_bound: usize,
    secret: String,
    expected: Vec<i64>,
}

#[test]
fn v_n_matches_reference() {
    for v in load::<VnVec>("vN") {
        let got = v_n(v.target.len(), v.len_bound, v.secret.as_bytes());
        assert_eq!(got, v.expected, "vN mismatch for secret={:?}", v.secret);
    }
}

#[derive(Deserialize)]
struct VqVec {
    salt: String,
    payload: String,
    offsets: Vec<i64>,
    expected: String,
}

#[test]
fn v_q_matches_reference() {
    for v in load::<VqVec>("vQ") {
        let got =
            v_q(v.salt.as_bytes(), v.payload.as_bytes(), &v.offsets).expect("vQ ok in fixture");
        assert_eq!(got.as_slice(), v.expected.as_bytes());
    }
}
