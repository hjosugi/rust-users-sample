//! Error type for the API.
//! Every error becomes an HTTP status plus a JSON body.

use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use shared::ErrorBody;

#[derive(Debug)]
pub enum AppError {
    /// 404: no user with this id.
    NotFound(u64),
    /// 400: the input breaks a rule (empty name, bad email).
    Validation(String),
    /// 409: the email is already used by another user.
    Conflict(String),
    /// Bad JSON body or bad path. axum gives us the status and message.
    Rejection { status: StatusCode, message: String },
}

/// `?` on a `Result<Json<T>, JsonRejection>` uses this to convert the error.
impl From<JsonRejection> for AppError {
    fn from(r: JsonRejection) -> Self {
        AppError::Rejection {
            status: r.status(),
            message: r.body_text(),
        }
    }
}

/// Same idea for a bad path, like `/users/abc`.
impl From<PathRejection> for AppError {
    fn from(r: PathRejection) -> Self {
        AppError::Rejection {
            status: r.status(),
            message: r.body_text(),
        }
    }
}

/// axum calls this to turn our error into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            AppError::NotFound(id) => (StatusCode::NOT_FOUND, format!("user {id} not found")),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Rejection { status, message } => (status, message),
        };
        (status, Json(ErrorBody { error })).into_response()
    }
}
