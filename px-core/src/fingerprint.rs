use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fingerprint {
    pub user_agent: String,
    pub accept_language: Vec<String>,
    pub screen_width: u32,
    pub screen_height: u32,
    pub device_pixel_ratio: u32,
    pub timezone: String,
    pub platform: String,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
}

impl Fingerprint {
    pub fn key_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.user_agent.hash(&mut hasher);
        self.platform.hash(&mut hasher);
        self.webgl_vendor.hash(&mut hasher);
        self.webgl_renderer.hash(&mut hasher);
        self.screen_width.hash(&mut hasher);
        self.screen_height.hash(&mut hasher);
        self.timezone.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn fp_for(ua: &str) -> Fingerprint {
        Fingerprint {
            user_agent: ua.into(),
            accept_language: vec!["en".into()],
            screen_width: 1920,
            screen_height: 1080,
            device_pixel_ratio: 1,
            timezone: "UTC".into(),
            platform: "Linux".into(),
            webgl_vendor: "Intel".into(),
            webgl_renderer: "Iris".into(),
        }
    }

    #[test]
    fn same_inputs_produce_same_key_hash() {
        let a = fp_for("Mozilla/5.0");
        let b = fp_for("Mozilla/5.0");
        assert_eq!(a.key_hash(), b.key_hash());
    }

    #[test]
    fn different_ua_changes_key_hash() {
        let a = fp_for("Mozilla/5.0 A");
        let b = fp_for("Mozilla/5.0 B");
        assert_ne!(a.key_hash(), b.key_hash());
    }
}
