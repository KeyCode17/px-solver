use crate::domain::native_solver::{NativeSolver, SolveContext};
use async_trait::async_trait;
use px_core::PxCookieBundle;
use px_errors::AppError;

pub struct NotImplementedNativeSolver;

impl NotImplementedNativeSolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NotImplementedNativeSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NativeSolver for NotImplementedNativeSolver {
    async fn solve(&self, _ctx: &SolveContext) -> Result<PxCookieBundle, AppError> {
        Err(AppError::NotImplemented(
            "native sensor generator is reserved for v2 (see ADR-0010)".into(),
        ))
    }
}
