//! Step 4: filter, map, sort, count with iterators.
//! The API has no search, so we get everything and filter on our side.
//!
//! cargo run -p sandbox --bin step4_filter

use std::collections::BTreeMap;

use shared::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    let users: Vec<User> =
        reqwest::get(format!("{base}/users"))
            .await?
            .error_for_status()?
            .json()
            .await?;

    // 1. filter: keep users whose email ends with "@example.org".
    //    `.iter()` borrows, so we get `&User` and `users` stays usable.
    let org: Vec<&User> = users
        .iter()
        .filter(|u| u.email.ends_with("@example.org"))
        .collect();
    print_users("email ends with @example.org", &org);

    // 2. filter, ignore case: name has "a" or "A".
    let has_a: Vec<&User> = users
        .iter()
        .filter(|u| u.name.to_lowercase().contains('a'))
        .collect();
    print_users("name contains 'a' (any case)", &has_a);

    // 3. two conditions + count. `count()` needs no Vec.
    let n = users
        .iter()
        .filter(|u| u.id > 3 && u.email.ends_with(".com"))
        .count();
    println!("== id > 3 and .com: {n} users\n");

    // 4. map: only the names, joined into one line.
    let names: Vec<&str> =
        users.iter().map(|u| u.name.as_str()).collect();
    println!("== names\n{}\n", names.join(", "));

    // 5. sort: newest (largest id) first. `sort_by_key` sorts in place, so clone refs first.
    let mut newest: Vec<&User> = users.iter().collect();
    newest.sort_by_key(|u| std::cmp::Reverse(u.id));
    print_users("newest 3", &newest[..newest.len().min(3)]);

    // 6. find: the first match, or `None`.
    match users.iter().find(|u| u.name == "Eve") {
        Some(u) => println!(
            "== find Eve\nid={} email={}\n",
            u.id, u.email
        ),
        None => println!("== find Eve\nnot found\n"),
    }

    // 7. group by email domain. BTreeMap keeps keys sorted.
    let mut by_domain: BTreeMap<&str, Vec<&str>> =
        BTreeMap::new();
    for u in &users {
        let domain =
            u.email.split_once('@').map_or("?", |(_, d)| d);
        by_domain.entry(domain).or_default().push(&u.name);
    }
    println!("== by domain");
    for (domain, names) in &by_domain {
        println!("{domain:<12} {}", names.join(", "));
    }

    Ok(())
}

fn print_users(title: &str, users: &[&User]) {
    println!("== {title} ({})", users.len());
    for u in users {
        println!("{:>3}  {:<8} {}", u.id, u.name, u.email);
    }
    println!();
}
