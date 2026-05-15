use crate::px_app_id::PxAppId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub domain: String,
    pub app_id: PxAppId,
    pub fingerprint_key: u64,
}

impl CacheKey {
    pub fn new(domain: impl Into<String>, app_id: PxAppId, fingerprint_key: u64) -> Self {
        Self {
            domain: domain.into(),
            app_id,
            fingerprint_key,
        }
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{:016x}",
            self.domain, self.app_id, self.fingerprint_key
        )
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn key(domain: &str, app: &str, fp: u64) -> CacheKey {
        let app_id = PxAppId::new(app).expect("valid app id");
        CacheKey::new(domain, app_id, fp)
    }

    #[test]
    fn equality_uses_all_fields() {
        assert_eq!(
            key("pedidosya.com.ar", "eT15wiaE", 7),
            key("pedidosya.com.ar", "eT15wiaE", 7)
        );
        assert_ne!(
            key("pedidosya.com.ar", "eT15wiaE", 7),
            key("hannaandersson.com", "eT15wiaE", 7)
        );
        assert_ne!(
            key("pedidosya.com.ar", "eT15wiaE", 7),
            key("pedidosya.com.ar", "feReLYy8", 7)
        );
    }

    #[test]
    fn display_format_is_stable() {
        let k = key("pedidosya.com.ar", "eT15wiaE", 0xdead);
        assert_eq!(k.to_string(), "pedidosya.com.ar|eT15wiaE|000000000000dead");
    }
}
