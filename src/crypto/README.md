# Crypto Module

The `crypto` module provides a compact set of cryptographic and adjacent helpers used by Neocrates. Its most important job today is password hashing and verification through Argon2.

See also: [root README](../../README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["crypto"] }
```

---

## What this module exposes

All public functions currently live on `crypto::core::Crypto`:

- `hash_password(...)`
- `verify_password(...)`
- `md5_string(...)`
- `generate_basic_auth_key(...)`
- `decode_basic_auth_key(...)`
- `zstd_compress(...)`
- `generate_aes_key(...)`

---

## Quick start

```rust
use neocrates::crypto::core::Crypto;

fn password_demo() -> Result<(), neocrates::argon2::password_hash::Error> {
    let hash = Crypto::hash_password("my-password-123")?;
    assert!(Crypto::verify_password("my-password-123", &hash));
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Hash passwords with Argon2

```rust
use neocrates::crypto::core::Crypto;

let hash = Crypto::hash_password("correct horse battery staple")?;
println!("{hash}");
```

This produces a PHC-format string with a generated salt. Store that string directly in your database.

## 2. Verify login attempts

```rust
let ok = Crypto::verify_password("correct horse battery staple", &hash);
assert!(ok);
```

If the stored hash string is malformed, verification simply returns `false`.

## 3. Use the legacy or utility helpers carefully

```rust
let digest = Crypto::md5_string("hello");
let encoded = Crypto::generate_basic_auth_key("user:password");
let decoded = Crypto::decode_basic_auth_key(&encoded)?;
let compressed = Crypto::zstd_compress(b"payload")?;
let key = Crypto::generate_aes_key();
println!("{digest} {decoded} {} {}", compressed.len(), key.len());
```

---

## Key points and gotchas

- For new password flows, prefer `hash_password()` and `verify_password()`.
- `md5_string()` is a legacy checksum-style helper, not a secure password or signature primitive.
- `generate_basic_auth_key()` uses a double-base64 scheme; that is a compatibility helper, not a standard HTTP Basic auth encoder.
- `generate_aes_key()` currently returns **32 hex characters** derived from 32 random bytes, so document and use it according to its actual output rather than assuming a full 64-hex-character key string.

---

## Roadmap

Potential next steps:

1. Add clearer key-size-specific generation helpers.
2. Add HMAC/HKDF-style wrappers when the project needs them.
3. Clarify naming for compatibility helpers vs security-recommended helpers.
4. Expand docs.rs examples around password migration and verification.
