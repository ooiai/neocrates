#![allow(unused_imports)]
// Dev-only Diesel SQL logging helpers with minimal call-site changes.
//
// `log_query` and `log_sql_str` are annotated with `#[track_caller]`.
// Because the macros expand at the user's call site, `Location::caller()`
// automatically resolves to the file/line/column where the macro was written —
// no extra parameters needed.
//
// Example output:
//   INFO sql: [src/services/user.rs:42:5]
//     sql | SELECT users.id FROM users WHERE users.id = $1 -- binds: [1]
//
// How to use:
// - Before:
//     let affected = diesel::update(users).set(name.eq("alice")).execute(conn)?;
//   After:
//     let affected = diesel_execute!(conn, diesel::update(users).set(name.eq("alice")))?;
//
// - Logging is enabled only in debug builds by default.
//     SQL_LOG=1  -> force enable
//     SQL_LOG=0  -> force disable

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
    cfg!(debug_assertions)
}

/// Log a Diesel query (with bound values).
///
/// `#[track_caller]` automatically captures the file/line/column of the
/// call site — no extra parameters required. When called from a macro the
/// location resolves to where the macro was written.
#[track_caller]
pub fn log_query<Q>(q: &Q)
where
    Q: QueryFragment<Pg>,
{
    if is_sql_logging_enabled() {
        let loc = std::panic::Location::caller();
        info!(
            target: "sql",
            "[{}:{}:{}]\n  sql | {}",
            loc.file(),
            loc.line(),
            loc.column(),
            debug_query::<Pg, _>(q),
        );
    }
}

/// Log a raw SQL string (for simple `sql_query` without binds).
///
/// `#[track_caller]` automatically captures the file/line/column of the
/// call site — no extra parameters required.
#[track_caller]
pub fn log_sql_str(sql: &str) {
    if is_sql_logging_enabled() {
        let loc = std::panic::Location::caller();
        info!(
            target: "sql",
            "[{}:{}:{}]\n  sql | {}",
            loc.file(),
            loc.line(),
            loc.column(),
            sql,
        );
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
        $crate::dieselhelper::logging::log_query(&__diesel_q);
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
        $crate::dieselhelper::logging::log_query(&__diesel_q);
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
        $crate::dieselhelper::logging::log_query(&__diesel_q);
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
        $crate::dieselhelper::logging::log_query(&__diesel_q);
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
        $crate::dieselhelper::logging::log_query(&__diesel_q);
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
        $crate::dieselhelper::logging::log_query(&__diesel_q);
        __diesel_q.get_result::<$ty>($conn).optional()
    }};
}

/// Convenience macro to execute a raw SQL string with caller location logging.
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

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::io;
    use std::sync::{Arc, Mutex, MutexGuard};

    // Serialize tests that mutate the process-global SQL_LOG_OVERRIDE flag.
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
    fn test_log_sql_str_output_contains_location() {
        let _guard = lock_log_state();
        set_sql_logging(true);

        let (sub, buf) = make_capture_subscriber();
        tracing::subscriber::with_default(sub, || {
            log_sql_str("SELECT 42"); // <-- this line number appears in output
        });

        let output = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
        // file path of THIS test file must appear in the log
        assert!(
            output.contains("logging.rs"),
            "output should contain the source file name; got: {output}"
        );
        assert!(
            output.contains("SELECT 42"),
            "output should contain the SQL; got: {output}"
        );

        set_sql_logging(false);
    }

    #[test]
    fn test_log_sql_str_silent_when_disabled() {
        let _guard = lock_log_state();
        set_sql_logging(false);

        let (sub, buf) = make_capture_subscriber();
        tracing::subscriber::with_default(sub, || {
            log_sql_str("SELECT secret");
        });

        let output = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
        assert!(
            output.is_empty(),
            "should produce no output when logging is disabled; got: {output}"
        );
    }
}
