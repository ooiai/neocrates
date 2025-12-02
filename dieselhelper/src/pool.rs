use deadpool_diesel::postgres::{Manager, Pool, Runtime};
use diesel::connection::Connection as DieselConnection;
use diesel::{PgConnection, RunQueryDsl, sql_query};
use thiserror::Error;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] deadpool_diesel::PoolError),

    #[error("Database query error: {0}")]
    QueryError(#[from] diesel::result::Error),

    #[error("Database interaction error: {0}")]
    InteractionError(#[from] deadpool_diesel::InteractError),

    #[error("Database initialization error: {0}")]
    InitializationError(String),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Clone)]
pub struct DieselPool {
    pool: Pool,
}

impl DieselPool {
    /// Create and initialize a new DieselPool.
    pub async fn new(url: impl Into<String>, max_size: usize) -> DatabaseResult<Self> {
        let manager = Manager::new(url.into(), Runtime::Tokio1);
        let pool = Pool::builder(manager)
            .max_size(max_size)
            .build()
            .map_err(|e| {
                DatabaseError::InitializationError(format!("Failed to build pool: {}", e))
            })?;

        // Set the timezone to UTC for all connections
        let conn = pool.get().await.map_err(DatabaseError::ConnectionError)?;
        conn.interact(|conn| sql_query("SET TIME ZONE 'UTC'").execute(conn))
            .await
            .map_err(DatabaseError::InteractionError)?
            .map_err(|e| {
                DatabaseError::InitializationError(format!(
                    "Failed to execute timezone query: {}",
                    e
                ))
            })?;

        Ok(Self { pool })
    }

    /// Get the underlying Pool reference.
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Get a connection object from the pool.
    pub async fn connection(&self) -> DatabaseResult<deadpool::managed::Object<Manager>> {
        self.pool
            .get()
            .await
            .map_err(DatabaseError::ConnectionError)
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
        E: Send + 'static,
    {
        let conn = self.connection().await?;
        conn.interact(|conn| f(conn).map_err(|_| diesel::result::Error::RollbackTransaction))
            .await
            .map_err(DatabaseError::InteractionError)?
            .map_err(|_| DatabaseError::QueryError(diesel::result::Error::RollbackTransaction))
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
