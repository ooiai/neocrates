# SMS Service Module

这个模块提供短信验证码发送与校验的能力，支持多短信服务商（当前已内置 **Aliyun** / **Tencent**），并将验证码写入 Redis 以便后续校验。

> 示例代码见：`examples/sms_example.rs`

---

## 功能特性

- 生成 6 位数字验证码
- 通过短信服务商发送验证码
- 发送成功后写入 Redis（避免“没收到短信却能登录”）
- 校验验证码（可选校验成功后删除，实现一次性验证码）
- `debug` 模式：不发短信，只写 Redis，便于本地联调

---

## 依赖与 Features

示例依赖 Redis，并会调用 HTTP（短信服务商 API）。

运行示例推荐开启 `full`：

```bash
cargo run --example sms_example --features full
```

如果你只想按需开启，请确保至少包含：
- `sms`
- `redis`
- `web`（因为 provider 内部依赖 `reqwest` / `urlencoding` 等）

> 由于项目的 feature 组合可能会调整，最稳妥是使用 `--features full` 来运行 examples。

---

## 环境准备

### 1) 启动 Redis

```bash
# macOS/Linux
redis-server

# Docker
docker run -d -p 6379:6379 redis:latest
```

可选：指定 Redis 连接串：

```bash
export REDIS_URL="redis://127.0.0.1:6379"
```

---

## 运行示例：sms_example

### 快速开始（推荐：debug 模式）

不想发真实短信时，可以开启 debug 模式：验证码只写 Redis，不会请求短信服务商。

```bash
export SMS_DEBUG=true
export SMS_PROVIDER=aliyun
export MOBILE=13800138000

cargo run --example sms_example --features full
```

---

## Provider 选择

示例通过环境变量 `SMS_PROVIDER` 选择短信服务商：

- `SMS_PROVIDER=aliyun`（默认）
- `SMS_PROVIDER=tencent`

```bash
export SMS_PROVIDER=aliyun
# 或
export SMS_PROVIDER=tencent
```

---

## Aliyun 配置（SMS_PROVIDER=aliyun）

需要以下环境变量：

```bash
export ALIYUN_SMS_ACCESS_KEY_ID="your-access-key-id"
export ALIYUN_SMS_ACCESS_KEY_SECRET="your-access-key-secret"
export ALIYUN_SMS_SIGN_NAME="your-sign-name"
export ALIYUN_SMS_TEMPLATE_CODE="SMS_xxxxxxxx"
```

运行：

```bash
export SMS_PROVIDER=aliyun
export MOBILE=13800138000

cargo run --example sms_example --features full
```

说明：

- `ALIYUN_SMS_SIGN_NAME`：短信签名
- `ALIYUN_SMS_TEMPLATE_CODE`：模板 Code
- 模板参数示例：`{"code":"123456"}`（示例内部会自动生成）

---

## Tencent 配置（SMS_PROVIDER=tencent）

需要以下环境变量：

```bash
export TENCENT_SMS_SECRET_ID="your-secret-id"
export TENCENT_SMS_SECRET_KEY="your-secret-key"
export TENCENT_SMS_APP_ID="your-sms-sdk-app-id"
export TENCENT_SMS_SIGN_NAME="your-sign-name"
export TENCENT_SMS_TEMPLATE_ID="your-template-id"
```

可选：region（默认 `ap-beijing`）：

```bash
export TENCENT_SMS_REGION="ap-beijing"
# 可选值示例：ap-nanjing / ap-guangzhou / 或任意字符串（会映射为 Region::Other）
```

运行：

```bash
export SMS_PROVIDER=tencent
export MOBILE=13800138000

cargo run --example sms_example --features full
```

说明：

- 腾讯云短信的 `PhoneNumberSet` 通常需要带国家码（如 `+86...`）
- 示例里做了兼容：如果 `MOBILE` 不以 `+` 开头，会自动拼接 `+86`

---

## 示例环境变量一览

通用：

- `REDIS_URL`（可选，默认 `redis://127.0.0.1:6379`）
- `SMS_PROVIDER`（可选，默认 `aliyun`）
- `SMS_DEBUG`（可选，`true/1` 开启 debug 模式）
- `MOBILE`（可选，默认 `13800138000`）

Aliyun：

- `ALIYUN_SMS_ACCESS_KEY_ID`
- `ALIYUN_SMS_ACCESS_KEY_SECRET`
- `ALIYUN_SMS_SIGN_NAME`
- `ALIYUN_SMS_TEMPLATE_CODE`

Tencent：

- `TENCENT_SMS_SECRET_ID`
- `TENCENT_SMS_SECRET_KEY`
- `TENCENT_SMS_APP_ID`
- `TENCENT_SMS_REGION`（可选）
- `TENCENT_SMS_SIGN_NAME`
- `TENCENT_SMS_TEMPLATE_ID`

---

## 验证码 Redis Key 约定

示例里使用：

- `redis_key_prefix = "captcha:sms:"`
- 最终 key = `captcha:sms:{mobile}`

例如：

- `captcha:sms:13800138000`

---

## 常见问题

### 1) 我不想真的发短信，怎么测试？
设置：

```bash
export SMS_DEBUG=true
```

这样 `send_captcha` 不会调用短信服务商，只会把验证码写入 Redis。

### 2) 为什么要“发送成功后”才写 Redis？
避免短信发送失败（或未到达）时仍然生成可用验证码，导致安全/体验问题。

### 3) 腾讯云手机号格式问题？
示例会自动给不带 `+` 的手机号加 `+86`。如果你要支持海外号码，建议直接传入形如 `+1...`、`+44...` 的号码。

---

## 许可证

MIT OR Apache-2.0
