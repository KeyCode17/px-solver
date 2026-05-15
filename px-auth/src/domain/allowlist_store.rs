use crate::domain::allowlist_entry::AllowlistEntry;
use async_trait::async_trait;
use px_errors::AppError;

#[async_trait]
pub trait AllowlistStore: Send + Sync {
    async fn lookup(&self, domain: &str) -> Result<Option<AllowlistEntry>, AppError>;
    async fn list(&self) -> Result<Vec<AllowlistEntry>, AppError>;
}
