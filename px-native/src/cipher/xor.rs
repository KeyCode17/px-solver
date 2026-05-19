//! `jw(t, n)` — single-byte XOR stream cipher used by the eT15wiaE
//! tenant for both the payload stream (key `IS = 50`) and the secret
//! feed (key `VJ = 10`). See `px-research/notes/eT15wiaE/r3-…`.

/// Payload-cipher XOR key.
pub const IS: u8 = 50;
/// Secret-feed XOR key.
pub const VJ: u8 = 10;

/// XOR every byte of `buf` with `key`.
///
/// JS source: `function jw(t, n) { ... String.fromCharCode(n ^ t.charCodeAt(r)) ... }`
/// All real call sites in the captured init.js pass ASCII-range inputs
/// (`hP` produces base64 bytes; `JSON.stringify` produces UTF-8 bytes
/// without surrogate pairs in our usage), so the JS UTF-16 semantics
/// collapse to byte XOR.
pub fn jw(buf: &[u8], key: u8) -> Vec<u8> {
    buf.iter().map(|b| b ^ key).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_is_involutive() {
        let plain = b"hello world";
        let cipher = jw(plain, IS);
        assert_eq!(jw(&cipher, IS), plain);
    }

    #[test]
    fn empty_input_returns_empty() {
        assert!(jw(b"", IS).is_empty());
    }
}
