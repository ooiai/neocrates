# Middlewares Module

The `middlewares` module contains the request-interception layer for Neocrates-based Axum services. It handles token lookup, optional BASIC auth for PMS-style routes, request IP extraction, and request-body enrichment for common auditing fields.

See also: [root README](../../README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["web", "crypto"] }
```

If you want Redis-backed token storage, enable `redis` too.

Practical note:

- the current interceptor references `crypto::core::Crypto`, so `crypto` is part of the practical feature set for this module today

---

## Main building blocks

- `interceptor::interceptor` — the Axum middleware function
- `token_store::TokenStore` — pluggable storage abstraction
- `token_store::InMemoryTokenStore` — default local implementation
- `token_store::RedisTokenStore` — Redis-backed implementation when `redis` is enabled
- `models::AuthModel` and `models::AuthTokenResult` — shared auth DTOs
- `models::MiddlewareConfig` — runtime configuration for the middleware
- `ip::get_request_host` — extract client IP and URI details

---

## Quick start

```rust
use std::sync::Arc;

use neocrates::axum::{Router, middleware, routing::get};
use neocrates::middlewares::{
    interceptor::interceptor,
    models::MiddlewareConfig,
    token_store::default_in_memory_store,
};

async fn health() -> &'static str {
    "ok"
}

fn router() -> Router {
    let config = Arc::new(MiddlewareConfig {
        token_store: default_in_memory_store(),
        ignore_urls: vec!["/health".into()],
        pms_ignore_urls: vec![],
        prefix: "app:".into(),
        auth_basics: vec![],
    });

    Router::new()
        .route("/health", get(health))
        .layer(middleware::from_fn_with_state(config, interceptor))
}
```

---

## Step-by-step tutorial

## 1. Decide where tokens live

For local development or tests:

```rust
use neocrates::middlewares::token_store::default_in_memory_store;

let store = default_in_memory_store();
```

For Redis-backed storage:

```rust
#[cfg(feature = "redis")]
use std::sync::Arc;
#[cfg(feature = "redis")]
use neocrates::middlewares::token_store::redis_store;
#[cfg(feature = "redis")]
use neocrates::rediscache::RedisPool;

#[cfg(feature = "redis")]
async fn redis_store_example() -> neocrates::anyhow::Result<()> {
    let pool = Arc::new(RedisPool::from_env().await?);
    let store = redis_store(pool, "app:");
    let _ = store;
    Ok(())
}
```

## 2. Configure bypass routes

`MiddlewareConfig` supports two bypass lists:

- `ignore_urls`: skip auth entirely
- `pms_ignore_urls`: bypass token auth, but require a BASIC auth header that matches `auth_basics`

```rust
use neocrates::middlewares::models::MiddlewareConfig;

let cfg = MiddlewareConfig {
    token_store: neocrates::middlewares::token_store::default_in_memory_store(),
    ignore_urls: vec!["/health".into(), "/auth/login".into()],
    pms_ignore_urls: vec!["/admin/internal".into()],
    prefix: "app:".into(),
    auth_basics: vec!["<double-base64 user:pass>".into()],
};
```

## 3. Understand what the interceptor does

For non-bypassed routes it:

1. Reads the token from `Authorization: Bearer ...` or `?accessToken=...`
2. Loads `AuthModel` from the configured `TokenStore`
3. Inserts `AuthModel` into request extensions
4. If the body is JSON, injects audit fields:
   - POST: `creator`, `creator_by`, `updater`, `updater_by`
   - PUT: `updater`, `updater_by`

This makes it convenient to build auditing-aware CRUD APIs without repeating the same body transformation logic in every handler.

---

## Key points and gotchas

- URL matching is **prefix-based** (`starts_with`), not regex-based.
- The middleware reads and rewrites JSON request bodies; non-JSON bodies pass through unchanged.
- `MiddlewareConfig.prefix` exists, but the current interceptor implementation hardcodes an empty prefix internally.
- Token lookup prefers the Bearer header over the query parameter fallback.
- BASIC auth entries in `auth_basics` must already be encoded in the format expected by `Crypto::decode_basic_auth_key`.

---

## Roadmap

Useful next improvements:

1. Make `prefix` active in the interceptor instead of ignored.
2. Add cookie and header customization for token extraction.
3. Extend audit-field injection to PATCH or make it configurable.
4. Add first-class permission hooks instead of leaving PMS logic partially specialized.
