use crate::domain::key_store::{ApiKeyRecord, KeyStore};
use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use px_errors::AppError;
use std::sync::Arc;

pub struct VerifyKey {
    store: Arc<dyn KeyStore>,
}

impl VerifyKey {
    pub fn new(store: Arc<dyn KeyStore>) -> Self {
        Self { store }
    }

    pub async fn execute(&self, key_id: &str, secret: &str) -> Result<ApiKeyRecord, AppError> {
        let record = self
            .store
            .find_by_id(key_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("unknown api key id".into()))?;
        let parsed = PasswordHash::new(&record.argon2_hash)
            .map_err(|e| AppError::InternalError(format!("hash parse: {e}")))?;
        Argon2::default()
            .verify_password(secret.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized("invalid api key secret".into()))?;
        Ok(record)
    }
}
