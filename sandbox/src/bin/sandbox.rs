use sandbox::api_client::ApiClient;
use shared::User;

#[tokio::main]
async fn main() {
    let users: Vec<User> = client()
        .await
        .get::<Vec<User>>("users")
        .await
        .unwrap();

    let user = &users[0];

    println!("user: {:?}", user);
}

async fn client() -> ApiClient {
    return ApiClient::new("http://127.0.0.1:3000");
}
