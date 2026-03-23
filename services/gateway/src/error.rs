use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("unavailable: {0}")]
    Unavailable(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            AppError::Unavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };

        let body = ErrorBody {
            error: error_code.to_string(),
            message: self.to_string(),
        };

        (status, axum::Json(body)).into_response()
    }
}

impl From<tonic::Status> for AppError {
    fn from(status: tonic::Status) -> Self {
        use tonic::Code;
        match status.code() {
            Code::NotFound => AppError::NotFound(status.message().to_string()),
            Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
            Code::Unauthenticated => AppError::Unauthorized(status.message().to_string()),
            Code::PermissionDenied => AppError::Forbidden(status.message().to_string()),
            Code::AlreadyExists => AppError::Conflict(status.message().to_string()),
            Code::Unavailable => AppError::Unavailable(status.message().to_string()),
            Code::DeadlineExceeded => {
                AppError::Unavailable(format!("timeout: {}", status.message()))
            }
            _ => AppError::Internal(status.message().to_string()),
        }
    }
}
