use deadpool_diesel::postgres::{Manager, Pool, Runtime};
use diesel::connection::Connection as DieselConnection;
use diesel::{PgConnection, QueryableByName, RunQueryDsl, sql_query, sql_types::Text};
use thiserror::Error;
use tracing::{error, info};
use url::Url;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] deadpool_diesel::PoolError),

    #[error("Database query error: {0}")]
    QueryError(#[from] diesel::result::Error),

    #[error("Database URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Database name missing in URL")]
    DatabaseNameMissing,

    #[error("Database interaction error: {0}")]
    InteractionError(#[from] deadpool_diesel::InteractError),

    #[error("Database initialization error: {0}")]
    InitializationError(String),

    #[error(transparent)]
    UserError(#[from] anyhow::Error),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(QueryableByName)]
pub struct DbRow {
    #[diesel(sql_type = Text)]
    pub datname: String,
}

async fn ensure_database_exists(database_url: &str) -> DatabaseResult<()> {
    let parsed = Url::parse(database_url)?;
    let db_name = parsed
        .path_segments()
        .and_then(|segments| segments.filter(|s| !s.is_empty()).last())
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
        .ok_or(DatabaseError::DatabaseNameMissing)?;

    let mut default_url = parsed.clone();
    default_url.set_path("/postgres");
    let default_url_string = default_url.to_string();
    let sanitized_db_name = db_name.replace('"', "\"\"");

    tokio::task::spawn_blocking(move || -> DatabaseResult<()> {
        let mut conn = PgConnection::establish(&default_url_string).map_err(|e| {
            DatabaseError::InitializationError(format!("Failed to connect to default db: {}", e))
        })?;

        let exists = !sql_query("SELECT datname FROM pg_database WHERE datname = $1")
            .bind::<Text, _>(db_name.clone())
            .load::<DbRow>(&mut conn)
            .map_err(DatabaseError::QueryError)?
            .is_empty();

        if exists {
            info!("Database '{}' already exists", db_name);
            return Ok(());
        }

        let create_query = format!("CREATE DATABASE \"{}\"", sanitized_db_name);
        sql_query(create_query)
            .execute(&mut conn)
            .map_err(DatabaseError::QueryError)?;
        info!("Database '{}' created", db_name);

        Ok(())
    })
    .await
    .map_err(|e| {
        DatabaseError::InitializationError(format!("Failed to ensure database exists: {}", e))
    })?
}

#[derive(Clone)]
pub struct DieselPool {
    pool: Pool,
}

impl DieselPool {
    /// Create and initialize a new DieselPool.
    pub async fn new(url: impl Into<String>, max_size: usize) -> DatabaseResult<Self> {
        let url = url.into();
        ensure_database_exists(&url).await?;

        let manager = Manager::new(url.clone(), Runtime::Tokio1);
        let pool = Pool::builder(manager)
            .max_size(max_size)
            .build()
            .map_err(|e| {
                DatabaseError::InitializationError(format!("Failed to build pool: {}", e))
            })?;

        // Set the timezone to UTC for all connections and enable dev-only server-side SQL logging
        let conn = pool.get().await.map_err(DatabaseError::ConnectionError)?;
        // Log and set timezone to UTC
        info!("SQL: SET TIME ZONE 'UTC'");
        conn.interact(|conn| sql_query("SET TIME ZONE 'UTC'").execute(conn))
            .await
            .map_err(DatabaseError::InteractionError)?
            .map_err(|e| {
                DatabaseError::InitializationError(format!(
                    "Failed to execute timezone query: {}",
                    e
                ))
            })?;
        // In development builds, optionally enable PostgreSQL server-side SQL logging for this session
        if cfg!(debug_assertions) && std::env::var("PG_LOG_STATEMENT").map_or(true, |v| v != "0") {
            info!("SQL: SET log_statement = 'all'");
            conn.interact(|conn| sql_query("SET log_statement = 'all'").execute(conn))
                .await
                .map_err(DatabaseError::InteractionError)?
                .map_err(|e| {
                    DatabaseError::InitializationError(format!(
                        "Failed to enable server-side SQL logging: {}",
                        e
                    ))
                })?;
        }

        Ok(Self { pool })
    }

    /// Get the underlying Pool reference.
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Get a connection object from the pool.
    pub async fn connection(&self) -> DatabaseResult<deadpool::managed::Object<Manager>> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(DatabaseError::ConnectionError)?;

        // Ensure per-connection session settings:
        // 1) Set timezone to UTC
        info!("SQL: SET TIME ZONE 'UTC'");
        conn.interact(|conn| sql_query("SET TIME ZONE 'UTC'").execute(conn))
            .await
            .map_err(DatabaseError::InteractionError)?
            .map_err(|e| {
                DatabaseError::InitializationError(format!(
                    "Failed to execute timezone query: {}",
                    e
                ))
            })?;

        // 2) In development builds, optionally enable server-side SQL logging
        if cfg!(debug_assertions) && std::env::var("PG_LOG_STATEMENT").map_or(true, |v| v != "0") {
            info!("SQL: SET log_statement = 'all'");
            conn.interact(|conn| sql_query("SET log_statement = 'all'").execute(conn))
                .await
                .map_err(DatabaseError::InteractionError)?
                .map_err(|e| {
                    DatabaseError::InitializationError(format!(
                        "Failed to enable server-side SQL logging: {}",
                        e
                    ))
                })?;
        }

        Ok(conn)
    }

    /// Check the status of the database connection.
    pub fn status(&self) -> deadpool::Status {
        self.pool.status()
    }

    /// This function is used to perform a health check on the database connection.
    pub async fn health_check(&self) -> DatabaseResult<()> {
        let conn = self.connection().await?;
        conn.interact(|conn| {
            sql_query("SELECT 1")
                .execute(conn)
                .map(|_| ())
                .map_err(DatabaseError::from)
        })
        .await?
        .map_err(|e| {
            error!("Diesel health check query failed: {}", e);
            e
        })?;
        info!("Diesel health check executed: db connection test successful");
        Ok(())
    }

    /// The function is used to interact with the database
    pub async fn interact<F, T, E>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&mut PgConnection) -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: Send + 'static + Into<DatabaseError>,
    {
        let conn = self.connection().await?;
        conn.interact(f)
            .await
            .map_err(DatabaseError::InteractionError)?
            .map_err(Into::into)
    }

    /// The transaction handler
    pub async fn transaction<F, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&mut PgConnection) -> diesel::result::QueryResult<T> + Send + 'static,
        T: Send + 'static,
    {
        self.interact(|conn| DieselConnection::transaction(conn, f))
            .await
    }
}
