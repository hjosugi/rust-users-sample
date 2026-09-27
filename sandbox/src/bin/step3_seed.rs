//! Step 3: POST some users, so filters have data to work on.
//! Safe to run again: the server says 409 for an email it already has.
//!
//! cargo run -p sandbox --bin step3_seed

use reqwest::StatusCode;
use shared::{CreateUser, ErrorBody, User};

const USERS: [(&str, &str); 6] = [
    ("Carol", "carol@example.org"),
    ("Dave", "dave@test.com"),
    ("Eve", "eve@example.org"),
    ("Frank", "frank@example.com"),
    ("Grace", "grace@test.com"),
    ("Heidi", "heidi@example.com"),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    // For anything other than a plain GET, make a `Client` and reuse it.
    let http = reqwest::Client::new();

    for (name, email) in USERS {
        let body = CreateUser {
            name: name.into(),
            email: email.into(),
        };
        // `.json(&body)` = serialize with serde + set `Content-Type: application/json`.
        let res = http
            .post(format!("{base}/users"))
            .json(&body)
            .send()
            .await?;

        match res.status() {
            StatusCode::CREATED => {
                let u: User = res.json().await?;
                println!(
                    "created  {:>3}  {}",
                    u.id, u.name
                );
            }
            StatusCode::CONFLICT => println!(
                "skip          {name} (already there)"
            ),
            other => {
                let e: ErrorBody = res.json().await?;
                println!("error    {other}  {}", e.error);
            }
        }
    }

    Ok(())
}
