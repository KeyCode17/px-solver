use async_trait::async_trait;
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerOutcome, PageHtml};

pub struct CloudflareHandler;

impl CloudflareHandler {
    pub fn new() -> Self {
        Self
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

    async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        Ok(HandlerOutcome::not_implemented(self.name()))
    }
}
