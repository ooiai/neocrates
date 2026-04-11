# Helper Module

The `helper` module is the always-available toolbox in Neocrates. It gathers reusable utilities for IDs, retries, pagination, serde normalization, config loading, text chunking, lightweight validation, and a few web-only Axum extractors.

See also: [root README](../../README.md)

---

## What this module contains

Top-level areas in `helper::core`:

- **IDs**: Snowflake, Sonyflake, and Crockford-style hashid encode/decode helpers
- **Request/data normalization**: serde deserialize/serialize helpers, page-size normalization, string/number coercion
- **Validation helpers**: mobile/landline/email checks and masking utilities
- **Retries**: reusable async retry helpers with exponential backoff
- **Config loading**: upward YAML file search based on `ENV`
- **Pagination**: `PageParams`, `PageResponse`, and offset/limit conversion
- **Text tooling**: chunk parsed text by length while preserving metadata
- **Web-only extras**: `LoggedJson<T>` and `DetailedJson<T>` Axum extractors

---

## Feature and compatibility notes

- `helper` itself is **always compiled**.
- `helper::core::axum_extractor` is only available with `web` or `full`.
- Most helpers are framework-agnostic and can be used without Axum.

---

## Quick start

```rust
use neocrates::helper::core::{
    page::to_offset_limit,
    retry::{RetryPolicy, retry_async},
    snowflake::generate_snowflake_id,
    utils::Utils,
};

async fn demo() -> neocrates::anyhow::Result<()> {
    let token = Utils::generate_token();
    let id = generate_snowflake_id();
    let (_current, _size, offset, limit) = to_offset_limit(2, 20);

    let policy = RetryPolicy::default();
    let value = retry_async(&policy, "demo", || async {
        Ok::<_, neocrates::anyhow::Error>(42)
    })
    .await?;

    println!("{token} {id} {offset} {limit} {value}");
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Load YAML config with automatic environment-aware lookup

`helper::core::loader::load_config()` searches upward from the current directory and checks files in this order:

1. `application.{ENV}.yml|yaml`
2. `config.{ENV}.yml|yaml`
3. `application.yml|yaml`
4. `config.yml|yaml`

Example:

```rust
use neocrates::helper::core::loader::load_config;
use neocrates::serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    server_port: u16,
    debug_mode: bool,
}

fn main() {
    std::env::set_var("ENV", "development");
    let config = load_config::<AppConfig>();
    println!("{config:?}");
}
```

## 2. Normalize IDs and pagination in request DTOs

`serde_helpers` lets you accept string-or-number inputs and normalize them at deserialize time.

```rust
use neocrates::helper::core::serde_helpers::{
    deserialize_option_i64,
    normalize_current,
    normalize_page_size,
};
use neocrates::serde::Deserialize;

#[derive(Debug, Deserialize)]
struct QueryDto {
    #[serde(default, deserialize_with = "normalize_current")]
    current: Option<i64>,
    #[serde(default, deserialize_with = "normalize_page_size")]
    size: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_option_i64")]
    user_id: Option<i64>,
}
```

## 3. Use retries around transient storage or network failures

```rust
use neocrates::helper::core::retry::{RetryPolicy, retry_async};

async fn fetch_with_retry() -> neocrates::anyhow::Result<Vec<u8>> {
    let policy = RetryPolicy::storage_io();

    retry_async(&policy, "download-avatar", || async {
        // Replace this with your real HTTP or storage operation.
        Ok::<_, neocrates::anyhow::Error>(b"ok".to_vec())
    })
    .await
}
```

## 4. Use the Axum JSON extractors when you want structured JSON parse errors

```rust
#[cfg(feature = "web")]
use neocrates::helper::core::axum_extractor::DetailedJson;
#[cfg(feature = "web")]
use neocrates::serde::Deserialize;

#[cfg(feature = "web")]
#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[cfg(feature = "web")]
async fn create_user(DetailedJson(payload): DetailedJson<CreateUser>) -> String {
    format!("created {}", payload.email)
}
```

See the runnable example: [`../../examples/axum_extractor_example.rs`](../../examples/axum_extractor_example.rs)

---

## Key points and gotchas

- `snowflake.rs` contains both a custom Snowflake generator and a Sonyflake wrapper.
- `hashid.rs` uses Crockford Base32-style encoding to present numeric IDs as compact strings.
- `Utils::is_cn_mobile()` and related helpers are pragmatic validations, not telecom-spec validators.
- `retry_async()` decides retryability from error-message text; use `retry_async_with()` when you need a custom predicate.
- `LoggedJson<T>` and `DetailedJson<T>` are helpful drop-in replacements for `axum::Json<T>` when you want structured parse failures.

---

## Roadmap

Possible future directions for `helper`:

1. Split large serde helpers into smaller, more focused submodules.
2. Add docs.rs examples for common DTO patterns.
3. Expose more framework-neutral utility traits instead of mostly static helpers.
4. Expand retry classification beyond string matching.
5. Add stronger typed helpers around loader search paths and config errors.
