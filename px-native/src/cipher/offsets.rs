//! `vN(target, lenBound, secret)` — offsets PRNG. See
//! `r3-sensor-grammar-2026-05-20.md` for the un-flattened VM trace.
//!
//! Produces the sorted offsets array consumed by `vQ` to splice salt
//! characters into the payload.

use crate::cipher::b64::h_p;
use crate::cipher::remap::v_m;
use crate::cipher::xor::{VJ, jw};

pub fn v_n(target_len: usize, len_bound: usize, secret: &[u8]) -> Vec<i64> {
    let a_f = jw(h_p(secret).as_bytes(), VJ);
    if a_f.is_empty() {
        return Vec::new();
    }
    let len_bound_i = len_bound as i64;
    let a_h = max_product(&a_f, target_len);
    let mut a_g: Vec<i64> = Vec::with_capacity(target_len);
    for a_m in 0..target_len {
        let a_n = (a_m / a_f.len()) + 1;
        let a_o = a_m % a_f.len();
        let mut a_p = i64::from(a_f[a_o]) * i64::from(a_f[a_n]);
        if a_p >= len_bound_i {
            a_p = v_m(a_p, 0, a_h, 0, len_bound_i - 1);
        }
        while a_g.contains(&a_p) {
            a_p += 1;
        }
        a_g.push(a_p);
    }
    a_g.sort_unstable();
    a_g
}

fn max_product(a_f: &[u8], target_len: usize) -> i64 {
    let mut a_h: i64 = -1;
    for a_i in 0..target_len {
        // JS: aJ = floor(aI / aF.length + 1) — the +1 is INSIDE the
        // division before floor(), matching the first VM arm.
        let a_j = ((a_i as f64) / (a_f.len() as f64) + 1.0).floor() as usize;
        let a_k = if a_i >= a_f.len() {
            a_i % a_f.len()
        } else {
            a_i
        };
        let a_l = i64::from(a_f[a_k]) * i64::from(a_f[a_j]);
        if a_l > a_h {
            a_h = a_l;
        }
    }
    a_h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_is_sorted_and_sized() {
        let offsets = v_n(50, 50, b"some-secret");
        assert_eq!(offsets.len(), 50);
        for w in offsets.windows(2) {
            assert!(w[0] <= w[1]);
        }
    }

    #[test]
    fn empty_secret_returns_empty() {
        assert!(v_n(10, 10, b"").is_empty());
    }
}
