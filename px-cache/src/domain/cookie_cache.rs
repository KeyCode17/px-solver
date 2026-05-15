use crate::domain::metrics::CacheMetrics;
use async_trait::async_trait;
use px_core::{CacheKey, PxCookieBundle};
use px_errors::AppError;

#[async_trait]
pub trait CookieCache: Send + Sync {
    async fn get(&self, key: &CacheKey) -> Result<Option<PxCookieBundle>, AppError>;
    async fn put(&self, key: CacheKey, bundle: PxCookieBundle) -> Result<(), AppError>;
    async fn invalidate(&self, domain: &str) -> Result<(), AppError>;
    fn metrics_snapshot(&self) -> CacheMetrics;
}
