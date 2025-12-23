# Neocrates

一个全面的 Rust 库，为 Web 开发、AWS 集成、数据库操作、缓存等提供统一的访问接口。Neocrates 作为门面 crate，重新导出多个内部模块的功能。

[![crates.io](https://img.shields.io/crates/v/neocrates.svg)](https://crates.io/crates/neocrates)
[![docs.rs](https://img.shields.io/docsrs/neocrates)](https://docs.rs/neocrates)
[![License](https://img.shields.io/crates/l/neocrates)](https://github.com/ooiai/neocrates/blob/main/LICENSE)

- **English Documentation**: [README.md](README.md)

---

## 🚀 功能特性

- **模块化设计**：通过特性标志按需启用功能
- **AWS 集成**：支持 Aliyun/Tencent Cloud 的 S3 和 STS 客户端
- **数据库助手**：Diesel 集成与连接池
- **缓存支持**：Redis 连接池和缓存工具
- **Web 工具**：日志记录、中间件、响应处理和验证
- **安全功能**：加密工具和短信功能
- **零成本**：未使用的功能不会增加二进制文件大小

---

## 📦 安装

在你的项目 `Cargo.toml` 中添加 Neocrates：

### 全功能版本（推荐用于快速开始）

```toml
[dependencies]
neocrates = "0.1"
```

### 按需选择功能（推荐用于生产环境）

```toml
[dependencies]
neocrates = { version = "0.1", default-features = false, features = ["awss3", "rediscache", "logger"] }
```

### 最低支持的 Rust 版本 (MSRV)

- Rust 1.84+（使用 `edition = "2024"`）

---

## 🔧 特性标志

Neocrates 使用特性标志来保持依赖精简。所有特性默认通过 `full` 特性启用。

| 特性           | 描述                        | 依赖                        |
| -------------- | --------------------------- | --------------------------- |
| `awss3`        | S3 客户端工具               | aws-sdk-s3, aws-config      |
| `awssts`       | STS 客户端 (Aliyun/Tencent) | aws-sdk-sts, hmac, sha2     |
| `crypto`       | 加密工具                    | openssl, ring, argon2       |
| `dieselhelper` | Diesel 数据库助手           | diesel, deadpool-diesel     |
| `helper`       | 通用工具                    | serde, validator, uuid      |
| `logger`       | 基于 Tracing 的日志         | tracing, tracing-subscriber |
| `middleware`   | Web 中间件                  | axum, tower-http            |
| `rediscache`   | Redis 缓存工具              | redis, bb8-redis, moka      |
| `response`     | 响应类型                    | axum, serde_json            |
| `sms`          | 短信工具                    | reqwest, hmac, sha2         |
| `full`         | 启用以上所有特性            | -                           |

**禁用默认特性：**

```toml
neocrates = { version = "0.1", default-features = false, features = ["awss3", "logger"] }
```

---

## 🎯 使用示例

### 基础设置

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志（需要 "logger" 特性）
    #[cfg(feature = "logger")]
    neocrates::logger::run().await;

    // 使用 S3 客户端（需要 "awss3" 特性）
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

        // 上传对象
        s3_client.put_object("uploads/file.txt", b"Hello, World!".to_vec()).await?;
    }

    // 使用 Redis 缓存（需要 "rediscache" 特性）
    #[cfg(feature = "rediscache")]
    {
        use neocrates::rediscache::RedisPool;

        let redis_pool = RedisPool::from_env().await?;
        let mut conn = redis_pool.get_connection().await?;

        // 设置和获取缓存
        neocrates::redis::cmd("SET").arg("key").arg("value").query_async(&mut *conn).await?;
        let value: String = neocrates::redis::cmd("GET").arg("key").query_async(&mut *conn).await?;
    }

    Ok(())
}
```

### AWS STS 客户端

```rust
// Aliyun STS 客户端
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
    println!("临时 AK: {}", credentials.credentials.access_key_id);

    Ok(())
}

// Tencent STS 客户端
#[cfg(feature = "awssts")]
async fn tencent_sts_example() -> anyhow::Result<()> {
    use neocrates::awssts::tencent::StsClient;

    let tencent_client = StsClient::new(
        "YOUR_SECRET_ID",
        "YOUR_SECRET_KEY",
        "ap-guangzhou"
    );

    // 注意：请查看文档以获取具体的方法签名
    // let credentials = tencent_client.get_temp_credentials(...).await?;

    Ok(())
}
```

### 数据库操作

```rust
#[cfg(feature = "dieselhelper")]
use neocrates::dieselhelper;

#[cfg(feature = "dieselhelper")]
async fn database_example() -> anyhow::Result<()> {
    // 初始化数据库连接池
    let pool = dieselhelper::create_pool("DATABASE_URL").await?;

    // 使用连接池中的连接
    dieselhelper::with_connection(&pool, |conn| {
        // 在这里执行数据库操作
        // 例如: User::find_by_id(conn, 1)?
        Ok::<(), neocrates::diesel::result::Error>(())
    }).await?;

    Ok(())
}
```

### Web 应用（带中间件）

```rust
#[cfg(all(feature = "axum", feature = "middleware"))]
use neocrates::{axum, middleware};

#[cfg(all(feature = "axum", feature = "middleware"))]
async fn web_app() -> anyhow::Result<()> {
    use axum::{routing::get, Router};

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .layer(middleware::trace_layer()) // 添加追踪中间件
        .layer(middleware::cors_layer()); // 添加 CORS 中间件

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## ⚙️ 配置

### 环境变量

许多模块支持基于环境的配置：

- **Redis**: `REDIS_URL`, `REDIS_POOL_SIZE`
- **数据库**: `DATABASE_URL`, `DATABASE_POOL_SIZE`
- **日志**: `RUST_LOG` (默认: "info")
- **AWS**: 标准 AWS 环境变量

### 自定义配置

对于高级用例，大多数模块接受自定义配置结构：

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

## 🛠️ 开发命令

### 构建

```bash
# 默认（所有特性）
cargo build -p neocrates

# 选择性特性
cargo build -p neocrates --no-default-features --features "awss3,rediscache,logger"

# 发布构建
cargo build --release -p neocrates
```

### 测试

```bash
# 运行所有测试
cargo test -p neocrates

# 测试特定特性
cargo test -p neocrates --features "awss3,rediscache"
```

### 代码检查

```bash
cargo clippy -p neocrates -- -D warnings
cargo fmt --check
```

### 文档

```bash
# 生成本地文档
cargo doc -p neocrates --open

# 检查文档链接
cargo doc -p neocrates --no-deps
```

---

## 📤 发布（维护者指南）

### 先决条件

1. 在 `Cargo.toml` 中填写完整的包元数据
2. 有效的许可证文件（`LICENSE-MIT`, `LICENSE-APACHE`）
3. 干净的 git 仓库（无未提交的更改）

### 发布序列

```bash
# 先测试发布
cargo publish -p neocrates --dry-run

# 发布到 crates.io
cargo publish -p neocrates --registry crates-io
```

### 版本管理

- 遵循语义化版本控制 (SemVer)
- 在工作区根目录 `Cargo.toml` 中更新版本
- 修改公共 API 时考虑破坏性变更

---

## 📚 文档

- **API 参考**: [docs.rs/neocrates](https://docs.rs/neocrates)
- **源代码**: [GitHub 仓库](https://github.com/ooiai/neocrates)
- **包信息**: [crates.io/neocrates](https://crates.io/crates/neocrates)

---

## 🤝 贡献

欢迎贡献！请遵循以下准则：

1. **特性标志**：新功能尽可能放在特性标志后面
2. **测试**：为新功能包含测试
3. **文档**：更新 README 并添加文档注释
4. **代码质量**：提交前运行 `cargo clippy` 和 `cargo fmt`

### 开发工作流

```bash
# 克隆和设置
git clone https://github.com/ooiai/neocrates.git
cd neocrates

# 构建和测试
cargo build -p neocrates
cargo test -p neocrates

# 验证发布准备就绪
cargo publish -p neocrates --dry-run
```

---

## 🛡️ 安全

- **凭据**：切勿在代码或示例中硬编码机密信息
- **依赖**：保持依赖更新以解决安全漏洞
- **最小权限原则**：为 AWS 角色和数据库用户使用最小权限
- **输入验证**：始终验证和清理用户输入

如果您发现安全漏洞，请直接联系维护者。

---

## 📄 许可证

Neocrates 采用双重许可证：

- **MIT 许可证** ([LICENSE-MIT](LICENSE-MIT))
- **Apache 许可证 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

SPDX-License-Identifier: MIT OR Apache-2.0

---

## 🙏 致谢

感谢 Rust 社区和我们所依赖的优秀 crate 的作者们：

- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
- [Axum](https://github.com/tokio-rs/axum)
- [Diesel](https://github.com/diesel-rs/diesel)
- [Redis-rs](https://github.com/redis-rs/redis-rs)
- [Tracing](https://github.com/tokio-rs/tracing)
- 以及其他许多优秀的项目！
