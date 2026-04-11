# AWS Service Module

The `aws` module is the higher-level storage and STS service layer in Neocrates. It sits above the low-level `awss3` and `awssts` clients and gives you config-driven helpers for:

- object upload/download
- presigned GET/PUT URLs
- config normalization for Aliyun, RustFS, and MinIO
- STS-style output through `CosService`

See also: [root README](../../README.md), [awss3 guide](../awss3/README.md), [awssts guide](../awssts/README.md)

---

## Feature and practical requirements

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["aws"] }
```

Practical notes:

- `AwsService` uses `response::error::{AppError, AppResult}` in the current implementation.
- `CosService` uses `RedisPool` for cached Aliyun STS results.
- In practice, apps using the high-level service layer usually work with `aws`, and often also with `web` and `redis` depending on which code path they need.

---

## Main building blocks

- `AwsConfig` — config carrier for Aliyun, RustFS, and MinIO fields
- `OssConfig` — normalized single-provider object-storage config
- `AwsService`
  - `init_from_env_config(...)`
  - `download_object(...)`
  - `put_object(...)`
  - `get_signed_url(...)`
  - `get_signed_put_url(...)`
  - `download_object_via_signed_url(...)`
  - `put_object_via_signed_url(...)`
- `CosService`
  - `get_cos_sts(...)`
  - `get_aliyun_sts(...)`
  - `get_rustfs_sts(...)`
  - `get_minio_sts(...)`

---

## Quick start

```rust
use std::sync::Arc;

use neocrates::aws::sts_service::AwsConfig;
use neocrates::aws::aws_service::AwsService;

fn init_storage() {
    let cfg = Arc::new(AwsConfig {
        cos_type: "minio".into(),
        aliyun_accesskey_id: "".into(),
        aliyun_accesskey_secret: "".into(),
        aliyun_role_arn: "".into(),
        aliyun_expiration: 3600,
        aliyun_role_session_name: "".into(),
        aliyun_endpoint: "".into(),
        aliyun_region_id: "".into(),
        aliyun_bucket: "".into(),
        rustfs_accesskey_id: "".into(),
        rustfs_accesskey_secret: "".into(),
        rustfs_endpoint: "".into(),
        rustfs_region_id: "".into(),
        rustfs_bucket: "".into(),
        rustfs_expiration: 3600,
        minio_accesskey_id: "minioadmin".into(),
        minio_accesskey_secret: "minioadmin".into(),
        minio_endpoint: "http://127.0.0.1:9000".into(),
        minio_region_id: "us-east-1".into(),
        minio_bucket: "uploads".into(),
        minio_expiration: 3600,
    });

    AwsService::init_from_env_config(&cfg);
}
```

---

## Step-by-step tutorial

## 1. Build an `AwsConfig`

`AwsConfig` is intentionally explicit. You fill the provider-specific fields you need and select the active provider through `cos_type`.

Supported `cos_type` branches:

- `aliyun`
- `rustfs`
- `minio`

## 2. Initialize the singleton-style storage config

```rust
AwsService::init_from_env_config(&aws_config);
```

This stores the normalized `OssConfig` in a `OnceCell`. All later `AwsService` calls read from that global config.

## 3. Use the storage helper

```rust
AwsService::put_object("docs/report.pdf", bytes).await?;
let body = AwsService::download_object("docs/report.pdf").await?;
let read_url = AwsService::get_signed_url("docs/report.pdf", 600).await?;
let write_url = AwsService::get_signed_put_url("uploads/client.bin", 600).await?;
```

## 4. Use the STS helper

For Aliyun, `CosService` will fetch and cache temporary credentials in Redis.

```rust
use std::sync::Arc;

use neocrates::aws::sts_service::CosService;
use neocrates::rediscache::RedisPool;

async fn fetch_cos_sts(config: Arc<AwsConfig>) -> neocrates::anyhow::Result<()> {
    let redis = Arc::new(RedisPool::from_env().await?);
    let sts = CosService::get_cos_sts(&config, &redis, 42).await?;
    println!("{}", sts.access_key_id);
    Ok(())
}
```

For `rustfs` and `minio`, `CosService` returns static config-driven credentials without a security token.

---

## Key points and gotchas

- `AwsService` must be initialized before use; otherwise it will panic when reading the `OnceCell`.
- `OssConfig::from_env_config(...)` panics on unsupported `cos_type`.
- `AwsService` redacts presigned URL query strings in its error messages.
- `CosService::get_aliyun_sts(...)` caches credentials in Redis; RustFS and MinIO branches do not perform a real STS call.
- If you only need a low-level client, prefer `awss3::aws::AwsClient`.

---

## Roadmap

Useful next improvements:

1. Replace the singleton-style config with an owned service object.
2. Add typed builders and validation for `AwsConfig`.
3. Add retry helpers around upload/download flows.
4. Clarify and tighten the practical feature dependencies for high-level AWS helpers.
