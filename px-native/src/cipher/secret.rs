//! `vL()` — secret-feed builder. JS reference:
//!
//! ```text
//! function vL() { return jw(hP(pf() || gC(365)), vJ); }   // vJ = 10
//! ```
//!
//! In JS, `pf()` returns the page fingerprint; here the caller passes
//! whatever bytes the runtime would have collected. The fallback
//! string is tenant-specific (decoded as `gC(365)` in the captured
//! init.js) — clients select the right fallback via the tenant
//! profile (ADR-0024 N5).

use crate::cipher::b64::h_p;
use crate::cipher::xor::{VJ, jw};

pub fn v_l(pf_or_fallback: &[u8]) -> Vec<u8> {
    jw(h_p(pf_or_fallback).as_bytes(), VJ)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_empty_feed() {
        assert!(v_l(b"").is_empty());
    }

    #[test]
    fn deterministic_for_same_input() {
        assert_eq!(v_l(b"pedidosya"), v_l(b"pedidosya"));
    }
}
