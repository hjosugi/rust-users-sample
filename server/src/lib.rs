//! User CRUD API.
//!
//! | Method | Path          | Success               |
//! |--------|---------------|-----------------------|
//! | GET    | /health       | 200 "ok"              |
//! | GET    | /users        | 200 [User]            |
//! | POST   | /users        | 201 User              |
//! | GET    | /users/{id}   | 200 User              |
//! | PUT    | /users/{id}   | 200 User              |
//! | DELETE | /users/{id}   | 204 (no body)         |
//!
//! Errors: 400 / 404 / 409 / 415 / 422 with `{"error": "..."}`.

pub mod error;
pub mod store;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{
        Path, State,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    routing::get,
};
use shared::{CreateUser, UpdateUser, User};
use tower_http::trace::TraceLayer;

pub use error::AppError;
pub use store::UserStore;

/// Shared state. `Arc` lets every request point to the same store.
pub type AppState = Arc<UserStore>;

/// Build the router. `main.rs` and the tests both call this.
pub fn app(store: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/users", get(list_users).post(create_user))
        // axum 0.8 writes path params as `{id}` (older versions used `:id`).
        .route(
            "/users/{id}",
            get(get_user)
                .put(update_user)
                .delete(delete_user),
        )
        // Unknown paths also get a JSON error, not an empty 404.
        .fallback(|| async {
            AppError::Rejection {
                status: StatusCode::NOT_FOUND,
                message: "route not found".into(),
            }
        })
        .layer(TraceLayer::new_for_http())
        .with_state(store)
}

// ---------- Handlers ----------
// Each handler is thin. It reads the input, calls the store, and returns.
//
// We take `Result<Json<T>, JsonRejection>` instead of `Json<T>`.
// Then `?` turns a bad body into our own JSON error (see error.rs).

async fn list_users(
    State(store): State<AppState>,
) -> Json<Vec<User>> {
    Json(store.list())
}

async fn create_user(
    State(store): State<AppState>,
    body: Result<Json<CreateUser>, JsonRejection>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let Json(input) = body?;
    let user = store.create(input)?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn get_user(
    State(store): State<AppState>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<Json<User>, AppError> {
    let Path(id) = id?;
    Ok(Json(store.get(id)?))
}

async fn update_user(
    State(store): State<AppState>,
    id: Result<Path<u64>, PathRejection>,
    body: Result<Json<UpdateUser>, JsonRejection>,
) -> Result<Json<User>, AppError> {
    let Path(id) = id?;
    let Json(input) = body?;
    Ok(Json(store.update(id, input)?))
}

async fn delete_user(
    State(store): State<AppState>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<StatusCode, AppError> {
    let Path(id) = id?;
    store.delete(id)?;
    Ok(StatusCode::NO_CONTENT)
}
