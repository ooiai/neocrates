# Neocrates

Neocrates is a **feature-gated Rust utility facade crate** that bundles practical building blocks for web services, storage, PostgreSQL, Redis, SMS, captcha, auth, and crypto workflows without forcing every dependency into every binary.

[![crates.io](https://img.shields.io/crates/v/neocrates.svg)](https://crates.io/crates/neocrates)
[![docs.rs](https://img.shields.io/docsrs/neocrates)](https://docs.rs/neocrates)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/ooiai/neocrates/blob/main/LICENSE)
[![Build](https://github.com/ooiai/neocrates/actions/workflows/rust.yml/badge.svg)](https://github.com/ooiai/neocrates/actions/workflows/rust.yml)

**Chinese overview**: [README.zh-CN.md](README.zh-CN.md)

---

## Why Neocrates

Neocrates is useful when you want one crate to provide a consistent set of infrastructure helpers:

- **Feature-gated modules** so you can keep binaries smaller than an “everything on” toolkit.
- **Facade-style re-exports** for common crates such as Tokio, Axum, Diesel, SQLx, Redis, Serde, tracing, and more.
- **Opinionated service helpers** for recurring backend work: Redis OTPs, request middleware, STS credentials, captcha storage, SQL logging, and password hashing.
- **Runnable examples** in `examples/` for Axum extractors, SMS flows, and captcha APIs.

This repository is a **single crate**, not a workspace. The root API is defined in [`src/lib.rs`](src/lib.rs).

---

## Module map

| Feature | Public module(s) | What you get | Module guide |
| --- | --- | --- | --- |
| default | `helper` | IDs, retry, pagination, serde helpers, config loading, utility validators | [`src/helper/README.md`](src/helper/README.md) |
| `logger` | `logger` | `tracing-subscriber` bootstrap with local timestamps | [`src/logger/README.md`](src/logger/README.md) |
| `web` | `middlewares`, `response` | Axum middleware, request auth, unified API errors and responses | [`src/middlewares/README.md`](src/middlewares/README.md), [`src/response/README.md`](src/response/README.md) |
| `diesel` | `dieselhelper` | PostgreSQL pool, auto-create DB, Diesel SQL logging macros | [`src/dieselhelper/README.md`](src/dieselhelper/README.md) |
| `sqlx` | `sqlxhelper` | PostgreSQL pool, migrations, SQLx SQL logging macros | [`src/sqlxhelper/README.md`](src/sqlxhelper/README.md) |
| `redis` | `rediscache` | Redis pool, pipelines, key helpers, distributed locks | [`src/rediscache/README.md`](src/rediscache/README.md) |
| `auth` | `auth` | Redis-backed token lifecycle and fingerprint helpers | [`src/auth/README.md`](src/auth/README.md) |
| `captcha` | `captcha` | Slider, numeric, and alphanumeric captcha flows | [`src/captcha/README.md`](src/captcha/README.md) |
| `awss3` | `awss3` | Low-level S3-compatible object client | [`src/awss3/README.md`](src/awss3/README.md) |
| `awssts` | `awssts` | Low-level Aliyun/Tencent STS clients | [`src/awssts/README.md`](src/awssts/README.md) |
| `aws` | `aws`, `awss3`, `awssts` | Higher-level storage/STS service layer | [`src/aws/README.md`](src/aws/README.md) |
| `sms` | `sms` | Aliyun/Tencent SMS providers and OTP workflow | [`src/sms/README.md`](src/sms/README.md) |
| `crypto` | `crypto` | Argon2 password hashing and misc crypto helpers | [`src/crypto/README.md`](src/crypto/README.md) |

### Practical feature combinations

The crate has a few **practical** combinations that are safer than turning on isolated flags blindly:

```toml
[dependencies]
# Explore everything
neocrates = { version = "0.1", features = ["full"] }

# Web API foundation
neocrates = { version = "0.1", default-features = false, features = ["web", "logger"] }

# Redis-backed auth + captcha
neocrates = { version = "0.1", default-features = false, features = ["web", "redis", "auth", "captcha"] }

# PostgreSQL with Diesel
neocrates = { version = "0.1", default-features = false, features = ["diesel"] }

# PostgreSQL with SQLx
neocrates = { version = "0.1", default-features = false, features = ["sqlx"] }

# Low-level S3-compatible client
neocrates = { version = "0.1", default-features = false, features = ["awss3"] }

# SMS OTP flow
neocrates = { version = "0.1", default-features = false, features = ["sms", "redis", "web"] }
```

Notes:

- `auth` is currently useful **with `web` and `redis`**.
- `captcha` is currently useful **with `web` and `redis`**.
- `middlewares` is currently safest to use with **`web` and `crypto`**.
- `sms` is typically used with **`web`** because provider implementations rely on the HTTP stack.
- `awss3` is the cleanest low-level storage entry point. The broader `aws` module adds higher-level configuration-driven helpers.

---

## Quick start

### 1. Bootstrap logging

```rust
#[cfg(feature = "logger")]
async fn init_logging() {
    neocrates::logger::run().await;
}
```

### 2. Return typed API errors in an Axum handler

```rust
#[cfg(feature = "web")]
use neocrates::response::error::{AppError, AppResult};

#[cfg(feature = "web")]
async fn health() -> AppResult<&'static str> {
    Ok("ok")
}

#[cfg(feature = "web")]
async fn guarded(flag: bool) -> AppResult<&'static str> {
    if !flag {
        return Err(AppError::Unauthorized);
    }
    Ok("allowed")
}
```

### 3. Initialize Redis and read/write a key

```rust
#[cfg(feature = "redis")]
async fn redis_demo() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = neocrates::rediscache::RedisPool::from_env().await?;
    pool.set("demo:key", "value").await?;
    let value: Option<String> = pool.get("demo:key").await?;
    println!("{value:?}");
    Ok(())
}
```

### 4. Hash and verify a password

```rust
#[cfg(feature = "crypto")]
fn password_demo() -> Result<(), neocrates::argon2::password_hash::Error> {
    let hash = neocrates::crypto::core::Crypto::hash_password("correct horse battery staple")?;
    assert!(neocrates::crypto::core::Crypto::verify_password(
        "correct horse battery staple",
        &hash
    ));
    Ok(())
}
```

---

## Detailed usage tutorials

## 1. Build an Axum service foundation

Recommended features:

```toml
neocrates = { version = "0.1", default-features = false, features = ["web", "logger", "crypto"] }
```

Suggested flow:

1. Initialize logging with `logger::run()` or `logger::init(LogConfig)`.
2. Return `AppResult<T>` from handlers so `AppError` can become a consistent JSON response.
3. Use `LoggedJson<T>` or `DetailedJson<T>` for request parsing when you want structured JSON error output.
4. Add `middlewares::interceptor` when you want Redis-backed or in-memory token validation and creator/updater injection.

Example:

```rust
use std::sync::Arc;

use neocrates::axum::{
    Router,
    middleware,
    routing::{get, post},
};
use neocrates::helper::core::axum_extractor::DetailedJson;
use neocrates::middlewares::{
    interceptor::interceptor,
    models::MiddlewareConfig,
    token_store::default_in_memory_store,
};
use neocrates::response::error::{AppResult, AppError};
use neocrates::serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CreateItem {
    name: String,
}

async fn health() -> AppResult<&'static str> {
    Ok("ok")
}

async fn create_item(DetailedJson(payload): DetailedJson<CreateItem>) -> AppResult<String> {
    if payload.name.trim().is_empty() {
        return Err(AppError::ValidationError("name is required".into()));
    }
    Ok(format!("created {}", payload.name))
}

#[tokio::main]
async fn main() {
    neocrates::logger::run().await;

    let config = Arc::new(MiddlewareConfig {
        token_store: default_in_memory_store(),
        ignore_urls: vec!["/health".into()],
        pms_ignore_urls: vec![],
        prefix: "app:".into(),
        auth_basics: vec![],
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/items", post(create_item))
        .layer(middleware::from_fn_with_state(config, interceptor));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    neocrates::axum::serve(listener, app).await.unwrap();
}
```

Related module docs:

- [`src/helper/README.md`](src/helper/README.md)
- [`src/middlewares/README.md`](src/middlewares/README.md)
- [`src/response/README.md`](src/response/README.md)
- [`examples/axum_extractor_example.rs`](examples/axum_extractor_example.rs)

## 2. Choose a PostgreSQL integration

You currently have two helper paths:

| If you want... | Use... |
| --- | --- |
| Diesel DSL + deadpool-diesel + logged Diesel macros | `dieselhelper` |
| SQLx + migrations + logged SQLx macros | `sqlxhelper` |

### Diesel path

```rust
use neocrates::dieselhelper::{logging::set_sql_logging, pool::DieselPool};

async fn diesel_flow() -> neocrates::anyhow::Result<()> {
    let pool = DieselPool::new("postgres://postgres:postgres@localhost/app", 10).await?;
    set_sql_logging(true);

    pool.health_check().await?;

    pool.transaction(|conn| {
        // Example:
        // let rows = neocrates::diesel_load!(conn, users::table.limit(10), User)?;
        Ok::<_, diesel::result::Error>(())
    })
    .await?;

    Ok(())
}
```

### SQLx path

```rust
use neocrates::sqlxhelper::{logging::set_sql_logging, pool::SqlxPool};

async fn sqlx_flow() -> neocrates::anyhow::Result<()> {
    let pool = SqlxPool::from_env().await?;
    set_sql_logging(true);

    pool.health_check().await?;

    let count: (i64,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool.pool())
        .await?;

    println!("{count:?}");
    Ok(())
}
```

Related module docs:

- [`src/dieselhelper/README.md`](src/dieselhelper/README.md)
- [`src/sqlxhelper/README.md`](src/sqlxhelper/README.md)

## 3. Use Redis for cache, auth, and captcha

Recommended features:

```toml
neocrates = { version = "0.1", default-features = false, features = ["web", "redis", "auth", "captcha"] }
```

Step-by-step:

1. Initialize a `RedisPool`.
2. Use `AuthHelper` to issue or rotate tokens.
3. Use `CaptchaService` to generate a numeric or slider challenge.
4. Reuse the same Redis deployment for OTPs, sessions, or lock keys.

Example:

```rust
use std::sync::Arc;

use neocrates::auth::auth_helper::AuthHelper;
use neocrates::captcha::CaptchaService;
use neocrates::middlewares::models::AuthModel;
use neocrates::rediscache::RedisPool;

async fn security_flow() -> neocrates::anyhow::Result<()> {
    let redis = Arc::new(RedisPool::from_env().await?);

    let auth_model = AuthModel {
        uid: 42,
        mobile: "13800138000".into(),
        nickname: "neo".into(),
        username: "neo".into(),
        tid: 1,
        tname: "tenant".into(),
        ouid: 10,
        ouname: "org".into(),
        rids: vec![1],
        pmsids: vec![100, 101],
    };

    let tokens = AuthHelper::generate_auth_token(&redis, "app:", 300, 86_400, auth_model).await?;
    println!("access token: {}", tokens.access_token);

    let captcha = CaptchaService::gen_numeric_captcha(&redis, "app:", "user@example.com", Some(6), Some(300)).await?;
    CaptchaService::validate_numeric_captcha(&redis, "app:", &captcha.id, &captcha.code, true).await?;

    Ok(())
}
```

Related module docs:

- [`src/rediscache/README.md`](src/rediscache/README.md)
- [`src/auth/README.md`](src/auth/README.md)
- [`src/captcha/README.md`](src/captcha/README.md)
- [`examples/captcha_example.rs`](examples/captcha_example.rs)

## 4. Work with object storage and temporary credentials

There are two layers:

- **`awss3`** for a direct S3-compatible client
- **`aws`** for the higher-level, config-driven service wrapper

### Low-level storage client

```rust
use std::time::Duration;

use neocrates::awss3::aws::AwsClient;

async fn s3_flow() -> neocrates::anyhow::Result<()> {
    let client = AwsClient::new(
        "my-bucket",
        "us-east-1",
        "https://s3.amazonaws.com",
        "access-key",
        "secret-key",
    )
    .await?;

    client.put_object("uploads/hello.txt", b"hello".to_vec()).await?;
    let url = client.get_presigned_put_url("uploads/client.bin", Duration::from_secs(600)).await?;
    println!("{url}");
    Ok(())
}
```

### High-level service wrapper

Use `AwsService` when your application already owns an `AwsConfig` and wants a singleton-like helper for object download/upload/presigned URLs. Use `CosService` when you want config-driven STS output and Redis caching for Aliyun credentials.

Related module docs:

- [`src/aws/README.md`](src/aws/README.md)
- [`src/awss3/README.md`](src/awss3/README.md)
- [`src/awssts/README.md`](src/awssts/README.md)

## 5. Send SMS verification codes

Recommended features:

```toml
neocrates = { version = "0.1", default-features = false, features = ["sms", "redis", "web"] }
```

Suggested flow:

1. Build a `SmsConfig` for Aliyun or Tencent.
2. Set `debug: true` locally to skip real SMS sending.
3. Call `SmsService::send_captcha(...)`.
4. Validate later with `SmsService::valid_auth_captcha(...)`.

Example:

```rust
use std::sync::Arc;

use neocrates::rediscache::RedisPool;
use neocrates::sms::sms_service::{
    AliyunSmsConfig, SmsConfig, SmsProviderConfig, SmsService,
};

async fn sms_flow() -> neocrates::anyhow::Result<()> {
    let redis = Arc::new(RedisPool::from_env().await?);
    let mobile_regex = regex::Regex::new(r"^1[3-9]\\d{9}$")?;

    let config = Arc::new(SmsConfig {
        debug: true,
        provider: SmsProviderConfig::Aliyun(AliyunSmsConfig {
            access_key_id: "ak".into(),
            access_key_secret: "sk".into(),
            sign_name: "MyApp".into(),
            template_code: "SMS_123456".into(),
        }),
    });

    SmsService::send_captcha(&config, &redis, "13800138000", "captcha:sms:", &mobile_regex).await?;
    let code = SmsService::get_captcha_code(&redis, "13800138000", "captcha:sms:").await?;
    println!("{code:?}");
    Ok(())
}
```

Related docs:

- [`src/sms/README.md`](src/sms/README.md)
- [`examples/sms_example.rs`](examples/sms_example.rs)

---

## Environment variables

### Core/library environment variables

| Variable | Used by | Notes |
| --- | --- | --- |
| `ENV` | `helper::core::loader` | Controls environment-specific YAML config file search. |
| `RUST_LOG` | `logger` | Overrides configured tracing level. |
| `DATABASE_URL` | `sqlxhelper::from_env`, examples | PostgreSQL connection string. |
| `DATABASE_POOL_SIZE` | `sqlxhelper::from_env`, examples | SQLx pool size. |
| `REDIS_URL` | `rediscache::from_env`, examples | Redis connection string. |
| `REDIS_MAX_SIZE` | `rediscache::from_env` | Max pool size. |
| `REDIS_MIN_IDLE` | `rediscache::from_env` | Min idle connections. |
| `REDIS_CONNECTION_TIMEOUT` | `rediscache::from_env` | Seconds. |
| `REDIS_IDLE_TIMEOUT` | `rediscache::from_env` | Seconds. |
| `REDIS_MAX_LIFETIME` | `rediscache::from_env` | Seconds. |

### Example-focused environment variables

These are used by example binaries rather than read automatically by the library:

| Variable | Example | Notes |
| --- | --- | --- |
| `SMS_PROVIDER` | `sms_example` | `aliyun` or `tencent`. |
| `SMS_DEBUG` | `sms_example` | `true/1` skips real SMS sending. |
| `MOBILE` | `sms_example` | Demo mobile number. |
| `ALIYUN_SMS_ACCESS_KEY_ID` | `sms_example` | Aliyun SMS credential. |
| `ALIYUN_SMS_ACCESS_KEY_SECRET` | `sms_example` | Aliyun SMS credential. |
| `ALIYUN_SMS_SIGN_NAME` | `sms_example` | Aliyun SMS sign name. |
| `ALIYUN_SMS_TEMPLATE_CODE` | `sms_example` | Aliyun template code. |
| `TENCENT_SMS_SECRET_ID` | `sms_example` | Tencent SMS credential. |
| `TENCENT_SMS_SECRET_KEY` | `sms_example` | Tencent SMS credential. |
| `TENCENT_SMS_APP_ID` | `sms_example` | Tencent SMS app ID. |
| `TENCENT_SMS_REGION` | `sms_example` | Example region such as `ap-beijing`. |
| `TENCENT_SMS_SIGN_NAME` | `sms_example` | Tencent sign name. |
| `TENCENT_SMS_TEMPLATE_ID` | `sms_example` | Tencent template ID. |

---

## Examples and module guides

### Runnable examples

- [`examples/axum_extractor_example.rs`](examples/axum_extractor_example.rs)
- [`examples/captcha_example.rs`](examples/captcha_example.rs)
- [`examples/sms_example.rs`](examples/sms_example.rs)

### Scenario guides

- [`examples/README_AXUM_EXTRACTOR.md`](examples/README_AXUM_EXTRACTOR.md)
- [`examples/README_CAPTCHA.md`](examples/README_CAPTCHA.md)
- [`examples/README_SMS.md`](examples/README_SMS.md)

### Module guides

- [`src/helper/README.md`](src/helper/README.md)
- [`src/logger/README.md`](src/logger/README.md)
- [`src/middlewares/README.md`](src/middlewares/README.md)
- [`src/response/README.md`](src/response/README.md)
- [`src/dieselhelper/README.md`](src/dieselhelper/README.md)
- [`src/sqlxhelper/README.md`](src/sqlxhelper/README.md)
- [`src/rediscache/README.md`](src/rediscache/README.md)
- [`src/auth/README.md`](src/auth/README.md)
- [`src/captcha/README.md`](src/captcha/README.md)
- [`src/aws/README.md`](src/aws/README.md)
- [`src/awss3/README.md`](src/awss3/README.md)
- [`src/awssts/README.md`](src/awssts/README.md)
- [`src/sms/README.md`](src/sms/README.md)
- [`src/crypto/README.md`](src/crypto/README.md)

---

## Development commands

```bash
make build
make build-full
make check
make test
make test-full
make lint
make fmt
make fmt-check
make doc
make dry-run
```

Direct Cargo equivalents:

```bash
cargo build -p neocrates
cargo build -p neocrates --features full
cargo test -p neocrates
cargo test -p neocrates --features full
cargo clippy -p neocrates --features full -- -D warnings
cargo fmt --check
```

---

## Roadmap

Neocrates already covers a broad backend toolkit surface. The most meaningful next steps are:

1. **Tighten feature wiring** so practical dependency sets such as `web + auth + redis`, `web + captcha + redis`, and `web + sms + redis` are clearer and safer.
2. **Expand module-level examples** for selective-feature builds, not only `full`.
3. **Improve typed configuration builders** for storage, STS, SMS, and auth workflows.
4. **Expose more cache ergonomics** around the existing `redis` + `moka` dependency set.
5. **Harden security-sensitive edges** such as STS HTTP client configuration and crypto helper naming/output clarity.
6. **Add deeper docs.rs coverage** with end-to-end examples for each major module.

---

## License

MIT. See [LICENSE](LICENSE).
