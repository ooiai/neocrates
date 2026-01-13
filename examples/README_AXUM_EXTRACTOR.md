# Axum Custom JSON Extractors

这个模块提供了两个增强的 JSON extractors，用于改进 Axum 应用中的错误处理和日志记录。

## 功能特性

### LoggedJson

一个简单的 JSON extractor，会在反序列化失败时记录详细的错误信息。

**特点：**

- 自动记录反序列化错误（使用 `tracing::error!`）
- 返回统一的错误响应格式
- 状态码：422 Unprocessable Entity

**错误响应格式：**

```json
{
  "error": "JSON deserialization failed",
  "message": "JSON 反序列化失败: ..."
}
```

### DetailedJson

提供更详细的错误信息，根据不同的错误类型返回相应的 HTTP 状态码。

**支持的错误类型：**

- **JsonDataError** (422) - 数据格式不匹配（如类型错误、缺失字段等）
- **JsonSyntaxError** (400) - JSON 语法错误
- **MissingJsonContentType** (415) - 缺少 `Content-Type: application/json` 头
- **BytesRejection** (500) - 无法读取请求体
- **其他错误** (400) - 未知错误

**错误响应格式：**

```json
{
  "error": "json_data_error",
  "message": "无效的 JSON 数据: missing field `email`",
  "status": 422
}
```

## 快速开始

### 1. 基本使用

```rust
use neocrates::axum::{Router, routing::post};
use neocrates::helper::core::axum_extractor::LoggedJson;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(LoggedJson(payload): LoggedJson<CreateUser>) -> String {
    format!("Created user: {}", payload.name)
}

let app = Router::new()
    .route("/users", post(create_user));
```

### 2. 使用 DetailedJson

```rust
use neocrates::axum::{Router, routing::post};
use neocrates::helper::core::axum_extractor::DetailedJson;
use serde::Deserialize;

#[derive(Deserialize)]
struct UpdateUser {
    id: i64,
    name: Option<String>,
    email: Option<String>,
}

async fn update_user(DetailedJson(payload): DetailedJson<UpdateUser>) -> String {
    format!("Updated user: {}", payload.id)
}

let app = Router::new()
    .route("/users", post(update_user));
```

## 运行示例

### 启动示例服务器

```bash
cargo run --example axum_extractor_example --features web
```

### 测试请求

#### 1. 成功请求

```bash
curl -X POST http://localhost:3000/users/logged \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com","age":30}'
```

**响应：**

```json
{
  "id": 1,
  "name": "Alice",
  "email": "alice@example.com",
  "age": 30
}
```

#### 2. JSON 语法错误

```bash
curl -X POST http://localhost:3000/users/detailed \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"'
```

**响应 (400)：**

```json
{
  "error": "json_syntax_error",
  "message": "JSON 语法错误: ...",
  "status": 400
}
```

#### 3. 缺失必需字段

```bash
curl -X POST http://localhost:3000/users/detailed \
  -H "Content-Type: application/json" \
  -d '{"name":"Bob"}'
```

**响应 (422)：**

```json
{
  "error": "json_data_error",
  "message": "无效的 JSON 数据: missing field `email`",
  "status": 422
}
```

#### 4. 类型错误

```bash
curl -X POST http://localhost:3000/users/detailed \
  -H "Content-Type: application/json" \
  -d '{"name":"Bob","email":"bob@example.com","age":"thirty"}'
```

**响应 (422)：**

```json
{
  "error": "json_data_error",
  "message": "无效的 JSON 数据: invalid type: string \"thirty\", expected u32",
  "status": 422
}
```

#### 5. 缺少 Content-Type 头

```bash
curl -X POST http://localhost:3000/users/detailed \
  -d '{"name":"Charlie","email":"charlie@example.com","age":25}'
```

**响应 (415)：**

```json
{
  "error": "missing_content_type",
  "message": "缺少 Content-Type: application/json 请求头: ...",
  "status": 415
}
```

## 使用场景

### 何时使用 LoggedJson

- 需要简单的错误日志记录
- 想要统一的错误响应格式
- 快速替换标准的 `Json` extractor
- 不需要区分不同的错误类型

### 何时使用 DetailedJson

- 需要详细的错误类型区分
- 不同错误类型需要不同的 HTTP 状态码
- 需要结构化的错误响应，便于前端处理
- 希望提供更好的 API 调试体验
- 构建面向用户的 API

## 对比标准 Json extractor

| 特性           | axum::Json | LoggedJson | DetailedJson |
| -------------- | ---------- | ---------- | ------------ |
| 错误日志       | ❌         | ✅         | ✅           |
| 详细错误类型   | ❌         | ❌         | ✅           |
| 自定义状态码   | ❌         | ❌         | ✅           |
| 结构化错误响应 | ❌         | ✅         | ✅           |
| 开箱即用       | ✅         | ✅         | ✅           |

## 集成建议

### 1. 全局错误处理

配合 Axum 的全局错误处理器使用：

```rust
use neocrates::axum::{Router, response::IntoResponse, http::StatusCode};

async fn handle_error() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
}

let app = Router::new()
    .route("/users", post(create_user))
    .fallback(handle_error);
```

### 2. 与验证库结合

配合 `validator` crate 使用：

```rust
use neocrates::helper::core::axum_extractor::DetailedJson;
use neocrates::validator::Validate;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1, max = 50))]
    name: String,
    #[validate(email)]
    email: String,
}

async fn create_user(DetailedJson(payload): DetailedJson<CreateUser>) -> Result<String, String> {
    payload.validate()
        .map_err(|e| format!("Validation error: {}", e))?;

    Ok(format!("Created user: {}", payload.name))
}
```

### 3. 日志配置

确保在应用启动时初始化 tracing：

```rust
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // ... 启动服务器
}
```

## API 文档

完整的 API 文档请查看：

```bash
cargo doc --open --features web
```

## 相关资源

- [Axum 官方文档](https://docs.rs/axum/)
- [Serde 文档](https://serde.rs/)
- [Tracing 文档](https://docs.rs/tracing/)

## 许可证

MIT OR Apache-2.0
