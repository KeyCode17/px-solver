use crate::domain::allowlist_entry::{AllowlistEntry, AllowlistEntryError};
use crate::domain::allowlist_store::AllowlistStore;
use async_trait::async_trait;
use px_errors::AppError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct AllowlistFile {
    entries: Vec<AllowlistEntry>,
}

pub struct YamlAllowlistStore {
    by_domain: HashMap<String, AllowlistEntry>,
}

impl YamlAllowlistStore {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let bytes = tokio::fs::read(path.as_ref())
            .await
            .map_err(|e| AppError::InternalError(format!("read allowlist yaml: {e}")))?;
        let parsed: AllowlistFile = serde_yaml::from_slice(&bytes)
            .map_err(|e| AppError::InternalError(format!("parse allowlist yaml: {e}")))?;
        for entry in &parsed.entries {
            entry
                .validate()
                .map_err(|e: AllowlistEntryError| AppError::InternalError(e.to_string()))?;
        }
        let by_domain = parsed
            .entries
            .into_iter()
            .map(|e| (e.domain.clone(), e))
            .collect();
        Ok(Self { by_domain })
    }

    pub fn from_entries(entries: Vec<AllowlistEntry>) -> Self {
        let by_domain = entries.into_iter().map(|e| (e.domain.clone(), e)).collect();
        Self { by_domain }
    }
}

#[async_trait]
impl AllowlistStore for YamlAllowlistStore {
    async fn lookup(&self, domain: &str) -> Result<Option<AllowlistEntry>, AppError> {
        Ok(self.by_domain.get(domain).cloned())
    }

    async fn list(&self) -> Result<Vec<AllowlistEntry>, AppError> {
        Ok(self.by_domain.values().cloned().collect())
    }
}
