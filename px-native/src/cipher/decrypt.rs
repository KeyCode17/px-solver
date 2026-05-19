//! Lossy decrypt of a captured `payload=…` body. The proper inverse
//! of `vQ` needs the per-call secret feed (we don't have it from the
//! capture), so this strips non-base64 chars heuristically — recovers
//! a readable JSON prefix until salt insertions corrupt the stream.
//!
//! Good enough to extract tag + field-key vocabulary from a real
//! capture. For a clean round-trip we'd need to hook the runtime
//! before `vQ` runs (open R-track item).

use crate::cipher::xor::IS;

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn decrypt_payload_lossy(payload: &str) -> String {
    let mut stripped: Vec<u8> = payload.bytes().filter(|b| B64.contains(b)).collect();
    while !stripped.len().is_multiple_of(4) {
        stripped.push(b'=');
    }
    let raw = match b64_decode(&stripped) {
        Some(v) => v,
        None => return String::new(),
    };
    let xored: Vec<u8> = raw.iter().map(|b| b ^ IS).collect();
    String::from_utf8_lossy(&xored).into_owned()
}

fn b64_decode(input: &[u8]) -> Option<Vec<u8>> {
    let mut table = [255u8; 256];
    for (i, &c) in B64.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in input {
        if b == b'=' {
            break;
        }
        let v = table[b as usize];
        if v == 255 {
            return None;
        }
        buf = (buf << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cipher::b64::h_p;
    use crate::cipher::xor::jw;

    #[test]
    fn round_trip_when_no_salt_inserted() {
        // Encrypt with the canonical XOR+b64 path. No vQ here — that
        // means our lossy decryptor should recover the plaintext
        // exactly.
        let plaintext = b"[{\"t\":\"AzNweUVTcEw=\",\"d\":{\"k\":\"v\"}}]";
        let encrypted = h_p(&jw(plaintext, IS));
        let recovered = decrypt_payload_lossy(&encrypted);
        assert_eq!(recovered.as_bytes(), plaintext);
    }
}
