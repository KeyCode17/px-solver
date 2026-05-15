use crate::domain::challenge_handler::ChallengeHandler;
use crate::domain::handler_outcome::HandlerOutcome;
use crate::domain::page_html::PageHtml;
use px_errors::AppError;
use std::sync::Arc;

#[derive(Clone)]
pub struct Pipeline {
    handlers: Vec<Arc<dyn ChallengeHandler>>,
    stop_on_solve: bool,
}

impl Pipeline {
    pub fn new(handlers: Vec<Arc<dyn ChallengeHandler>>) -> Self {
        Self {
            handlers,
            stop_on_solve: true,
        }
    }

    pub fn with_stop_on_solve(mut self, stop: bool) -> Self {
        self.stop_on_solve = stop;
        self
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    pub async fn run(&self, page: &PageHtml) -> Result<Vec<HandlerOutcome>, AppError> {
        let mut outcomes = Vec::with_capacity(self.handlers.len());
        for handler in &self.handlers {
            if handler.detects(page).await? {
                let outcome = handler.solve(page).await?;
                let solved = matches!(
                    outcome.status,
                    crate::domain::handler_outcome::HandlerStatus::Solved
                );
                outcomes.push(outcome);
                if solved && self.stop_on_solve {
                    return Ok(outcomes);
                }
            } else {
                outcomes.push(HandlerOutcome::skipped(handler.name()));
            }
        }
        Ok(outcomes)
    }
}
