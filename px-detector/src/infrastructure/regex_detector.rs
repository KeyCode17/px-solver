use crate::domain::detector::{Detected, Detector};
use px_core::{BlockClass, PxAppId, PxDetection, PxMode};
use regex::Regex;
use std::sync::OnceLock;

pub struct RegexDetector;

impl RegexDetector {
    pub fn new() -> Self {
        Self
    }

    fn block_class_for(html: &str) -> BlockClass {
        if has_captcha_marker(html) {
            BlockClass::Captcha
        } else if has_block_marker(html) {
            BlockClass::Block
        } else {
            BlockClass::None
        }
    }

    fn mode_for(html: &str) -> PxMode {
        if first_party_enabled(html) {
            PxMode::ReverseProxy
        } else {
            PxMode::Hosted
        }
    }
}

impl Default for RegexDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for RegexDetector {
    fn detect(&self, html: &str) -> Detected {
        let mode = Self::mode_for(html);
        let block_class = Self::block_class_for(html);

        if let Some(app_id) = extract_app_id(html)
            && let Ok(typed) = PxAppId::new(&app_id)
        {
            let init_js_path = format!("/{app_id}/init.js");
            let collector_base = format!("/{app_id}/xhr");
            return Detected::Yes(PxDetection::with_app_id(
                typed,
                init_js_path,
                collector_base,
                mode,
                block_class,
            ));
        }

        if has_px_marker(html) {
            return Detected::Yes(PxDetection::marker_only(mode, block_class));
        }

        Detected::No
    }
}

#[allow(clippy::expect_used)]
fn app_id_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r#"_pxAppId\s*=\s*['"]PX([A-Za-z0-9]{6,12})['"]"#)
            .expect("static appId regex is valid")
    })
}

#[allow(clippy::expect_used)]
fn first_party_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r#"_pxFirstPartyEnabled\s*=\s*true"#).expect("static first-party regex is valid")
    })
}

#[allow(clippy::expect_used)]
fn host_init_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r#"/([A-Za-z0-9]{6,12})/init\.js"#).expect("static host_init regex is valid")
    })
}

fn extract_app_id(html: &str) -> Option<String> {
    if let Some(caps) = app_id_re().captures(html) {
        return Some(caps.get(1)?.as_str().to_string());
    }
    host_init_re()
        .captures(html)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

fn first_party_enabled(html: &str) -> bool {
    first_party_re().is_match(html)
}

fn has_captcha_marker(html: &str) -> bool {
    html.contains("px-captcha") || html.contains("captcha.js")
}

fn has_block_marker(html: &str) -> bool {
    html.contains("Access to this page has been denied") || html.contains("PerimeterX")
}

fn has_px_marker(html: &str) -> bool {
    html.contains("PX2") || html.contains("PerimeterX") || html.contains("_pxAppId")
}
