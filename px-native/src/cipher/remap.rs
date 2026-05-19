//! `vM(t, n, e, r, g)` — linear remap from `[n, e]` to `[r, g]`,
//! `Math.floor`-rounded. JS reference:
//!
//! ```text
//! function vM(t, n, e, r, g) {
//!   return Math.floor((t - n) / (e - n) * (g - r) + r);
//! }
//! ```
//!
//! All call sites we observe pass non-negative integers and `n < e`,
//! so the integer-cast path is exact; we still go through `f64` to
//! match JS rounding when intermediate products overflow u32.

#[inline]
pub fn v_m(t: i64, n: i64, e: i64, r: i64, g: i64) -> i64 {
    let denom = (e - n) as f64;
    let numer = (t - n) as f64;
    let scale = (g - r) as f64;
    let bias = r as f64;
    (numer / denom * scale + bias).floor() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remap_midpoint() {
        assert_eq!(v_m(50, 0, 100, 0, 10), 5);
    }

    #[test]
    fn remap_zero_zero() {
        assert_eq!(v_m(0, 0, 1, 0, 1), 0);
    }
}
