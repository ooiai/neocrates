#![allow(unused_imports)]
// Dev-only Diesel SQL logging helpers with minimal call-site changes
//
// Goal:
// - Print the actual SQL (with bound values) every time a Diesel query is executed,
//   without changing Postgres config or docker-compose.
// - Keep call-site changes minimal: replace `.execute` / `.load::<T>` / `.get_result::<T>`
//   with small macros.
//
// How to use (minimal changes at call sites):
// - Before:
//     let affected = diesel::update(users).set(name.eq("alice")).execute(conn)?;
//   After:
//     let affected = diesel_execute!(conn, diesel::update(users).set(name.eq("alice")))?;
//
// - Before:
//     let rows = users.filter(id.eq(1)).load::<User>(conn)?;
//   After:
//     let rows = diesel_load!(conn, users.filter(id.eq(1)), User)?;
//
// - Before:
//     let user = users.filter(id.eq(1)).first::<User>(conn)?;
//   After:
//     let user = diesel_first!(conn, users.filter(id.eq(1)), User)?;
//
// - For native SQL without binds:
//     diesel_execute_sql!(conn, "SET TIME ZONE 'UTC'")?;
//
// Notes:
// - Logging is enabled only in debug builds by default. You can override with env:
//     SQL_LOG=1  -> force enable
//     SQL_LOG=0  -> force disable
// - For Diesel DSL and most `sql_query(...).bind(...)`, we use `diesel::debug_query::<Pg, _>`
//   which renders SQL with values inlined for Postgres.

use diesel::{debug_query, pg::Pg, query_builder::QueryFragment};
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

static SQL_LOG_OVERRIDE: OnceCell<AtomicBool> = OnceCell::new();

/// Programmatic override for SQL logging. Call early during app init.
pub fn set_sql_logging(enabled: bool) {
    SQL_LOG_OVERRIDE
        .get_or_init(|| AtomicBool::new(enabled))
        .store(enabled, Ordering::Relaxed);
}

/// Returns true if SQL logging should be enabled for this build/environment.
/// Prefers programmatic override; falls back to debug builds.
#[inline]
pub fn is_sql_logging_enabled() -> bool {
    if let Some(flag) = SQL_LOG_OVERRIDE.get() {
        return flag.load(Ordering::Relaxed);
    }
    // Default: enabled only in debug builds
    cfg!(debug_assertions)
}

/// Log a Diesel query (with bound values) if logging is enabled.
#[inline]
pub fn log_query<Q>(q: &Q)
where
    Q: QueryFragment<Pg>,
{
    if is_sql_logging_enabled() {
        info!("{}", debug_query::<Pg, _>(q));
    }
}

/// Convenience: log a raw SQL string (for simple `sql_query` without binds).
#[inline]
pub fn log_sql_str(sql: &str) {
    if is_sql_logging_enabled() {
        info!("SQL: {}", sql);
    }
}

/// Execute a Diesel query and log its SQL (with bound values) in dev builds.
///
/// Usage:
///   let n = diesel_execute!(conn, diesel::update(users).set(name.eq("alice")))?;
#[macro_export]
macro_rules! diesel_execute {
    ($conn:expr, $q:expr) => {{
        $crate::dieselhelper::logging::log_query(&$q);
        $q.execute($conn)
    }};
}

/// Load rows for a Diesel query and log its SQL in dev builds.
///
/// Usage:
///   let rows = diesel_load!(conn, users.filter(id.eq(1)), User)?;
#[macro_export]
macro_rules! diesel_load {
    ($conn:expr, $q:expr, $ty:ty) => {{
        $crate::dieselhelper::logging::log_query(&$q);
        $q.load::<$ty>($conn)
    }};
}

/// Get a single result for a Diesel query and log its SQL in dev builds.
///
/// Usage:
///   let user = diesel_get_result!(conn, users.filter(id.eq(1)), User)?;
#[macro_export]
macro_rules! diesel_get_result {
    ($conn:expr, $q:expr, $ty:ty) => {{
        $crate::dieselhelper::logging::log_query(&$q);
        $q.get_result::<$ty>($conn)
    }};
}

/// Get multiple results for a Diesel query and log its SQL in dev builds.
///
/// Usage:
///   let users = diesel_get_results!(conn, users.filter(active.eq(true)), User)?;
#[macro_export]
macro_rules! diesel_get_results {
    ($conn:expr, $q:expr, $ty:ty) => {{
        $crate::dieselhelper::logging::log_query(&$q);
        $q.get_results::<$ty>($conn)
    }};
}

/// First row helper that logs SQL and returns the first row.
///
/// Usage:
///   let user = diesel_first!(conn, users.order(id.asc()), User)?;
#[macro_export]
macro_rules! diesel_first {
    ($conn:expr, $q:expr, $ty:ty) => {{
        $crate::dieselhelper::logging::log_query(&$q);
        $q.first::<$ty>($conn)
    }};
}

/// Optional helper: run a query, log SQL, and convert NotFound into Ok(None).
///
/// Usage:
///   let maybe_user = diesel_optional!(conn, users.filter(id.eq(1)), User)?;
#[macro_export]
macro_rules! diesel_optional {
    ($conn:expr, $q:expr, $ty:ty) => {{
        use diesel::OptionalExtension;
        $crate::dieselhelper::logging::log_query(&$q);
        $q.get_result::<$ty>($conn).optional()
    }};
}

/// Convenience macro to execute a raw SQL string with logging.
/// Good for statements without binds (e.g. SET TIME ZONE).
///
/// Usage:
///   diesel_execute_sql!(conn, "SET TIME ZONE 'UTC'")?;
#[macro_export]
macro_rules! diesel_execute_sql {
    ($conn:expr, $sql:expr) => {{
        $crate::dieselhelper::logging::log_sql_str($sql);
        diesel::sql_query($sql).execute($conn)
    }};
}
