//! `vQ(salt, payload, offsets)` — interleave salt characters into the
//! base64 payload at byte positions produced by `vN`. Final step of
//! the eT15wiaE sensor encryption stack.
//!
//! JS reference (un-flattened from the switch/with/case VM at line
//! 6065 of the captured init.js):
//!
//! ```text
//! function vQ(salt, payload, offsets) {
//!   const bf = salt.split('');
//!   let bd = '', be = 0;
//!   for (let bg = 0; bg < salt.length; bg++) {
//!     bd += payload.substring(be, offsets[bg] - bg - 1) + bf[bg];
//!     be = offsets[bg] - bg - 1;
//!   }
//!   bd += payload.substring(be);
//!   return bd;
//! }
//! ```

use px_errors::AppError;

/// Returns the spliced payload. Errors if any offset would index
/// outside the payload — that indicates `vN` was called with a
/// `len_bound` that does not match `payload.len()`.
pub fn v_q(salt: &[u8], payload: &[u8], offsets: &[i64]) -> Result<Vec<u8>, AppError> {
    if offsets.len() < salt.len() {
        return Err(AppError::InternalError(format!(
            "vQ: need {} offsets for salt, got {}",
            salt.len(),
            offsets.len()
        )));
    }
    let mut bd: Vec<u8> = Vec::with_capacity(payload.len() + salt.len());
    let mut be: usize = 0;
    for bg in 0..salt.len() {
        let cut = offsets[bg]
            .checked_sub(bg as i64)
            .and_then(|x| x.checked_sub(1))
            .and_then(|x| usize::try_from(x).ok())
            .filter(|&x| x <= payload.len())
            .ok_or_else(|| AppError::InternalError(format!("vQ: bad offset at {bg}")))?;
        if cut < be {
            return Err(AppError::InternalError(format!(
                "vQ: non-monotonic offset {cut} < {be} at {bg}"
            )));
        }
        bd.extend_from_slice(&payload[be..cut]);
        bd.push(salt[bg]);
        be = cut;
    }
    bd.extend_from_slice(&payload[be..]);
    Ok(bd)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::cipher::offsets::v_n;

    #[test]
    fn round_trip_against_offsets_prng() {
        let salt = b"tag";
        let payload = b"AAAAAAAAAA";
        let offsets = v_n(payload.len(), payload.len(), b"seed");
        let out = v_q(salt, payload, &offsets).expect("splice");
        assert_eq!(out.len(), payload.len() + salt.len());
    }

    #[test]
    fn empty_salt_returns_payload() {
        let out = v_q(b"", b"abcdef", &[]).expect("splice");
        assert_eq!(out, b"abcdef");
    }
}
