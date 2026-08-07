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
use px_pipeline::{ChallengeHandler, HandlerMetrics, HandlerOutcome, PageHtml, SolveAction};
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

    async fn solve(&self, action: &SolveAction) -> Result<HandlerOutcome, AppError> {
        let Some(harvester) = self.harvester.as_ref() else {
            return Ok(HandlerOutcome::not_implemented(self.name()));
        };
        let start = Instant::now();
        let request = HarvestRequest::new(action.url()).with_proxy(action.proxy.clone());
        let result = harvester.harvest(request).await?;
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
    use tokio::sync::Mutex;

    struct FakeHarvester {
        ua: String,
        cookies: Vec<HarvestedCookie>,
        html: String,
        seen_proxy: Mutex<Option<String>>,
    }

    impl FakeHarvester {
        fn new(ua: &str, cookies: Vec<HarvestedCookie>, html: &str) -> Self {
            Self {
                ua: ua.into(),
                cookies,
                html: html.into(),
                seen_proxy: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl Harvester for FakeHarvester {
        async fn harvest(&self, req: HarvestRequest) -> Result<HarvestResult, AppError> {
            *self.seen_proxy.lock().await = req.proxy.clone();
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
        let action = SolveAction::new(PageHtml::new("https://x.com", ""));
        let oc = h.solve(&action).await.expect("solve");
        assert_eq!(oc.status, px_pipeline::HandlerStatus::NotImplemented);
    }

    /// Regression: the request's proxy has to reach the harvester. It used
    /// to be parsed at the edge and dropped before any browser saw it.
    #[tokio::test]
    async fn solve_forwards_the_requested_proxy_to_the_harvester() {
        let fake = Arc::new(FakeHarvester::new(
            "ua",
            vec![cookie("cf_clearance")],
            "page",
        ));
        let h = CloudflareHandler::with_harvester(Arc::clone(&fake) as Arc<dyn Harvester>);
        let action = SolveAction::new(PageHtml::new("https://x.com", "<challenge>"))
            .with_proxy(Some("socks5://127.0.0.1:9050".into()));
        let _ = h.solve(&action).await.expect("solve");
        assert_eq!(
            fake.seen_proxy.lock().await.as_deref(),
            Some("socks5://127.0.0.1:9050")
        );
    }

    #[tokio::test]
    async fn solve_with_harvester_returns_session_cookies_and_ua() {
        let fake = Arc::new(FakeHarvester::new(
            "Mozilla/5.0 Camoufox",
            vec![
                cookie("cf_clearance"),
                cookie("__cf_bm"),
                cookie("_px3"),
                cookie("_pxhd"),
                cookie("unrelated_session"),
            ],
            "real page",
        ));
        let h = CloudflareHandler::with_harvester(fake);
        let action = SolveAction::new(PageHtml::new("https://x.com", "<challenge>"));
        let oc = h.solve(&action).await.expect("solve");
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
