# rust-user-crud

A small User CRUD system. Everything is Rust.
It is made for learning Rust.

- `server` : JSON API for users (axum)
- `client` : a web page that calls the API and shows the raw result (axum + reqwest)
- `shared` : types that both sides use (serde)

## How it works

```
 Browser                client (:8080)                     server (:3000)
 -------                --------------                     --------------
   |  GET /                  |                                   |
   |------------------------>|  GET /users                       |
   |                         |---------------------------------->|
   |                         |          200 [ {...}, {...} ]     |
   |                         |<----------------------------------|
   |   HTML page             |                                   |
   |   (request + response)  |                                   |
   |<------------------------|                                   |
   |                         |                                   |
   |  POST /run              |                                   |
   |  op=create&name=..      |  POST /users  {"name":..}         |
   |------------------------>|---------------------------------->|
   |                         |          201 {"id":3,..}          |
   |   HTML page             |<----------------------------------|
   |<------------------------|                                   |
```

The browser never talks to the API directly.
The client app calls the API with `reqwest`, then puts the status and the body into HTML.

## Run

You need Rust 1.85 or newer (edition 2024).

```sh
# Terminal 1
cargo run -p server        # http://127.0.0.1:3000

# Terminal 2
cargo run -p client        # http://127.0.0.1:8080  <- open this
```

The server starts with two sample users (Alice, Bob).
Data is in memory. It is lost when the server stops.

Settings:

| Env var        | Used by | Default                 |
|----------------|---------|-------------------------|
| `PORT`         | both    | 3000 / 8080             |
| `API_BASE_URL` | client  | `http://127.0.0.1:3000` |
| `RUST_LOG`     | both    | `info`                  |

Try the API with curl too:

```sh
curl -s localhost:3000/users
curl -s -X POST localhost:3000/users -H 'content-type: application/json' \
     -d '{"name":"Carol","email":"carol@example.com"}'
curl -s -X PUT localhost:3000/users/3 -H 'content-type: application/json' -d '{"name":"Caroline"}'
curl -s -X DELETE -i localhost:3000/users/3
```

## API

| Method | Path          | Body                          | Success          |
|--------|---------------|-------------------------------|------------------|
| GET    | `/health`     | -                             | 200 `ok`         |
| GET    | `/users`      | -                             | 200 `[User]`     |
| POST   | `/users`      | `{"name","email"}`            | 201 `User`       |
| GET    | `/users/{id}` | -                             | 200 `User`       |
| PUT    | `/users/{id}` | `{"name"?, "email"?}`         | 200 `User`       |
| DELETE | `/users/{id}` | -                             | 204 (no body)    |

Every error has the same shape: `{"error": "..."}`.

| Status | When                                           |
|--------|------------------------------------------------|
| 400    | empty name, bad email, id is not a number      |
| 404    | no user with this id, or unknown route         |
| 409    | email is already used by another user          |
| 415    | no `Content-Type: application/json`            |
| 422    | JSON is valid but a field is missing or wrong  |

## Files

```
rust-user-crud/
├── Cargo.toml            workspace (3 crates)
├── shared/src/lib.rs     User, CreateUser, UpdateUser, ErrorBody
├── server/
│   ├── src/main.rs       start the server, add sample users
│   ├── src/lib.rs        router + handlers
│   ├── src/store.rs      in-memory store + validation
│   ├── src/error.rs      AppError -> HTTP status + JSON
│   └── tests/api.rs      API tests (no network)
└── client/
    ├── src/main.rs       start the client app
    ├── src/lib.rs        routes, form -> API request
    ├── src/api.rs        reqwest call, keep request + response
    ├── src/render.rs     HTML by hand, HTML escape
    └── tests/e2e.rs      real server on a random port + client
```

## Test

```sh
cargo test                              # all 12 tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

## Learning map

Where to look for each Rust topic.

| Topic                                  | Where                                                                 |
|----------------------------------------|-----------------------------------------------------------------------|
| `String` vs `&str`, borrowing          | `store.rs` `validate_name(&str) -> String`                            |
| Lifetimes                              | `render.rs` `field_value<'a>(v: &'a FormValues, ..) -> &'a str`       |
| `Option` / `Result` / `?`              | `store.rs` `update()` (`map(..).transpose()?`)                        |
| `enum` + `match`                       | `error.rs` `AppError`, `lib.rs` (client) `Op`                         |
| Traits you implement                   | `error.rs` `From<JsonRejection>`, `IntoResponse`                      |
| Generics / `impl Trait`                | client `to_json<T: Serialize>`, `app(api_base: impl Into<String>)`    |
| Shared state across threads            | server `Arc<UserStore>` + `RwLock` in `store.rs`                      |
| Iterators + `collect()`                | `render.rs` `FORMS.iter().map(..).collect::<String>()`                |
| serde attributes                       | `shared` `skip_serializing_if`, client `flatten`, `rename_all`        |
| async / await                          | every handler, `api.rs` `call()`                                      |
| Error `source()` chain                 | `api.rs` `error_chain()`                                              |
| Workspace + path dependencies          | root `Cargo.toml`, `client/Cargo.toml` (`shared`, dev-dep `server`)   |
| Testing a router without network       | `server/tests/api.rs` (`oneshot`)                                     |
| Testing with a real port               | `client/tests/e2e.rs` (bind to port 0)                                |

## Next steps (exercises)

1. Add search: `GET /users?name=ali`. Use the `Query` extractor.
2. Add `created_at` to `User`. Try the `time` or `chrono` crate.
3. Make a `UserRepo` trait. Then add a SQLite version with `sqlx`.
4. Add paging: `GET /users?limit=10&offset=0`.
5. Rewrite the client as a WASM app (for example Leptos). Keep using the `shared` crate.
