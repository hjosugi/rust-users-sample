use std::sync::Arc;

use server::{UserStore, app};
use shared::CreateUser;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Log level comes from RUST_LOG. Default shows each request.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")),
        )
        .init();

    // Add two sample users, so the list is not empty on first run.
    let store = Arc::new(UserStore::new());
    for (name, email) in [("Alice", "alice@example.com"), ("Bob", "bob@example.com")] {
        store
            .create(CreateUser {
                name: name.into(),
                email: email.into(),
            })
            .expect("seed data must be valid");
    }

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");
    tracing::info!("API server on http://{addr}");

    // Ctrl+C stops the server. Running requests finish first.
    axum::serve(listener, app(store))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("shutting down");
        })
        .await
        .expect("server error");
}
