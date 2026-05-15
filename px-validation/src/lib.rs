use axum::body::Bytes;
use axum::extract::FromRequest;
use axum::http::Request;
use px_errors::AppError;
use serde::de::DeserializeOwned;

pub trait Validate {
    fn validate(&self) -> Result<(), String>;
}

#[derive(Debug)]
pub struct Validated<T>(pub T);

impl<T, S> FromRequest<S> for Validated<T>
where
    T: DeserializeOwned + Validate + 'static,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request<axum::body::Body>, state: &S) -> Result<Self, AppError> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("body read failed: {e}")))?;
        let value: T = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::BadRequest(format!("invalid JSON: {e}")))?;
        value.validate().map_err(AppError::ValidationError)?;
        Ok(Validated(value))
    }
}
