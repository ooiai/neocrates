# AGENTS.md

This document provides guidelines for AI agents (GitHub Copilot, Claude, Codex, etc.) working in the **Neocrates** codebase.

---

## Project Overview

**Neocrates** is a comprehensive Rust utility library (crate) that acts as a facade over multiple internal modules. It targets web development, AWS integrations, database operations, Redis caching, cryptography, SMS, and auth scenarios — all gated behind Cargo feature flags.

- **Language**: Rust (edition 2024, MSRV 1.84+)
- **Async Runtime**: Tokio
- **Repository**: https://github.com/ooiai/neocrates
- **Crates.io**: https://crates.io/crates/neocrates

---

## Repository Layout

```
src/
├── lib.rs              # Crate root: feature-gated re-exports and module declarations
├── helper/             # Core utilities (always compiled)
├── logger/             # Tracing-based logger          [feature: logger]
├── middlewares/        # Axum middleware, interceptors  [feature: web]
├── response/           # Unified response/error types  [feature: web]
├── aws/                # Shared AWS service helpers    [feature: aws | awssts]
├── awss3/              # S3 client                     [feature: awss3 | aws]
├── awssts/             # STS clients (Aliyun, Tencent) [feature: awssts | aws]
├── dieselhelper/       # Diesel + connection pooling   [feature: diesel]
├── rediscache/         # Redis pool + Moka cache       [feature: redis]
├── crypto/             # Argon2, HMAC, SHA2, Ring      [feature: crypto]
├── sms/                # SMS (Aliyun, Tencent)         [feature: sms]
├── captcha/            # CAPTCHA service               [feature: captcha]
├── auth/               # Auth helpers / JWT            [feature: auth | redis]
└── sqlx/               # SQLx helpers (experimental)
```

---

## Feature Flags

Every module is behind a Cargo feature. Always use the narrowest feature set needed.

| Feature   | Modules enabled                        |
|-----------|----------------------------------------|
| `web`     | `middlewares`, `response`              |
| `aws`     | `aws`, `awss3`, `awssts`               |
| `awss3`   | `awss3`                                |
| `awssts`  | `awssts`, `aws`                        |
| `diesel`  | `dieselhelper`                         |
| `redis`   | `rediscache`                           |
| `crypto`  | `crypto`                               |
| `sms`     | `sms`                                  |
| `logger`  | `logger`                               |
| `captcha` | `captcha`                              |
| `auth`    | `auth`                                 |
| `full`    | all of the above                       |

When adding new functionality:
- Gate it behind an appropriate feature flag in `Cargo.toml` and `lib.rs`.
- Mark new crate dependencies as `optional = true`.

---

## Build, Test & Lint Commands

```bash
# Build (default features)
cargo build -p neocrates

# Build with all features
cargo build -p neocrates --features full

# Run tests
cargo test -p neocrates

# Lint (no warnings allowed)
cargo clippy -p neocrates -- -D warnings

# Format check
cargo fmt --check

# Dry-run publish
cargo publish -p neocrates --dry-run --registry crates-io

# Makefile shortcuts
make build           # cargo build
make build-full      # all features
make test            # cargo test
make lint            # clippy -D warnings
make fmt             # cargo fmt
make doc             # generate docs
make dry-run         # test publish
make publish m="release: v0.1.x"
```

---

## Coding Conventions

1. **Async-first**: All I/O-heavy code must be `async`. Use `tokio` primitives.
2. **Error handling**: Use `anyhow::Result` for application code and `thiserror` for library error types.
3. **Feature guards**: Every module that uses an optional dependency must be wrapped in `#[cfg(feature = "...")]`.
4. **No panics**: Avoid `unwrap()` / `expect()` in library code; propagate errors with `?`.
5. **Builder pattern**: Use the [`bon`](https://crates.io/crates/bon) crate for struct builders.
6. **IDs**: Use `uuid` (v4) or `sonyflake`/`snowflaker` for distributed IDs.
7. **Serde**: All public data types exposed over APIs should derive `Serialize, Deserialize`.
8. **Validation**: Use the `validator` crate for request/input validation structs.
9. **Formatting**: Run `cargo fmt` before every commit. The project follows standard Rust style.
10. **Clippy**: All code must pass `cargo clippy -- -D warnings`.

---

## Module-Specific Notes

### `middlewares`
- Contains Axum layers: request tracing, CORS, IP extraction, token-based auth interceptor.
- `TokenStore` is pluggable: in-memory by default, Redis-backed when `redis` feature is active.
- Do not add hard dependencies on Redis inside `middlewares`; use trait objects.

### `awss3` / `awssts`
- Clients wrap the official AWS SDK.
- Aliyun and Tencent STS clients live in `awssts/aliyun.rs` and `awssts/tencent.rs`.
- Credentials must **never** be hardcoded; always read from environment variables or config.

### `dieselhelper`
- Uses `deadpool-diesel` for async connection pooling with PostgreSQL.
- Migrations are managed via `diesel_migrations`; never modify schema directly.

### `rediscache`
- `RedisPool` wraps `bb8-redis`.
- `Moka` is used for in-process caching (LRU/TTL).
- Read connection config from `REDIS_URL` env var by default.

### `crypto`
- Password hashing: Argon2.
- HMAC/Signatures: `hmac` + `sha2`.
- Low-level crypto: `ring`.
- Never roll custom crypto primitives.

### `auth`
- Provides JWT helpers and session management.
- Depends on `redis` feature for token persistence.

---

## Environment Variables

| Variable              | Used by        | Description                          |
|-----------------------|----------------|--------------------------------------|
| `DATABASE_URL`        | `dieselhelper` | PostgreSQL connection string         |
| `DATABASE_POOL_SIZE`  | `dieselhelper` | Connection pool size                 |
| `REDIS_URL`           | `rediscache`   | Redis connection URL                 |
| `REDIS_POOL_SIZE`     | `rediscache`   | Redis pool size                      |
| `RUST_LOG`            | `logger`       | Log level filter (default: `info`)   |

---

## Security Rules

- **No secrets in source code** — credentials belong in environment variables or secret managers.
- **Input validation** — validate all external inputs with the `validator` crate before processing.
- **Least privilege** — AWS IAM roles and DB users should have minimal permissions.
- **Dependency hygiene** — run `cargo audit` periodically; address HIGH/CRITICAL advisories promptly.

---

## CI / CD

CI runs on every push and PR to `main` via GitHub Actions (`.github/workflows/rust.yml`):

1. `cargo build --verbose`
2. `cargo test --verbose`

When adding new features, ensure they compile and test cleanly under CI.

---

## Adding a New Module

1. Create `src/<module>/mod.rs` (and sub-files as needed).
2. Add the feature flag in `Cargo.toml` `[features]` section.
3. Mark new crate deps as `optional = true` in `[dependencies]`.
4. Add the `#[cfg(feature = "...")]` pub mod declaration in `src/lib.rs`.
5. Add re-exports for any external crate your module depends on in `lib.rs`.
6. Update `README.md` feature table.
7. Update this `AGENTS.md` module layout table.
