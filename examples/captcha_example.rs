//! Captcha service usage examples
//!
//! This example demonstrates how to use the captcha service for generating
//! and validating different types of captchas.
//!
//! Run with:
//! ```bash
//! cargo run --example captcha_example --features web,redis
//! ```

use neocrates::axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use neocrates::captcha::{CaptchaData, CaptchaService};
use neocrates::rediscache::RedisPool;
use neocrates::serde::{Deserialize, Serialize};
use neocrates::tokio;
use std::sync::Arc;

// ==================== Request/Response DTOs ====================

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct GenerateCaptchaRequest {
    account: String,
    #[serde(default = "default_length")]
    length: usize,
}

fn default_length() -> usize {
    6
}

#[derive(Debug, Deserialize)]
struct ValidateCaptchaRequest {
    id: String,
    code: String,
}

#[derive(Debug, Deserialize)]
struct ValidateSliderRequest {
    account: String,
    code: String,
}

#[derive(Debug, Deserialize)]
struct GenerateSliderRequest {
    account: String,
    code: String,
}

// ==================== Application State ====================

#[derive(Clone)]
struct AppState {
    redis_pool: Arc<RedisPool>,
}

// ==================== Route Handlers ====================

/// Health check endpoint
async fn health_check() -> Response {
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some("OK".to_string()),
            message: "Service is healthy".to_string(),
        }),
    )
        .into_response()
}

/// Generate a numeric captcha
#[neocrates::axum::debug_handler]
async fn generate_numeric_captcha(
    State(state): State<AppState>,
    Json(payload): Json<GenerateCaptchaRequest>,
) -> Response {
    match CaptchaService::gen_numeric_captcha(
        &state.redis_pool,
        &payload.account,
        Some(payload.length),
    )
    .await
    {
        Ok(captcha) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(captcha),
                message: "Numeric captcha generated successfully".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<CaptchaData> {
                success: false,
                data: None,
                message: format!("Failed to generate captcha: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Generate an alphanumeric captcha
#[neocrates::axum::debug_handler]
async fn generate_alphanumeric_captcha(
    State(state): State<AppState>,
    Json(payload): Json<GenerateCaptchaRequest>,
) -> Response {
    match CaptchaService::gen_alphanumeric_captcha(
        &state.redis_pool,
        &payload.account,
        Some(payload.length),
    )
    .await
    {
        Ok(captcha) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(captcha),
                message: "Alphanumeric captcha generated successfully".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<CaptchaData> {
                success: false,
                data: None,
                message: format!("Failed to generate captcha: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Generate a slider captcha
#[neocrates::axum::debug_handler]
async fn generate_slider_captcha(
    State(state): State<AppState>,
    Json(payload): Json<GenerateSliderRequest>,
) -> Response {
    match CaptchaService::gen_captcha_slider(&state.redis_pool, &payload.code, &payload.account)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some("Slider captcha generated".to_string()),
                message: "Slider captcha generated successfully".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String> {
                success: false,
                data: None,
                message: format!("Failed to generate slider captcha: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Validate numeric captcha
#[neocrates::axum::debug_handler]
async fn validate_numeric_captcha(
    State(state): State<AppState>,
    Json(payload): Json<ValidateCaptchaRequest>,
) -> Response {
    match CaptchaService::validate_numeric_captcha(
        &state.redis_pool,
        &payload.id,
        &payload.code,
        true, // Delete after validation
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some("Valid".to_string()),
                message: "Captcha validation successful".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String> {
                success: false,
                data: None,
                message: format!("Captcha validation failed: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Validate alphanumeric captcha
#[neocrates::axum::debug_handler]
async fn validate_alphanumeric_captcha(
    State(state): State<AppState>,
    Json(payload): Json<ValidateCaptchaRequest>,
) -> Response {
    match CaptchaService::validate_alphanumeric_captcha(
        &state.redis_pool,
        &payload.id,
        &payload.code,
        true, // Delete after validation
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some("Valid".to_string()),
                message: "Captcha validation successful".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String> {
                success: false,
                data: None,
                message: format!("Captcha validation failed: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Validate slider captcha
#[neocrates::axum::debug_handler]
async fn validate_slider_captcha(
    State(state): State<AppState>,
    Json(payload): Json<ValidateSliderRequest>,
) -> Response {
    match CaptchaService::captcha_slider_valid(
        &state.redis_pool,
        &payload.code,
        &payload.account,
        true, // Delete after validation
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some("Valid".to_string()),
                message: "Slider captcha validation successful".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String> {
                success: false,
                data: None,
                message: format!("Slider captcha validation failed: {}", e),
            }),
        )
            .into_response(),
    }
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    // Initialize Redis pool
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    println!("Connecting to Redis at: {}", redis_url);

    let redis_config = neocrates::rediscache::RedisConfig {
        url: redis_url.clone(),
        max_size: 10,
        min_idle: Some(1),
        connection_timeout: std::time::Duration::from_secs(5),
        idle_timeout: Some(std::time::Duration::from_secs(600)),
        max_lifetime: Some(std::time::Duration::from_secs(3600)),
    };

    let redis_pool = match RedisPool::new(redis_config).await {
        Ok(pool) => Arc::new(pool),
        Err(e) => {
            eprintln!("Failed to connect to Redis: {}", e);
            eprintln!("Please make sure Redis is running at {}", redis_url);
            eprintln!("You can start Redis with: redis-server");
            std::process::exit(1);
        }
    };

    println!("Successfully connected to Redis");

    // Create application state
    let state = AppState { redis_pool };

    // Build the application router
    let app = Router::new()
        .route("/", get(health_check))
        // Numeric captcha routes
        .route(
            "/api/captcha/numeric/generate",
            post(generate_numeric_captcha),
        )
        .route(
            "/api/captcha/numeric/validate",
            post(validate_numeric_captcha),
        )
        // Alphanumeric captcha routes
        .route(
            "/api/captcha/alphanumeric/generate",
            post(generate_alphanumeric_captcha),
        )
        .route(
            "/api/captcha/alphanumeric/validate",
            post(validate_alphanumeric_captcha),
        )
        // Slider captcha routes
        .route(
            "/api/captcha/slider/generate",
            post(generate_slider_captcha),
        )
        .route(
            "/api/captcha/slider/validate",
            post(validate_slider_captcha),
        )
        .with_state(state);

    // Start the server
    let listener = neocrates::tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("\n🚀 Server running on http://127.0.0.1:3000");
    println!("\n📝 Available endpoints:");
    println!("  - GET  /                                       (health check)");
    println!("  - POST /api/captcha/numeric/generate          (generate numeric captcha)");
    println!("  - POST /api/captcha/numeric/validate          (validate numeric captcha)");
    println!("  - POST /api/captcha/alphanumeric/generate     (generate alphanumeric captcha)");
    println!("  - POST /api/captcha/alphanumeric/validate     (validate alphanumeric captcha)");
    println!("  - POST /api/captcha/slider/generate           (generate slider captcha)");
    println!("  - POST /api/captcha/slider/validate           (validate slider captcha)");
    println!("\n💡 Example curl commands:");
    println!("  # Generate numeric captcha:");
    println!(r#"  curl -X POST http://localhost:3000/api/captcha/numeric/generate \\"#);
    println!(r#"    -H "Content-Type: application/json" \\"#);
    println!(r#"    -d '{{"account":"user@example.com","length":6}}'"#);
    println!("\n  # Validate numeric captcha:");
    println!(r#"  curl -X POST http://localhost:3000/api/captcha/numeric/validate \\"#);
    println!(r#"    -H "Content-Type: application/json" \\"#);
    println!(r#"    -d '{{"id":"<captcha-id>","code":"123456}}'"#);
    println!("\n  # Generate slider captcha:");
    println!(r#"  curl -X POST http://localhost:3000/api/captcha/slider/generate \\"#);
    println!(r#"    -H "Content-Type: application/json" \\"#);
    println!(r#"    -d '{{"account":"user@example.com","code":"abc123"}}'"#);
    println!("\n  # Validate slider captcha:");
    println!(r#"  curl -X POST http://localhost:3000/api/captcha/slider/validate \\"#);
    println!(r#"    -H "Content-Type: application/json" \\"#);
    println!(r#"    -d '{{"account":"user@example.com","code":"abc123"}}'"#);

    neocrates::axum::serve(listener, app).await.unwrap();
}
