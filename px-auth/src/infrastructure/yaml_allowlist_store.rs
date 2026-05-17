use crate::domain::allowlist_entry::{AllowlistEntry, AllowlistEntryError};
use crate::domain::allowlist_store::AllowlistStore;
use async_trait::async_trait;
use px_errors::AppError;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct AllowlistFile {
    entries: Vec<AllowlistEntry>,
}

/// `BTreeMap` keeps `list()` output in domain-sorted order so operator output
/// (`px-cli allowlist list`) and audit diffs are reproducible across reloads.
pub struct YamlAllowlistStore {
    by_domain: BTreeMap<String, AllowlistEntry>,
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

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn entry(domain: &str) -> AllowlistEntry {
        AllowlistEntry {
            domain: domain.into(),
            tos_reviewed: true,
            justification: "test".into(),
            handler: None,
        }
    }

    #[tokio::test]
    async fn list_returns_entries_sorted_by_domain() {
        let store = YamlAllowlistStore::from_entries(vec![
            entry("zeta.example"),
            entry("alpha.example"),
            entry("mu.example"),
        ]);
        let listed: Vec<String> = store
            .list()
            .await
            .expect("list")
            .into_iter()
            .map(|e| e.domain)
            .collect();
        assert_eq!(listed, vec!["alpha.example", "mu.example", "zeta.example"]);
    }

    #[tokio::test]
    async fn lookup_finds_inserted_entry() {
        let store = YamlAllowlistStore::from_entries(vec![entry("example.com")]);
        let hit = store.lookup("example.com").await.expect("lookup");
        assert_eq!(hit.map(|e| e.domain).as_deref(), Some("example.com"));
        let miss = store.lookup("missing.example").await.expect("lookup");
        assert!(miss.is_none());
    }
}
