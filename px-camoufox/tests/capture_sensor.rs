#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Live ground-truth capture for the px-3 sensor.
//!
//! Run manually with:
//!   CAPTURE_PX=1 \
//!     [CAPTURE_URL=https://www.pedidosya.com.ar/] \
//!     [CAPTURE_WAIT_MS=12000] \
//!     [CAPTURE_OUT=px-research/captures/eT15wiaE/<date>.json] \
//!     [PX_PROXIES=socks5://...] \
//!     cargo test -q -p pxsolver-camoufox --test capture_sensor -- --ignored --nocapture
//!
//! The output JSON contains:
//!   * `plaintext_events`: every `[{t, d}, …]` batch the runtime
//!     JSON-stringified during the wait window (= the input to `vP`).
//!   * `xhr_sends`: URL + body of every XHR that hit `/b/s` (= the
//!     wire-format payload after `vP`).
//!   * `cookies`: post-solve cookie jar.
//!   * `user_agent`: navigator.userAgent observed in the session.
//!
//! Use the captured batches to calibrate `px-native::events::default_batch`
//! (ADR-0024 N3 follow-up).

use px_camoufox::{CamoufoxConfig, capture_sensor};
use std::path::PathBuf;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn live_capture_for_calibration() {
    if std::env::var("CAPTURE_PX").ok().as_deref() != Some("1") {
        eprintln!("set CAPTURE_PX=1 to run");
        return;
    }
    let url =
        std::env::var("CAPTURE_URL").unwrap_or_else(|_| "https://www.pedidosya.com.ar/".into());
    let wait_ms: u64 = std::env::var("CAPTURE_WAIT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12_000);
    let out: PathBuf = std::env::var("CAPTURE_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            PathBuf::from(format!("../px-research/captures/eT15wiaE/{secs}.json"))
        });

    let mut cfg = CamoufoxConfig::from_env();
    cfg.navigate_timeout = Duration::from_secs(60);
    eprintln!(
        "capturing: url={url} wait_ms={wait_ms} out={}",
        out.display()
    );

    let proxy = std::env::var("PX_PROXIES")
        .ok()
        .and_then(|s| s.split(',').next().map(|p| p.trim().to_string()));

    let result = tokio::time::timeout(
        Duration::from_secs(120),
        capture_sensor(&cfg, proxy.as_deref(), &url, wait_ms),
    )
    .await
    .expect("capture timeout")
    .expect("capture ok");

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create capture dir");
    }
    let json = serde_json::to_string_pretty(&result).expect("serialize");
    std::fs::write(&out, json).expect("write capture");

    eprintln!(
        "\n=== CAPTURE_PX ok ===\n  plaintext_events: {}\n  all_stringify:    {}\n  xhr_sends:        {}\n  cookies:          {}\n  user_agent:       {}\n  wrote:            {}",
        result.plaintext_events.len(),
        result.all_stringify.len(),
        result.xhr_sends.len(),
        result.cookies.len(),
        result.user_agent,
        out.display(),
    );
}
