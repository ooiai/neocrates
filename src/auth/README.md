# Auth Module

The `auth` module provides Redis-backed helpers for issuing, rotating, storing, and deleting token pairs, plus a small fingerprint-binding API for associating devices or client fingerprints with users.

See also: [root README](../../README.md)

---

## Feature and runtime requirements

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["web", "auth", "redis"] }
```

Important note:

- The current implementation is **Redis-backed in practice**.
- It also returns `AppError`/`AppResult`, so use it together with `web`.
- Treat `auth` as a **`web + auth + redis`** feature set, even though the Cargo feature table does not encode that coupling perfectly yet.

---

## Main types

- `AuthHelper` — static helper API
- `middlewares::models::AuthModel` — user/session identity payload
- `middlewares::models::AuthTokenResult` — access/refresh token pair and TTL data

Redis key families used by the module:

- `{prefix}:auth:uid:{uid}` → current `AuthTokenResult`
- `{prefix}:auth:token:{access_token}` → serialized `AuthModel`
- `{prefix}:auth:refresh_token:{refresh_token}` → serialized `AuthModel`
- `{prefix}:auth:fp:uid:{fingerprint}` → user ID
- `{prefix}:auth:uid:fp:{uid}` → fingerprint

---

## Quick start

```rust
use std::sync::Arc;

use neocrates::auth::auth_helper::AuthHelper;
use neocrates::middlewares::models::AuthModel;
use neocrates::rediscache::RedisPool;

async fn login() -> neocrates::anyhow::Result<()> {
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
    println!("{:?}", tokens.access_token);
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Issue tokens during login

```rust
let tokens = AuthHelper::generate_auth_token(
    &redis_pool,
    "app:",
    300,        // access-token TTL in seconds
    86_400,     // refresh-token TTL in seconds
    auth_model,
)
.await?;
```

This helper:

1. deletes any existing token pair for the user
2. generates a new access token and refresh token
3. stores the `AuthModel` and `AuthTokenResult` into Redis

## 2. Refresh a token pair

```rust
let rotated = AuthHelper::refresh_auth(
    &redis_pool,
    "app:",
    300,
    86_400,
    &old_access_token,
    &refresh_token,
)
.await?;
```

This verifies the old pair against Redis and rotates to a new pair.

## 3. Bind a fingerprint to a user

```rust
AuthHelper::bind_fingerprint(&redis_pool, "app:", 42, "browser-fingerprint").await?;
let uid = AuthHelper::get_uid_by_fingerprint(&redis_pool, "app:", "browser-fingerprint").await?;
println!("{uid:?}");
```

## 4. Logout and revoke tokens

```rust
AuthHelper::delete_token(&redis_pool, "app:", 42).await?;
```

---

## Key points and gotchas

- `generate_auth_token()` invalidates the previous session for the same user before issuing a new one.
- Prefix handling is entirely caller-driven; use a stable namespace such as `"app:"` or `"tenant-a:"`.
- `bind_fingerprint()` rejects empty fingerprint strings.
- The module manages token state, but it does **not** perform HTTP request parsing or middleware injection; that belongs to `middlewares`.

---

## Roadmap

Useful next improvements:

1. Support multiple active sessions per user.
2. Add a first-class config type for TTLs and prefixes.
3. Add cookie/session helpers for web apps.
4. Improve compile-time feature wiring so `auth` explicitly pulls in the backend it depends on.
