# Redis Cache Module

The `rediscache` module is the Redis integration layer for Neocrates. It wraps `bb8-redis` with a friendly async API for pool creation, common Redis operations, pattern deletion, pipelines, distributed locks, and a simple global singleton.

See also: [root README](../../README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["redis"] }
```

---

## What this module exposes

### Configuration and pool

- `RedisConfig`
- `RedisPool::new(config)`
- `RedisPool::from_env()`
- `RedisPool::get_connection()`
- `RedisPool::get_pool_status()`

### Common operations

- `set`, `setex`, `get`, `del`, `exists`, `expire`, `ttl`
- `pipeline(...)`
- `del_by_pattern(pattern)`
- `del_prefix(prefix)`

### Lock helpers

- `acquire_lock(...)`
- `release_lock(...)`
- `lock_key(namespace, resource)`
- `try_acquire_lock_with_retry(...)`
- `release_lock_if(...)`

### Global pool helpers

- `init_redis_pool(config)`
- `get_redis_pool()`

### Key builders

- `RedisUtils::cache_key(...)`
- `RedisUtils::user_token_key(...)`
- `RedisUtils::user_session_key(...)`
- `RedisUtils::rate_limit_key(...)`
- `RedisUtils::thread_lock_key(...)`

---

## Quick start

```rust
use neocrates::rediscache::RedisPool;

async fn demo() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = RedisPool::from_env().await?;
    pool.set("demo:key", "value").await?;
    let value: Option<String> = pool.get("demo:key").await?;
    println!("{value:?}");
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Start from environment variables

Supported variables:

- `REDIS_URL`
- `REDIS_MAX_SIZE`
- `REDIS_MIN_IDLE`
- `REDIS_CONNECTION_TIMEOUT`
- `REDIS_IDLE_TIMEOUT`
- `REDIS_MAX_LIFETIME`

Example:

```bash
export REDIS_URL="redis://127.0.0.1:6379"
export REDIS_MAX_SIZE=20
```

Then create the pool:

```rust
let pool = neocrates::rediscache::RedisPool::from_env().await?;
```

## 2. Read, write, and expire keys

```rust
pool.set("user:42", "neo").await?;
pool.setex("session:abc", "payload", 300).await?;

let user: Option<String> = pool.get("user:42").await?;
let ttl = pool.ttl("session:abc").await?;
println!("{user:?} {ttl}");
```

## 3. Delete a key range safely

```rust
let removed = pool.del_by_pattern("cache:*").await?;
println!("removed {removed} keys");

let removed2 = pool.del_prefix("tmp:").await?;
println!("removed {removed2} keys");
```

`del_by_pattern()` uses `SCAN` internally and attempts `UNLINK` before falling back to `DEL`.

## 4. Use a distributed lock

```rust
use std::time::Duration;

let key = neocrates::rediscache::RedisPool::lock_key("jobs", "daily-report");

if let Some(token) = pool.acquire_lock(&key, Duration::from_secs(10), None).await? {
    // critical section
    pool.release_lock(&key, &token).await?;
}
```

## 5. Initialize the global singleton if your app wants one shared pool

```rust
use neocrates::rediscache::{RedisConfig, init_redis_pool, get_redis_pool};

let config = RedisConfig::default();
init_redis_pool(config).await?;
let pool = get_redis_pool().unwrap();
```

---

## Key points and gotchas

- `RedisConfig::default()` panics if `REDIS_URL` is missing; `RedisPool::from_env()` is the safer option for most apps.
- The lock helpers use `SET NX PX` for acquisition and a Lua compare-and-delete script for release.
- `del_by_pattern()` is safer than `KEYS ...`-style deletion on large keyspaces, but it is still operational work you should use intentionally.
- The `redis` feature currently enables the `moka` dependency, but this module does not yet expose a high-level Moka wrapper.

---

## Roadmap

Useful next improvements:

1. Add typed JSON helpers for common cache value patterns.
2. Add a first-class Moka integration layer.
3. Add metrics/tracing around pool saturation and command timing.
4. Add namespace builders so callers do less manual key formatting.
