# Captcha Module

The `captcha` module provides Redis-backed captcha generation and validation for three common challenge types:

- slider captcha
- numeric captcha
- alphanumeric captcha

See also: [root README](../../README.md)

---

## Feature and runtime requirements

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["web", "captcha", "redis"] }
```

For an HTTP demo, see: [`../../examples/captcha_example.rs`](../../examples/captcha_example.rs)

---

## What the module exposes

- `CaptchaType` — `Slider | Numeric | Alphanumeric`
- `CaptchaData` — `{ id, code, expires_in }`
- `CaptchaService` — static helper with generation and validation methods

Default behavior:

- default TTL: **120 seconds**
- numeric length clamps to **4-8**
- alphanumeric length clamps to **4-10**
- slider challenges store **MD5(code)** instead of the raw code

---

## Quick start

```rust
use std::sync::Arc;

use neocrates::captcha::CaptchaService;
use neocrates::rediscache::RedisPool;

async fn numeric_demo() -> neocrates::anyhow::Result<()> {
    let redis = Arc::new(RedisPool::from_env().await?);

    let captcha = CaptchaService::gen_numeric_captcha(&redis, "app:", "user@example.com", Some(6), Some(300)).await?;
    CaptchaService::validate_numeric_captcha(&redis, "app:", &captcha.id, &captcha.code, true).await?;
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Generate a numeric captcha

```rust
let captcha = CaptchaService::gen_numeric_captcha(
    &redis_pool,
    "app:",
    "user@example.com",
    Some(6),
    Some(300),
)
.await?;

println!("id = {}", captcha.id);
println!("code = {}", captcha.code);
```

The generated Redis key looks like:

```text
app::captcha:numeric:{id}
```

## 2. Validate and delete it

```rust
CaptchaService::validate_numeric_captcha(
    &redis_pool,
    "app:",
    &captcha.id,
    &user_input_code,
    true,
)
.await?;
```

If `delete` is `true`, the captcha is removed after a successful validation.

## 3. Generate an alphanumeric captcha

```rust
let captcha = CaptchaService::gen_alphanumeric_captcha(
    &redis_pool,
    "app:",
    "user@example.com",
    Some(6),
    Some(300),
)
.await?;
```

Validation is case-insensitive:

```rust
CaptchaService::validate_alphanumeric_captcha(
    &redis_pool,
    "app:",
    &captcha.id,
    "a3k7m9",
    true,
)
.await?;
```

## 4. Store and verify a slider captcha

```rust
CaptchaService::gen_captcha_slider(
    &redis_pool,
    "app:",
    "slider-offset",
    "account-42",
    Some(120),
)
.await?;

CaptchaService::captcha_slider_valid(
    &redis_pool,
    "app:",
    "slider-offset",
    "account-42",
    true,
)
.await?;
```

---

## Key points and gotchas

- Slider captchas hash the submitted code with MD5 before storing it. This is obfuscation, not strong cryptography.
- Numeric and alphanumeric generation use UUID bytes as the randomness source.
- The module does not include generation throttling or attempt counters; pair it with rate limiting if needed.
- Redis is the persistence layer; expired captchas disappear because Redis TTLs expire them.

---

## Roadmap

Potential next steps:

1. Add image-based or puzzle-based slider generation helpers.
2. Add built-in retry/attempt counters and rate limiting hooks.
3. Add pluggable storage so non-Redis backends can be supported.
4. Add configurable charsets and security levels for alphanumeric captchas.
