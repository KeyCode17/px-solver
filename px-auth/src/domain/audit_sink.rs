use crate::domain::audit_event::AuditEvent;
use async_trait::async_trait;
use px_errors::AppError;

#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn record(&self, event: &AuditEvent) -> Result<(), AppError>;
}
