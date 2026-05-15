use async_trait::async_trait;
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerOutcome, PageHtml};

pub struct CaptchaHandler;

impl CaptchaHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CaptchaHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChallengeHandler for CaptchaHandler {
    fn name(&self) -> &'static str {
        "captcha"
    }

    async fn detects(&self, page: &PageHtml) -> Result<bool, AppError> {
        let h = &page.html;
        Ok(h.contains("hcaptcha.com/1/api.js")
            || h.contains("recaptcha/api.js")
            || h.contains("class=\"h-captcha\"")
            || h.contains("class=\"g-recaptcha\""))
    }

    async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        Ok(HandlerOutcome::not_implemented(self.name()))
    }
}
