//! Step 5: take the filter from the command line.
//!
//! cargo run -p sandbox --bin step5_args                 # all users
//! cargo run -p sandbox --bin step5_args -- ali          # name or email has "ali"
//! cargo run -p sandbox --bin step5_args -- e test.com   # ... and domain is test.com

use shared::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // args[0] is the program path, so skip it.
    let args: Vec<String> =
        std::env::args().skip(1).collect();
    let keyword = args.first().map(|s| s.to_lowercase());
    let domain = args.get(1);

    let base = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    let users: Vec<User> =
        reqwest::get(format!("{base}/users"))
            .await?
            .error_for_status()?
            .json()
            .await?;

    // `None` filter = match everything. `is_none_or` is true for `None`.
    let hits: Vec<&User> = users
        .iter()
        .filter(|u| {
            keyword.as_deref().is_none_or(|k| {
                u.name.to_lowercase().contains(k)
                    || u.email.to_lowercase().contains(k)
            })
        })
        .filter(|u| {
            domain.is_none_or(|d| {
                u.email.ends_with(&format!("@{d}"))
            })
        })
        .collect();

    println!(
        "keyword={keyword:?} domain={domain:?} -> {} of {}",
        hits.len(),
        users.len()
    );
    for u in hits {
        println!("{:>3}  {:<8} {}", u.id, u.name, u.email);
    }

    Ok(())
}
