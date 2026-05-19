//! `hP / hQ` — standard base64 encode over the alphabet `hI`
//! captured at line 583 of the eT15wiaE init.js:
//!
//! `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=`
//!
//! Encoder matches `=` padding length per JS reference. We do not pull
//! the `base64` crate so the cipher core stays dependency-free.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn h_p(input: &[u8]) -> String {
    if input.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let o =
            (u32::from(input[i]) << 16) | (u32::from(input[i + 1]) << 8) | u32::from(input[i + 2]);
        out.push(ALPHABET[((o >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((o >> 12) & 63) as usize] as char);
        out.push(ALPHABET[((o >> 6) & 63) as usize] as char);
        out.push(ALPHABET[(o & 63) as usize] as char);
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let o = u32::from(input[i]) << 16;
        out.push(ALPHABET[((o >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((o >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let o = (u32::from(input[i]) << 16) | (u32::from(input[i + 1]) << 8);
        out.push(ALPHABET[((o >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((o >> 12) & 63) as usize] as char);
        out.push(ALPHABET[((o >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc4648_vectors() {
        assert_eq!(h_p(b""), "");
        assert_eq!(h_p(b"f"), "Zg==");
        assert_eq!(h_p(b"fo"), "Zm8=");
        assert_eq!(h_p(b"foo"), "Zm9v");
        assert_eq!(h_p(b"foob"), "Zm9vYg==");
        assert_eq!(h_p(b"fooba"), "Zm9vYmE=");
        assert_eq!(h_p(b"foobar"), "Zm9vYmFy");
    }
}
