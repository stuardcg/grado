use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use grado_store::StoreError;

use crate::trail::TrailError;

/// A JSON error response: `{"error": "..."}`.
#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub String);

impl ApiError {
    pub fn unauthorized() -> Self {
        Self(StatusCode::UNAUTHORIZED, "Sign in first.".into())
    }

    pub fn forbidden() -> Self {
        Self(StatusCode::FORBIDDEN, "Not allowed.".into())
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, message.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

impl From<TrailError> for ApiError {
    fn from(error: TrailError) -> Self {
        match error {
            TrailError::NotFound(m) => Self(StatusCode::NOT_FOUND, m),
            TrailError::NotYet(m) => Self(StatusCode::CONFLICT, m),
            TrailError::Invalid(m) => Self(StatusCode::BAD_REQUEST, m),
            TrailError::Store(e) => e.into(),
        }
    }
}

impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        tracing::error!(%error, "storage error");
        Self(StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong on our side.".into())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
