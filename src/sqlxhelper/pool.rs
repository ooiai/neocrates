use sqlx::postgres::{PgPool, PgPoolOptions};
use thiserror::Error;
use tracing::{error, info};
use url::Url;

// ── Error & Result ────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SqlxError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Database URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Database name missing in URL")]
    DatabaseNameMissing,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type SqlxResult<T> = Result<T, SqlxError>;

// ── Internal helper ───────────────────────────────────────────────────────────

/// Connect to the system `postgres` database and create `db_name` if it does
/// not yet exist. Mirrors the behaviour of `dieselhelper::pool::ensure_database_exists`.
async fn ensure_database_exists(url: &str) -> SqlxResult<()> {
    let parsed = Url::parse(url)?;

    let db_name = parsed
        .path_segments()
        .and_then(|segs| segs.filter(|s| !s.is_empty()).last())
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
        .ok_or(SqlxError::DatabaseNameMissing)?;

    // Build a URL that points to the system database
    let mut system_url = parsed.clone();
    system_url.set_path("/postgres");

    let system_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(system_url.as_str())
        .await?;

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(&db_name)
            .fetch_one(&system_pool)
            .await?;

    if exists {
        info!("Database '{}' already exists", db_name);
    } else {
        let sanitized = db_name.replace('"', "\"\"");
        sqlx::query(&format!("CREATE DATABASE \"{}\"", sanitized))
            .execute(&system_pool)
            .await?;
        info!("Database '{}' created", db_name);
    }

    system_pool.close().await;
    Ok(())
}

// ── SqlxPool ──────────────────────────────────────────────────────────────────

/// Async PostgreSQL connection pool backed by [`sqlx::PgPool`].
///
/// # Quick start
/// ```rust,ignore
/// let pool = SqlxPool::from_env().await?;
/// let row: (i64,) = sqlx::query_as("SELECT 1").fetch_one(pool.pool()).await?;
/// ```
#[derive(Clone, Debug)]
pub struct SqlxPool {
    pool: PgPool,
}

impl SqlxPool {
    /// Create a new pool, auto-creating the target database if it does not exist.
    ///
    /// # Arguments
    /// * `url` – full PostgreSQL connection string (e.g. `postgres://user:pass@host/db`)
    /// * `max_connections` – maximum number of connections in the pool
    pub async fn new(url: &str, max_connections: u32) -> SqlxResult<Self> {
        ensure_database_exists(url).await?;

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("SET TIME ZONE 'UTC'")
                        .execute(&mut *conn)
                        .await
                        .map(|_| ())
                })
            })
            .connect(url)
            .await?;

        Ok(Self { pool })
    }

    /// Create a pool from environment variables.
    ///
    /// Reads `DATABASE_URL` (required) and `DATABASE_POOL_SIZE` (optional, default 10).
    pub async fn from_env() -> SqlxResult<Self> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| SqlxError::Config("DATABASE_URL env var not set".into()))?;
        let max_connections = std::env::var("DATABASE_POOL_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(10);
        Self::new(&url, max_connections).await
    }

    /// Returns a reference to the underlying [`PgPool`].
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Begin a new database transaction.
    ///
    /// The transaction is automatically rolled back when dropped if not committed.
    pub async fn begin(&self) -> SqlxResult<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.pool.begin().await.map_err(SqlxError::from)
    }

    /// Run all pending SQLx migrations from the given migrator.
    ///
    /// # Example
    /// ```rust,ignore
    /// use sqlx::migrate::Migrator;
    /// use std::path::Path;
    ///
    /// let migrator = Migrator::new(Path::new("./migrations")).await?;
    /// pool.run_migrations(&migrator).await?;
    /// ```
    pub async fn run_migrations(
        &self,
        migrator: &sqlx::migrate::Migrator,
    ) -> SqlxResult<()> {
        migrator.run(&self.pool).await.map_err(SqlxError::from)
    }

    /// Returns the current number of connections in the pool (idle + active).
    pub fn size(&self) -> u32 {
        self.pool.size()
    }

    /// Returns the number of idle connections.
    pub fn idle(&self) -> u32 {
        self.pool.num_idle() as u32
    }

    /// Perform a lightweight connectivity check (`SELECT 1`).
    pub async fn health_check(&self) -> SqlxResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!("SQLx health check failed: {}", e);
                SqlxError::from(e)
            })?;
        info!("SQLx health check passed: db connection test successful");
        Ok(())
    }
}

// Allow `SqlxPool` to be used directly wherever `&PgPool` is accepted.
impl std::ops::Deref for SqlxPool {
    type Target = PgPool;
    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
