# Neocrates

一个全面的 Rust 工具库，通过模块化方式提供 Web 开发、AWS 集成、数据库操作、Redis 缓存、加密、认证等功能。Neocrates 作为门面 crate，对生态系统中的优秀 crate 进行封装和重新导出 —— 通过 Feature 标志按需启用，你只为用到的功能付费。

[![crates.io](https://img.shields.io/crates/v/neocrates.svg)](https://crates.io/crates/neocrates)
[![docs.rs](https://img.shields.io/docsrs/neocrates)](https://docs.rs/neocrates)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/ooiai/neocrates/blob/main/LICENSE)
[![Build](https://github.com/ooiai/neocrates/actions/workflows/rust.yml/badge.svg)](https://github.com/ooiai/neocrates/actions/workflows/rust.yml)

**English Documentation**: [README.md](README.md)

---

## ✨ 功能特性

| Feature    | 说明                                                           |
|------------|----------------------------------------------------------------|
| `web`      | Axum + Tower + Hyper、reqwest HTTP 客户端、URL 工具、中间件与响应类型 |
| `aws`      | 完整 AWS 套件（S3 + STS，支持 AWS / 阿里云 / 腾讯云）          |
| `awss3`    | 仅 S3 客户端                                                   |
| `awssts`   | 仅 STS 客户端（阿里云 & 腾讯云）                               |
| `diesel`   | Diesel ORM + deadpool-diesel 连接池（PostgreSQL）               |
| `redis`    | bb8-redis 连接池 + Moka 进程内缓存                              |
| `crypto`   | Argon2 密码哈希、HMAC、SHA-2、Ring 底层加密                     |
| `sms`      | 短信助手（阿里云 & 腾讯云）                                     |
| `captcha`  | 验证码服务                                                     |
| `auth`     | JWT 与会话认证助手                                              |
| `logger`   | 结构化 tracing-subscriber 日志                                  |
| `full`     | 以上全部                                                       |

**最低支持 Rust 版本（MSRV）**：Rust 1.84+（edition 2024）

---

## 📦 安装

```toml
[dependencies]
# 全功能 —— 快速上手
neocrates = { version = "0.1", features = ["full"] }

# 按需选择 —— 推荐用于生产环境（更小的二进制体积）
neocrates = { version = "0.1", default-features = false, features = ["web", "redis", "logger"] }
```

---

## 🚀 快速开始

### 日志

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "logger")]
    neocrates::logger::run().await;

    neocrates::tracing::info!("neocrates 已就绪");
    Ok(())
}
```

### S3 客户端

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

### 阿里云 / 腾讯云 STS

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
    println!("临时 AK: {}", creds.credentials.access_key_id);
}
```

### Redis 缓存

```rust
#[cfg(feature = "redis")]
{
    use neocrates::rediscache::RedisPool;

    let pool = RedisPool::from_env().await?;          // 读取 REDIS_URL
    let mut conn = pool.get_connection().await?;

    neocrates::redis::cmd("SET").arg("key").arg("val").query_async(&mut *conn).await?;
    let v: String = neocrates::redis::cmd("GET").arg("key").query_async(&mut *conn).await?;
    println!("{v}");
}
```

### Diesel（PostgreSQL）

```rust
#[cfg(feature = "diesel")]
{
    use neocrates::dieselhelper;

    let pool = dieselhelper::create_pool(&std::env::var("DATABASE_URL")?).await?;
    dieselhelper::with_connection(&pool, |conn| {
        // diesel 操作 …
        Ok::<(), neocrates::diesel::result::Error>(())
    }).await?;
}
```

### Axum Web 应用

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

## ⚙️ 环境变量

| 变量名                    | 模块         | 说明                          |
|--------------------------|--------------|-------------------------------|
| `DATABASE_URL`           | `diesel`     | PostgreSQL 连接字符串         |
| `DATABASE_POOL_SIZE`     | `diesel`     | 连接池大小                    |
| `REDIS_URL`              | `redis`      | Redis 连接 URL                |
| `REDIS_POOL_SIZE`        | `redis`      | Redis 连接池大小              |
| `RUST_LOG`               | `logger`     | 日志级别过滤（默认：`info`）  |
| `AWS_ACCESS_KEY_ID`      | `awss3`      | AWS / S3 兼容访问密钥         |
| `AWS_SECRET_ACCESS_KEY`  | `awss3`      | AWS / S3 兼容密钥             |

---

## 🛠️ 开发命令

```bash
make build          # cargo build
make build-full     # cargo build --features full
make test           # cargo test
make test-full      # cargo test --features full
make lint           # cargo clippy -D warnings
make fmt            # cargo fmt
make doc            # cargo doc --open
make audit          # cargo audit（需安装 cargo-audit）
make dry-run        # 测试发布
make publish m="release: v0.1.x"
```

运行 `make help` 查看所有可用目标。

---

## 🤝 贡献指南

1. 新功能应放在 Feature 标志后面。
2. 为新功能添加测试（异步测试使用 `#[tokio::test]`）。
3. 提交 PR 前运行 `make lint` 和 `make fmt`。
4. 添加新模块时同步更新本 README 和 `AGENTS.md`。

```bash
git clone https://github.com/ooiai/neocrates.git
cd neocrates
make build-full && make test-full
```

---

## 🛡️ 安全

- 切勿将凭据硬编码在代码中 —— 使用环境变量或密钥管理服务。
- 在处理前始终验证和清理用户输入。
- 密码哈希请使用 Argon2（通过 `crypto` feature），禁止使用 MD5 或 SHA-1。
- 如发现安全漏洞，请直接联系维护者。

---

## 📚 相关资源

- **API 文档**：[docs.rs/neocrates](https://docs.rs/neocrates)
- **crates.io**：[crates.io/crates/neocrates](https://crates.io/crates/neocrates)
- **源代码**：[github.com/ooiai/neocrates](https://github.com/ooiai/neocrates)
- **使用示例**：[USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)

---

## 📄 许可证

MIT License — Copyright © 2026 [ooiai](https://github.com/ooiai)

完整许可证文本见 [LICENSE](LICENSE)。
