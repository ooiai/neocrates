use std::sync::Arc;

use crate::helper::core::utils::Utils;
use crate::middlewares::models::{
    AuthModel, AuthTokenResult, CACHE_AUTH_FP_UID, CACHE_AUTH_REFRESH_TOKEN, CACHE_AUTH_TOKEN,
    CACHE_AUTH_UID, CACHE_AUTH_UID_FP,
};
use crate::rediscache::RedisPool;
use crate::response::error::{AppError, AppResult};

pub struct AuthHelper;

impl AuthHelper {
    /// Generate a random token.
    pub fn generate_token() -> String {
        Utils::generate_token()
    }

    /// Generate a random refresh token.
    pub fn generate_refresh_token() -> String {
        Utils::generate_token()
    }

    /// Delete token and associated data from Redis for a specific user.
    pub async fn delete_token(rdpool: &Arc<RedisPool>, prefix: &str, uid: i64) -> AppResult<()> {
        let auth_uid_key = format!("{}{}{}", prefix, CACHE_AUTH_UID, uid);
        let auth_result_str: Option<String> = rdpool
            .get::<_, String>(&auth_uid_key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        let auth_result: Option<AuthTokenResult> = if let Some(s) = auth_result_str {
            match serde_json::from_str(&s) {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::error!("Failed to deserialize AuthTokenResult: {}", e);
                    None
                }
            }
        } else {
            None
        };

        if let Some(auth_result) = auth_result {
            let token_key = format!("{}{}{}", prefix, CACHE_AUTH_TOKEN, auth_result.access_token);
            let refresh_token_key = format!(
                "{}{}{}",
                prefix, CACHE_AUTH_REFRESH_TOKEN, auth_result.refresh_token
            );
            rdpool
                .del(token_key)
                .await
                .map_err(|e| AppError::RedisError(e.to_string()))?;
            rdpool
                .del(refresh_token_key)
                .await
                .map_err(|e| AppError::RedisError(e.to_string()))?;
        }
        rdpool
            .del(auth_uid_key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        Ok(())
    }

    /// Get AuthModel from Redis using the provided key (usually a token key).
    pub async fn get_auth_model(rdpool: &Arc<RedisPool>, redis_key: &str) -> AppResult<AuthModel> {
        match rdpool.get::<_, String>(redis_key).await {
            Ok(Some(t)) => serde_json::from_str(&t).map_err(|e| {
                tracing::error!("Failed to deserialize AuthModel: {}", e);
                AppError::TokenExpired
            }),
            Ok(None) => Err(AppError::Unauthorized),
            Err(e) => {
                tracing::warn!("Failed to get token from redis error: {}", e);
                Err(AppError::TokenExpired)
            }
        }
    }

    /// Get AuthTokenResult from Redis.
    pub async fn get_auth_token_result(
        rdpool: &Arc<RedisPool>,
        redis_key: &str,
    ) -> AppResult<AuthTokenResult> {
        match rdpool.get::<_, String>(redis_key).await {
            Ok(Some(t)) => serde_json::from_str(&t).map_err(|e| {
                tracing::error!("Failed to deserialize AuthTokenResult: {}", e);
                AppError::TokenExpired
            }),
            Ok(None) => Err(AppError::TokenExpired),
            Err(e) => {
                tracing::warn!("Failed to get token from redis error: {}", e);
                Err(AppError::TokenExpired)
            }
        }
    }

    /// Bind fingerprint to user ID.
    pub async fn bind_fingerprint(
        rdpool: &Arc<RedisPool>,
        prefix: &str,
        uid: i64,
        fp: &str,
    ) -> AppResult<()> {
        if fp.is_empty() {
            return Err(AppError::ClientError(
                "Fingerprint cannot be empty".to_string(),
            ));
        }
        let fp_key = format!("{}{}{}", prefix, CACHE_AUTH_FP_UID, fp);
        let uid_key = format!("{}{}{}", prefix, CACHE_AUTH_UID_FP, uid);

        rdpool
            .set(fp_key, uid.to_string())
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        rdpool
            .set(uid_key, fp.to_string())
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        Ok(())
    }

    /// Get UID by fingerprint.
    pub async fn get_uid_by_fingerprint(
        rdpool: &Arc<RedisPool>,
        prefix: &str,
        fp: &str,
    ) -> AppResult<Option<i64>> {
        let fp_key = format!("{}{}{}", prefix, CACHE_AUTH_FP_UID, fp);
        let s: Option<String> = rdpool
            .get::<_, String>(&fp_key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        if let Some(v) = s {
            match v.parse::<i64>() {
                Ok(uid) => Ok(Some(uid)),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Get fingerprint by UID.
    pub async fn get_fingerprint_by_uid(
        rdpool: &Arc<RedisPool>,
        prefix: &str,
        uid: i64,
    ) -> AppResult<Option<String>> {
        let uid_key = format!("{}{}{}", prefix, CACHE_AUTH_UID_FP, uid);
        let s: Option<String> = rdpool
            .get::<_, String>(&uid_key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        Ok(s)
    }

    /// Unbind fingerprint by fingerprint string.
    pub async fn unbind_fingerprint_by_fp(
        rdpool: &Arc<RedisPool>,
        prefix: &str,
        fp: &str,
    ) -> AppResult<()> {
        let fp_key = format!("{}{}{}", prefix, CACHE_AUTH_FP_UID, fp);
        if let Some(uid) = Self::get_uid_by_fingerprint(rdpool, prefix, fp).await? {
            let uid_key = format!("{}{}{}", prefix, CACHE_AUTH_UID_FP, uid);
            let _ = rdpool.del(uid_key).await;
        }
        rdpool
            .del(fp_key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        Ok(())
    }

    /// Unbind fingerprint by UID.
    pub async fn unbind_fingerprint_by_uid(
        rdpool: &Arc<RedisPool>,
        prefix: &str,
        uid: i64,
    ) -> AppResult<()> {
        let uid_key = format!("{}{}{}", prefix, CACHE_AUTH_UID_FP, uid);
        if let Ok(Some(fp)) = rdpool.get::<_, String>(&uid_key).await {
            let fp_key = format!("{}{}{}", prefix, CACHE_AUTH_FP_UID, fp);
            let _ = rdpool.del(fp_key).await;
        }
        rdpool
            .del(uid_key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        Ok(())
    }

    /// Store authentication tokens and model in Redis.
    pub async fn store_token(
        rdpool: &Arc<RedisPool>,
        prefix: &str,
        auth_model: &AuthModel,
        auth_token: &AuthTokenResult,
    ) -> AppResult<()> {
        let auth_str =
            serde_json::to_string(&auth_model).map_err(|e| AppError::ClientError(e.to_string()))?;
        let auth_result_str =
            serde_json::to_string(&auth_token).map_err(|e| AppError::ClientError(e.to_string()))?;
        let auth_uid_key = format!("{}{}{}", prefix, CACHE_AUTH_UID, auth_model.uid);
        let token_key = format!("{}{}{}", prefix, CACHE_AUTH_TOKEN, auth_token.access_token);
        let refresh_token_key = format!(
            "{}{}{}",
            prefix, CACHE_AUTH_REFRESH_TOKEN, auth_token.refresh_token
        );

        rdpool
            .setex(token_key, &auth_str, auth_token.expires_at)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        rdpool
            .setex(refresh_token_key, &auth_str, auth_token.refresh_expires_at)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        rdpool
            .set(auth_uid_key, auth_result_str)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;
        Ok(())
    }

    /// Generate and store new authentication tokens for the given AuthModel.
    ///
    /// This method is independent of database models (Users/Spaces).
    /// It cleans up old tokens for the user before creating new ones.
    pub async fn generate_auth_token(
        rdpool: &Arc<RedisPool>,
        prefix: &str,
        expires_at: u64,
        refresh_expires_at: u64,
        auth_model: AuthModel,
    ) -> AppResult<AuthTokenResult> {
        // Delete previous token information for this user
        Self::delete_token(rdpool, prefix, auth_model.uid).await?;

        let token = Self::generate_token();
        let refresh_token = Self::generate_refresh_token();
        let auth_token = AuthTokenResult {
            access_token: token,
            refresh_token,
            expires_at,
            refresh_expires_at,
        };

        Self::store_token(rdpool, prefix, &auth_model, &auth_token).await?;

        tracing::info!(
            "Auth token generated successfully for uid: {}",
            auth_model.uid
        );
        Ok(auth_token)
    }

    /// Refresh the authentication token.
    ///
    /// Validates access_token and refresh_token against Redis records.
    /// If valid, rotates the tokens using the existing AuthModel in Redis.
    /// Note: This does not refresh user data from the database.
    pub async fn refresh_auth(
        rdpool: &Arc<RedisPool>,
        prefix: &str,
        expires_at: u64,
        refresh_expires_at: u64,
        access_token: &str,
        refresh_token: &str,
    ) -> AppResult<AuthTokenResult> {
        let refresh_token_key = format!("{}{}{}", prefix, CACHE_AUTH_REFRESH_TOKEN, refresh_token);
        let auth_model: AuthModel =
            Self::get_auth_model(rdpool, refresh_token_key.as_str()).await?;

        let auth_uid_key = format!("{}{}{}", prefix, CACHE_AUTH_UID, auth_model.uid);
        let auth_result: AuthTokenResult =
            Self::get_auth_token_result(rdpool, auth_uid_key.as_str()).await?;

        if auth_result.access_token != access_token {
            tracing::error!("Access token mismatch for uid {}", auth_model.uid);
            return Err(AppError::Unauthorized);
        }
        if auth_result.refresh_token != refresh_token {
            tracing::error!("Refresh token mismatch for uid {}", auth_model.uid);
            return Err(AppError::Unauthorized);
        }

        // Generate auth token using existing model
        Self::generate_auth_token(rdpool, prefix, expires_at, refresh_expires_at, auth_model).await
    }
}
