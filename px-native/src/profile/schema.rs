//! TOML-backed tenant profile. Profiles live in
//! `px-native/profiles/<app_id>.toml`; the runtime loads them from a
//! directory passed via configuration.

use std::path::Path;

use px_errors::AppError;
use serde::{Deserialize, Serialize};

/// XOR keys for the two cipher passes the eT15wiaE family of tenants
/// uses. Other tenants can override either or both at load time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct XorKeys {
    /// Payload-cipher byte XOR key (JS `iS`). Default 50.
    pub payload: u8,
    /// Secret-feed byte XOR key (JS `vJ`). Default 10.
    pub secret_feed: u8,
}

impl Default for XorKeys {
    fn default() -> Self {
        Self {
            payload: 50,
            secret_feed: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TenantProfile {
    /// `_pxAppId` value (with `PX` prefix), e.g. `PXeT15wiaE`.
    pub app_id: String,
    /// Lower-cased tenant tag used in URL paths (the appId stripped
    /// of the `PX` prefix), e.g. `eT15wiaE`.
    pub app_id_tag: String,
    /// Sensor POST endpoint, relative to the chosen origin. Joined
    /// after the tenant tag at runtime to form
    /// `${origin}/${app_id_tag}${sensor_path}`.
    pub sensor_path: String,
    /// String the JS reference uses when `pf()` is empty
    /// (decoded `gC(365)` in the captured init.js).
    pub pf_fallback: String,
    /// XOR keys. See [`XorKeys`].
    #[serde(default)]
    pub xor_keys: XorKeys,
}

impl TenantProfile {
    pub fn from_toml(src: &str) -> Result<Self, AppError> {
        toml::from_str::<Self>(src)
            .map_err(|e| AppError::InternalError(format!("tenant profile parse: {e}")))
    }

    pub fn load(path: &Path) -> Result<Self, AppError> {
        let bytes = std::fs::read_to_string(path)
            .map_err(|e| AppError::InternalError(format!("read {}: {e}", path.display())))?;
        Self::from_toml(&bytes)
    }

    /// Build the absolute sensor URL given the page origin.
    pub fn sensor_url(&self, origin: &str) -> String {
        format!(
            "{}/{}{}",
            origin.trim_end_matches('/'),
            self.app_id_tag,
            self.sensor_path,
        )
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_et15wiae_profile() {
        let src = r#"
app_id      = "PXeT15wiaE"
app_id_tag  = "eT15wiaE"
sensor_path = "/xhr/b/s"
pf_fallback = "fallback-pf"

[xor_keys]
payload     = 50
secret_feed = 10
"#;
        let p = TenantProfile::from_toml(src).expect("parse");
        assert_eq!(p.app_id, "PXeT15wiaE");
        assert_eq!(
            p.sensor_url("https://www.pedidosya.com.ar"),
            "https://www.pedidosya.com.ar/eT15wiaE/xhr/b/s"
        );
        assert_eq!(p.xor_keys, XorKeys::default());
    }

    #[test]
    fn xor_keys_default_when_section_omitted() {
        let src = r#"
app_id      = "PXabc"
app_id_tag  = "abc"
sensor_path = "/xhr/b/s"
pf_fallback = ""
"#;
        let p = TenantProfile::from_toml(src).expect("parse");
        assert_eq!(p.xor_keys, XorKeys::default());
    }
}
