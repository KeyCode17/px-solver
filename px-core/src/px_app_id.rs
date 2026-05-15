use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PxAppId(String);

impl PxAppId {
    pub fn new(value: impl Into<String>) -> Result<Self, PxAppIdError> {
        let v = value.into();
        if v.is_empty() {
            return Err(PxAppIdError::Empty);
        }
        if !v.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(PxAppIdError::InvalidChars);
        }
        if v.len() < 6 || v.len() > 12 {
            return Err(PxAppIdError::WrongLength(v.len()));
        }
        Ok(Self(v))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PxAppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PxAppIdError {
    Empty,
    InvalidChars,
    WrongLength(usize),
}

impl fmt::Display for PxAppIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "PxAppId is empty"),
            Self::InvalidChars => write!(f, "PxAppId must be ascii alphanumeric"),
            Self::WrongLength(n) => write!(f, "PxAppId length {n} outside 6..=12"),
        }
    }
}

impl std::error::Error for PxAppIdError {}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_app_ids() {
        assert!(PxAppId::new("eT15wiaE").is_ok());
        assert!(PxAppId::new("feReLYy8").is_ok());
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(PxAppId::new(""), Err(PxAppIdError::Empty));
    }

    #[test]
    fn rejects_special_chars() {
        assert_eq!(PxAppId::new("abc-def"), Err(PxAppIdError::InvalidChars));
    }

    #[test]
    fn rejects_too_short() {
        assert!(matches!(
            PxAppId::new("abc"),
            Err(PxAppIdError::WrongLength(3))
        ));
    }

    #[test]
    fn rejects_too_long() {
        assert!(matches!(
            PxAppId::new("abcdefghijklm"),
            Err(PxAppIdError::WrongLength(13))
        ));
    }

    #[test]
    fn display_round_trip() {
        let id = PxAppId::new("eT15wiaE").expect("valid");
        assert_eq!(format!("{id}"), "eT15wiaE");
    }
}
