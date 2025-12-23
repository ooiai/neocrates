use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenStoreError {
    #[error("Backend error: {0}")]
    Backend(String),
    #[error("JSON error: {0}")]
    Json(String),
}

impl From<serde_json::Error> for TokenStoreError {
    fn from(err: serde_json::Error) -> Self {
        TokenStoreError::Json(err.to_string())
    }
}

/// Token storage abstraction used by middleware to persist and fetch token payloads.
///
/// The store deals with raw JSON payload strings keyed by application-defined keys.
/// It also provides convenience helpers for typed get/set using serde.
#[async_trait]
pub trait TokenStore: Send + Sync + 'static {
    /// Get the raw JSON payload for a key. Returns None if the key does not exist or has expired.
    async fn get_raw(&self, key: &str) -> Result<Option<String>, TokenStoreError>;

    /// Set the raw JSON payload for a key. If ttl_secs is Some, the entry expires after ttl seconds.
    async fn set_raw(
        &self,
        key: &str,
        value: &str,
        ttl_secs: Option<u64>,
    ) -> Result<(), TokenStoreError>;

    /// Delete a key. Returns true if the key existed and was deleted.
    async fn delete(&self, key: &str) -> Result<bool, TokenStoreError>;
}

/// Deserialize JSON value from a TokenStore into type T.
pub async fn store_get<T>(store: &dyn TokenStore, key: &str) -> Result<Option<T>, TokenStoreError>
where
    T: DeserializeOwned,
{
    match store.get_raw(key).await? {
        Some(json) => Ok(Some(serde_json::from_str::<T>(&json)?)),
        None => Ok(None),
    }
}

/// Serialize and store value T into TokenStore as JSON.
pub async fn store_set<T>(
    store: &dyn TokenStore,
    key: &str,
    value: &T,
    ttl_secs: Option<u64>,
) -> Result<(), TokenStoreError>
where
    T: Serialize + Sync,
{
    let json = serde_json::to_string(value)?;
    store.set_raw(key, &json, ttl_secs).await
}

/// In-memory token store (fallback when Redis is not available).
///
/// - Thread-safe and lock-free via DashMap
/// - Optional TTL support (checked lazily on read)
/// - Intended for tests and non-distributed setups
pub struct InMemoryTokenStore {
    map: crate::dashmap::DashMap<String, Entry>,
}

struct Entry {
    json: String,
    // Expiration time. None means no expiration.
    // We use std::time::Instant to avoid clock changes affecting expiration.
    expires_at: Option<std::time::Instant>,
}

impl InMemoryTokenStore {
    pub fn new() -> Self {
        Self {
            map: crate::dashmap::DashMap::new(),
        }
    }

    fn is_expired(expires_at: Option<std::time::Instant>) -> bool {
        match expires_at {
            Some(deadline) => std::time::Instant::now() >= deadline,
            None => false,
        }
    }
}

#[async_trait]
impl TokenStore for InMemoryTokenStore {
    async fn get_raw(&self, key: &str) -> Result<Option<String>, TokenStoreError> {
        if let Some(entry) = self.map.get(key) {
            if Self::is_expired(entry.expires_at) {
                drop(entry);
                self.map.remove(key);
                Ok(None)
            } else {
                Ok(Some(entry.json.clone()))
            }
        } else {
            Ok(None)
        }
    }

    async fn set_raw(
        &self,
        key: &str,
        value: &str,
        ttl_secs: Option<u64>,
    ) -> Result<(), TokenStoreError> {
        let expires_at =
            ttl_secs.map(|s| std::time::Instant::now() + std::time::Duration::from_secs(s));
        self.map.insert(
            key.to_string(),
            Entry {
                json: value.to_string(),
                expires_at,
            },
        );
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, TokenStoreError> {
        Ok(self.map.remove(key).is_some())
    }
}

/// Redis-backed token store (enabled when the `redis` feature is active).
///
/// It leverages the crate's `RedisPool` and stores raw JSON payloads.
/// Keys are automatically namespaced using the optional `prefix`, which
/// is prepended to the provided key.
///
/// Note: This type is only compiled when the `redis` feature (or `full`) is enabled.
#[cfg(any(feature = "redis", feature = "full"))]
pub struct RedisTokenStore {
    pool: Arc<crate::rediscache::RedisPool>,
    prefix: String,
}

#[cfg(any(feature = "redis", feature = "full"))]
impl RedisTokenStore {
    /// Create a RedisTokenStore with the given pool and key prefix namespace.
    ///
    /// Example:
    /// - prefix = "auth:token:" => final key = "auth:token:{key}"
    pub fn new(pool: Arc<crate::rediscache::RedisPool>, prefix: impl Into<String>) -> Self {
        Self {
            pool,
            prefix: prefix.into(),
        }
    }

    fn build_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

#[cfg(any(feature = "redis", feature = "full"))]
#[async_trait]
impl TokenStore for RedisTokenStore {
    async fn get_raw(&self, key: &str) -> Result<Option<String>, TokenStoreError> {
        let redis_key = self.build_key(key);
        self.pool
            .get::<_, String>(redis_key)
            .await
            .map_err(|e| TokenStoreError::Backend(e.to_string()))
    }

    async fn set_raw(
        &self,
        key: &str,
        value: &str,
        ttl_secs: Option<u64>,
    ) -> Result<(), TokenStoreError> {
        let redis_key = self.build_key(key);
        match ttl_secs {
            Some(secs) => self
                .pool
                .setex(redis_key, value, secs)
                .await
                .map_err(|e| TokenStoreError::Backend(e.to_string())),
            None => self
                .pool
                .set(redis_key, value)
                .await
                .map_err(|e| TokenStoreError::Backend(e.to_string())),
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, TokenStoreError> {
        let redis_key = self.build_key(key);
        self.pool
            .del(redis_key)
            .await
            .map_err(|e| TokenStoreError::Backend(e.to_string()))
    }
}

/// A boxed trait object alias for dynamic dispatch.
pub type DynTokenStore = Arc<dyn TokenStore>;

/// Helper to build a default TokenStore implementation:
/// - When `redis` feature is enabled, prefer RedisTokenStore
/// - Otherwise, fall back to InMemoryTokenStore
///
/// Note: This function cannot instantiate RedisTokenStore by itself since it
/// requires a RedisPool. It returns the in-memory store when Redis is not
/// available.
pub fn default_in_memory_store() -> DynTokenStore {
    Arc::new(InMemoryTokenStore::new())
}

#[cfg(any(feature = "redis", feature = "full"))]
pub fn redis_store(
    pool: Arc<crate::rediscache::RedisPool>,
    prefix: impl Into<String>,
) -> DynTokenStore {
    Arc::new(RedisTokenStore::new(pool, prefix))
}
