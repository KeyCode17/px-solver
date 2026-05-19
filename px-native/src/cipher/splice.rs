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

/// Returns the spliced payload. Matches the JS `String.prototype.substring`
/// semantics that the reference implementation relies on: arguments
/// less than 0 become 0, greater than `len` become `len`, and if
/// `start > end` they are swapped before slicing.
pub fn v_q(salt: &[u8], payload: &[u8], offsets: &[i64]) -> Result<Vec<u8>, AppError> {
    if offsets.len() < salt.len() {
        return Err(AppError::InternalError(format!(
            "vQ: need {} offsets for salt, got {}",
            salt.len(),
            offsets.len()
        )));
    }
    let mut bd: Vec<u8> = Vec::with_capacity(payload.len() + salt.len());
    let mut be: i64 = 0;
    let plen = payload.len();
    for (bg, salt_byte) in salt.iter().enumerate() {
        let cut = offsets[bg] - (bg as i64) - 1;
        bd.extend_from_slice(substring(payload, be, cut, plen));
        bd.push(*salt_byte);
        be = cut;
    }
    let plen_i = plen as i64;
    bd.extend_from_slice(substring(payload, be, plen_i, plen));
    Ok(bd)
}

/// JS `String.prototype.substring(start, end)` over a byte slice.
fn substring(buf: &[u8], start: i64, end: i64, len: usize) -> &[u8] {
    let len_i = len as i64;
    let mut s = start.clamp(0, len_i);
    let mut e = end.clamp(0, len_i);
    if s > e {
        std::mem::swap(&mut s, &mut e);
    }
    &buf[s as usize..e as usize]
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
