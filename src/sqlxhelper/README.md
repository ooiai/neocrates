# SQLx Helper Module

`sqlxhelper` is the SQLx-oriented PostgreSQL integration layer in Neocrates. It provides a small wrapper around `sqlx::PgPool`, plus migration support and SQL logging macros.

See also: [root README](../../README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["sqlx"] }
```

---

## What this module exposes

### Pooling API

- `SqlxPool::new(url, max_connections)`
- `SqlxPool::from_env()`
- `SqlxPool::pool()`
- `SqlxPool::begin()`
- `SqlxPool::run_migrations(...)`
- `SqlxPool::size()`
- `SqlxPool::idle()`
- `SqlxPool::health_check()`

### Error type

- `SqlxError`
- `SqlxResult<T>`

### SQL logging helpers

From `sqlxhelper::logging`:

- `set_sql_logging(enabled)`
- `is_sql_logging_enabled()`
- `sqlx_fetch_all!(exec, query)`
- `sqlx_fetch_one!(exec, query)`
- `sqlx_fetch_optional!(exec, query)`
- `sqlx_execute!(exec, query)`
- `sqlx_fetch_scalar!(exec, query)`

---

## Quick start

```rust
use neocrates::sqlxhelper::pool::SqlxPool;

async fn connect() -> neocrates::anyhow::Result<()> {
    let pool = SqlxPool::from_env().await?;
    pool.health_check().await?;
    Ok(())
}
```

Required environment variable:

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost/app"
```

Optional:

```bash
export DATABASE_POOL_SIZE=10
```

---

## Step-by-step tutorial

## 1. Create the pool

```rust
let pool = neocrates::sqlxhelper::pool::SqlxPool::new(
    "postgres://postgres:postgres@localhost/neocrates_demo",
    10,
)
.await?;
```

Like `dieselhelper`, startup will:

1. ensure the target database exists
2. connect to the target database
3. run `SET TIME ZONE 'UTC'` through `after_connect`

## 2. Run typed queries

```rust
let row: (i64,) = sqlx::query_as("SELECT 1")
    .fetch_one(pool.pool())
    .await?;
println!("{row:?}");
```

`SqlxPool` also dereferences to `PgPool`, so you can pass `&*pool` in many places.

## 3. Add logging around queries

```rust
use neocrates::sqlxhelper::logging::set_sql_logging;

set_sql_logging(true);

let row: (i64,) = neocrates::sqlx_fetch_one!(
    pool.pool(),
    sqlx::query_as::<_, (i64,)>("SELECT 1")
)
.await?;
```

The macros log the SQL string and return the SQLx future. You still call `.await` at the call site.

## 4. Run migrations

```rust
use sqlx::migrate::Migrator;
use std::path::Path;

let migrator = Migrator::new(Path::new("./migrations")).await?;
pool.run_migrations(&migrator).await?;
```

---

## Key points and gotchas

- `from_env()` reads `DATABASE_URL` and optional `DATABASE_POOL_SIZE`.
- SQL logging is toggled programmatically with `set_sql_logging(...)`.
- Logging macros return futures, not final results, so they must be followed with `.await`.
- `run_migrations()` expects a caller-provided SQLx `Migrator`.

---

## Roadmap

Potential next steps:

1. Add a builder-style pool config instead of only constructor args/env loading.
2. Add timing/metrics around logged queries.
3. Add examples for transactions and migration bootstrapping.
4. Extend helper coverage if non-PostgreSQL backends are added later.
