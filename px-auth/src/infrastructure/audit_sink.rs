use crate::domain::audit_event::AuditEvent;
use crate::domain::audit_sink::AuditSink;
use async_trait::async_trait;
use px_errors::AppError;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

pub struct FileAuditSink {
    path: PathBuf,
    lock: Mutex<()>,
}

impl FileAuditSink {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Mutex::new(()),
        }
    }
}

#[async_trait]
impl AuditSink for FileAuditSink {
    async fn record(&self, event: &AuditEvent) -> Result<(), AppError> {
        let line = event
            .redacted_json()
            .map_err(|e| AppError::InternalError(format!("audit json: {e}")))?;
        let _guard = self.lock.lock().await;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .map_err(|e| AppError::InternalError(format!("audit open: {e}")))?;
        file.write_all(line.as_bytes())
            .await
            .map_err(|e| AppError::InternalError(format!("audit write: {e}")))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| AppError::InternalError(format!("audit newline: {e}")))?;
        Ok(())
    }
}

pub struct StdoutAuditSink;

impl StdoutAuditSink {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdoutAuditSink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditSink for StdoutAuditSink {
    async fn record(&self, event: &AuditEvent) -> Result<(), AppError> {
        let line = event
            .redacted_json()
            .map_err(|e| AppError::InternalError(format!("audit json: {e}")))?;
        tracing::info!(target: "audit", "{line}");
        Ok(())
    }
}
