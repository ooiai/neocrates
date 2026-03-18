# Neocrates

A comprehensive Rust utility library providing modular access to web development, AWS integrations, database operations, Redis caching, cryptography, authentication, and more. Neocrates acts as a facade crate that re-exports functionality from well-maintained ecosystem crates — gated behind feature flags so you only pay for what you use.

[![crates.io](https://img.shields.io/crates/v/neocrates.svg)](https://crates.io/crates/neocrates)
[![docs.rs](https://img.shields.io/docsrs/neocrates)](https://docs.rs/neocrates)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/ooiai/neocrates/blob/main/LICENSE)
[![Build](https://github.com/ooiai/neocrates/actions/workflows/rust.yml/badge.svg)](https://github.com/ooiai/neocrates/actions/workflows/rust.yml)

**中文文档**: [README.zh-CN.md](README.zh-CN.md)

---

## ✨ Features

| Feature    | What it gives you                                              |
|------------|----------------------------------------------------------------|
| `web`      | Axum + Tower + Hyper, reqwest HTTP client, URL helpers, middleware & response types |
| `aws`      | Full AWS suite (S3 + STS for AWS / Aliyun / Tencent Cloud)    |
| `awss3`    | S3 client only                                                 |
| `awssts`   | STS clients only (Aliyun & Tencent Cloud)                      |
| `diesel`   | Diesel ORM + deadpool-diesel connection pooling (PostgreSQL)   |
| `redis`    | bb8-redis connection pool + Moka in-process cache              |
| `crypto`   | Argon2 password hashing, HMAC, SHA-2, Ring low-level crypto    |
| `sms`      | SMS helpers (Aliyun & Tencent Cloud)                           |
| `captcha`  | CAPTCHA service                                                |
| `auth`     | JWT and session auth helpers                                   |
| `logger`   | Structured tracing-subscriber logger                           |
| `full`     | All of the above                                               |

**MSRV**: Rust 1.84+ (edition 2024)

---

## 📦 Installation

```toml
[dependencies]
# All features — great for getting started
neocrates = { version = "0.1", features = ["full"] }

# Selective — recommended for production (smaller binaries)
neocrates = { version = "0.1", default-features = false, features = ["web", "redis", "logger"] }
```

---

## 🚀 Quick Start

### Logger

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "logger")]
    neocrates::logger::run().await;

    neocrates::tracing::info!("neocrates ready");
    Ok(())
}
```

### S3 Client

```rust
#[cfg(feature = "awss3")]
{
    use neocrates::awss3::aws::AwsClient;

    let s3 = AwsClient::new(
        "my-bucket", "us-east-1",
        "https://s3.amazonaws.com",
        &std::env::var("AWS_ACCESS_KEY_ID")?,
        &std::env::var("AWS_SECRET_ACCESS_KEY")?,
    ).await?;

    s3.put_object("uploads/hello.txt", b"Hello, World!".to_vec()).await?;
}
```

### Aliyun / Tencent STS

```rust
#[cfg(feature = "awssts")]
{
    use neocrates::awssts::aliyun::StsClient;

    let sts = StsClient::new(
        &std::env::var("ALI_AK")?,
        &std::env::var("ALI_SK")?,
        "acs:ram::123456789012:role/OSSRole",
        "session-name",
    );
    let creds = sts.assume_role(3600).await?;
    println!("Temporary AK: {}", creds.credentials.access_key_id);
}
```

### Redis Cache

```rust
#[cfg(feature = "redis")]
{
    use neocrates::rediscache::RedisPool;

    let pool = RedisPool::from_env().await?;         // reads REDIS_URL
    let mut conn = pool.get_connection().await?;

    neocrates::redis::cmd("SET").arg("key").arg("val").query_async(&mut *conn).await?;
    let v: String = neocrates::redis::cmd("GET").arg("key").query_async(&mut *conn).await?;
    println!("{v}");
}
```

### Diesel (PostgreSQL)

```rust
#[cfg(feature = "diesel")]
{
    use neocrates::dieselhelper;

    let pool = dieselhelper::create_pool(&std::env::var("DATABASE_URL")?).await?;
    dieselhelper::with_connection(&pool, |conn| {
        // diesel operations …
        Ok::<(), neocrates::diesel::result::Error>(())
    }).await?;
}
```

### Axum Web App

```rust
#[cfg(feature = "web")]
{
    use neocrates::axum::{routing::get, Router};

    let app = Router::new()
        .route("/health", get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    neocrates::axum::serve(listener, app).await?;
}
```

---

## ⚙️ Environment Variables

| Variable             | Module       | Description                          |
|----------------------|--------------|--------------------------------------|
| `DATABASE_URL`       | `diesel`     | PostgreSQL connection string         |
| `DATABASE_POOL_SIZE` | `diesel`     | Connection pool size                 |
| `REDIS_URL`          | `redis`      | Redis connection URL                 |
| `REDIS_POOL_SIZE`    | `redis`      | Redis pool size                      |
| `RUST_LOG`           | `logger`     | Log level filter (default: `info`)   |
| `AWS_ACCESS_KEY_ID`  | `awss3`      | AWS / S3-compatible access key       |
| `AWS_SECRET_ACCESS_KEY` | `awss3`   | AWS / S3-compatible secret key       |

---

## 🛠️ Development

```bash
make build          # cargo build
make build-full     # cargo build --features full
make test           # cargo test
make test-full      # cargo test --features full
make lint           # cargo clippy -D warnings
make fmt            # cargo fmt
make doc            # cargo doc --open
make audit          # cargo audit (requires cargo-audit)
make dry-run        # test publish
make publish m="release: v0.1.x"
```

Run `make help` to see all available targets.

---

## 🤝 Contributing

1. New functionality should be behind a feature flag.
2. Add tests for new features (`#[tokio::test]` for async).
3. Run `make lint` and `make fmt` before opening a PR.
4. Update this README and `AGENTS.md` when adding modules.

```bash
git clone https://github.com/ooiai/neocrates.git
cd neocrates
make build-full && make test-full
```

---

## 🛡️ Security

- Never hardcode credentials — use environment variables or a secrets manager.
- Always validate and sanitize user input before processing.
- Use Argon2 (via the `crypto` feature) for password hashing; never MD5 or SHA-1.
- Report security vulnerabilities directly to the maintainers.

---

## 📚 Resources

- **API Docs**: [docs.rs/neocrates](https://docs.rs/neocrates)
- **crates.io**: [crates.io/crates/neocrates](https://crates.io/crates/neocrates)
- **Source**: [github.com/ooiai/neocrates](https://github.com/ooiai/neocrates)
- **Usage Examples**: [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)

---

## 📄 License

MIT License — Copyright © 2026 [ooiai](https://github.com/ooiai)

See [LICENSE](LICENSE) for the full text.
