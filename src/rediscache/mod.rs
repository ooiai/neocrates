use bb8::Pool;
use bb8_redis::{RedisConnectionManager, bb8::RunError};
use redis::{AsyncCommands, RedisError, Script};
use std::{env, sync::Arc};
use tokio::sync::OnceCell;
use tracing::info;

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub max_size: u32,
    pub min_idle: Option<u32>,
    pub connection_timeout: std::time::Duration,
    pub idle_timeout: Option<std::time::Duration>,
    pub max_lifetime: Option<std::time::Duration>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        let url = env::var("REDIS_URL")
            .unwrap_or_else(|_| panic!("REDIS_URL environment variable must be set!"));
        Self {
            url,
            max_size: 10,
            min_idle: Some(1),
            connection_timeout: std::time::Duration::from_secs(5),
            idle_timeout: Some(std::time::Duration::from_secs(600)),
            max_lifetime: Some(std::time::Duration::from_secs(3600)),
        }
    }
}

#[derive(Clone)]
pub struct RedisPool {
    pool: Arc<Pool<RedisConnectionManager>>,
    max_size: u32,
}

impl RedisPool {
    pub async fn new(
        config: RedisConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let manager = RedisConnectionManager::new(config.url.clone())?;

        let pool = Pool::builder()
            .max_size(config.max_size)
            .min_idle(config.min_idle)
            .connection_timeout(config.connection_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .build(manager)
            .await?;
        {
            let mut conn = pool.get().await?;
            let _: String = conn.ping().await?;
        }

        info!(
            "Redis connection pool initialized successfully with {} max connections",
            config.max_size
        );

        Ok(Self {
            pool: Arc::new(pool),
            max_size: config.max_size,
        })
    }

    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = RedisConfig {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            max_size: std::env::var("REDIS_MAX_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            min_idle: std::env::var("REDIS_MIN_IDLE")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Some)
                .unwrap_or(Some(1)),
            connection_timeout: std::time::Duration::from_secs(
                std::env::var("REDIS_CONNECTION_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5),
            ),
            idle_timeout: Some(std::time::Duration::from_secs(
                std::env::var("REDIS_IDLE_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(600),
            )),
            max_lifetime: Some(std::time::Duration::from_secs(
                std::env::var("REDIS_MAX_LIFETIME")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3600),
            )),
        };

        Self::new(config).await
    }

    pub async fn get_connection(
        &self,
    ) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>, RunError<RedisError>> {
        self.pool.get().await
    }

    pub async fn set<K, V>(
        &self,
        key: K,
        value: V,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        K: redis::ToRedisArgs + Send + Sync,
        V: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let _: () = conn.set(&key, &value).await?;
        Ok(())
    }

    pub async fn setex<K, V>(
        &self,
        key: K,
        value: V,
        seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        K: redis::ToRedisArgs + Send + Sync,
        V: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let _: () = redis::cmd("SETEX")
            .arg(&key)
            .arg(seconds)
            .arg(&value)
            .query_async(&mut *conn)
            .await?;
        Ok(())
    }

    pub async fn get<K, V>(
        &self,
        key: K,
    ) -> Result<Option<V>, Box<dyn std::error::Error + Send + Sync>>
    where
        K: redis::ToRedisArgs + Send + Sync,
        V: redis::FromRedisValue,
    {
        let mut conn = self.get_connection().await?;
        let result: Option<V> = conn.get(&key).await?;
        Ok(result)
    }

    pub async fn del<K>(&self, key: K) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
    where
        K: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result: i32 = conn.del(&key).await?;
        Ok(result > 0)
    }

    pub async fn exists<K>(&self, key: K) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
    where
        K: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result: bool = conn.exists(&key).await?;
        Ok(result)
    }

    pub async fn expire<K>(
        &self,
        key: K,
        seconds: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
    where
        K: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result: bool = conn.expire(&key, seconds as i64).await?;
        Ok(result)
    }

    pub async fn ttl<K>(&self, key: K) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>
    where
        K: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let result: i64 = conn.ttl(&key).await?;
        Ok(result)
    }

    pub fn get_pool_status(&self) -> PoolStatus {
        let state = self.pool.state();
        PoolStatus {
            connections: state.connections,
            idle_connections: state.idle_connections,
            max_size: self.max_size,
        }
    }

    pub async fn pipeline<T>(
        &self,
        build: impl FnOnce(&mut redis::Pipeline) + Send,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: redis::FromRedisValue,
    {
        let mut conn = self.get_connection().await?;
        let mut pipe = redis::Pipeline::new();
        build(&mut pipe);
        let result = pipe.query_async(&mut *conn).await?;
        Ok(result)
    }

    pub async fn del_by_pattern(
        &self,
        pattern: &str,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_connection().await?;

        // Parameters that can be adjusted: the number of items SCAN tries to return per batch, and the number of keys to submit per batch when deleting
        const SCAN_COUNT: usize = 5000;
        const DELETE_BATCH_SIZE: usize = 1024;

        let mut cursor: u64 = 0;
        let mut total_deleted: u64 = 0;

        // Cache within a single call: if the first UNLINK fails, subsequent calls will use DEL directly to avoid additional overhead from repeated errors
        let mut unlink_supported = true;

        loop {
            // SCAN cursor MATCH pattern COUNT N
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(SCAN_COUNT)
                .query_async(&mut *conn)
                .await?;

            if !keys.is_empty() {
                // Batch delete in chunks to avoid overly long single commands
                for chunk in keys.chunks(DELETE_BATCH_SIZE) {
                    if unlink_supported {
                        let unlink_res: Result<i64, redis::RedisError> = redis::cmd("UNLINK")
                            .arg(chunk)
                            .query_async(&mut *conn)
                            .await;

                        match unlink_res {
                            Ok(n) => {
                                total_deleted += n as u64;
                                continue;
                            }
                            Err(_) => {
                                // Current Redis does not support UNLINK or it is disabled, use DEL directly for subsequent calls
                                unlink_supported = false;
                            }
                        }
                    }

                    // Backup plan: use DEL if UNLINK is not supported
                    let n: i64 = redis::cmd("DEL").arg(chunk).query_async(&mut *conn).await?;
                    total_deleted += n as u64;
                }
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(total_deleted)
    }

    // Added: Delete by prefix (equivalent to pattern = "prefix*")
    pub async fn del_prefix(
        &self,
        prefix: &str,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let pattern = format!("{}*", prefix);
        self.del_by_pattern(&pattern).await
    }

    /// Acquire a distributed lock using SET NX PX. Returns Some(token) if acquired, None otherwise.
    pub async fn acquire_lock(
        &self,
        key: &str,
        ttl: std::time::Duration,
        token: Option<&str>,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_connection().await?;
        let lock_value = match token {
            Some(t) => t.to_string(),
            None => {
                let pid = std::process::id();
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let current_thread = std::thread::current();
                let thread_name = current_thread.name().unwrap_or("thread");
                format!("{}:{}:{}", pid, thread_name, ts)
            }
        };
        let ttl_ms = ttl.as_millis() as u64;
        let mut cmd = redis::cmd("SET");
        cmd.arg(key)
            .arg(lock_value.clone())
            .arg("NX")
            .arg("PX")
            .arg(ttl_ms);
        let res: Option<String> = cmd.query_async(&mut *conn).await?;
        if res.is_some() {
            Ok(Some(lock_value))
        } else {
            Ok(None)
        }
    }
}

impl RedisPool {
    /// Release a distributed lock via Lua script (only if token matches).
    pub async fn release_lock(
        &self,
        key: &str,
        token: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.get_connection().await?;
        let script = Script::new(
            r#"if redis.call("GET", KEYS[1]) == ARGV[1] then
    return redis.call("DEL", KEYS[1])
else
    return 0
end"#,
        );
        let deleted: i32 = script.key(key).arg(token).invoke_async(&mut *conn).await?;
        Ok(deleted > 0)
    }

    /// Build a namespaced lock key.
    pub fn lock_key(namespace: &str, resource: &str) -> String {
        format!("lock:{}:{}", namespace, resource)
    }

    /// Try to acquire a lock with retry and backoff. Returns Some(token) on success.
    pub async fn try_acquire_lock_with_retry(
        &self,
        key: &str,
        ttl: std::time::Duration,
        retries: u32,
        backoff: std::time::Duration,
        token: Option<&str>,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        for _ in 0..retries {
            match self.acquire_lock(key, ttl, token).await? {
                Some(t) => return Ok(Some(t)),
                None => tokio::time::sleep(backoff).await,
            }
        }
        Ok(None)
    }

    /// Release a lock only if token is Some.
    pub async fn release_lock_if(&self, key: &str, token: Option<&str>) {
        if let Some(t) = token {
            let _ = self.release_lock(key, t).await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub connections: u32,
    pub idle_connections: u32,
    pub max_size: u32,
}

static REDIS_POOL: OnceCell<RedisPool> = OnceCell::const_new();

pub async fn init_redis_pool(
    config: RedisConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = RedisPool::new(config).await?;
    REDIS_POOL
        .set(pool)
        .map_err(|_| "Redis pool already initialized")?;
    Ok(())
}

pub fn get_redis_pool() -> Option<&'static RedisPool> {
    REDIS_POOL.get()
}

pub struct RedisUtils;

impl RedisUtils {
    pub fn cache_key(prefix: &str, id: &str) -> String {
        format!("{}:{}", prefix, id)
    }

    pub fn user_token_key(token: &str) -> String {
        format!("auth:token:{}", token)
    }

    pub fn user_session_key(user_id: i64) -> String {
        format!("auth:session:{}", user_id)
    }

    pub fn rate_limit_key(ip: &str, endpoint: &str) -> String {
        format!("rate_limit:{}:{}", ip, endpoint)
    }

    /// Build a lock key for current-thread operations per user/agent/channel type.
    pub fn thread_lock_key(uid: i64, aid: i64, chtype: &str) -> String {
        format!("lock:thread:{}:{}:{}", uid, aid, chtype)
    }
}

// #[tokio::test]
// async fn test_redis_pool_basic_ops() {
//     let pool = RedisPool::new(RedisConfig::default())
//         .await
//         .expect("Failed to create Redis pool");

//     let key = "test:key";
//     let value = "hello world";

//     // set
//     pool.set(key, value)
//         .await
//         .expect("Failed to set value");

//     // get
//     let got: Option<String> = pool
//         .get(key)
//         .await
//         .expect("Failed to get value");
//     assert_eq!(got, Some(value.to_string()));

//     // exists
//     let exists = pool.exists(key).await.expect("Failed to check exists");
//     assert!(exists);

//     // del
//     let deleted = pool.del(key).await.expect("Failed to delete key");
//     assert!(deleted);

//     // get again
//     let got: Option<String> = pool.get(key).await.expect("Failed to get value");
//     assert_eq!(got, None);
// }
