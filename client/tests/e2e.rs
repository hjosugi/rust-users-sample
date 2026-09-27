//! End-to-end tests: a real API server on a random port + the client router.

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Start the real API server on a free port. Return its base URL.
async fn start_api_server() -> String {
    // Port 0 = "OS, please pick a free port".
    let listener =
        tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();
    let addr = listener.local_addr().unwrap();
    let api =
        server::app(Arc::new(server::UserStore::new()));
    // Run the server in the background for the rest of the test.
    tokio::spawn(async move {
        axum::serve(listener, api).await.unwrap()
    });
    format!("http://{addr}")
}

async fn get_page(app: &Router) -> String {
    let req =
        Request::get("/").body(Body::empty()).unwrap();
    read(app.clone().oneshot(req).await.unwrap()).await
}

/// Send a form like the browser does.
async fn post_form(app: &Router, form: &str) -> String {
    let req = Request::post("/run")
        .header(
            "content-type",
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(form.to_string()))
        .unwrap();
    read(app.clone().oneshot(req).await.unwrap()).await
}

async fn read(res: axum::response::Response) -> String {
    assert_eq!(res.status(), StatusCode::OK);
    let bytes =
        res.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn index_shows_list_result() {
    let app = client::app(start_api_server().await);
    let html = get_page(&app).await;
    assert!(html.contains("GET http://127.0.0.1:"));
    assert!(html.contains("200 OK"));
    assert!(html.contains("<pre>[]</pre>")); // empty list, raw JSON
}

#[tokio::test]
async fn live_reload_is_opt_in_and_never_repeats_form_posts()
 {
    let api_base = start_api_server().await;
    let app = client::app(&api_base);
    let html = get_page(&app).await;
    assert_eq!(
        html.contains("data-event-stream="),
        cfg!(feature = "dev")
    );

    let stream = app
        .clone()
        .oneshot(
            Request::get("/_tower-livereload/event-stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    if cfg!(feature = "dev") {
        assert_eq!(stream.status(), StatusCode::OK);
        assert_eq!(
            stream.headers()["content-type"],
            "text/event-stream"
        );
    } else {
        assert_eq!(stream.status(), StatusCode::NOT_FOUND);
    }
    drop(stream); // The event stream stays open until the browser disconnects.

    let html = post_form(
        &app,
        "op=create&name=Alice&email=alice%40example.com",
    )
    .await;
    assert!(html.contains("201 Created"));
    assert!(!html.contains("data-event-stream="));
}

#[tokio::test]
async fn full_crud_through_the_page() {
    let app = client::app(start_api_server().await);

    // Create. `%40` is "@" in a form body.
    let html = post_form(
        &app,
        "op=create&name=Alice&email=alice%40example.com",
    )
    .await;
    assert!(html.contains("POST http://"));
    assert!(html.contains("201 Created"));
    assert!(
        html.contains(
            "&quot;name&quot;: &quot;Alice&quot;"
        )
    );
    // Only the Create form keeps the input. The Update form stays empty.
    assert_eq!(html.matches(r#"value="Alice""#).count(), 1);

    // Get.
    let html = post_form(&app, "op=get&id=1").await;
    assert!(html.contains("/users/1"));
    assert!(html.contains("200 OK"));

    // Update: empty email is not sent.
    let html = post_form(
        &app,
        "op=update&id=1&name=Alicia&email=",
    )
    .await;
    assert!(html.contains("200 OK"));
    assert!(
        html.contains(
            "&quot;name&quot;: &quot;Alicia&quot;"
        )
    );
    assert!(
        !html.contains("&quot;email&quot;: &quot;&quot;")
    );

    // Delete: 204 has no body.
    let html = post_form(&app, "op=delete&id=1").await;
    assert!(html.contains("204 No Content"));
    assert!(html.contains("(empty body)"));

    // Now it is gone. The server's JSON error is shown as is.
    let html = post_form(&app, "op=get&id=1").await;
    assert!(html.contains("404 Not Found"));
    assert!(html.contains("user 1 not found"));
}

#[tokio::test]
async fn server_errors_are_shown_raw() {
    let app = client::app(start_api_server().await);

    let html =
        post_form(&app, "op=create&name=&email=x").await;
    assert!(html.contains("400 Bad Request"));
    assert!(html.contains("name must not be empty"));

    // Non-numeric id goes to the server as is (encoded). The server says 400.
    let html = post_form(&app, "op=get&id=abc").await;
    assert!(html.contains("400 Bad Request"));
}

#[tokio::test]
async fn html_is_escaped() {
    let app = client::app(start_api_server().await);
    let html = post_form(
        &app,
        "op=create&name=%3Cscript%3Ealert(1)%3C%2Fscript%3E&email=x%40example.com",
    )
    .await;
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(
        html.contains(
            "&lt;script&gt;alert(1)&lt;/script&gt;"
        )
    );
}

#[tokio::test]
async fn api_down_shows_error() {
    // Nothing listens on port 9. The call fails, but the page still renders.
    let app = client::app("http://127.0.0.1:9");
    let html = get_page(&app).await;
    assert!(html.contains("request failed"));
    assert!(html.contains("caused by"));
}
