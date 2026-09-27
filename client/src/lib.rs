//! A small web app that calls the User API and shows the raw result.
//!
//! Browser --(HTML form POST)--> this app --(reqwest, JSON)--> API server
//! Browser <--(HTML page)------- this app <--(status + JSON)--- API server

pub mod api;
pub mod render;

use std::time::Duration;

use axum::{
    Form, Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use reqwest::Method;
use serde::Deserialize;
use shared::{CreateUser, UpdateUser};

/// Shared state for all requests.
/// `reqwest::Client` keeps a connection pool. Make it once and reuse it.
/// Its clone is cheap because it uses `Arc` inside.
#[derive(Clone)]
pub struct ClientState {
    http: reqwest::Client,
    api_base: String,
}

/// Which API call the user picked. The hidden `op` field in each form sets it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    List,
    Get,
    Create,
    Update,
    Delete,
}

impl Op {
    /// The value of the hidden `op` field. Must match `rename_all` above.
    pub fn as_str(self) -> &'static str {
        match self {
            Op::List => "list",
            Op::Get => "get",
            Op::Create => "create",
            Op::Update => "update",
            Op::Delete => "delete",
        }
    }
}

/// The HTML form data. Missing fields become empty strings.
#[derive(Debug, Deserialize)]
pub struct RunForm {
    pub op: Op,
    #[serde(flatten)]
    pub values: FormValues,
}

/// Input values. We put them back in the form you sent, so you can repeat a call.
#[derive(Debug, Default, Deserialize)]
pub struct FormValues {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
}

pub fn app(api_base: impl Into<String>) -> Router {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("failed to build HTTP client");
    let state = ClientState {
        http,
        api_base: api_base
            .into()
            .trim_end_matches('/')
            .to_string(),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/run", post(run))
        .with_state(state);

    // Opt in with --features dev. Never auto-reload a form POST: the browser
    // could repeat a create/update/delete request when reloading that page.
    #[cfg(feature = "dev")]
    let app = app.layer(
        tower_livereload::LiveReloadLayer::new()
            .request_predicate(
                |request: &axum::http::Request<
                    axum::body::Body,
                >| {
                    request.method() == Method::GET
                },
            ),
    );

    app
}

/// GET / -> run "list users" once, so the first page is not empty.
async fn index(
    State(state): State<ClientState>,
) -> Html<String> {
    let values = FormValues::default();
    let call = state.execute(Op::List, &values).await;
    Html(render::page(
        &state.api_base,
        Op::List,
        &values,
        &call,
    ))
}

/// POST /run -> call the API, then show the result.
async fn run(
    State(state): State<ClientState>,
    Form(form): Form<RunForm>,
) -> Html<String> {
    let call = state.execute(form.op, &form.values).await;
    Html(render::page(
        &state.api_base,
        form.op,
        &form.values,
        &call,
    ))
}

impl ClientState {
    /// Turn (op, form values) into one HTTP request, then send it.
    async fn execute(
        &self,
        op: Op,
        v: &FormValues,
    ) -> api::ApiCall {
        let base = &self.api_base;
        // Encode the id, so input like "1/2" or "a?b" cannot change the URL.
        let id = encode_segment(v.id.trim());

        let (method, url, body) = match op {
            Op::List => {
                (Method::GET, format!("{base}/users"), None)
            }
            Op::Get => (
                Method::GET,
                format!("{base}/users/{id}"),
                None,
            ),
            Op::Create => {
                // Send the input as is. The server does the validation.
                let body = CreateUser {
                    name: v.name.clone(),
                    email: v.email.clone(),
                };
                (
                    Method::POST,
                    format!("{base}/users"),
                    Some(to_json(&body)),
                )
            }
            Op::Update => {
                // Empty input means "do not change this field".
                let body = UpdateUser {
                    name: non_empty(&v.name),
                    email: non_empty(&v.email),
                };
                (
                    Method::PUT,
                    format!("{base}/users/{id}"),
                    Some(to_json(&body)),
                )
            }
            Op::Delete => (
                Method::DELETE,
                format!("{base}/users/{id}"),
                None,
            ),
        };

        api::call(&self.http, method, url, body).await
    }
}

/// Any `Serialize` type -> `serde_json::Value`. Generic, so it works for both bodies.
fn to_json<T: serde::Serialize>(
    body: &T,
) -> serde_json::Value {
    serde_json::to_value(body)
        .expect("our types always serialize")
}

fn non_empty(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

/// Percent-encode one path segment. Keep only safe ASCII chars as they are.
fn encode_segment(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b)
        {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_segment_keeps_safe_chars() {
        assert_eq!(encode_segment("42"), "42");
        assert_eq!(encode_segment("a/b?c"), "a%2Fb%3Fc");
        assert_eq!(encode_segment("あ"), "%E3%81%82");
    }

    #[test]
    fn non_empty_trims() {
        assert_eq!(non_empty("  "), None);
        assert_eq!(non_empty(" Bob "), Some("Bob".into()));
    }
}
