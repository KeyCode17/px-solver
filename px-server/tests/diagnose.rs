#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use px_harvester::{ChromiumoxidePool, HarvestRequest, Harvester, PoolConfig};
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn diagnose_pedidosya() {
    if std::env::var("DIAGNOSE").ok().as_deref() != Some("1") {
        eprintln!("set DIAGNOSE=1 to run");
        return;
    }
    let target =
        std::env::var("DIAGNOSE_URL").unwrap_or_else(|_| "https://www.pedidosya.com.ar/".into());

    let headless = std::env::var("DIAGNOSE_HEADLESS")
        .ok()
        .as_deref()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    eprintln!("headless = {headless}");
    let pool = ChromiumoxidePool::new(PoolConfig {
        max_concurrent: 1,
        navigate_timeout: Duration::from_secs(30),
        headless,
    });
    let wait_ms: u64 = std::env::var("DIAGNOSE_WAIT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    let mut req = HarvestRequest::new(&target);
    req.wait_ms = wait_ms;
    eprintln!("wait_ms = {wait_ms}");
    let result = tokio::time::timeout(Duration::from_secs(60), pool.harvest(req))
        .await
        .expect("timeout")
        .expect("harvest err");

    eprintln!("\n=== DIAGNOSE: {target} ===");
    eprintln!("cookies ({}):", result.cookies.len());
    for c in &result.cookies {
        let value_preview: String = c.value.chars().take(40).collect();
        eprintln!(
            "  - {} = {}{}  (domain={}, path={})",
            c.name,
            value_preview,
            if c.value.len() > 40 { "..." } else { "" },
            c.domain,
            c.path
        );
    }
    eprintln!("html length: {}", result.html.len());
    let title_marker = result.html.find("<title>").and_then(|i| {
        result.html[i + 7..]
            .find("</title>")
            .map(|j| &result.html[i + 7..i + 7 + j])
    });
    eprintln!("title: {title_marker:?}");

    let blocked_markers = [
        "Access to this page has been denied",
        "Just a moment",
        "__cf_chl_jschl",
        "Sorry, you have been blocked",
        "Captcha",
        "cf-browser-verification",
    ];
    for m in blocked_markers {
        if result.html.contains(m) {
            eprintln!("MARKER FOUND: {m}");
        }
    }
}
