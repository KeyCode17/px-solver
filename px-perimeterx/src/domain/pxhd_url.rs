use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PxHdUrl {
    head: String,
    tail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PxHdUrlParseError {
    MissingColon,
    TooManyColons,
    EmptyHead,
    EmptyTail,
    InvalidCharsInHead,
    InvalidCharsInTail,
}

impl fmt::Display for PxHdUrlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColon => f.write_str("pxhd URL form missing ':' separator"),
            Self::TooManyColons => f.write_str("pxhd URL form has more than one ':'"),
            Self::EmptyHead => f.write_str("pxhd URL form head is empty"),
            Self::EmptyTail => f.write_str("pxhd URL form tail is empty"),
            Self::InvalidCharsInHead => f.write_str("pxhd head contains non base64-ish chars"),
            Self::InvalidCharsInTail => f.write_str("pxhd tail contains non base64-ish chars"),
        }
    }
}

impl std::error::Error for PxHdUrlParseError {}

fn is_b64ish(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=' | '-' | '_')
}

impl PxHdUrl {
    pub fn parse(s: &str) -> Result<Self, PxHdUrlParseError> {
        let mut parts = s.split(':');
        let head = parts.next().ok_or(PxHdUrlParseError::MissingColon)?;
        let tail = parts.next().ok_or(PxHdUrlParseError::MissingColon)?;
        if parts.next().is_some() {
            return Err(PxHdUrlParseError::TooManyColons);
        }
        if head.is_empty() {
            return Err(PxHdUrlParseError::EmptyHead);
        }
        if tail.is_empty() {
            return Err(PxHdUrlParseError::EmptyTail);
        }
        if !head.chars().all(is_b64ish) {
            return Err(PxHdUrlParseError::InvalidCharsInHead);
        }
        if !tail.chars().all(is_b64ish) {
            return Err(PxHdUrlParseError::InvalidCharsInTail);
        }
        Ok(Self {
            head: head.to_string(),
            tail: tail.to_string(),
        })
    }

    pub fn head(&self) -> &str {
        &self.head
    }

    pub fn tail(&self) -> &str {
        &self.tail
    }
}

impl fmt::Display for PxHdUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.head, self.tail)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    const HAVEN_PXHD_URL: &str = "NyR4JwB5SECKy8JJleb52Z19g4hQa8vWnJyxAxdmgS7itr7L0us8mc/xzz8MaFD1ve4jCP7KzkNY6n56Zw2ZCA==:0UFopDiXm-3tcYf0DHvKC4a/ZaKLJWmk2TqQnbfn3joC4rHlJdZmoJOSWFTHJD3ocgy-RcwKpbVrxgc2WoiN7SfezW7F1Y9OWNN6S/OpwfI=";
    const PED_PXHD_URL: &str = "uvgtwm/pJXuzshUHm/uGZyF/8QqarE-/35paNE20eatLsAo7XbkuF/2hinVRV6efdjXydloZjn4ru8svh7bgoA==:lXcAiXdsMIsfobUMjncmdJkoxUlSiwgGLF3tYsIphOY2KQ1qwKZq27vuf559W79sAd/YCi1SM/KEtK9/iPmZZNxXvheqLgyrN5lIdKG0pRs=";

    #[test]
    fn parses_haven_pxhd_url_from_recon() {
        let p = PxHdUrl::parse(HAVEN_PXHD_URL).expect("haven pxhd url");
        assert!(!p.head().is_empty());
        assert!(!p.tail().is_empty());
    }

    #[test]
    fn parses_ped_pxhd_url_from_recon() {
        let p = PxHdUrl::parse(PED_PXHD_URL).expect("ped pxhd url");
        assert!(!p.head().is_empty());
        assert!(!p.tail().is_empty());
    }

    #[test]
    fn round_trip_via_display() {
        let p = PxHdUrl::parse(HAVEN_PXHD_URL).expect("parse");
        assert_eq!(p.to_string(), HAVEN_PXHD_URL);
    }

    #[test]
    fn missing_colon_errors() {
        assert_eq!(
            PxHdUrl::parse("nocolonhere"),
            Err(PxHdUrlParseError::MissingColon)
        );
    }

    #[test]
    fn too_many_colons_errors() {
        assert_eq!(
            PxHdUrl::parse("aaa:bbb:ccc"),
            Err(PxHdUrlParseError::TooManyColons)
        );
    }

    #[test]
    fn empty_head_errors() {
        assert_eq!(PxHdUrl::parse(":tail"), Err(PxHdUrlParseError::EmptyHead));
    }

    #[test]
    fn empty_tail_errors() {
        assert_eq!(PxHdUrl::parse("head:"), Err(PxHdUrlParseError::EmptyTail));
    }

    #[test]
    fn rejects_non_b64ish_head() {
        assert_eq!(
            PxHdUrl::parse("ab cd:efgh"),
            Err(PxHdUrlParseError::InvalidCharsInHead)
        );
    }

    #[test]
    fn rejects_non_b64ish_tail() {
        assert_eq!(
            PxHdUrl::parse("abcd:ef gh"),
            Err(PxHdUrlParseError::InvalidCharsInTail)
        );
    }
}
