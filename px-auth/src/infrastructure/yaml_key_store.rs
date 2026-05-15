use crate::domain::key_store::{ApiKeyRecord, KeyStore};
use async_trait::async_trait;
use px_errors::AppError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct KeysFile {
    keys: Vec<ApiKeyRecord>,
}

pub struct YamlKeyStore {
    by_id: HashMap<String, ApiKeyRecord>,
}

impl YamlKeyStore {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let bytes = tokio::fs::read(path.as_ref())
            .await
            .map_err(|e| AppError::InternalError(format!("read keys yaml: {e}")))?;
        let parsed: KeysFile = serde_yaml::from_slice(&bytes)
            .map_err(|e| AppError::InternalError(format!("parse keys yaml: {e}")))?;
        let mut by_id = HashMap::new();
        for k in parsed.keys {
            by_id.insert(k.id.clone(), k);
        }
        Ok(Self { by_id })
    }

    pub fn from_records(records: Vec<ApiKeyRecord>) -> Self {
        let by_id = records.into_iter().map(|r| (r.id.clone(), r)).collect();
        Self { by_id }
    }
}

#[async_trait]
impl KeyStore for YamlKeyStore {
    async fn find_by_id(&self, id: &str) -> Result<Option<ApiKeyRecord>, AppError> {
        Ok(self.by_id.get(id).cloned())
    }

    async fn list_ids(&self) -> Result<Vec<String>, AppError> {
        Ok(self.by_id.keys().cloned().collect())
    }
}
