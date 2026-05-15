use async_trait::async_trait;
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerOutcome, PageHtml};

pub struct DataDomeHandler;

impl DataDomeHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DataDomeHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChallengeHandler for DataDomeHandler {
    fn name(&self) -> &'static str {
        "datadome"
    }

    async fn detects(&self, page: &PageHtml) -> Result<bool, AppError> {
        let h = &page.html;
        Ok(h.contains("DD_OPTIONS")
            || h.contains("datadome.co/captcha")
            || h.contains("ddg_datadome"))
    }

    async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        Ok(HandlerOutcome::not_implemented(self.name()))
    }
}
