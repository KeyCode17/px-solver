use crate::domain::handler_outcome::HandlerOutcome;
use crate::domain::page_html::PageHtml;
use crate::domain::solve_action::SolveAction;
use async_trait::async_trait;
use px_errors::AppError;

pub use crate::domain::handler_outcome::HandlerName;

#[async_trait]
pub trait ChallengeHandler: Send + Sync {
    fn name(&self) -> HandlerName;
    async fn detects(&self, page: &PageHtml) -> Result<bool, AppError>;
    async fn solve(&self, action: &SolveAction) -> Result<HandlerOutcome, AppError>;
}
