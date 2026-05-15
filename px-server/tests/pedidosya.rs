#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use px_harvester::{ChromiumoxidePool, HarvestRequest, Harvester, PoolConfig};
use std::time::Duration;

#[derive(Debug)]
enum CanaryFailure {
    Network(String),
    PxBlock(String),
    CookieShapeMismatch(String),
    HarvesterTimeout(String),
}

impl std::fmt::Display for CanaryFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(m) => write!(f, "network: {m}"),
            Self::PxBlock(m) => write!(f, "px-block: {m}"),
            Self::CookieShapeMismatch(m) => write!(f, "cookie-shape-mismatch: {m}"),
            Self::HarvesterTimeout(m) => write!(f, "harvester-timeout: {m}"),
        }
    }
}

#[tokio::test]
#[ignore]
async fn pedidosya_solve_returns_px_cookies() {
    if std::env::var("CI_CANARY").ok().as_deref() != Some("1") {
        eprintln!("skipping canary: CI_CANARY=1 not set");
        return;
    }

    let pool = ChromiumoxidePool::new(PoolConfig {
        max_concurrent: 1,
        navigate_timeout: Duration::from_secs(30),
        headless: true,
    });
    let req = HarvestRequest::new("https://www.pedidosya.com.ar/");

    let outcome = match tokio::time::timeout(Duration::from_secs(45), pool.harvest(req)).await {
        Err(_) => Err(CanaryFailure::HarvesterTimeout("45s timeout".into())),
        Ok(Err(e)) => Err(CanaryFailure::Network(e.to_string())),
        Ok(Ok(r)) => Ok(r),
    };
    let result = outcome.expect("canary harvest");

    let has_px3 = result.cookies.iter().any(|c| c.name == "_px3");
    let title_blocked = result.html.contains("Access to this page has been denied");

    if title_blocked && !has_px3 {
        panic!("{}", CanaryFailure::PxBlock("blocked page returned".into()));
    }
    if !has_px3 {
        panic!(
            "{}",
            CanaryFailure::CookieShapeMismatch(format!(
                "expected _px3 in returned cookies; got {} cookies",
                result.cookies.len()
            ))
        );
    }
}
