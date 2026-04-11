# Diesel Helper Module

`dieselhelper` is the Diesel-oriented PostgreSQL integration layer in Neocrates. It combines:

- async connection pooling via `deadpool-diesel`
- automatic database creation when the target DB is missing
- UTC timezone initialization for new connections
- query logging macros for Diesel DSL calls

See also: [root README](../../README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["diesel"] }
```

---

## What this module exposes

### Pooling API

- `DieselPool::new(url, max_size)`
- `DieselPool::pool()`
- `DieselPool::connection()`
- `DieselPool::status()`
- `DieselPool::health_check()`
- `DieselPool::interact(...)`
- `DieselPool::transaction(...)`
- `DieselPool::run(...)`

### Error type

- `DatabaseError`
- `DatabaseResult<T>`

### SQL logging helpers

From `dieselhelper::logging`:

- `set_sql_logging(enabled)`
- `is_sql_logging_enabled()`
- `diesel_execute!(conn, query)`
- `diesel_load!(conn, query, T)`
- `diesel_get_result!(conn, query, T)`
- `diesel_get_results!(conn, query, T)`
- `diesel_first!(conn, query, T)`
- `diesel_optional!(conn, query, T)`
- `diesel_execute_sql!(conn, "SQL")`

---

## Quick start

```rust
use neocrates::dieselhelper::pool::DieselPool;

async fn connect() -> neocrates::anyhow::Result<()> {
    let pool = DieselPool::new("postgres://postgres:postgres@localhost/app", 10).await?;
    pool.health_check().await?;
    Ok(())
}
```

---

## Step-by-step tutorial

## 1. Create the pool

```rust
use neocrates::dieselhelper::pool::DieselPool;

let pool = DieselPool::new(
    "postgres://postgres:postgres@localhost/neocrates_demo",
    10,
)
.await?;
```

What happens during startup:

1. the URL is parsed
2. the target database name is extracted
3. the helper connects to the system `postgres` database
4. it creates the target database if it does not exist yet
5. it builds the deadpool-diesel pool
6. it runs `SET TIME ZONE 'UTC'` on the first connection

## 2. Run a transaction

```rust
pool.transaction(|conn| {
    // Place normal Diesel DSL code here.
    // Example:
    // diesel::insert_into(users::table).values(&new_user).execute(conn)?;
    Ok::<_, diesel::result::Error>(())
})
.await?;
```

## 3. Turn on SQL logging

You can enable Diesel SQL logging programmatically:

```rust
use neocrates::dieselhelper::logging::set_sql_logging;

set_sql_logging(true);
```

If you also use the `logger` module, `logger::LogSettings { sql_log: Some(true), .. }` can toggle the same flag.

## 4. Replace query calls with logging macros

```rust
let rows = neocrates::diesel_load!(conn, users::table.limit(10), User)?;
let one = neocrates::diesel_first!(conn, users::table.find(42), User)?;
let maybe = neocrates::diesel_optional!(conn, users::table.find(42), User)?;
let affected = neocrates::diesel_execute!(conn, diesel::delete(users::table.find(42)))?;
```

The macros log the SQL at the macro call site and then execute the query.

---

## Key points and gotchas

- SQL logging is **debug-build or programmatic-toggle behavior**, not an env-var-driven feature in the current code.
- `debug_query` works best with Diesel DSL queries; raw SQL with binds may need additional manual logging.
- `DieselPool::new(...)` takes an explicit pool size; unlike `sqlxhelper`, this module does not provide a `from_env()` helper today.
- Database creation and timezone initialization are built into startup behavior, so document that when wiring it into an app.

---

## Roadmap

Useful next improvements:

1. Add a `from_env()` constructor mirroring `sqlxhelper`.
2. Add migration helpers or guide integration around `diesel_migrations`.
3. Add pool metrics and optional query timing.
4. Expand docs.rs examples for fully typed Diesel usage.
