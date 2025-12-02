# Neocrates Usage Examples

This document provides comprehensive examples for using Neocrates in various scenarios.

## Table of Contents

1. [Basic Setup](#basic-setup)
2. [AWS S3 Operations](#aws-s3-operations)
3. [AWS STS Integration](#aws-sts-integration)
4. [Redis Caching](#redis-caching)
5. [Database Operations](#database-operations)
6. [Web Application](#web-application)
7. [Logging Configuration](#logging-configuration)
8. [Error Handling](#error-handling)

## Basic Setup

### Minimal Setup

```rust
use neocrates::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Your application logic here
    Ok(())
}
```

### Selective Features

```toml
# Cargo.toml
[dependencies]
neocrates = { version = "0.1", default-features = false, features = ["logger", "helper"] }
```

```rust
// main.rs
use neocrates::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "logger")]
    init_logger().await;

    // Only logger and helper features are available
    Ok(())
}
```

## AWS S3 Operations

### S3 Client Initialization

```rust
use neocrates::prelude::*;

#[cfg(feature = "awss3")]
async fn create_s3_client() -> anyhow::Result<S3Client> {
    let client = S3Client::new(
        "my-bucket",
        "us-east-1",
        "https://s3.amazonaws.com",
        "ACCESS_KEY_ID",
        "SECRET_ACCESS_KEY"
    ).await?;

    Ok(client)
}
```

### File Upload/Download

```rust
#[cfg(feature = "awss3")]
async fn s3_operations() -> anyhow::Result<()> {
    let s3 = S3Client::new("bucket", "region", "endpoint", "ak", "sk").await?;

    // Upload file
    let data = b"Hello, World!";
    s3.put_object("uploads/hello.txt", data).await?;

    // Download file
    let downloaded_data = s3.get_object("uploads/hello.txt").await?;
    println!("Downloaded: {:?}", String::from_utf8_lossy(&downloaded_data));

    // List objects
    let objects = s3.list_objects("uploads/").await?;
    for object in objects {
        println!("Object: {}", object.key);
    }

    Ok(())
}
```

### Aliyun OSS Example

```rust
#[cfg(feature = "awss3")]
async fn aliyun_oss_example() -> anyhow::Result<()> {
    let oss = S3Client::new(
        "my-bucket",
        "cn-hangzhou",
        "https://oss-cn-hangzhou.aliyuncs.com",
        "YOUR_ACCESS_KEY",
        "YOUR_SECRET_KEY"
    ).await?;

    oss.put_object("test/file.txt", b"Aliyun OSS test").await?;
    Ok(())
}
```

## AWS STS Integration

### Aliyun STS

```rust
use neocrates::prelude::*;

#[cfg(feature = "awssts")]
async fn aliyun_sts_workflow() -> anyhow::Result<()> {
    let sts = AliyunStsClient::new(
        "LTAI5tEXAMPLE",
        "EXAMPLE_SECRET",
        "acs:ram::123456789012:role/OSSAccessRole",
        "session-name"
    );

    let credentials = sts.assume_role(3600).await?;

    // Use temporary credentials with S3
    let s3 = S3Client::new(
        "my-bucket",
        "cn-hangzhou",
        "https://oss-cn-hangzhou.aliyuncs.com",
        &credentials.access_key_id,
        &credentials.access_key_secret
    ).await?;

    s3.put_object("temp/upload.txt", b"Temporary upload").await?;
    Ok(())
}
```

### Tencent STS

```rust
#[cfg(feature = "awssts")]
async fn tencent_sts_workflow() -> anyhow::Result<()> {
    let sts = TencentStsClient::new(
        "AKIDEXAMPLE",
        "EXAMPLE_SECRET",
        "ap-guangzhou"
    );

    let credentials = sts
        .get_temp_credentials("my-session", None, Some(7200))
        .await?;

    println!("Temporary secret ID: {}", credentials.tmp_secret_id);
    Ok(())
}
```

## Redis Caching

### Basic Redis Setup

```rust
use neocrates::prelude::*;

#[cfg(feature = "rediscache")]
async fn redis_basic() -> anyhow::Result<()> {
    // From environment variables
    let pool = RedisPool::from_env().await?;

    // Or with custom config
    let config = neocrates::rediscache::RedisConfig {
        url: "redis://localhost:6379".to_string(),
        pool_size: 10,
        connection_timeout: std::time::Duration::from_secs(5),
    };
    let pool = RedisPool::new(config).await?;

    let mut conn = pool.get_connection().await?;

    // Set key
    redis::cmd("SET")
        .arg("user:1:name")
        .arg("John Doe")
        .query_async(&mut *conn)
        .await?;

    // Get key
    let name: String = redis::cmd("GET")
        .arg("user:1:name")
        .query_async(&mut *conn)
        .await?;

    println!("User name: {}", name);
    Ok(())
}
```

### Cache Helper Functions

```rust
#[cfg(feature = "rediscache")]
async fn cache_helper_example() -> anyhow::Result<()> {
    let pool = RedisPool::from_env().await?;

    // Set with TTL
    neocrates::rediscache::set_with_ttl(
        &pool,
        "cache:key",
        "cached_value",
        std::time::Duration::from_secs(300)
    ).await?;

    // Get or compute
    let value = neocrates::rediscache::get_or_compute(
        &pool,
        "expensive:computation",
        || async {
            // Expensive operation
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Ok::<String, anyhow::Error>("computed_result".to_string())
        },
        std::time::Duration::from_secs(3600)
    ).await?;

    println!("Cached value: {}", value);
    Ok(())
}
```

## Database Operations

### Diesel Connection Pool

```rust
#[cfg(feature = "dieselhelper")]
async fn database_setup() -> anyhow::Result<()> {
    // Create connection pool
    let pool = neocrates::dieselhelper::create_pool("DATABASE_URL").await?;

    // Use connection
    neocrates::dieselhelper::with_connection(&pool, |conn| {
        // Your database operations
        // Example: let users = User::all(conn)?;
        Ok::<(), diesel::result::Error>(())
    }).await?;

    Ok(())
}
```

### Transaction Management

```rust
#[cfg(feature = "dieselhelper")]
async fn transaction_example() -> anyhow::Result<()> {
    let pool = neocrates::dieselhelper::create_pool("DATABASE_URL").await?;

    neocrates::dieselhelper::with_transaction(&pool, |conn| {
        // First operation
        // User::create(conn, &user_data)?;

        // Second operation
        // Order::create(conn, &order_data)?;

        // If any operation fails, the entire transaction rolls back
        Ok::<(), diesel::result::Error>(())
    }).await?;

    Ok(())
}
```

## Web Application

### Basic Axum App

```rust
#[cfg(all(feature = "axum", feature = "middleware"))]
use neocrates::{axum, middleware, prelude::*};

#[cfg(all(feature = "axum", feature = "middleware"))]
async fn web_app() -> anyhow::Result<()> {
    use axum::{
        routing::{get, post},
        Json, Router
    };
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct CreateUser {
        name: String,
        email: String,
    }

    #[derive(Serialize)]
    struct ApiResponse {
        success: bool,
        message: String,
    }

    async fn health_check() -> &'static str {
        "OK"
    }

    async fn create_user(Json(payload): Json<CreateUser>) -> Json<ApiResponse> {
        // Process user creation
        Json(ApiResponse {
            success: true,
            message: format!("User {} created", payload.name),
        })
    }

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .layer(middleware::trace_layer())
        .layer(middleware::cors_layer());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### Error Handling Middleware

```rust
#[cfg(feature = "response")]
use neocrates::response::ApiError;

#[cfg(feature = "response")]
async fn error_handling_example() -> Result<(), ApiError> {
    // Custom error types
    let err = ApiError::BadRequest("Invalid input".to_string());

    // Convert from other errors
    let io_error = std::io::Error::new(std::io::ErrorKind::Other, "IO error");
    let api_error: ApiError = io_error.into();

    Err(err)
}
```

## Logging Configuration

### Basic Logging

```rust
use neocrates::prelude::*;

#[cfg(feature = "logger")]
async fn basic_logging() -> anyhow::Result<()> {
    // Initialize with default configuration
    init_logger().await;

    tracing::info!("Application started");
    tracing::debug!("Debug information");
    tracing::error!("An error occurred");

    Ok(())
}
```

### Custom Logging Configuration

```rust
#[cfg(feature = "logger")]
async fn custom_logging() -> anyhow::Result<()> {
    use neocrates::logger::{LoggerConfig, LogFormat};

    let config = LoggerConfig {
        level: "debug".to_string(),
        format: LogFormat::Json,
        service_name: "my-service".to_string(),
    };

    neocrates::logger::init_with_config(config).await;

    tracing::info!(service = "my-service", "Custom logging initialized");
    Ok(())
}
```

## Error Handling

### Unified Error Types

```rust
use neocrates::prelude::*;

#[cfg(feature = "thiserror")]
#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[cfg(feature = "anyhow")]
async fn unified_error_handling() -> anyhow::Result<()> {
    // Convert custom errors to anyhow
    let my_error = MyError::Validation("Invalid email".to_string());
    anyhow::bail!(my_error);

    // Or use directly
    Err(MyError::Database("Connection failed".to_string()))?;
}
```

### Validation Errors

```rust
#[cfg(feature = "validator")]
use neocrates::validator::{Validate, ValidationError};

#[cfg(feature = "validator")]
#[derive(Validate)]
struct UserInput {
    #[validate(email)]
    email: String,
    #[validate(length(min = 6))]
    password: String,
}

#[cfg(feature = "validator")]
fn validate_user_input() -> Result<(), neocrates::validator::ValidationErrors> {
    let input = UserInput {
        email: "invalid-email".to_string(),
        password: "123".to_string(),
    };

    input.validate() // Returns validation errors
}
```

## Feature Combination Examples

### Full-Featured Microservice

```rust
use neocrates::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    #[cfg(feature = "logger")]
    init_logger().await;

    // Initialize Redis cache
    #[cfg(feature = "rediscache")]
    let redis_pool = RedisPool::from_env().await?;

    // Initialize database
    #[cfg(feature = "dieselhelper")]
    let db_pool = neocrates::dieselhelper::create_pool("DATABASE_URL").await?;

    // Start web server
    #[cfg(all(feature = "axum", feature = "middleware"))]
    {
        let app = create_app(redis_pool, db_pool).await?;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}

#[cfg(all(feature = "axum", feature = "middleware", feature = "rediscache", feature = "dieselhelper"))]
async fn create_app(
    redis_pool: neocrates::rediscache::RedisPool,
    db_pool: neocrates::dieselhelper::DbPool,
) -> anyhow::Result<axum::Router> {
    use axum::{routing::get, Router};

    let state = AppState { redis_pool, db_pool };

    let app = Router::new()
        .route("/api/users", get(get_users))
        .with_state(state)
        .layer(middleware::trace_layer())
        .layer(middleware::cors_layer());

    Ok(app)
}

#[cfg(all(feature = "rediscache", feature = "dieselhelper"))]
struct AppState {
    redis_pool: neocrates::rediscache::RedisPool,
    db_pool: neocrates::dieselhelper::DbPool,
}
```
