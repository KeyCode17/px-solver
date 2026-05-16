#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use px_core::{BlockClass, PxMode};
use px_detector::{Detected, Detector, RegexDetector};

const PEDIDOSYA: &str = include_str!("fixtures/pedidosya-block.html");
const HANNA: &str = include_str!("fixtures/hannaandersson-block.html");
const NON_PX: &str = include_str!("fixtures/non-px-control.html");
const HAVEN: &str = include_str!("fixtures/havenwellwithin-non-px.html");

fn detector() -> RegexDetector {
    RegexDetector::new()
}

#[test]
fn pedidosya_block_page_detected_marker_only() {
    match detector().detect(PEDIDOSYA) {
        Detected::Yes(d) => {
            assert!(
                d.app_id.is_none(),
                "block-page only; app_id not available from HTML"
            );
            assert!(d.init_js_path.is_none());
            assert!(d.collector_base.is_none());
        }
        Detected::No => panic!("expected PX marker-only detection for pedidosya block fixture"),
    }
}

#[test]
fn hannaandersson_detected_with_app_id_and_captcha() {
    match detector().detect(HANNA) {
        Detected::Yes(d) => {
            assert_eq!(d.app_id.as_ref().map(|a| a.as_str()), Some("feReLYy8"));
            assert_eq!(d.init_js_path.as_deref(), Some("/feReLYy8/init.js"));
            assert_eq!(d.collector_base.as_deref(), Some("/feReLYy8/xhr"));
            assert_eq!(d.mode, PxMode::ReverseProxy);
            assert_eq!(d.block_class, BlockClass::Captcha);
        }
        Detected::No => panic!("expected PX detection for hannaandersson fixture"),
    }
}

#[test]
fn non_px_control_returns_no() {
    assert_eq!(detector().detect(NON_PX), Detected::No);
}

#[test]
fn havenwellwithin_unchallenged_response_returns_no() {
    // The site IS PerimeterX-protected (appId PX12Ew76qT, captcha.px-cdn.net
    // hosted-mode). PX did not fire on this captured GET — see
    // px-research/notes/12Ew76qT/r0-notes.md. Detector correctly returns No
    // because this specific HTML carries no markers.
    assert_eq!(detector().detect(HAVEN), Detected::No);
}
