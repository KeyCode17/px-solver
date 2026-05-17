//! Cloudflare challenge handler.
//!
//! Detects Cloudflare interstitials in fetched HTML and, when constructed
//! with a CF-bypass harvester (e.g. [`px-camoufox::CamoufoxPool`]), re-harvests
//! the URL to recover `cf_clearance` / `__cf_bm` plus any downstream PX cookies
//! that the same fetch happens to set.
//!
//! See ADR-0015 (handler stub) and ADR-0020 (Camoufox path).

use async_trait::async_trait;
use px_core::{CookieJarDelta, NamedCookie};
use px_errors::AppError;
use px_harvester::{HarvestRequest, Harvester};
use px_pipeline::{ChallengeHandler, HandlerMetrics, HandlerOutcome, PageHtml};
use std::sync::Arc;
use std::time::Instant;

mod cookie_extractor;
pub use cookie_extractor::{extract_session_cookies, is_session_cookie};

pub struct CloudflareHandler {
    harvester: Option<Arc<dyn Harvester>>,
}

impl CloudflareHandler {
    pub fn new() -> Self {
        Self { harvester: None }
    }

    pub fn with_harvester(harvester: Arc<dyn Harvester>) -> Self {
        Self {
            harvester: Some(harvester),
        }
    }
}

impl Default for CloudflareHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChallengeHandler for CloudflareHandler {
    fn name(&self) -> &'static str {
        "cloudflare"
    }

    async fn detects(&self, page: &PageHtml) -> Result<bool, AppError> {
        let h = &page.html;
        Ok(h.contains("cdn-cgi/challenge-platform")
            || h.contains("cf-mitigated")
            || h.contains("cf_clearance"))
    }

    async fn solve(&self, page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        let Some(harvester) = self.harvester.as_ref() else {
            return Ok(HandlerOutcome::not_implemented(self.name()));
        };
        let start = Instant::now();
        let result = harvester.harvest(HarvestRequest::new(&page.url)).await?;
        let session_cookies: Vec<NamedCookie> = extract_session_cookies(&result.cookies)
            .into_iter()
            .map(|c| NamedCookie {
                name: c.name,
                value: c.value,
                domain: c.domain,
                path: c.path,
            })
            .collect();
        let delta = CookieJarDelta {
            set: session_cookies,
            removed: Vec::new(),
        };
        let metrics = HandlerMetrics {
            detect_us: 0,
            solve_ms: start.elapsed().as_millis() as u64,
            bytes_read: result.html.len() as u64,
        };
        Ok(HandlerOutcome::solved_with_ua(
            self.name(),
            delta,
            Vec::new(),
            metrics,
            result.user_agent,
        ))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use px_harvester::{HarvestResult, HarvestedCookie};

    struct FakeHarvester {
        ua: String,
        cookies: Vec<HarvestedCookie>,
        html: String,
    }

    #[async_trait]
    impl Harvester for FakeHarvester {
        async fn harvest(&self, _req: HarvestRequest) -> Result<HarvestResult, AppError> {
            Ok(HarvestResult {
                html: self.html.clone(),
                user_agent: self.ua.clone(),
                cookies: self.cookies.clone(),
            })
        }
    }

    fn cookie(name: &str) -> HarvestedCookie {
        HarvestedCookie {
            name: name.into(),
            value: "v".into(),
            domain: "x.com".into(),
            path: "/".into(),
        }
    }

    #[tokio::test]
    async fn solve_without_harvester_is_not_implemented() {
        let h = CloudflareHandler::new();
        let page = PageHtml::new("https://x.com", "");
        let oc = h.solve(&page).await.expect("solve");
        assert_eq!(oc.status, px_pipeline::HandlerStatus::NotImplemented);
    }

    #[tokio::test]
    async fn solve_with_harvester_returns_session_cookies_and_ua() {
        let fake = Arc::new(FakeHarvester {
            ua: "Mozilla/5.0 Camoufox".into(),
            cookies: vec![
                cookie("cf_clearance"),
                cookie("__cf_bm"),
                cookie("_px3"),
                cookie("_pxhd"),
                cookie("unrelated_session"),
            ],
            html: "real page".into(),
        });
        let h = CloudflareHandler::with_harvester(fake);
        let page = PageHtml::new("https://x.com", "<challenge>");
        let oc = h.solve(&page).await.expect("solve");
        assert_eq!(oc.status, px_pipeline::HandlerStatus::Solved);
        assert_eq!(oc.user_agent.as_deref(), Some("Mozilla/5.0 Camoufox"));
        let names: Vec<&str> = oc.cookies.set.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"cf_clearance"));
        assert!(names.contains(&"__cf_bm"));
        assert!(names.contains(&"_px3"));
        assert!(names.contains(&"_pxhd"));
        assert!(!names.contains(&"unrelated_session"));
    }
}
