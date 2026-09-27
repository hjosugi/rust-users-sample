//! Step 1: GET /users and print the body as it is.
//!
//! cargo run -p sandbox --bin step1_raw

// `#[tokio::main]` makes `main` async. reqwest needs an async runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    let url = format!("{base}/users");

    // Two `.await`s: one for the response head, one for the whole body.
    let res = reqwest::get(&url).await?;
    println!("GET {url} -> {}", res.status());

    let body = res.text().await?;
    println!("{body}");

    Ok(())
}
