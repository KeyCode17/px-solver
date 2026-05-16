use std::fmt;
use uuid::Uuid;

const VS_BASE: u32 = 0xE0100;
const VS_DIGIT_0: u32 = VS_BASE + 0x30;
const VS_DIGIT_9: u32 = VS_BASE + 0x39;
const UUID_STR_LEN: usize = 36;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sid {
    uuid: Uuid,
    ms_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidError {
    UuidTooShort,
    UuidParse(String),
    EmptyWatermark,
    UnexpectedCodepoint(u32),
    DigitParse,
}

impl fmt::Display for SidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UuidTooShort => f.write_str("sid is shorter than the 36-char UUID prefix"),
            Self::UuidParse(m) => write!(f, "sid UUID parse: {m}"),
            Self::EmptyWatermark => f.write_str("sid carries no variation-selector watermark"),
            Self::UnexpectedCodepoint(cp) => {
                write!(f, "sid contains non-VS codepoint U+{cp:05X} after UUID")
            }
            Self::DigitParse => f.write_str("sid watermark digits do not form a u64 ms epoch"),
        }
    }
}

impl std::error::Error for SidError {}

impl Sid {
    pub fn new(uuid: Uuid, ms_epoch: u64) -> Self {
        Self { uuid, ms_epoch }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn ms_epoch(&self) -> u64 {
        self.ms_epoch
    }

    pub fn to_watermarked(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.uuid.to_string());
        for d in self.ms_epoch.to_string().bytes() {
            let cp = VS_BASE + u32::from(d);
            if let Some(c) = char::from_u32(cp) {
                out.push(c);
            }
        }
        out
    }

    pub fn parse_watermarked(s: &str) -> Result<Self, SidError> {
        if s.len() < UUID_STR_LEN {
            return Err(SidError::UuidTooShort);
        }
        let uuid =
            Uuid::parse_str(&s[..UUID_STR_LEN]).map_err(|e| SidError::UuidParse(e.to_string()))?;
        let tail = &s[UUID_STR_LEN..];
        let mut digits = String::new();
        for c in tail.chars() {
            let cp = c as u32;
            if (VS_DIGIT_0..=VS_DIGIT_9).contains(&cp) {
                let ascii_byte = (cp - VS_BASE) as u8;
                digits.push(ascii_byte as char);
            } else {
                return Err(SidError::UnexpectedCodepoint(cp));
            }
        }
        if digits.is_empty() {
            return Err(SidError::EmptyWatermark);
        }
        let ms_epoch = digits.parse::<u64>().map_err(|_| SidError::DigitParse)?;
        Ok(Self { uuid, ms_epoch })
    }
}

impl fmt::Display for Sid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_watermarked())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    const HAVEN_SID: &str = "2d86b9bc-50d9-11f1-889b-8b3f65c0b297\u{E0131}\u{E0137}\u{E0137}\u{E0138}\u{E0139}\u{E0130}\u{E0132}\u{E0139}\u{E0132}\u{E0132}\u{E0136}\u{E0139}\u{E0139}";
    const PED_SID: &str = "0be0e2ba-50df-11f1-b5b6-61f398873cea\u{E0131}\u{E0137}\u{E0137}\u{E0138}\u{E0139}\u{E0130}\u{E0135}\u{E0134}\u{E0134}\u{E0130}\u{E0131}\u{E0138}\u{E0130}";

    #[test]
    fn parses_haven_sid_from_recon() {
        let sid = Sid::parse_watermarked(HAVEN_SID).expect("parse haven sid");
        assert_eq!(sid.ms_epoch(), 1_778_902_922_699);
        assert_eq!(
            sid.uuid().to_string(),
            "2d86b9bc-50d9-11f1-889b-8b3f65c0b297"
        );
    }

    #[test]
    fn parses_ped_sid_from_recon() {
        let sid = Sid::parse_watermarked(PED_SID).expect("parse ped sid");
        assert_eq!(sid.ms_epoch(), 1_778_905_440_180);
        assert_eq!(
            sid.uuid().to_string(),
            "0be0e2ba-50df-11f1-b5b6-61f398873cea"
        );
    }

    #[test]
    fn round_trip_preserves_uuid_and_epoch() {
        let uuid = Uuid::parse_str("2d86b9bc-50d9-11f1-889b-8b3f65c0b297").expect("uuid");
        let sid = Sid::new(uuid, 1_778_902_922_699);
        let s = sid.to_watermarked();
        let back = Sid::parse_watermarked(&s).expect("round-trip");
        assert_eq!(back, sid);
    }

    #[test]
    fn watermarked_has_invisible_tail_only() {
        let uuid = Uuid::parse_str("2d86b9bc-50d9-11f1-889b-8b3f65c0b297").expect("uuid");
        let sid = Sid::new(uuid, 1).to_watermarked();
        assert_eq!(&sid[..UUID_STR_LEN], "2d86b9bc-50d9-11f1-889b-8b3f65c0b297");
        let tail: Vec<u32> = sid[UUID_STR_LEN..].chars().map(|c| c as u32).collect();
        assert_eq!(tail, vec![0xE0131]);
    }

    #[test]
    fn rejects_no_watermark() {
        let bare = "2d86b9bc-50d9-11f1-889b-8b3f65c0b297";
        assert_eq!(Sid::parse_watermarked(bare), Err(SidError::EmptyWatermark));
    }

    #[test]
    fn rejects_too_short() {
        assert_eq!(Sid::parse_watermarked("short"), Err(SidError::UuidTooShort));
    }

    #[test]
    fn rejects_unexpected_codepoint_in_tail() {
        let bad = "2d86b9bc-50d9-11f1-889b-8b3f65c0b297\u{E0131}\u{1F600}";
        match Sid::parse_watermarked(bad) {
            Err(SidError::UnexpectedCodepoint(0x1_F600)) => {}
            other => panic!("expected UnexpectedCodepoint(0x1F600), got {other:?}"),
        }
    }
}
