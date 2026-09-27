//! Step 2: turn the JSON into `Vec<User>` and print one line per user.
//!
//! cargo run -p sandbox --bin step2_typed

use shared::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".into());

    // `error_for_status()` turns 4xx/5xx into an `Err`, so `?` stops here.
    // `.json::<Vec<User>>()` parses the body with serde. Wrong shape = `Err`.
    let users = reqwest::get(format!("{base}/users"))
        .await?
        .error_for_status()?
        .json::<Vec<User>>()
        .await?;

    println!("{} users", users.len());
    for u in &users {
        // `{:>3}` = right-align in 3 columns, `{:<8}` = left-align in 8.
        println!("{:>3}  {:<8} {}", u.id, u.name, u.email);
    }

    // `{:#?}` = Debug, pretty. Works because `User` derives `Debug`.
    println!("\nfirst user (Debug): {:#?}", users.first());

    Ok(())
}
