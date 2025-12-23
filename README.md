# Neocrates

A comprehensive Rust library providing unified access to essential utilities for web development, AWS integration, database operations, caching, and more. Neocrates acts as a facade crate that re-exports functionality from multiple internal modules.

[![crates.io](https://img.shields.io/crates/v/neocrates.svg)](https://crates.io/crates/neocrates)
[![docs.rs](https://img.shields.io/docsrs/neocrates)](https://docs.rs/neocrates)
[![License](https://img.shields.io/crates/l/neocrates)](https://github.com/ooiai/neocrates/blob/main/LICENSE)

- **中文文档**: [README.zh-CN.md](README.zh-CN.md)

---

## 🚀 Features

- **Modular Design**: Enable only what you need with feature flags
- **AWS Integration**: S3 and STS clients for Aliyun/Tencent Cloud
- **Database Helpers**: Diesel integration with connection pooling
- **Caching**: Redis connection pool and cache utilities
- **Web Utilities**: Logging, middleware, response handling, and validation
- **Security**: Cryptographic utilities and SMS functionality
- **Zero Cost**: Unused features don't add to your binary size

---

## 📦 Installation

Add Neocrates to your `Cargo.toml`:

### Full Feature Set (Recommended for getting started)

```toml
[dependencies]
neocrates = "0.1"
```

### Selective Features (Recommended for production)

```toml
[dependencies]
neocrates = { version = "0.1", default-features = false, features = ["awss3", "rediscache", "logger"] }
```

### Minimum Supported Rust Version (MSRV)

- Rust 1.84+ (uses `edition = "2024"`)

---

## 🔧 Feature Flags

Neocrates uses feature flags to keep your dependencies lean. All features are enabled by default via the `full` feature.

| Feature        | Description                  | Dependencies                |
| -------------- | ---------------------------- | --------------------------- |
| `awss3`        | S3 client utilities          | aws-sdk-s3, aws-config      |
| `awssts`       | STS clients (Aliyun/Tencent) | aws-sdk-sts, hmac, sha2     |
| `crypto`       | Cryptography helpers         | openssl, ring, argon2       |
| `dieselhelper` | Diesel database helpers      | diesel, deadpool-diesel     |
| `helper`       | Common utilities             | serde, validator, uuid      |
| `logger`       | Tracing-based logger         | tracing, tracing-subscriber |
| `middleware`   | Web middlewares              | axum, tower-http            |
| `rediscache`   | Redis cache utilities        | redis, bb8-redis, moka      |
| `response`     | Response types               | axum, serde_json            |
| `sms`          | SMS utilities                | reqwest, hmac, sha2         |
| `full`         | All features above           | -                           |

**Disable default features:**

```toml
neocrates = { version = "0.1", default-features = false, features = ["awss3", "logger"] }
```

---

## 🎯 Usage Examples

### Basic Setup

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logger (requires "logger" feature)
    #[cfg(feature = "logger")]
    neocrates::logger::run().await;

    // Use S3 client (requires "awss3" feature)
    #[cfg(feature = "awss3")]
    {
        use neocrates::awss3::aws::AwsClient;

        let s3_client = AwsClient::new(
            "my-bucket",
            "us-east-1",
            "https://s3.amazonaws.com",
            "ACCESS_KEY",
            "SECRET_KEY"
        ).await?;

        // Upload object
        s3_client.put_object("uploads/file.txt", b"Hello, World!".to_vec()).await?;
    }

    // Use Redis cache (requires "rediscache" feature)
    #[cfg(feature = "rediscache")]
    {
        use neocrates::rediscache::RedisPool;

        let redis_pool = RedisPool::from_env().await?;
        let mut conn = redis_pool.get_connection().await?;

        // Set and get cache
        neocrates::redis::cmd("SET").arg("key").arg("value").query_async(&mut *conn).await?;
        let value: String = neocrates::redis::cmd("GET").arg("key").query_async(&mut *conn).await?;
    }

    Ok(())
}
```

### AWS STS Clients

```rust
// Aliyun STS Client
#[cfg(feature = "awssts")]
async fn aliyun_sts_example() -> anyhow::Result<()> {
    use neocrates::awssts::aliyun::StsClient;

    let aliyun_client = StsClient::new(
        "YOUR_ACCESS_KEY_ID",
        "YOUR_ACCESS_KEY_SECRET",
        "acs:ram::123456789012:role/my-role",
        "session-name"
    );

    let credentials = aliyun_client.assume_role(3600).await?;
    println!("Temporary AK: {}", credentials.credentials.access_key_id);

    Ok(())
}

// Tencent STS Client
#[cfg(feature = "awssts")]
async fn tencent_sts_example() -> anyhow::Result<()> {
    use neocrates::awssts::tencent::StsClient;

    let tencent_client = StsClient::new(
        "YOUR_SECRET_ID",
        "YOUR_SECRET_KEY",
        "ap-guangzhou"
    );

    // Note: Check specific method signatures in documentation
    // let credentials = tencent_client.get_temp_credentials(...).await?;

    Ok(())
}
```

### Database Operations

```rust
#[cfg(feature = "dieselhelper")]
use neocrates::dieselhelper;

#[cfg(feature = "dieselhelper")]
async fn database_example() -> anyhow::Result<()> {
    // Initialize database pool
    let pool = dieselhelper::create_pool("DATABASE_URL").await?;

    // Use connection from pool
    dieselhelper::with_connection(&pool, |conn| {
        // Your database operations here
        // Example: User::find_by_id(conn, 1)?
        Ok::<(), neocrates::diesel::result::Error>(())
    }).await?;

    Ok(())
}
```

### Web Application with Middleware

```rust
#[cfg(all(feature = "axum", feature = "middleware"))]
use neocrates::{axum, middleware};

#[cfg(all(feature = "axum", feature = "middleware"))]
async fn web_app() -> anyhow::Result<()> {
    use axum::{routing::get, Router};

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .layer(middleware::trace_layer()) // Add tracing middleware
        .layer(middleware::cors_layer()); // Add CORS middleware

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## ⚙️ Configuration

### Environment Variables

Many modules support environment-based configuration:

- **Redis**: `REDIS_URL`, `REDIS_POOL_SIZE`
- **Database**: `DATABASE_URL`, `DATABASE_POOL_SIZE`
- **Logging**: `RUST_LOG` (default: "info")
- **AWS**: Standard AWS environment variables

### Custom Configuration

For advanced use cases, most modules accept custom configuration structs:

```rust
#[cfg(feature = "rediscache")]
{
    use neocrates::rediscache::{RedisConfig, RedisPool};

    let config = RedisConfig {
        url: "redis://localhost:6379".to_string(),
        max_size: 10,
        min_idle: Some(1),
        connection_timeout: std::time::Duration::from_secs(5),
        idle_timeout: Some(std::time::Duration::from_secs(600)),
        max_lifetime: Some(std::time::Duration::from_secs(3600)),
    };

    let pool = RedisPool::new(config).await?;
}
```

---

## 🛠️ Development Commands

### Build

```bash
# Default (all features)
cargo build -p neocrates

# Selective features
cargo build -p neocrates --no-default-features --features "awss3,rediscache,logger"

# Release build
cargo build --release -p neocrates
```

### Test

```bash
# Run all tests
cargo test -p neocrates

# Test specific features
cargo test -p neocrates --features "awss3,rediscache"
```

### Lint

```bash
cargo clippy -p neocrates -- -D warnings
cargo fmt --check
```

### Documentation

```bash
# Generate local docs
cargo doc -p neocrates --open

# Check documentation links
cargo doc -p neocrates --no-deps
```

---

## 📤 Publishing (For Maintainers)

### Prerequisites

1. Complete package metadata in `Cargo.toml`
2. Valid license files (`LICENSE-MIT`, `LICENSE-APACHE`)
3. Clean git repository (no uncommitted changes)

### Publish Sequence

```bash
# Test publish first
cargo publish -p neocrates --dry-run

# Publish to crates.io
cargo publish -p neocrates --registry crates-io
```

### Version Management

- Follow Semantic Versioning (SemVer)
- Update version in workspace root `Cargo.toml`
- Consider breaking changes when modifying public APIs

---

## 📚 Documentation

- **API Reference**: [docs.rs/neocrates](https://docs.rs/neocrates)
- **Source Code**: [GitHub Repository](https://github.com/ooiai/neocrates)
- **Package**: [crates.io/neocrates](https://crates.io/crates/neocrates)

---

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

1. **Feature Flags**: New functionality should be behind feature flags when possible
2. **Testing**: Include tests for new features
3. **Documentation**: Update README and add doc comments
4. **Code Quality**: Run `cargo clippy` and `cargo fmt` before submitting

### Development Workflow

```bash
# Clone and setup
git clone https://github.com/ooiai/neocrates.git
cd neocrates

# Build and test
cargo build -p neocrates
cargo test -p neocrates

# Verify publish readiness
cargo publish -p neocrates --dry-run
```

---

## 🛡️ Security

- **Credentials**: Never hardcode secrets in code or examples
- **Dependencies**: Keep dependencies updated to address security vulnerabilities
- **Principle of Least Privilege**: Use minimal permissions for AWS roles and database users
- **Input Validation**: Always validate and sanitize user input

If you discover a security vulnerability, please contact the maintainers directly.

---

## 📄 License

Neocrates is dual-licensed under:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

SPDX-License-Identifier: MIT OR Apache-2.0

---

## 🙏 Acknowledgements

Thanks to the Rust community and the authors of the excellent crates we build upon:

- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
- [Axum](https://github.com/tokio-rs/axum)
- [Diesel](https://github.com/diesel-rs/diesel)
- [Redis-rs](https://github.com/redis-rs/redis-rs)
- [Tracing](https://github.com/tokio-rs/tracing)
- And many others!
