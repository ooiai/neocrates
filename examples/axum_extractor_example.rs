//! Example demonstrating the usage of custom JSON extractors
//!
//! This example shows how to use `LoggedJson` and `DetailedJson` extractors
//! in Axum route handlers for better error handling.
//!
//! Run with:
//! ```bash
//! cargo run --example axum_extractor_example --features web
//! ```
//!
//! Test with curl:
//! ```bash
//! # Valid request
//! curl -X POST http://localhost:3000/users/logged \
//!   -H "Content-Type: application/json" \
//!   -d '{"name":"Alice","email":"alice@example.com","age":30}'
//!
//! # Invalid JSON syntax
//! curl -X POST http://localhost:3000/users/logged \
//!   -H "Content-Type: application/json" \
//!   -d '{"name":"Alice","email":"alice@example.com"'
//!
//! # Missing required field
//! curl -X POST http://localhost:3000/users/detailed \
//!   -H "Content-Type: application/json" \
//!   -d '{"name":"Bob"}'
//!
//! # Wrong data type
//! curl -X POST http://localhost:3000/users/detailed \
//!   -H "Content-Type: application/json" \
//!   -d '{"name":"Bob","email":"bob@example.com","age":"thirty"}'
//!
//! # Missing Content-Type header
//! curl -X POST http://localhost:3000/users/detailed \
//!   -d '{"name":"Charlie","email":"charlie@example.com","age":25}'
//! ```

use neocrates::axum::{
    Router,
    routing::{get, post},
};
use neocrates::helper::core::axum_extractor::{DetailedJson, LoggedJson};
use neocrates::serde::{Deserialize, Serialize};
use neocrates::tokio;

#[derive(Debug, Deserialize, Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    age: u32,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: u64,
    name: String,
    email: String,
    age: u32,
}

// Route handler using LoggedJson
async fn create_user_logged(LoggedJson(payload): LoggedJson<CreateUserRequest>) -> String {
    println!("Creating user with LoggedJson: {:?}", payload);

    let response = UserResponse {
        id: 1,
        name: payload.name,
        email: payload.email,
        age: payload.age,
    };

    neocrates::serde_json::to_string_pretty(&response).unwrap()
}

// Route handler using DetailedJson
async fn create_user_detailed(DetailedJson(payload): DetailedJson<CreateUserRequest>) -> String {
    println!("Creating user with DetailedJson: {:?}", payload);

    let response = UserResponse {
        id: 2,
        name: payload.name,
        email: payload.email,
        age: payload.age,
    };

    neocrates::serde_json::to_string_pretty(&response).unwrap()
}

// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    // Build the application router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/users/logged", post(create_user_logged))
        .route("/users/detailed", post(create_user_detailed));

    // Start the server
    let listener = neocrates::tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");
    println!("Try the following endpoints:");
    println!("  - GET  /              (health check)");
    println!("  - POST /users/logged  (using LoggedJson)");
    println!("  - POST /users/detailed (using DetailedJson)");

    neocrates::axum::serve(listener, app).await.unwrap();
}
