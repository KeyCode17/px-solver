use async_trait::async_trait;
use px_errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiKeyRecord {
    pub id: String,
    pub argon2_hash: String,
    pub note: Option<String>,
}

#[async_trait]
pub trait KeyStore: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<ApiKeyRecord>, AppError>;
    async fn list_ids(&self) -> Result<Vec<String>, AppError>;
}
