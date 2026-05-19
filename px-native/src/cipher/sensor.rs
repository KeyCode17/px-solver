//! `vP` — top-level sensor encryptor. Chains the six primitives in the
//! captured init.js (line 6050) into the single output the runtime
//! POSTs to `/<tenant>/xhr/b/s`:
//!
//! ```text
//! events_json ── jw(·, IS=50) ── h_p ── v_q(secret_feed, ·, offsets) ── payload
//!                                          ▲
//!                                          │
//!                          v_n(secret_feed.len(), payload_b64.len(), cu)
//!                                          ▲
//!                                          │
//!                          v_l(pf)  (= jw(b64(pf), VJ=10))
//! ```

use px_errors::AppError;

use crate::cipher::b64::h_p;
use crate::cipher::offsets::v_n;
use crate::cipher::secret::v_l;
use crate::cipher::splice::v_q;
use crate::cipher::xor::{IS, jw};

/// Encrypt a fully serialised sensor event batch.
///
/// * `events_json` — UTF-8 bytes of `JSON.stringify(events)` output
///   (`hY` in the JS reference is JSON.stringify for our input shapes).
/// * `pf` — `pf()` page-fingerprint bytes (or the tenant fallback when
///   the runtime would have used `gC(365)`).
/// * `cu` — the `ctx.cu` salt (the `vK` field).
pub fn encrypt_sensor(events_json: &[u8], pf: &[u8], cu: &[u8]) -> Result<Vec<u8>, AppError> {
    let secret_feed = v_l(pf);
    let encrypted = h_p(&jw(events_json, IS)).into_bytes();
    let offsets = v_n(secret_feed.len(), encrypted.len(), cu);
    v_q(&secret_feed, &encrypted, &offsets)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_same_inputs() {
        let a = encrypt_sensor(b"[]", b"pedidosya.com.ar", b"cu-1").expect("encrypt");
        let b = encrypt_sensor(b"[]", b"pedidosya.com.ar", b"cu-1").expect("encrypt");
        assert_eq!(a, b);
    }

    #[test]
    fn output_differs_for_distinct_pf() {
        let a = encrypt_sensor(b"[]", b"pf-A", b"cu-shared").expect("encrypt");
        let b = encrypt_sensor(b"[]", b"pf-B", b"cu-shared").expect("encrypt");
        assert_ne!(a, b);
    }
}
