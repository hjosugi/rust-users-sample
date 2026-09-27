//! Types shared by the server and the client.
//! Both sides use the same structs, so the JSON shape always matches.

use serde::{Deserialize, Serialize};

/// A user. The server returns this as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

/// Body for `POST /users`. Both fields are required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
}

/// Body for `PUT /users/{id}`.
/// Every field is optional. The server only changes the fields it gets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateUser {
    // Skip `None` when we send JSON, so the body only has the set fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Body the server sends on any error. Example: `{"error":"user 9 not found"}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}
