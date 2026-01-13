# dieselhelper README

This tutorial shows how to print every Diesel SQL (with bound values) in development without changing your Postgres or docker-compose. It uses a small set of macros that log the query via `tracing` before executing it. You only need minimal call-site changes.

## Why this

- You want: every DB query prints the actual SQL with values for debugging.
- You don’t want: to touch Postgres config, docker-compose, or wire up multiple logging systems.
- Diesel internally uses prepared statements; “automatic” interception won’t print parameters reliably. The safest way is to log right before execution, which the macros do.

## Key features

- Dev-only logging by default. Production stays clean.
- Logs the real SQL with parameters using `diesel::debug_query::<Pg, _>`.
- Minimal changes: replace `.execute/.load/.first/.get_result` with macros.

## Macros

These macros are available in `neocrates::dieselhelper::logging` and exported for direct use:

- `diesel_execute!(conn, query)` → executes and logs SQL
- `diesel_load!(conn, query, T)` → loads rows and logs SQL
- `diesel_get_result!(conn, query, T)` → single row and logs SQL
- `diesel_get_results!(conn, query, T)` → multiple rows and logs SQL
- `diesel_first!(conn, query, T)` → first row and logs SQL
- `diesel_optional!(conn, query, T)` → optional row and logs SQL (`NotFound` → `Ok(None)`)
- `diesel_execute_sql!(conn, "SQL")` → logs and executes a raw SQL string (no binds)

All macros print the SQL only when logging is enabled (see “Enable/disable logging” below).

## Enable/disable logging

- Default:
  - Enabled in debug builds (`cfg!(debug_assertions)`), disabled in release.
- Environment override:
  - `SQL_LOG=1` → force enable
  - `SQL_LOG=0` → force disable

Examples:

- macOS/Linux: `SQL_LOG=1 cargo run`
- Windows PowerShell: `$env:SQL_LOG=1; cargo run`

## Usage examples

You can invoke macros by path (`neocrates::diesel_execute!`) or `use` them. With the 2018+ edition, path invocation works without `use`.

### Load rows

Before:

```rust
let rows = users.filter(active.eq(true)).load::<User>(conn)?;
```

After (minimal):

```rust
let rows = neocrates::diesel_load!(conn, users.filter(active.eq(true)), User)?;
```

### Single result

Before:

```rust
let user = users.filter(id.eq(user_id)).get_result::<User>(conn)?;
```

After:

```rust
let user = neocrates::diesel_get_result!(conn, users.filter(id.eq(user_id)), User)?;
```

Or using `first`:

```rust
let user = neocrates::diesel_first!(conn, users.filter(id.eq(user_id)), User)?;
```

### Optional single result

Before:

```rust
let maybe = users.filter(id.eq(user_id)).first::<User>(conn).optional()?;
```

After:

```rust
let maybe = neocrates::diesel_optional!(conn, users.filter(id.eq(user_id)), User)?;
```

### Insert / Update / Delete

Before:

```rust
let affected = diesel::insert_into(users).values(name.eq("alice")).execute(conn)?;
```

After:

```rust
let affected = neocrates::diesel_execute!(conn, diesel::insert_into(users).values(name.eq("alice")))?;
```

### Raw SQL (no binds)

```rust
neocrates::diesel_execute_sql!(conn, "SET TIME ZONE 'UTC'")?;
```

### In transactions (deadpool_diesel)

Works the same inside `pool.transaction(|conn| { ... })`:

```rust
pool.transaction(|conn| {
    neocrates::diesel_first!(conn, users.find(42), User)
}).await?;
```

## Minimal migration guide

Search/replace patterns (safe and incremental):

- `.execute(conn)?` → `diesel_execute!(conn, <expr>)?`
- `.load::<T>(conn)?` → `diesel_load!(conn, <expr>, T)?`
- `.get_result::<T>(conn)?` → `diesel_get_result!(conn, <expr>, T)?`
- `.first::<T>(conn)?` → `diesel_first!(conn, <expr>, T)?`
- `.get_results::<T>(conn)?` → `diesel_get_results!(conn, <expr>, T)?`
- `.first::<T>(conn).optional()?` → `diesel_optional!(conn, <expr>, T)?`

Do this for the hotspots first (e.g., repo/service layers). You don’t need to touch schema or model definitions.

## Logging output

- The macros use `tracing::info!` to print.
- Ensure you initialize your logger (e.g., via your existing `tracing_subscriber` setup).
- Control verbosity (e.g., `RUST_LOG=info` or `RUST_LOG=debug`) as you normally do.

Example log line (dev):

```
SELECT "id", "name" FROM "users" WHERE "id" = 42
```

This is produced by Diesel’s `debug_query`, with parameters inlined for Postgres.

## Notes and caveats

- Diesel DSL queries render well with `debug_query`. For `sql_query("...").bind(...)`, Diesel may not inline bound values in all cases; if you rely heavily on raw SQL with binds, consider logging the SQL string plus the bind values yourself for those spots.
- The macros add negligible overhead, and only log in dev unless you force-enable via `SQL_LOG=1`.
- No Postgres or container changes are needed. Everything stays at the application layer.

## Example: repository method

Original:

```rust
async fn find_by_id(&self, id: i64) -> AppResult<Option<RoleAssignment>> {
    let pool = self.pool.clone();

    pool.transaction(move |conn| {
        role_assignments::table
            .find(id)
            .first::<RoleAssignmentRecord>(conn)
            .optional()
            .map(|opt| opt.map(RoleAssignment::from))
    })
    .await
    .map_err(|e| {
        tracing::error!("Failed to find role assignment by id {}: {}", id, e);
        AppError::DbError(e.to_string())
    })
}
```

Minimal change:

```rust
async fn find_by_id(&self, id: i64) -> AppResult<Option<RoleAssignment>> {
    let pool = self.pool.clone();

    pool.transaction(move |conn| {
        neocrates::diesel_optional!(conn, role_assignments::table.find(id), RoleAssignmentRecord)
            .map(|opt| opt.map(RoleAssignment::from))
    })
    .await
    .map_err(|e| {
        tracing::error!("Failed to find role assignment by id {}: {}", id, e);
        AppError::DbError(e.to_string())
    })
}
```

## Troubleshooting

- “I don’t see SQL logs.”
  - Confirm you’re in a debug build or set `SQL_LOG=1`.
  - Ensure your `tracing_subscriber` is initialized and `RUST_LOG` allows `info`.
- “I need SQL in production logs.”
  - Set `SQL_LOG=1` in production, but be aware of verbosity and potential log volume.

## Summary

These macros give you deterministic, dev-friendly SQL logging with minimal code changes and no infrastructure tweaks. Adopt them incrementally in your data access paths, and you’ll get reliable, parameter-inclusive SQL logs exactly where you need them.
