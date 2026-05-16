#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Camoufox harvester probe.
//!
//! Run with:
//!   DIAGNOSE_CAMOUFOX=1 [DIAGNOSE_URL=https://...] [DIAGNOSE_WAIT_MS=10000]
//!     cargo test -q -p px-camoufox --test diagnose -- --ignored --nocapture

use px_camoufox::{CamoufoxConfig, CamoufoxPool};
use px_harvester::{HarvestRequest, Harvester};
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn diagnose_camoufox_harvest() {
    if std::env::var("DIAGNOSE_CAMOUFOX").ok().as_deref() != Some("1") {
        eprintln!("set DIAGNOSE_CAMOUFOX=1 to run");
        return;
    }
    let url =
        std::env::var("DIAGNOSE_URL").unwrap_or_else(|_| "https://www.havenwellwithin.com/".into());
    let wait_ms: u64 = std::env::var("DIAGNOSE_WAIT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);

    let mut cfg = CamoufoxConfig::from_env();
    cfg.headless = std::env::var("DIAGNOSE_HEADLESS")
        .ok()
        .as_deref()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    cfg.navigate_timeout = Duration::from_secs(60);
    eprintln!(
        "camoufox_bin = {}\ngeckodriver_bin = {}\nheadless = {}",
        cfg.camoufox_bin.display(),
        cfg.geckodriver_bin.display(),
        cfg.headless
    );

    let pool = CamoufoxPool::new(cfg).expect("pool");
    let mut req = HarvestRequest::new(&url);
    req.wait_ms = wait_ms;

    let result = tokio::time::timeout(Duration::from_secs(90), pool.harvest(req))
        .await
        .expect("timeout")
        .expect("harvest");

    eprintln!("\n=== DIAGNOSE_CAMOUFOX: {url} ===");
    eprintln!("user_agent: {}", result.user_agent);
    eprintln!("cookies ({}):", result.cookies.len());
    for c in &result.cookies {
        let v: String = c.value.chars().take(40).collect();
        eprintln!(
            "  - {} = {}{}  (domain={}, path={})",
            c.name,
            v,
            if c.value.len() > 40 { "..." } else { "" },
            c.domain,
            c.path
        );
    }
    eprintln!("html length: {}", result.html.len());
    if let Some(i) = result.html.find("<title>") {
        let rest = &result.html[i + 7..];
        if let Some(j) = rest.find("</title>") {
            eprintln!("title: {:?}", &rest[..j]);
        }
    }
    for m in [
        "Access to this page has been denied",
        "Just a moment",
        "__cf_chl_jschl",
        "Verify you are human",
        "cf-browser-verification",
    ] {
        if result.html.contains(m) {
            eprintln!("MARKER FOUND: {m}");
        }
    }
    let has_px3 = result.cookies.iter().any(|c| c.name == "_px3");
    let has_pxhd = result.cookies.iter().any(|c| c.name == "_pxhd");
    let has_cf_clear = result.cookies.iter().any(|c| c.name == "cf_clearance");
    let has_cf_bm = result.cookies.iter().any(|c| c.name == "__cf_bm");
    eprintln!("has _px3:         {has_px3}");
    eprintln!("has _pxhd:        {has_pxhd}");
    eprintln!("has cf_clearance: {has_cf_clear}");
    eprintln!("has __cf_bm:      {has_cf_bm}");
}
