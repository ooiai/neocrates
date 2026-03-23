// SQLx SQL logging helpers — zero call-site overhead.
//
// `log_query` is annotated with `#[track_caller]`.  Because the macros below
// expand at the user's call site, `Location::caller()` automatically resolves
// to the file/line/column where the macro was written — no extra parameters.
//
// Example output:
//   INFO sql: [src/services/user.rs:42:5]
//     sql | SELECT users.id FROM users WHERE users.id = $1
//
// How to use:
//
//   // fetch all rows (typed)
//   let users: Vec<User> = sqlx_fetch_all!(
//       pool.pool(),
//       sqlx::query_as::<_, User>("SELECT * FROM users")
//   ).await?;
//
//   // fetch one row with bind
//   let user: User = sqlx_fetch_one!(
//       pool.pool(),
//       sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(42i64)
//   ).await?;
//
//   // optional (returns Option<T>)
//   let maybe: Option<User> = sqlx_fetch_optional!(
//       pool.pool(),
//       sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(42i64)
//   ).await?;
//
//   // execute (INSERT / UPDATE / DELETE)
//   let result = sqlx_execute!(
//       pool.pool(),
//       sqlx::query("DELETE FROM users WHERE id = $1").bind(42i64)
//   ).await?;
//
//   // scalar value
//   let count: i64 = sqlx_fetch_scalar!(
//       pool.pool(),
//       sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
//   ).await?;
//
//   // inside a transaction
//   let mut tx = pool.begin().await?;
//   let user: User = sqlx_fetch_one!(
//       &mut *tx,
//       sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(1i64)
//   ).await?;
//   tx.commit().await?;
//
// Logging can be enabled / disabled at runtime:
//   SQL_LOG override → call `set_sql_logging(true/false)` during app init.
//   Default          → enabled only in debug builds.

use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

static SQL_LOG_OVERRIDE: OnceCell<AtomicBool> = OnceCell::new();

/// Programmatic override for SQL logging.  Call early during app init.
pub fn set_sql_logging(enabled: bool) {
    SQL_LOG_OVERRIDE
        .get_or_init(|| AtomicBool::new(enabled))
        .store(enabled, Ordering::Relaxed);
}

/// Returns `true` if SQL logging is currently enabled.
/// Prefers a programmatic override; falls back to `cfg!(debug_assertions)`.
#[inline]
pub fn is_sql_logging_enabled() -> bool {
    if let Some(flag) = SQL_LOG_OVERRIDE.get() {
        return flag.load(Ordering::Relaxed);
    }
    cfg!(debug_assertions)
}

/// Log a SQL string together with its caller location.
///
/// `#[track_caller]` resolves the location to the direct call site.  When
/// called from inside a macro the location is the line where the macro was
/// written — exactly what you want for query tracing.
#[track_caller]
pub fn log_query(sql: &str) {
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

// ── Macros ────────────────────────────────────────────────────────────────────
//
// All macros follow the same pattern:
//   1. Bind the query expression to a local so it is only evaluated once.
//   2. Log the SQL via `log_query` (location is the macro call site).
//   3. Return the future — the caller must `.await` it.
//
// `$exec` can be any sqlx `Executor`:
//   • `pool.pool()` or `&*pool`  (SqlxPool / &PgPool)
//   • `&mut tx`                  (inside a transaction)

/// Fetch all matching rows.
///
/// Returns `impl Future<Output = Result<Vec<Row>>>`.  Append `.await?` at the call site.
///
/// ```rust,ignore
/// let users: Vec<User> = sqlx_fetch_all!(
///     pool.pool(),
///     sqlx::query_as::<_, User>("SELECT * FROM users")
/// ).await?;
/// ```
#[macro_export]
macro_rules! sqlx_fetch_all {
    ($exec:expr, $q:expr) => {{
        let __q = $q;
        $crate::sqlxhelper::logging::log_query(__q.sql());
        __q.fetch_all($exec)
    }};
}

/// Fetch exactly one row (returns an error if zero or more than one row found).
///
/// ```rust,ignore
/// let user: User = sqlx_fetch_one!(
///     pool.pool(),
///     sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(1i64)
/// ).await?;
/// ```
#[macro_export]
macro_rules! sqlx_fetch_one {
    ($exec:expr, $q:expr) => {{
        let __q = $q;
        $crate::sqlxhelper::logging::log_query(__q.sql());
        __q.fetch_one($exec)
    }};
}

/// Fetch zero or one row (`None` if not found, error if more than one).
///
/// ```rust,ignore
/// let maybe: Option<User> = sqlx_fetch_optional!(
///     pool.pool(),
///     sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1").bind(42i64)
/// ).await?;
/// ```
#[macro_export]
macro_rules! sqlx_fetch_optional {
    ($exec:expr, $q:expr) => {{
        let __q = $q;
        $crate::sqlxhelper::logging::log_query(__q.sql());
        __q.fetch_optional($exec)
    }};
}

/// Execute a statement (INSERT / UPDATE / DELETE) and return the query result.
///
/// ```rust,ignore
/// let result = sqlx_execute!(
///     pool.pool(),
///     sqlx::query("DELETE FROM users WHERE id = $1").bind(42i64)
/// ).await?;
/// println!("rows affected: {}", result.rows_affected());
/// ```
#[macro_export]
macro_rules! sqlx_execute {
    ($exec:expr, $q:expr) => {{
        let __q = $q;
        $crate::sqlxhelper::logging::log_query(__q.sql());
        __q.execute($exec)
    }};
}

/// Fetch a single scalar value.
///
/// ```rust,ignore
/// let count: i64 = sqlx_fetch_scalar!(
///     pool.pool(),
///     sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
/// ).await?;
/// ```
#[macro_export]
macro_rules! sqlx_fetch_scalar {
    ($exec:expr, $q:expr) => {{
        let __q = $q;
        $crate::sqlxhelper::logging::log_query(__q.sql());
        __q.fetch_one($exec)
    }};
}

// ── Tests ─────────────────────────────────────────────────────────────────────

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
        assert!(is_sql_logging_enabled(), "should be enabled");

        set_sql_logging(false);
        assert!(!is_sql_logging_enabled(), "should be disabled");
    }

    #[test]
    fn test_log_query_output_contains_location() {
        let _guard = lock_log_state();
        set_sql_logging(true);

        let (sub, buf) = make_capture_subscriber();
        tracing::subscriber::with_default(sub, || {
            log_query("SELECT 99"); // ← this line number appears in output
        });

        let output = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
        assert!(
            output.contains("logging.rs"),
            "output should contain the source file; got: {output}"
        );
        assert!(
            output.contains("SELECT 99"),
            "output should contain the SQL; got: {output}"
        );

        set_sql_logging(false);
    }

    #[test]
    fn test_log_query_silent_when_disabled() {
        let _guard = lock_log_state();
        set_sql_logging(false);

        let (sub, buf) = make_capture_subscriber();
        tracing::subscriber::with_default(sub, || {
            log_query("SELECT secret");
        });

        let output = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
        assert!(
            output.is_empty(),
            "should produce no output when disabled; got: {output}"
        );
    }
}
