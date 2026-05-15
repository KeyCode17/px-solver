use async_trait::async_trait;
use px_core::{Fingerprint, PxAppId, PxCookieBundle};
use px_errors::AppError;

#[derive(Debug, Clone)]
pub struct SolveContext {
    pub url: String,
    pub app_id: PxAppId,
    pub fingerprint: Fingerprint,
}

impl SolveContext {
    pub fn new(url: impl Into<String>, app_id: PxAppId, fingerprint: Fingerprint) -> Self {
        Self {
            url: url.into(),
            app_id,
            fingerprint,
        }
    }
}

#[async_trait]
pub trait NativeSolver: Send + Sync {
    async fn solve(&self, ctx: &SolveContext) -> Result<PxCookieBundle, AppError>;
}
