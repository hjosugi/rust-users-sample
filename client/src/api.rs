//! Call the API with reqwest. Keep everything we need to show on the page.

use std::time::Instant;

use reqwest::Method;
use serde_json::Value;

/// One API call: what we sent and what we got back.
pub struct ApiCall {
    pub method: Method,
    pub url: String,
    /// The JSON body we sent (pretty). `None` for GET and DELETE.
    pub request_body: Option<String>,
    /// `Ok` = the server answered (any status, even 404).
    /// `Err` = no answer at all (server down, timeout, and so on).
    pub result: Result<ApiResponse, String>,
    pub elapsed_ms: u128,
}

pub struct ApiResponse {
    pub status: reqwest::StatusCode,
    /// Body as text. Pretty-printed if it is JSON.
    pub body: String,
}

/// Send one request and record everything.
/// This function never fails. Errors become part of `ApiCall`.
pub async fn call(
    http: &reqwest::Client,
    method: Method,
    url: String,
    body: Option<Value>,
) -> ApiCall {
    let request_body = body.as_ref().map(to_pretty);

    let mut req = http.request(method.clone(), &url);
    if let Some(b) = &body {
        // `.json()` sets `Content-Type: application/json` for us.
        req = req.json(b);
    }

    let start = Instant::now();
    let result = match req.send().await {
        Ok(res) => {
            let status = res.status();
            // Read the body as text. We show it even if it is not JSON.
            let text = res.text().await.unwrap_or_default();
            Ok(ApiResponse {
                status,
                body: pretty_if_json(&text),
            })
        }
        Err(e) => Err(error_chain(&e)),
    };

    ApiCall {
        method,
        url,
        request_body,
        result,
        elapsed_ms: start.elapsed().as_millis(),
    }
}

fn to_pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

/// Try to parse as JSON and pretty-print. If that fails, keep the text as is.
fn pretty_if_json(text: &str) -> String {
    match serde_json::from_str::<Value>(text) {
        Ok(v) => to_pretty(&v),
        Err(_) => text.to_string(),
    }
}

/// reqwest errors often hide the real cause in `source()`.
/// We walk the chain, so the page shows "connection refused" and not only "error sending request".
fn error_chain(e: &dyn std::error::Error) -> String {
    let mut msg = e.to_string();
    let mut cur = e.source();
    while let Some(src) = cur {
        msg.push_str("\n  caused by: ");
        msg.push_str(&src.to_string());
        cur = src.source();
    }
    msg
}
