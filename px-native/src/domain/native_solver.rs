use async_trait::async_trait;
use px_core::{Fingerprint, PxAppId, PxCookieBundle};
use px_errors::AppError;

#[derive(Debug, Clone)]
pub struct SolveContext {
    pub url: String,
    pub app_id: PxAppId,
    pub fingerprint: Fingerprint,
    pub proxy: Option<String>,
}

impl SolveContext {
    pub fn new(url: impl Into<String>, app_id: PxAppId, fingerprint: Fingerprint) -> Self {
        Self {
            url: url.into(),
            app_id,
            fingerprint,
            proxy: None,
        }
    }

    /// Route this solve's sensor POST through one egress proxy.
    #[must_use]
    pub fn with_proxy(mut self, proxy: Option<String>) -> Self {
        self.proxy = proxy;
        self
    }
}

#[async_trait]
pub trait NativeSolver: Send + Sync {
    async fn solve(&self, ctx: &SolveContext) -> Result<PxCookieBundle, AppError>;
}
