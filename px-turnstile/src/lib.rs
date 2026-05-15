use async_trait::async_trait;
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerOutcome, PageHtml};

pub struct TurnstileHandler;

impl TurnstileHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TurnstileHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChallengeHandler for TurnstileHandler {
    fn name(&self) -> &'static str {
        "turnstile"
    }

    async fn detects(&self, page: &PageHtml) -> Result<bool, AppError> {
        let h = &page.html;
        Ok(h.contains("challenges.cloudflare.com/turnstile") || h.contains("cf-turnstile"))
    }

    async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        Ok(HandlerOutcome::not_implemented(self.name()))
    }
}
