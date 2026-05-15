use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiKeyHash([u8; 32]);

impl ApiKeyHash {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for ApiKeyHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKeyHash(<redacted>)")
    }
}

impl fmt::Display for ApiKeyHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<redacted>")
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn does_not_leak_bytes_in_display_or_debug() {
        let h = ApiKeyHash::new([7u8; 32]);
        assert_eq!(format!("{h}"), "<redacted>");
        assert_eq!(format!("{h:?}"), "ApiKeyHash(<redacted>)");
    }

    #[test]
    fn equality_compares_bytes() {
        let a = ApiKeyHash::new([1u8; 32]);
        let b = ApiKeyHash::new([1u8; 32]);
        let c = ApiKeyHash::new([2u8; 32]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
