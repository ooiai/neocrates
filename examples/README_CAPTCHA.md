# Captcha Service Module

这个模块提供了多种验证码的生成和验证功能，支持滑动验证码、数字验证码和字母数字验证码。

## 功能特性

### 支持的验证码类型

1. **滑动验证码 (Slider Captcha)** - 用于滑动验证场景
2. **数字验证码 (Numeric Captcha)** - 4-8 位纯数字验证码
3. **字母数字验证码 (Alphanumeric Captcha)** - 4-10 位字母+数字组合（排除易混淆字符）

## 快速开始

### 前置要求

确保你的项目启用了 `web` 和 `redis` features：

```toml
[dependencies]
neocrates = { version = "0.1", features = ["web", "redis"] }
```

### 环境准备

1. **启动 Redis 服务**：

```bash
# macOS/Linux
redis-server

# Docker
docker run -d -p 6379:6379 redis:latest
```

2. **设置环境变量**（可选）：

```bash
export REDIS_URL="redis://127.0.0.1:6379"
```

## 使用示例

### 1. 数字验证码

#### 生成验证码

```rust
use neocrates::captcha::CaptchaService;
use neocrates::rediscache::{RedisPool, RedisConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // 初始化 Redis
    let config = RedisConfig {
        url: "redis://127.0.0.1:6379".to_string(),
        max_size: 10,
        min_idle: Some(1),
        connection_timeout: std::time::Duration::from_secs(5),
        idle_timeout: Some(std::time::Duration::from_secs(600)),
        max_lifetime: Some(std::time::Duration::from_secs(3600)),
    };

    let redis_pool = Arc::new(RedisPool::new(config).await.unwrap());

    // 生成 6 位数字验证码
    let captcha = CaptchaService::gen_numeric_captcha(
        &redis_pool,
        "user@example.com",  // 账户标识
        Some(6)               // 验证码长度
    ).await.unwrap();

    println!("验证码 ID: {}", captcha.id);
    println!("验证码内容: {}", captcha.code);
    println!("有效期: {} 秒", captcha.expires_in);
}
```

#### 验证验证码

```rust
// 验证用户输入的验证码
let result = CaptchaService::validate_numeric_captcha(
    &redis_pool,
    &captcha_id,     // 验证码 ID
    "123456",        // 用户输入的验证码
    true             // 验证成功后是否删除
).await;

match result {
    Ok(_) => println!("验证成功！"),
    Err(e) => println!("验证失败: {}", e),
}
```

### 2. 字母数字验证码

```rust
// 生成字母数字验证码（排除易混淆字符如 0/O, 1/I/l）
let captcha = CaptchaService::gen_alphanumeric_captcha(
    &redis_pool,
    "user@example.com",
    Some(6)  // 6 位验证码
).await.unwrap();

println!("验证码: {}", captcha.code);  // 例如: "A3K7M9"

// 验证（不区分大小写）
let result = CaptchaService::validate_alphanumeric_captcha(
    &redis_pool,
    &captcha.id,
    "a3k7m9",  // 小写也可以
    true
).await;
```

### 3. 滑动验证码

```rust
// 生成滑动验证码
CaptchaService::gen_captcha_slider(
    &redis_pool,
    "abc123",            // 滑动验证码（由前端计算）
    "user@example.com"   // 账户标识
).await.unwrap();

// 验证滑动验证码
let result = CaptchaService::captcha_slider_valid(
    &redis_pool,
    "abc123",            // 用户输入的验证码
    "user@example.com",  // 账户标识
    true                 // 验证成功后删除
).await;
```

### 4. 手动删除验证码

```rust
// 删除滑动验证码
CaptchaService::captcha_slider_delete(
    &redis_pool,
    "user@example.com"
).await.unwrap();
```

## 运行示例应用

### 启动示例服务器

```bash
cargo run --example captcha_example --features web,redis
```

服务器将在 `http://127.0.0.1:3000` 启动。

### API 端点

| 方法 | 路径                                 | 描述               |
| ---- | ------------------------------------ | ------------------ |
| GET  | `/`                                  | 健康检查           |
| POST | `/api/captcha/numeric/generate`      | 生成数字验证码     |
| POST | `/api/captcha/numeric/validate`      | 验证数字验证码     |
| POST | `/api/captcha/alphanumeric/generate` | 生成字母数字验证码 |
| POST | `/api/captcha/alphanumeric/validate` | 验证字母数字验证码 |
| POST | `/api/captcha/slider/generate`       | 生成滑动验证码     |
| POST | `/api/captcha/slider/validate`       | 验证滑动验证码     |

## 测试示例

### 1. 生成数字验证码

```bash
curl -X POST http://localhost:3000/api/captcha/numeric/generate \
  -H "Content-Type: application/json" \
  -d '{
    "account": "user@example.com",
    "length": 6
  }'
```

**响应示例**：

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "code": "123456",
    "expires_in": 120
  },
  "message": "Numeric captcha generated successfully"
}
```

### 2. 验证数字验证码

```bash
curl -X POST http://localhost:3000/api/captcha/numeric/validate \
  -H "Content-Type: application/json" \
  -d '{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "code": "123456"
  }'
```

**成功响应**：

```json
{
  "success": true,
  "data": "Valid",
  "message": "Captcha validation successful"
}
```

**失败响应**：

```json
{
  "success": false,
  "data": null,
  "message": "Captcha validation failed: Numeric captcha verification failed"
}
```

### 3. 生成字母数字验证码

```bash
curl -X POST http://localhost:3000/api/captcha/alphanumeric/generate \
  -H "Content-Type: application/json" \
  -d '{
    "account": "user@example.com",
    "length": 6
  }'
```

**响应示例**：

```json
{
  "success": true,
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "code": "A3K7M9",
    "expires_in": 120
  },
  "message": "Alphanumeric captcha generated successfully"
}
```

### 4. 验证字母数字验证码（不区分大小写）

```bash
curl -X POST http://localhost:3000/api/captcha/alphanumeric/validate \
  -H "Content-Type: application/json" \
  -d '{
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "code": "a3k7m9"
  }'
```

### 5. 生成滑动验证码

```bash
curl -X POST http://localhost:3000/api/captcha/slider/generate \
  -H "Content-Type: application/json" \
  -d '{
    "account": "user@example.com",
    "code": "abc123"
  }'
```

### 6. 验证滑动验证码

```bash
curl -X POST http://localhost:3000/api/captcha/slider/validate \
  -H "Content-Type: application/json" \
  -d '{
    "account": "user@example.com",
    "code": "abc123"
  }'
```

## 配置说明

### Redis 配置

```rust
use neocrates::rediscache::RedisConfig;

let config = RedisConfig {
    url: "redis://127.0.0.1:6379".to_string(),
    max_size: 10,                                           // 连接池最大连接数
    min_idle: Some(1),                                      // 最小空闲连接数
    connection_timeout: std::time::Duration::from_secs(5),  // 连接超时
    idle_timeout: Some(std::time::Duration::from_secs(600)), // 空闲超时（10分钟）
    max_lifetime: Some(std::time::Duration::from_secs(3600)), // 最大生命周期（1小时）
};
```

### 验证码配置

验证码默认有效期为 **120 秒（2 分钟）**。

可以通过修改 `CaptchaService::DEFAULT_EXPIRATION` 常量来调整。

## 数据结构

### CaptchaData

```rust
pub struct CaptchaData {
    /// 验证码 ID（用于后续验证）
    pub id: String,

    /// 验证码内容（实际的验证码值）
    pub code: String,

    /// 有效期（秒）
    pub expires_in: u64,
}
```

### CaptchaType

```rust
pub enum CaptchaType {
    /// 滑动验证码
    Slider,

    /// 数字验证码（4-8 位）
    Numeric,

    /// 字母数字验证码（4-10 位）
    Alphanumeric,
}
```

## Redis 存储键格式

- **数字验证码**: `captcha:numeric:{id}`
- **字母数字验证码**: `captcha:alpha:{id}`
- **滑动验证码**: `captcha:slider:{account}`

## 安全性说明

1. **哈希存储**：滑动验证码使用 MD5 哈希存储，防止明文泄露
2. **自动过期**：所有验证码默认 120 秒后自动过期
3. **一次性使用**：验证成功后可选择立即删除（推荐）
4. **字符排除**：字母数字验证码排除易混淆字符（0/O, 1/I/l）

## 最佳实践

### 1. 生成验证码时

```rust
// ✅ 推荐：记录验证码 ID，不要返回验证码内容给前端
let captcha = CaptchaService::gen_numeric_captcha(&redis_pool, account, Some(6)).await?;

// 只返回 ID 给前端
return Json(json!({
    "captcha_id": captcha.id,
    "expires_in": captcha.expires_in
}));
```

### 2. 验证验证码时

```rust
// ✅ 推荐：验证成功后立即删除
CaptchaService::validate_numeric_captcha(
    &redis_pool,
    &id,
    &code,
    true  // 删除验证码，防止重复使用
).await?;
```

### 3. 错误处理

```rust
match CaptchaService::validate_numeric_captcha(&redis_pool, &id, &code, true).await {
    Ok(_) => {
        // 验证成功，继续后续流程
    }
    Err(AppError::ClientError(msg)) => {
        // 客户端错误（验证码错误、已过期等）
        return Err(StatusCode::BAD_REQUEST);
    }
    Err(AppError::RedisError(msg)) => {
        // Redis 错误（连接失败等）
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Err(_) => {
        // 其他错误
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
}
```

### 4. 与 Axum 集成

```rust
use neocrates::axum::{Router, routing::post, extract::State};
use neocrates::captcha::CaptchaService;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    redis_pool: Arc<RedisPool>,
}

async fn generate_captcha(
    State(state): State<AppState>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<CaptchaData>, StatusCode> {
    CaptchaService::gen_numeric_captcha(&state.redis_pool, &req.account, Some(6))
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

let app = Router::new()
    .route("/captcha/generate", post(generate_captcha))
    .with_state(AppState { redis_pool });
```

## 常见问题

### Q: 验证码总是验证失败？

A: 检查以下几点：

1. Redis 是否正常运行
2. 验证码是否已过期（默认 120 秒）
3. 验证码 ID 是否正确
4. 字母数字验证码是否大小写匹配（不区分大小写）

### Q: 如何调整验证码有效期？

A: 修改 `CaptchaService::DEFAULT_EXPIRATION` 常量，或在生成时自定义过期时间。

### Q: 如何自定义验证码长度？

A: 在调用生成函数时传入 `Some(length)` 参数：

```rust
// 生成 8 位数字验证码
let captcha = CaptchaService::gen_numeric_captcha(&redis_pool, account, Some(8)).await?;
```

### Q: 验证码可以重复使用吗？

A: 不推荐。验证时将 `delete` 参数设置为 `true` 可以确保验证码只能使用一次。

### Q: 如何处理并发验证？

A: Redis 操作是原子性的，多个请求同时验证同一个验证码时，只有第一个会成功（如果启用了删除）。

## 性能建议

1. **连接池大小**：根据并发量调整 `max_size`，推荐值为 10-50
2. **批量操作**：如果需要批量生成验证码，考虑使用 Pipeline
3. **监控**：监控 Redis 连接池使用情况和验证码验证失败率

## 相关资源

- [Axum 官方文档](https://docs.rs/axum/)
- [Redis 文档](https://redis.io/documentation)
- [Neocrates 完整文档](https://docs.rs/neocrates/)

## 许可证

MIT OR Apache-2.0
