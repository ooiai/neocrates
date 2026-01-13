//! Captcha service module providing various types of captcha generation and validation
//!
//! Supports multiple captcha types:
//! - Slider captcha (滑动验证码)
//! - Numeric captcha (数字验证码)
//! - Alphanumeric captcha (字母数字验证码)

use std::sync::Arc;

#[cfg(any(feature = "redis", feature = "full"))]
use crate::rediscache::RedisPool;
use crate::response::error::{AppError, AppResult};

/// Captcha type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaType {
    /// Slider captcha (滑动验证码)
    Slider,
    /// Numeric captcha (4-6 digit numbers)
    Numeric,
    /// Alphanumeric captcha (letters and numbers)
    Alphanumeric,
}

/// Captcha generation result
#[derive(Debug, Clone, crate::serde::Serialize, crate::serde::Deserialize)]
pub struct CaptchaData {
    /// Captcha ID for validation
    pub id: String,
    /// Captcha code (for validation, may be hidden for security)
    pub code: String,
    /// Expiration time in seconds
    pub expires_in: u64,
}

/// Captcha service for generating and validating various types of captchas
pub struct CaptchaService;

impl CaptchaService {
    const CACHE_PREFIX_SLIDER: &'static str = "captcha:slider:";
    const CACHE_PREFIX_NUMERIC: &'static str = "captcha:numeric:";
    const CACHE_PREFIX_ALPHA: &'static str = "captcha:alpha:";

    /// Default expiration time (2 minutes)
    const DEFAULT_EXPIRATION: u64 = 120;

    // ==================== Slider Captcha ====================

    /// Generate a slider captcha for the given account
    ///
    /// # Arguments
    /// * `redis_pool` - Redis connection pool
    /// * `code` - Verification code to store
    /// * `account` - Account identifier (email, phone, etc.)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(AppError)` on failure
    ///
    /// # Example
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use neocrates::captcha::CaptchaService;
    ///
    /// async fn example(redis_pool: Arc<RedisPool>) {
    ///     let result = CaptchaService::gen_captcha_slider(
    ///         &redis_pool,
    ///         "abc123",
    ///         "user@example.com"
    ///     ).await;
    /// }
    /// ```
    #[cfg(any(feature = "redis", feature = "full"))]
    pub async fn gen_captcha_slider(
        redis_pool: &Arc<RedisPool>,
        code: &str,
        account: &str,
    ) -> AppResult<()> {
        let key = format!("{}{}", Self::CACHE_PREFIX_SLIDER, account);
        let value = Self::hash_code(code);
        let seconds = Self::DEFAULT_EXPIRATION;

        redis_pool
            .setex(key, value.clone(), seconds)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        crate::tracing::info!(
            "gen_captcha_slider success for account: {}, value: {}",
            account,
            value
        );
        Ok(())
    }

    /// Validate the slider captcha for the given account
    ///
    /// # Arguments
    /// * `redis_pool` - Redis connection pool
    /// * `code` - Code to validate
    /// * `account` - Account identifier
    /// * `delete` - Whether to delete the captcha after validation
    ///
    /// # Returns
    /// * `Ok(())` if validation succeeds
    /// * `Err(AppError)` if validation fails
    #[cfg(any(feature = "redis", feature = "full"))]
    pub async fn captcha_slider_valid(
        redis_pool: &Arc<RedisPool>,
        code: &str,
        account: &str,
        delete: bool,
    ) -> AppResult<()> {
        let key = format!("{}{}", Self::CACHE_PREFIX_SLIDER, account);
        let result = redis_pool
            .get::<_, String>(&key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        match result {
            Some(stored_code) => {
                let hashed_input = Self::hash_code(code);
                if stored_code != hashed_input {
                    return Err(AppError::ClientError(
                        "Slider captcha verification failed, please refresh and try again"
                            .to_string(),
                    ));
                }
            }
            None => {
                return Err(AppError::ClientError(
                    "Captcha expired or not found".to_string(),
                ));
            }
        }

        // Delete the captcha code from Redis after validation
        if delete {
            redis_pool
                .del(&key)
                .await
                .map_err(|e| AppError::RedisError(e.to_string()))?;
        }

        crate::tracing::info!("captcha_slider_valid success for account: {}", account);
        Ok(())
    }

    /// Delete the slider captcha from Redis
    ///
    /// # Arguments
    /// * `redis_pool` - Redis connection pool
    /// * `account` - Account identifier
    #[cfg(any(feature = "redis", feature = "full"))]
    pub async fn captcha_slider_delete(
        redis_pool: &Arc<RedisPool>,
        account: &str,
    ) -> AppResult<()> {
        let key = format!("{}{}", Self::CACHE_PREFIX_SLIDER, account);
        redis_pool
            .del(&key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        Ok(())
    }

    // ==================== Numeric Captcha ====================

    /// Generate a numeric captcha (4-6 digits)
    ///
    /// # Arguments
    /// * `redis_pool` - Redis connection pool
    /// * `account` - Account identifier
    /// * `length` - Length of the numeric code (default: 6)
    ///
    /// # Returns
    /// * `Ok(CaptchaData)` containing the captcha ID and code
    ///
    /// # Example
    /// ```rust,ignore
    /// use neocrates::captcha::CaptchaService;
    ///
    /// async fn example(redis_pool: Arc<RedisPool>) {
    ///     let captcha = CaptchaService::gen_numeric_captcha(
    ///         &redis_pool,
    ///         "user@example.com",
    ///         Some(6)
    ///     ).await.unwrap();
    ///
    ///     println!("Captcha ID: {}", captcha.id);
    ///     println!("Captcha Code: {}", captcha.code);
    /// }
    /// ```
    #[cfg(any(feature = "redis", feature = "full"))]
    pub async fn gen_numeric_captcha(
        redis_pool: &Arc<RedisPool>,
        account: &str,
        length: Option<usize>,
    ) -> AppResult<CaptchaData> {
        let len = length.unwrap_or(6).clamp(4, 8);

        // Generate random numeric code using uuid for randomness (Send-safe)
        let uuid = crate::uuid::Uuid::new_v4();
        let uuid_bytes = uuid.as_bytes();
        let code: String = (0..len)
            .map(|i| (uuid_bytes[i % 16] % 10).to_string())
            .collect();

        let id = crate::uuid::Uuid::new_v4().to_string();
        let key = format!("{}{}", Self::CACHE_PREFIX_NUMERIC, id);

        redis_pool
            .setex(&key, code.clone(), Self::DEFAULT_EXPIRATION)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        crate::tracing::info!(
            "gen_numeric_captcha success for account: {}, id: {}",
            account,
            id
        );

        Ok(CaptchaData {
            id,
            code,
            expires_in: Self::DEFAULT_EXPIRATION,
        })
    }

    /// Validate numeric captcha
    ///
    /// # Arguments
    /// * `redis_pool` - Redis connection pool
    /// * `id` - Captcha ID
    /// * `code` - Code to validate
    /// * `delete` - Whether to delete after validation
    #[cfg(any(feature = "redis", feature = "full"))]
    pub async fn validate_numeric_captcha(
        redis_pool: &Arc<RedisPool>,
        id: &str,
        code: &str,
        delete: bool,
    ) -> AppResult<()> {
        let key = format!("{}{}", Self::CACHE_PREFIX_NUMERIC, id);
        let result = redis_pool
            .get::<_, String>(&key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        match result {
            Some(stored_code) => {
                if stored_code != code {
                    return Err(AppError::ClientError(
                        "Numeric captcha verification failed".to_string(),
                    ));
                }
            }
            None => {
                return Err(AppError::ClientError(
                    "Captcha expired or not found".to_string(),
                ));
            }
        }

        if delete {
            redis_pool
                .del(&key)
                .await
                .map_err(|e| AppError::RedisError(e.to_string()))?;
        }

        crate::tracing::info!("validate_numeric_captcha success for id: {}", id);
        Ok(())
    }

    // ==================== Alphanumeric Captcha ====================

    /// Generate an alphanumeric captcha (letters and numbers)
    ///
    /// # Arguments
    /// * `redis_pool` - Redis connection pool
    /// * `account` - Account identifier
    /// * `length` - Length of the code (default: 6)
    ///
    /// # Returns
    /// * `Ok(CaptchaData)` containing the captcha ID and code
    ///
    /// # Example
    /// ```rust,ignore
    /// use neocrates::captcha::CaptchaService;
    ///
    /// async fn example(redis_pool: Arc<RedisPool>) {
    ///     let captcha = CaptchaService::gen_alphanumeric_captcha(
    ///         &redis_pool,
    ///         "user@example.com",
    ///         Some(6)
    ///     ).await.unwrap();
    ///
    ///     println!("Captcha Code: {}", captcha.code); // e.g., "A3K7M9"
    /// }
    /// ```
    #[cfg(any(feature = "redis", feature = "full"))]
    pub async fn gen_alphanumeric_captcha(
        redis_pool: &Arc<RedisPool>,
        account: &str,
        length: Option<usize>,
    ) -> AppResult<CaptchaData> {
        let len = length.unwrap_or(6).clamp(4, 10);

        // Generate random alphanumeric code (excluding confusing characters: 0, O, I, l, 1)
        let charset = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

        // Use uuid for randomness (Send-safe)
        let uuid = crate::uuid::Uuid::new_v4();
        let uuid_bytes = uuid.as_bytes();
        let code: String = (0..len)
            .map(|i| {
                let idx = (uuid_bytes[i % 16] as usize) % charset.len();
                charset[idx] as char
            })
            .collect();

        let id = crate::uuid::Uuid::new_v4().to_string();
        let key = format!("{}{}", Self::CACHE_PREFIX_ALPHA, id);

        redis_pool
            .setex(&key, code.clone(), Self::DEFAULT_EXPIRATION)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        crate::tracing::info!(
            "gen_alphanumeric_captcha success for account: {}, id: {}",
            account,
            id
        );

        Ok(CaptchaData {
            id,
            code,
            expires_in: Self::DEFAULT_EXPIRATION,
        })
    }

    /// Validate alphanumeric captcha (case-insensitive)
    ///
    /// # Arguments
    /// * `redis_pool` - Redis connection pool
    /// * `id` - Captcha ID
    /// * `code` - Code to validate
    /// * `delete` - Whether to delete after validation
    #[cfg(any(feature = "redis", feature = "full"))]
    pub async fn validate_alphanumeric_captcha(
        redis_pool: &Arc<RedisPool>,
        id: &str,
        code: &str,
        delete: bool,
    ) -> AppResult<()> {
        let key = format!("{}{}", Self::CACHE_PREFIX_ALPHA, id);
        let result = redis_pool
            .get::<_, String>(&key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        match result {
            Some(stored_code) => {
                if stored_code.to_uppercase() != code.to_uppercase() {
                    return Err(AppError::ClientError(
                        "Captcha verification failed".to_string(),
                    ));
                }
            }
            None => {
                return Err(AppError::ClientError(
                    "Captcha expired or not found".to_string(),
                ));
            }
        }

        if delete {
            redis_pool
                .del(&key)
                .await
                .map_err(|e| AppError::RedisError(e.to_string()))?;
        }

        crate::tracing::info!("validate_alphanumeric_captcha success for id: {}", id);
        Ok(())
    }

    // ==================== Helper Functions ====================

    /// Hash a code using MD5 (for simple obfuscation, not cryptographic security)
    fn hash_code(code: &str) -> String {
        use crate::md5;
        format!("{:x}", md5::compute(code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captcha_type() {
        assert_eq!(CaptchaType::Slider, CaptchaType::Slider);
        assert_ne!(CaptchaType::Numeric, CaptchaType::Alphanumeric);
    }

    #[test]
    fn test_hash_code() {
        let hash1 = CaptchaService::hash_code("test123");
        let hash2 = CaptchaService::hash_code("test123");
        let hash3 = CaptchaService::hash_code("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
