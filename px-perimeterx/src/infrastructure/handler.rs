use crate::application::solve_px::SolvePx;
use async_trait::async_trait;
use px_detector::{Detected, Detector, RegexDetector};
use px_errors::AppError;
use px_harvester::Harvester;
use px_pipeline::{ChallengeHandler, HandlerMetrics, HandlerOutcome, PageHtml};
use std::sync::Arc;

pub struct PerimeterxHandler {
    detector: RegexDetector,
    solver: SolvePx,
}

impl PerimeterxHandler {
    pub fn new(harvester: Arc<dyn Harvester>) -> Self {
        Self {
            detector: RegexDetector::new(),
            solver: SolvePx::new(harvester),
        }
    }
}

#[async_trait]
impl ChallengeHandler for PerimeterxHandler {
    fn name(&self) -> &'static str {
        "perimeterx"
    }

    async fn detects(&self, page: &PageHtml) -> Result<bool, AppError> {
        Ok(matches!(self.detector.detect(&page.html), Detected::Yes(_)))
    }

    async fn solve(&self, page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        let out = self.solver.execute(&page.url).await?;
        let metrics = HandlerMetrics {
            detect_us: 0,
            solve_ms: out.solve_ms,
            bytes_read: 0,
        };
        Ok(HandlerOutcome::solved_with_ua(
            self.name(),
            out.cookies,
            Vec::new(),
            metrics,
            out.user_agent,
        ))
    }
}
