use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("not implemented: {0}")]
    NotImplemented(String),
    #[error("internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
    pub message: String,
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::ValidationError(_) => "validation_error",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::Conflict(_) => "conflict",
            Self::NotImplemented(_) => "not_implemented",
            Self::InternalError(_) => "internal_error",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) | Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::NotFound(m)
            | Self::BadRequest(m)
            | Self::ValidationError(m)
            | Self::Unauthorized(m)
            | Self::Forbidden(m)
            | Self::Conflict(m)
            | Self::NotImplemented(m)
            | Self::InternalError(m) => m.clone(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorResponse {
            error: self.kind(),
            message: self.message(),
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes_map_correctly() {
        assert_eq!(
            AppError::NotFound("x".into()).status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::BadRequest("x".into()).status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::ValidationError("x".into()).status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Unauthorized("x".into()).status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Forbidden("x".into()).status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::Conflict("x".into()).status(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            AppError::NotImplemented("x".into()).status(),
            StatusCode::NOT_IMPLEMENTED
        );
        assert_eq!(
            AppError::InternalError("x".into()).status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn kind_strings_are_stable() {
        assert_eq!(AppError::NotFound("x".into()).kind(), "not_found");
        assert_eq!(
            AppError::NotImplemented("x".into()).kind(),
            "not_implemented"
        );
    }

    #[test]
    fn message_round_trip() {
        let m = "boom";
        let err = AppError::InternalError(m.into());
        assert_eq!(err.message(), m);
        assert!(format!("{err}").contains(m));
    }
}
