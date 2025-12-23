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
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logger
    neocrates::logger::run().await;

    // Your application logic here
    Ok(())
}
```

## AWS S3 Operations

### S3 Client Initialization

```rust
#[cfg(feature = "awss3")]
async fn create_s3_client() -> anyhow::Result<neocrates::awss3::aws::AwsClient> {
    use neocrates::awss3::aws::AwsClient;

    let client = AwsClient::new(
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
    use neocrates::awss3::aws::AwsClient;

    let s3 = AwsClient::new("bucket", "region", "endpoint", "ak", "sk").await?;

    // Upload file
    let data = b"Hello, World!".to_vec();
    s3.put_object("uploads/hello.txt", data).await?;

    // Download file
    let downloaded_data = s3.get_object("uploads/hello.txt").await?;
    println!("Downloaded: {:?}", String::from_utf8_lossy(&downloaded_data));

    // List objects
    let objects = s3.list_objects(Some("uploads/")).await?;
    for key in objects {
        println!("Object: {}", key);
    }

    Ok(())
}
```

## AWS STS Integration

### Aliyun STS

```rust
#[cfg(feature = "awssts")]
async fn aliyun_sts_workflow() -> anyhow::Result<()> {
    use neocrates::awssts::aliyun::StsClient;
    use neocrates::awss3::aws::AwsClient;

    let sts = StsClient::new(
        "LTAI5tEXAMPLE",
        "EXAMPLE_SECRET",
        "acs:ram::123456789012:role/OSSAccessRole",
        "session-name"
    );

    let response = sts.assume_role(3600).await?;
    let credentials = response.credentials;

    // Use temporary credentials with S3
    let s3 = AwsClient::new(
        "my-bucket",
        "cn-hangzhou",
        "https://oss-cn-hangzhou.aliyuncs.com",
        &credentials.access_key_id,
        &credentials.access_key_secret
    ).await?;

    s3.put_object("temp/upload.txt", b"Temporary upload".to_vec()).await?;
    Ok(())
}
```

### Tencent STS

```rust
#[cfg(feature = "awssts")]
async fn tencent_sts_workflow() -> anyhow::Result<()> {
    use neocrates::awssts::tencent::StsClient;

    let sts = StsClient::new(
        "AKIDEXAMPLE",
        "EXAMPLE_SECRET",
        "ap-guangzhou"
    );

    // Note: Check specific method signatures in documentation
    // let credentials = sts.get_temp_credentials("my-session", None, Some(7200)).await?;
    // println!("Temporary secret ID: {}", credentials.tmp_secret_id);
    Ok(())
}
```

## Redis Caching

### Basic Redis Setup

```rust
#[cfg(feature = "rediscache")]
async fn redis_basic() -> anyhow::Result<()> {
    use neocrates::rediscache::{RedisPool, RedisConfig};

    // From environment variables
    let pool = RedisPool::from_env().await?;

    // Or with custom config
    let config = RedisConfig {
        url: "redis://localhost:6379".to_string(),
        max_size: 10,
        min_idle: Some(1),
        connection_timeout: std::time::Duration::from_secs(5),
        idle_timeout: Some(std::time::Duration::from_secs(600)),
        max_lifetime: Some(std::time::Duration::from_secs(3600)),
    };
    let pool = RedisPool::new(config).await?;

    let mut conn = pool.get_connection().await?;

    // Set key
    neocrates::redis::cmd("SET")
        .arg("user:1:name")
        .arg("John Doe")
        .query_async(&mut *conn)
        .await?;

    // Get key
    let name: String = neocrates::redis::cmd("GET")
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
    use neocrates::rediscache::RedisPool;

    let pool = RedisPool::from_env().await?;

    // Set with TTL
    pool.setex("cache:key", "cached_value", 300).await?;

    // Get
    let value: Option<String> = pool.get("cache:key").await?;

    if let Some(v) = value {
        println!("Cached value: {}", v);
    }

    Ok(())
}
```

## Database Operations

### Diesel Connection Pool

```rust
#[cfg(feature = "database")]
async fn database_setup() -> anyhow::Result<()> {
    use neocrates::dieselhelper;

    // Create connection pool
    let pool = dieselhelper::create_pool("DATABASE_URL").await?;

    // Use connection
    dieselhelper::with_connection(&pool, |conn| {
        // Your database operations
        // Example: let users = User::all(conn)?;
        Ok::<(), neocrates::diesel::result::Error>(())
    }).await?;

    Ok(())
}
```

## Web Application

### Basic Axum App

```rust
#[cfg(all(feature = "web", feature = "axum"))]
async fn web_app() -> anyhow::Result<()> {
    use neocrates::{axum, middleware};
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
        .route("/users", post(create_user));
        // .layer(middleware::trace_layer())
        // .layer(middleware::cors_layer());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## Logging Configuration

### Basic Logging

```rust
async fn basic_logging() -> anyhow::Result<()> {
    // Initialize with default configuration
    neocrates::logger::run().await;

    neocrates::tracing::info!("Application started");
    neocrates::tracing::debug!("Debug information");
    neocrates::tracing::error!("An error occurred");

    Ok(())
}
```

## Error Handling

### Unified Error Types

```rust
#[cfg(feature = "thiserror")]
#[derive(neocrates::thiserror::Error, Debug)]
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
    neocrates::anyhow::bail!(my_error);
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
