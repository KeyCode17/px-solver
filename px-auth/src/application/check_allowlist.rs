use crate::domain::allowlist_entry::AllowlistEntry;
use crate::domain::allowlist_store::AllowlistStore;
use px_errors::AppError;
use std::sync::Arc;

pub struct CheckAllowlist {
    store: Arc<dyn AllowlistStore>,
}

impl CheckAllowlist {
    pub fn new(store: Arc<dyn AllowlistStore>) -> Self {
        Self { store }
    }

    pub async fn execute(&self, domain: &str) -> Result<AllowlistEntry, AppError> {
        let entry = self
            .store
            .lookup(domain)
            .await?
            .ok_or_else(|| AppError::Forbidden(format!("domain not in allowlist: {domain}")))?;
        entry
            .validate()
            .map_err(|e| AppError::Forbidden(e.to_string()))?;
        Ok(entry)
    }
}
