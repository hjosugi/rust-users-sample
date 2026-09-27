//! API tests. They call the router in memory. No real network.

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server::{UserStore, app};
use tower::ServiceExt; // gives us `oneshot`

fn new_app() -> Router {
    app(Arc::new(UserStore::new()))
}

/// Send one request. Return the status and the body as JSON (Null if empty).
async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req =
        Request::builder().method(method).uri(uri);
    let body = match body {
        Some(v) => {
            req = req
                .header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    // `oneshot` takes ownership, so we clone the router. The clone is cheap.
    let res = app
        .clone()
        .oneshot(req.body(body).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes =
        res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes)
            .unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn crud_flow() {
    let app = new_app();

    // Create. Name is trimmed. Email is lowercased.
    let (s, b) = send(
        &app,
        "POST",
        "/users",
        Some(json!({"name": " Alice ", "email": "Alice@Example.com"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    assert_eq!(
        b,
        json!({"id": 1, "name": "Alice", "email": "alice@example.com"})
    );

    // Read one.
    let (s, b) = send(&app, "GET", "/users/1", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b["name"], "Alice");

    // Update only the name. Email stays.
    let (s, b) = send(
        &app,
        "PUT",
        "/users/1",
        Some(json!({"name": "Alice Liddell"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        b,
        json!({"id": 1, "name": "Alice Liddell", "email": "alice@example.com"})
    );

    // List is sorted by id.
    send(
        &app,
        "POST",
        "/users",
        Some(json!({"name": "Bob", "email": "bob@example.com"})),
    )
    .await;
    let (s, b) = send(&app, "GET", "/users", None).await;
    assert_eq!(s, StatusCode::OK);
    let ids: Vec<u64> = b
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["id"].as_u64().unwrap())
        .collect();
    assert_eq!(ids, vec![1, 2]);

    // Delete, then it is gone.
    let (s, b) =
        send(&app, "DELETE", "/users/1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    assert_eq!(b, Value::Null);
    let (s, _) = send(&app, "GET", "/users/1", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn not_found_is_json() {
    let app = new_app();
    for (method, body) in [
        ("GET", None),
        ("PUT", Some(json!({"name": "x"}))),
        ("DELETE", None),
    ] {
        let (s, b) =
            send(&app, method, "/users/99", body).await;
        assert_eq!(s, StatusCode::NOT_FOUND, "{method}");
        assert_eq!(
            b,
            json!({"error": "user 99 not found"})
        );
    }
}

#[tokio::test]
async fn validation_errors_are_400() {
    let app = new_app();
    let (s, b) = send(
        &app,
        "POST",
        "/users",
        Some(json!({"name": "  ", "email": "a@b.c"})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert_eq!(b["error"], "name must not be empty");

    let (s, b) = send(
        &app,
        "POST",
        "/users",
        Some(json!({"name": "A", "email": "no-at-mark"})),
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(
        b["error"]
            .as_str()
            .unwrap()
            .starts_with("invalid email")
    );
}

#[tokio::test]
async fn duplicate_email_is_409() {
    let app = new_app();
    send(
        &app,
        "POST",
        "/users",
        Some(
            json!({"name": "A", "email": "a@example.com"}),
        ),
    )
    .await;
    send(
        &app,
        "POST",
        "/users",
        Some(
            json!({"name": "B", "email": "b@example.com"}),
        ),
    )
    .await;

    // Same email on create (case does not matter).
    let (s, _) = send(
        &app,
        "POST",
        "/users",
        Some(
            json!({"name": "C", "email": "A@example.com"}),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);

    // Take another user's email on update.
    let (s, _) = send(
        &app,
        "PUT",
        "/users/2",
        Some(json!({"email": "a@example.com"})),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);

    // Keeping your own email is fine.
    let (s, _) = send(
        &app,
        "PUT",
        "/users/1",
        Some(json!({"email": "a@example.com"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
}

#[tokio::test]
async fn bad_input_gets_json_error_too() {
    let app = new_app();

    // Missing field -> 422 from the Json extractor, but in our JSON format.
    let (s, b) = send(
        &app,
        "POST",
        "/users",
        Some(json!({"name": "A"})),
    )
    .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(b["error"].as_str().unwrap().contains("email"));

    // Not a number -> 400 from the Path extractor.
    let (s, b) =
        send(&app, "GET", "/users/abc", None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(b["error"].is_string());

    // Unknown route -> 404 JSON.
    let (s, b) = send(&app, "GET", "/nope", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    assert_eq!(b, json!({"error": "route not found"}));
}
