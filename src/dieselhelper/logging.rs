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
//
// Log output format (macros capture and print caller location automatically):
//
//   INFO sql: [my_crate::services::user  src/services/user.rs:42]
//     expr | users.filter(id.eq(1))
//     sql  | SELECT users.id FROM users WHERE users.id = $1 -- binds: [1]

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

/// Log a Diesel query (with bound values) and full caller location.
///
/// Prefer using the macros (`diesel_execute!`, `diesel_load!`, etc.) so that
/// `file`, `line`, `module_path`, and `query_expr` are captured automatically
/// at the macro call site.
#[inline]
pub fn log_query_at<Q>(
    q: &Q,
    file: &'static str,
    line: u32,
    module_path: &'static str,
    query_expr: &'static str,
) where
    Q: QueryFragment<Pg>,
{
    if is_sql_logging_enabled() {
        info!(
            target: "sql",
            "[{}  {}:{}]\n  expr | {}\n  sql  | {}",
            module_path,
            file,
            line,
            query_expr,
            debug_query::<Pg, _>(q),
        );
    }
}

/// Log a raw SQL string and full caller location.
///
/// Prefer using `diesel_execute_sql!` so that location is captured automatically.
#[inline]
pub fn log_sql_str_at(sql: &str, file: &'static str, line: u32, module_path: &'static str) {
    if is_sql_logging_enabled() {
        info!(
            target: "sql",
            "[{}  {}:{}]\n  sql  | {}",
            module_path,
            file,
            line,
            sql,
        );
    }
}

/// Log a Diesel query (with bound values) if logging is enabled.
/// Does **not** include call-site location; use the macros instead.
#[inline]
pub fn log_query<Q>(q: &Q)
where
    Q: QueryFragment<Pg>,
{
    if is_sql_logging_enabled() {
        info!(target: "sql", "sql | {}", debug_query::<Pg, _>(q));
    }
}

/// Convenience: log a raw SQL string (for simple `sql_query` without binds).
/// Does **not** include call-site location; use `diesel_execute_sql!` instead.
#[inline]
pub fn log_sql_str(sql: &str) {
    if is_sql_logging_enabled() {
        info!(target: "sql", "sql | {}", sql);
    }
}

/// Execute a Diesel query and log its SQL with caller location in dev builds.
///
/// Usage:
///   let n = diesel_execute!(conn, diesel::update(users).set(name.eq("alice")))?;
#[macro_export]
macro_rules! diesel_execute {
    ($conn:expr, $q:expr) => {{
        let __diesel_q = $q;
        $crate::dieselhelper::logging::log_query_at(
            &__diesel_q,
            file!(),
            line!(),
            module_path!(),
            stringify!($q),
        );
        __diesel_q.execute($conn)
    }};
}

/// Load rows for a Diesel query and log its SQL with caller location in dev builds.
///
/// Usage:
///   let rows = diesel_load!(conn, users.filter(id.eq(1)), User)?;
#[macro_export]
macro_rules! diesel_load {
    ($conn:expr, $q:expr, $ty:ty) => {{
        let __diesel_q = $q;
        $crate::dieselhelper::logging::log_query_at(
            &__diesel_q,
            file!(),
            line!(),
            module_path!(),
            stringify!($q),
        );
        __diesel_q.load::<$ty>($conn)
    }};
}

/// Get a single result for a Diesel query and log its SQL with caller location in dev builds.
///
/// Usage:
///   let user = diesel_get_result!(conn, users.filter(id.eq(1)), User)?;
#[macro_export]
macro_rules! diesel_get_result {
    ($conn:expr, $q:expr, $ty:ty) => {{
        let __diesel_q = $q;
        $crate::dieselhelper::logging::log_query_at(
            &__diesel_q,
            file!(),
            line!(),
            module_path!(),
            stringify!($q),
        );
        __diesel_q.get_result::<$ty>($conn)
    }};
}

/// Get multiple results for a Diesel query and log its SQL with caller location in dev builds.
///
/// Usage:
///   let users = diesel_get_results!(conn, users.filter(active.eq(true)), User)?;
#[macro_export]
macro_rules! diesel_get_results {
    ($conn:expr, $q:expr, $ty:ty) => {{
        let __diesel_q = $q;
        $crate::dieselhelper::logging::log_query_at(
            &__diesel_q,
            file!(),
            line!(),
            module_path!(),
            stringify!($q),
        );
        __diesel_q.get_results::<$ty>($conn)
    }};
}

/// First row helper that logs SQL with caller location and returns the first row.
///
/// Usage:
///   let user = diesel_first!(conn, users.order(id.asc()), User)?;
#[macro_export]
macro_rules! diesel_first {
    ($conn:expr, $q:expr, $ty:ty) => {{
        let __diesel_q = $q;
        $crate::dieselhelper::logging::log_query_at(
            &__diesel_q,
            file!(),
            line!(),
            module_path!(),
            stringify!($q),
        );
        __diesel_q.first::<$ty>($conn)
    }};
}

/// Optional helper: run a query, log SQL with caller location, and convert NotFound into Ok(None).
///
/// Usage:
///   let maybe_user = diesel_optional!(conn, users.filter(id.eq(1)), User)?;
#[macro_export]
macro_rules! diesel_optional {
    ($conn:expr, $q:expr, $ty:ty) => {{
        use diesel::OptionalExtension;
        let __diesel_q = $q;
        $crate::dieselhelper::logging::log_query_at(
            &__diesel_q,
            file!(),
            line!(),
            module_path!(),
            stringify!($q),
        );
        __diesel_q.get_result::<$ty>($conn).optional()
    }};
}

/// Convenience macro to execute a raw SQL string with caller location logging.
/// Good for statements without binds (e.g. SET TIME ZONE).
///
/// Usage:
///   diesel_execute_sql!(conn, "SET TIME ZONE 'UTC'")?;
#[macro_export]
macro_rules! diesel_execute_sql {
    ($conn:expr, $sql:expr) => {{
        $crate::dieselhelper::logging::log_sql_str_at($sql, file!(), line!(), module_path!());
        diesel::sql_query($sql).execute($conn)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::io;
    use std::sync::{Arc, Mutex, MutexGuard};

    // Serialize all tests that mutate the process-global SQL_LOG_OVERRIDE flag
    // so they don't race with each other.
    static LOG_STATE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn lock_log_state() -> MutexGuard<'static, ()> {
        LOG_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    // ── tracing output capture ────────────────────────────────────────────────

    struct BufWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct MakeBufWriter(Arc<Mutex<Vec<u8>>>);

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MakeBufWriter {
        type Writer = BufWriter;
        fn make_writer(&'a self) -> BufWriter {
            BufWriter(Arc::clone(&self.0))
        }
    }

    fn make_capture_subscriber() -> (impl tracing::Subscriber, Arc<Mutex<Vec<u8>>>) {
        let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_writer(MakeBufWriter(Arc::clone(&buf)))
            .with_ansi(false)
            .finish();
        (subscriber, buf)
    }

    // ── tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_sql_logging_override() {
        let _guard = lock_log_state();

        set_sql_logging(true);
        assert!(is_sql_logging_enabled(), "should be enabled after set_sql_logging(true)");

        set_sql_logging(false);
        assert!(!is_sql_logging_enabled(), "should be disabled after set_sql_logging(false)");
    }

    #[test]
    fn test_log_sql_str_at_output_contains_location() {
        let _guard = lock_log_state();
        set_sql_logging(true);

        let (sub, buf) = make_capture_subscriber();
        tracing::subscriber::with_default(sub, || {
            log_sql_str_at("SELECT 42", "src/services/user.rs", 99, "myapp::services::user");
        });

        let output = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
        assert!(
            output.contains("src/services/user.rs"),
            "output should contain file path; got: {output}"
        );
        assert!(
            output.contains("99"),
            "output should contain line number; got: {output}"
        );
        assert!(
            output.contains("myapp::services::user"),
            "output should contain module path; got: {output}"
        );
        assert!(
            output.contains("SELECT 42"),
            "output should contain the SQL; got: {output}"
        );

        set_sql_logging(false);
    }

    #[test]
    fn test_log_sql_str_at_silent_when_disabled() {
        let _guard = lock_log_state();
        set_sql_logging(false);

        let (sub, buf) = make_capture_subscriber();
        tracing::subscriber::with_default(sub, || {
            log_sql_str_at("SELECT secret", "src/test.rs", 1, "myapp");
        });

        let output = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
        assert!(
            output.is_empty(),
            "should produce no output when logging is disabled; got: {output}"
        );
    }

    #[test]
    fn test_log_sql_str_legacy_does_not_panic() {
        let _guard = lock_log_state();
        set_sql_logging(true);
        let (sub, buf) = make_capture_subscriber();
        tracing::subscriber::with_default(sub, || {
            log_sql_str("SELECT 1");
        });
        let output = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
        assert!(output.contains("SELECT 1"), "legacy log_sql_str should still emit output");
        set_sql_logging(false);
    }
}
