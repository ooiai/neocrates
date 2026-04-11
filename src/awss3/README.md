# AWS S3-Compatible Client Module

The `awss3` module exposes the low-level `AwsClient`, a small wrapper over the AWS SDK S3 client that works with AWS S3 and other S3-compatible backends such as Aliyun OSS, MinIO, or RustFS.

See also: [root README](../../README.md), [high-level aws guide](../aws/README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["awss3"] }
```

You can also get it through the broader `aws` feature.

---

## What you get

- `AwsClient::new(...)`
- `AwsClient::new_with_options(...)`
- `put_object(...)`
- `get_object(...)`
- `get_presigned_url(...)`
- `get_presigned_put_url(...)`
- `head_object(...)`
- `delete_object(...)`
- `list_objects(...)`

---

## Quick start

```rust
use std::time::Duration;

use neocrates::awss3::aws::AwsClient;

async fn demo() -> neocrates::anyhow::Result<()> {
    let client = AwsClient::new(
        "my-bucket",
        "us-east-1",
        "https://s3.amazonaws.com",
        "access-key",
        "secret-key",
    )
    .await?;

    client.put_object("uploads/hello.txt", b"hello".to_vec()).await?;
    let url = client.get_presigned_url("uploads/hello.txt", Duration::from_secs(300)).await?;
    println!("{url}");
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Choose the endpoint and URL style

For AWS S3:

```rust
let client = AwsClient::new(
    "my-bucket",
    "us-east-1",
    "https://s3.amazonaws.com",
    "access-key",
    "secret-key",
)
.await?;
```

For MinIO or another backend that needs path-style URLs:

```rust
let client = AwsClient::new_with_options(
    "uploads",
    "us-east-1",
    "http://127.0.0.1:9000",
    "minioadmin",
    "minioadmin",
    true,
)
.await?;
```

## 2. Upload and download objects

```rust
client.put_object("avatars/u42.png", image_bytes).await?;
let bytes = client.get_object("avatars/u42.png").await?;
```

## 3. Use presigned URLs for browser or mobile uploads

```rust
use std::time::Duration;

let read_url = client
    .get_presigned_url("avatars/u42.png", Duration::from_secs(600))
    .await?;

let write_url = client
    .get_presigned_put_url("uploads/client.bin", Duration::from_secs(600))
    .await?;
```

## 4. Inspect or list objects

```rust
let meta = client.head_object("avatars/u42.png").await?;
let keys = client.list_objects(Some("avatars/")).await?;
println!("{:?} {}", meta.content_length(), keys.len());
```

---

## Key points and gotchas

- Credentials and endpoints are passed explicitly; the module does not read env vars on its own.
- `force_path_style` matters for S3-compatible backends that do not support virtual-host-style URLs.
- The API returns boxed errors, so downstream code usually wraps them into its own error surface.
- If you need a config-driven, singleton-style service layer, use the higher-level `aws` module instead.

---

## Roadmap

Potential next steps:

1. Add multipart upload helpers.
2. Add streaming upload/download ergonomics.
3. Add documented retry patterns for transient object-storage failures.
4. Add more examples for Aliyun OSS and MinIO compatibility.
