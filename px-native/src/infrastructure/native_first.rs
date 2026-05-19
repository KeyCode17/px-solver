//! `NativeFirstHandler` — decorator that tries a native handler and
//! falls back to a browser-based one on error.

use std::sync::Arc;

use async_trait::async_trait;
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerName, HandlerOutcome, HandlerStatus, PageHtml};

pub struct NativeFirstHandler {
    native: Arc<dyn ChallengeHandler>,
    fallback: Arc<dyn ChallengeHandler>,
    name: HandlerName,
}

impl NativeFirstHandler {
    pub fn new(native: Arc<dyn ChallengeHandler>, fallback: Arc<dyn ChallengeHandler>) -> Self {
        Self {
            native,
            fallback,
            name: "perimeterx-native-first",
        }
    }
}

#[async_trait]
impl ChallengeHandler for NativeFirstHandler {
    fn name(&self) -> HandlerName {
        self.name
    }

    async fn detects(&self, page: &PageHtml) -> Result<bool, AppError> {
        self.fallback.detects(page).await
    }

    async fn solve(&self, page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        match self.native.solve(page).await {
            Ok(out) if matches!(out.status, HandlerStatus::Solved) => Ok(out),
            Ok(out) => {
                tracing::info!(
                    target: "px_native",
                    status = ?out.status,
                    "native handler not solved, falling back"
                );
                self.fallback.solve(page).await
            }
            Err(e) => {
                tracing::warn!(
                    target: "px_native",
                    error = %e,
                    "native handler error, falling back"
                );
                self.fallback.solve(page).await
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use px_core::CookieJarDelta;
    use px_pipeline::HandlerMetrics;

    struct SolvedHandler(&'static str);
    struct FailingHandler;

    #[async_trait]
    impl ChallengeHandler for SolvedHandler {
        fn name(&self) -> HandlerName {
            self.0
        }
        async fn detects(&self, _page: &PageHtml) -> Result<bool, AppError> {
            Ok(true)
        }
        async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
            Ok(HandlerOutcome::solved_with_ua(
                self.0,
                CookieJarDelta::default(),
                Vec::new(),
                HandlerMetrics::default(),
                "ua",
            ))
        }
    }

    #[async_trait]
    impl ChallengeHandler for FailingHandler {
        fn name(&self) -> HandlerName {
            "failing"
        }
        async fn detects(&self, _page: &PageHtml) -> Result<bool, AppError> {
            Ok(true)
        }
        async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
            Err(AppError::InternalError("synthetic".into()))
        }
    }

    #[tokio::test]
    async fn prefers_native_when_ok() {
        let h = NativeFirstHandler::new(
            Arc::new(SolvedHandler("native")),
            Arc::new(SolvedHandler("fallback")),
        );
        let out = h
            .solve(&PageHtml::new("https://x/", ""))
            .await
            .expect("solve");
        assert_eq!(out.handler, "native");
    }

    #[tokio::test]
    async fn falls_back_on_error() {
        let h = NativeFirstHandler::new(Arc::new(FailingHandler), Arc::new(SolvedHandler("fb")));
        let out = h
            .solve(&PageHtml::new("https://x/", ""))
            .await
            .expect("solve");
        assert_eq!(out.handler, "fb");
    }
}
