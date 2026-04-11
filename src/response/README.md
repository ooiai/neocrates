# Response Module

The `response` module defines the common HTTP-facing error and response model used across Neocrates web-oriented modules.

See also: [root README](../../README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["web"] }
```

---

## Core types

- `AppError` — typed application error enum
- `AppResult<T>` — alias for `Result<T, AppError>`
- `ApiResponse<T>` — serialized response payload `{ code, message, data }`
- `AppResultExt` — helpers for attaching consistent `AppError` context to fallible operations

Important `AppError` families:

- client-facing issues: `ValidationError`, `Unauthorized`, `TokenExpired`, `Forbidden`, `NotFound`, `Conflict`, `ClientError`, `ClientDataError`
- business/control-flow responses: `UnprocessableEntity`, `RateLimit`, `EasterEgg`
- server-side issues: `DbError`, `RedisError`, `MqError`, `ExternalError`, `Internal`
- custom business-code path: `DataError(code, message)`

---

## Quick start

```rust
use neocrates::response::error::{AppError, AppResult};

async fn health() -> AppResult<&'static str> {
    Ok("ok")
}

async fn get_secret(authenticated: bool) -> AppResult<&'static str> {
    if !authenticated {
        return Err(AppError::Unauthorized);
    }
    Ok("classified")
}
```

In Axum, returning `AppError` automatically becomes a JSON response through `IntoResponse`.

---

## Step-by-step tutorial

## 1. Use `AppResult<T>` in handlers

```rust
use neocrates::response::error::{AppError, AppResult};

async fn find_user(found: bool) -> AppResult<&'static str> {
    if !found {
        return Err(AppError::not_found_here("user not found"));
    }
    Ok("user")
}
```

## 2. Turn validation failures into a consistent response

`AppError` implements `From<validator::ValidationErrors>`.

```rust
use neocrates::validator::Validate;
use neocrates::response::error::AppResult;

#[derive(Validate)]
struct CreateUser {
    #[validate(email)]
    email: String,
}

fn validate_input(input: &CreateUser) -> AppResult<()> {
    input.validate()?;
    Ok(())
}
```

## 3. Add call-site context to arbitrary errors

```rust
use neocrates::response::error::AppResultExt;

async fn external_call() -> Result<(), neocrates::anyhow::Error> {
    Ok(())
}

async fn wrapped() -> neocrates::response::error::AppResult<()> {
    external_call().await.client_context()?;
    Ok(())
}
```

## 4. Use custom business codes when HTTP status alone is not enough

```rust
use neocrates::response::error::AppError;

fn conflict() -> AppError {
    AppError::DataError(AppError::BIZ_DATA_DUPLICATE, "duplicate record".into())
}
```

---

## Key points and gotchas

- `AppError::IntoResponse` always serializes the JSON shape `{ code, message, data }`.
- `ClientError` currently maps to HTTP **417 Expectation Failed**.
- `DataError(code, msg)` always maps to HTTP **409 Conflict**, even though the business code is custom.
- The `*_here(...)` constructors use `#[track_caller]` so the message includes source location.

---

## Roadmap

Potential next steps:

1. Add response builders for success payloads, not only error conversions.
2. Add optional RFC 7807/problem-details serialization.
3. Add i18n-aware message formatting hooks.
4. Provide a clearer distinction between client misuse and upstream service failure helpers.
