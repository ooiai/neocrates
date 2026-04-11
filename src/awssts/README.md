# AWS STS Providers Module

The `awssts` module contains the low-level temporary-credential clients for Aliyun and Tencent Cloud. It is the provider-facing STS layer beneath the higher-level `aws::sts_service::CosService`.

See also: [root README](../../README.md), [high-level aws guide](../aws/README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["awssts"] }
```

You also get this module through the broader `aws` feature.

---

## What this module exposes

### Aliyun client

- `aliyun::StsClient::new(access_key_id, access_key_secret, role_arn, session_name)`
- `aliyun::StsClient::assume_role(duration_seconds)`
- response types such as `Credentials`, `AssumedRoleUser`, and `Response`

### Tencent client

- `tencent::StsClient::new(secret_id, secret_key, region)`
- `tencent::StsClient::get_temp_credentials(name, policy, duration_seconds)`
- response types such as `StsCredential`, `StsResponse`, and `Credentials`

---

## Quick start

### Aliyun

```rust
use neocrates::awssts::aliyun::StsClient;

async fn aliyun_demo() -> neocrates::anyhow::Result<()> {
    let client = StsClient::new(
        "access-key-id",
        "access-key-secret",
        "acs:ram::123456789012:role/demo",
        "demo-session",
    );

    let resp = client.assume_role(3600).await?;
    println!("{}", resp.credentials.access_key_id);
    Ok(())
}
```

### Tencent

```rust
use neocrates::awssts::tencent::StsClient;

async fn tencent_demo() -> neocrates::anyhow::Result<()> {
    let client = StsClient::new("secret-id", "secret-key", "ap-guangzhou");
    let creds = client.get_temp_credentials("demo-session", None, Some(3600)).await?;
    println!("{}", creds.tmp_secret_id);
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Use Aliyun `AssumeRole`

The Aliyun implementation:

1. builds the canonical query string
2. signs it with HMAC-SHA1
3. sends a GET request to `https://sts.aliyuncs.com/`
4. parses the JSON response into the public response structs

Example:

```rust
let client = neocrates::awssts::aliyun::StsClient::new(
    ak,
    sk,
    role_arn,
    "neocrates-session",
);

let resp = client.assume_role(1800).await?;
println!("{}", resp.credentials.expiration);
```

## 2. Use Tencent `GetFederationToken`

The Tencent implementation:

1. builds a signed POST request using the TC3-HMAC-SHA256 flow
2. optionally includes a policy JSON document
3. converts the returned expiration timestamp into `DateTime<Utc>`

Example:

```rust
let client = neocrates::awssts::tencent::StsClient::new(secret_id, secret_key, "ap-beijing");
let creds = client
    .get_temp_credentials(
        "frontend-upload",
        Some(r#"{"version":"2.0","statement":[]}"#),
        Some(7200),
    )
    .await?;

println!("{}", creds.expiration);
```

## 3. Use the higher-level wrapper when you want caching and provider dispatch

If you want a provider-dispatching helper and Redis caching for Aliyun credentials, move up to `aws::sts_service::CosService`.

---

## Key points and gotchas

- Aliyun and Tencent use completely different request-signing schemes and response shapes.
- The current Aliyun client builds a reqwest client with `danger_accept_invalid_certs(true)`, which is a security-sensitive implementation detail you should review before production use.
- The module is low-level: it gives you credentials and provider responses, not a full policy-management or upload-flow abstraction.

---

## Roadmap

Useful next improvements:

1. Make HTTP client configuration injectable.
2. Remove or gate insecure certificate-acceptance behavior in the Aliyun client.
3. Add typed policy builders instead of raw JSON strings.
4. Add more docs.rs examples for end-to-end browser/mobile upload flows.
