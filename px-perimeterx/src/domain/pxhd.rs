use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PxHd {
    hex: String,
    vid: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PxHdParseError {
    MissingColon,
    HexLengthInvalid(usize),
    HexNotLowercase,
    VidParse(String),
}

impl fmt::Display for PxHdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColon => f.write_str("_pxhd missing ':' separator"),
            Self::HexLengthInvalid(n) => write!(f, "_pxhd hex length {n} != 64"),
            Self::HexNotLowercase => f.write_str("_pxhd hex contains non lowercase-hex chars"),
            Self::VidParse(m) => write!(f, "_pxhd UUID parse: {m}"),
        }
    }
}

impl std::error::Error for PxHdParseError {}

impl PxHd {
    pub fn parse(s: &str) -> Result<Self, PxHdParseError> {
        let (hex, vid_str) = s.split_once(':').ok_or(PxHdParseError::MissingColon)?;
        if hex.len() != 64 {
            return Err(PxHdParseError::HexLengthInvalid(hex.len()));
        }
        if !hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        {
            return Err(PxHdParseError::HexNotLowercase);
        }
        let vid = Uuid::parse_str(vid_str).map_err(|e| PxHdParseError::VidParse(e.to_string()))?;
        Ok(Self {
            hex: hex.to_string(),
            vid,
        })
    }

    pub fn hex(&self) -> &str {
        &self.hex
    }

    pub fn vid(&self) -> Uuid {
        self.vid
    }

    pub fn synthesize_seed() -> String {
        format!(":{}", Uuid::now_v1(&[0u8; 6]))
    }
}

impl fmt::Display for PxHd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.hex, self.vid)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    const VALID: &str = "63269414886d52c66f141cdfabc15cdc9c83ecb567019be97d56d9a75154dcce:0e429506-50a0-11f1-909a-164f0fd4a91d";

    #[test]
    fn parses_pedidosya_pxhd_shape() {
        let p = PxHd::parse(VALID).expect("valid pxhd");
        assert_eq!(p.hex().len(), 64);
    }

    #[test]
    fn round_trips_via_display() {
        let p = PxHd::parse(VALID).expect("valid pxhd");
        assert_eq!(p.to_string(), VALID);
    }

    #[test]
    fn missing_colon_errors() {
        assert!(matches!(
            PxHd::parse("nocolonhere"),
            Err(PxHdParseError::MissingColon)
        ));
    }

    #[test]
    fn wrong_hex_length_errors() {
        let bad = format!("{}:{}", "ab", "0e429506-50a0-11f1-909a-164f0fd4a91d");
        assert!(matches!(
            PxHd::parse(&bad),
            Err(PxHdParseError::HexLengthInvalid(2))
        ));
    }

    #[test]
    fn uppercase_hex_rejected() {
        let bad = format!(
            "{}:{}",
            "A".repeat(64),
            "0e429506-50a0-11f1-909a-164f0fd4a91d"
        );
        assert_eq!(PxHd::parse(&bad), Err(PxHdParseError::HexNotLowercase));
    }

    #[test]
    fn invalid_uuid_errors() {
        let bad = format!("{}:notauuid", "a".repeat(64));
        assert!(matches!(
            PxHd::parse(&bad),
            Err(PxHdParseError::VidParse(_))
        ));
    }

    #[test]
    fn synthesize_seed_has_empty_hex_half() {
        let s = PxHd::synthesize_seed();
        assert!(s.starts_with(':'));
        let after = &s[1..];
        assert!(Uuid::parse_str(after).is_ok());
    }
}
