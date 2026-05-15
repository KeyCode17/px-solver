#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use async_trait::async_trait;
use px_core::{CookieJarDelta, NamedToken};
use px_errors::AppError;
use px_pipeline::{
    ChallengeHandler, HandlerMetrics, HandlerOutcome, HandlerStatus, PageHtml, Pipeline,
};
use std::sync::Arc;

struct SkippingHandler;

#[async_trait]
impl ChallengeHandler for SkippingHandler {
    fn name(&self) -> &'static str {
        "skip"
    }
    async fn detects(&self, _page: &PageHtml) -> Result<bool, AppError> {
        Ok(false)
    }
    async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        Err(AppError::InternalError(
            "should not be called when detects=false".into(),
        ))
    }
}

struct SolvingHandler;

#[async_trait]
impl ChallengeHandler for SolvingHandler {
    fn name(&self) -> &'static str {
        "solve"
    }
    async fn detects(&self, _page: &PageHtml) -> Result<bool, AppError> {
        Ok(true)
    }
    async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        Ok(HandlerOutcome::solved(
            "solve",
            CookieJarDelta::default(),
            vec![NamedToken::new("t", "v")],
            HandlerMetrics::default(),
        ))
    }
}

struct StubHandler;

#[async_trait]
impl ChallengeHandler for StubHandler {
    fn name(&self) -> &'static str {
        "stub"
    }
    async fn detects(&self, _page: &PageHtml) -> Result<bool, AppError> {
        Ok(true)
    }
    async fn solve(&self, _page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        Ok(HandlerOutcome::not_implemented("stub"))
    }
}

#[tokio::test]
async fn pipeline_skips_then_solves_and_stops() {
    let pipeline = Pipeline::new(vec![Arc::new(SkippingHandler), Arc::new(SolvingHandler)]);
    let page = PageHtml::new("https://x", "<html/>");
    let outcomes = pipeline.run(&page).await.expect("run ok");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].status, HandlerStatus::Skipped);
    assert_eq!(outcomes[1].status, HandlerStatus::Solved);
}

#[tokio::test]
async fn stop_on_solve_false_continues_past_solver() {
    let pipeline = Pipeline::new(vec![Arc::new(SolvingHandler), Arc::new(StubHandler)])
        .with_stop_on_solve(false);
    let page = PageHtml::new("https://x", "<html/>");
    let outcomes = pipeline.run(&page).await.expect("run ok");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].status, HandlerStatus::Solved);
    assert_eq!(outcomes[1].status, HandlerStatus::NotImplemented);
}

#[tokio::test]
async fn first_solver_stops_pipeline() {
    let pipeline = Pipeline::new(vec![Arc::new(SolvingHandler), Arc::new(StubHandler)]);
    let page = PageHtml::new("https://x", "<html/>");
    let outcomes = pipeline.run(&page).await.expect("run ok");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].status, HandlerStatus::Solved);
}
