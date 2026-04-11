# SMS Module

The `sms` module provides provider-specific SMS clients for Aliyun and Tencent, plus a higher-level OTP workflow service that generates verification codes, sends them, and stores them in Redis for later validation.

See also: [root README](../../README.md), [`../../examples/sms_example.rs`](../../examples/sms_example.rs)

---

## Feature and runtime requirements

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["sms", "redis", "web"] }
```

Practical note:

- The current provider implementations rely on the HTTP stack, so `sms` is most safely used with `web`.
- OTP storage is Redis-backed, so real workflows normally include `redis`.

---

## Main building blocks

### High-level OTP service

- `SmsConfig`
- `SmsProviderConfig`
- `AliyunSmsConfig`
- `TencentSmsConfig`
- `SmsSendResult`
- `SmsService::send_captcha(...)`
- `SmsService::send_captcha_with_options(...)`
- `SmsService::valid_auth_captcha(...)`
- `SmsService::store_captcha_code(...)`
- `SmsService::get_captcha_code(...)`
- `SmsService::delete_captcha_code(...)`

### Low-level providers

- `aliyun::Aliyun`
- `tencent::Tencent`
- `tencent::Region`

---

## Quick start

```rust
use std::sync::Arc;

use neocrates::rediscache::RedisPool;
use neocrates::sms::sms_service::{
    AliyunSmsConfig, SmsConfig, SmsProviderConfig, SmsService,
};

async fn demo() -> neocrates::anyhow::Result<()> {
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
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Choose a provider config

Aliyun:

```rust
use neocrates::sms::sms_service::{AliyunSmsConfig, SmsConfig, SmsProviderConfig};

let config = SmsConfig {
    debug: false,
    provider: SmsProviderConfig::Aliyun(AliyunSmsConfig {
        access_key_id: "ak".into(),
        access_key_secret: "sk".into(),
        sign_name: "MyApp".into(),
        template_code: "SMS_123456".into(),
    }),
};
```

Tencent:

```rust
use neocrates::sms::sms_service::{SmsConfig, SmsProviderConfig, TencentSmsConfig};
use neocrates::sms::tencent::Region;

let config = SmsConfig {
    debug: false,
    provider: SmsProviderConfig::Tencent(TencentSmsConfig {
        secret_id: "secret-id".into(),
        secret_key: "secret-key".into(),
        sms_app_id: "app-id".into(),
        region: Region::Beijing,
        sign_name: "MyApp".into(),
        template_id: "template-id".into(),
    }),
};
```

## 2. Send a code

```rust
use std::sync::Arc;

let config = Arc::new(config);
let redis_key_prefix = "captcha:sms:";
let mobile_regex = regex::Regex::new(r"^1[3-9]\\d{9}$")?;

SmsService::send_captcha(
    &config,
    &redis_pool,
    "13800138000",
    redis_key_prefix,
    &mobile_regex,
)
.await?;
```

## 3. Validate a code

```rust
SmsService::valid_auth_captcha(
    &redis_pool,
    "13800138000",
    "123456",
    "captcha:sms:",
    true,
)
.await?;
```

## 4. Use debug mode for local development

```rust
let config = Arc::new(SmsConfig {
    debug: true,
    provider: SmsProviderConfig::Aliyun(aliyun_cfg),
});
```

With `debug: true`, the service skips the real SMS API call and still stores the generated code in Redis so you can test the end-to-end OTP flow.

---

## Key points and gotchas

- The OTP service always generates a 6-digit numeric code.
- Tencent phone numbers are normalized by auto-prepending `+86` when the input does not already start with `+`.
- Aliyun and Tencent expect different template-parameter shapes internally.
- `valid_auth_captcha(...)` deletes the stored code on mismatch, which is a deliberate anti-brute-force behavior.
- The module does not include rate limiting or resend throttling; add that at the application layer.

---

## Roadmap

Potential next steps:

1. Add provider traits so adding new vendors is cleaner.
2. Add resend throttling and per-number rate-limiting helpers.
3. Add more example coverage for Tencent flows.
4. Expose a cleaner template-parameter abstraction for multi-parameter SMS templates.
