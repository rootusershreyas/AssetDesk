use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Validation(String),
    NotFound(&'static str),
    Conflict {
        code: &'static str,
        message: &'static str,
    },
    Database(sqlx::Error),
    Unavailable,
    UnsupportedMediaType,
    PayloadTooLarge,
}

impl AppError {
    pub fn conflict(code: &'static str, message: &'static str) -> Self {
        Self::Conflict { code, message }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::Validation(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                message,
            ),
            Self::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("{resource} does not exist."),
            ),
            Self::Conflict { code, message } => (StatusCode::CONFLICT, code, message.into()),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database_unavailable",
                "Database is unavailable.".into(),
            ),
            Self::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media_type",
                "Use Content-Type: application/json.".into(),
            ),
            Self::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "Request body exceeds the 2 MiB limit.".into(),
            ),
            Self::Database(error) => {
                let unavailable = matches!(
                    error,
                    sqlx::Error::PoolClosed | sqlx::Error::PoolTimedOut | sqlx::Error::Io(_)
                );
                // Keep credentials and raw SQL/database messages out of responses and logs.
                tracing::error!(unavailable, "database operation failed");
                if unavailable {
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "database_unavailable",
                        "Database is unavailable.".into(),
                    )
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal_error",
                        "An unexpected error occurred.".into(),
                    )
                }
            }
        };
        (
            status,
            Json(ErrorResponse {
                error: ErrorDetails {
                    code: code.into(),
                    message,
                },
            }),
        )
            .into_response()
    }
}
