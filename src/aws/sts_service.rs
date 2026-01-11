use std::sync::Arc;

use crate::{
    awssts::aliyun::StsClient,
    rediscache::RedisPool,
    response::error::{AppError, AppResult},
};

pub const CACHE_ALIYUN_STS: &str = ":aliyun_sts:";
pub const CACHE_COS_STS: &str = ":cos_sts:";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwsStsVo {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub security_token: String,
    pub expiration: String,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
}
#[derive(Debug, Clone)]
pub struct AwsConfig {
    pub cos_type: String,

    pub aliyun_accesskey_id: String,
    pub aliyun_accesskey_secret: String,
    pub aliyun_role_arn: String,
    pub aliyun_expiration: u32,
    pub aliyun_role_session_name: String,
    pub aliyun_endpoint: String,
    pub aliyun_region_id: String,
    pub aliyun_bucket: String,

    pub rustfs_accesskey_id: String,
    pub rustfs_accesskey_secret: String,
    pub rustfs_endpoint: String,
    pub rustfs_region_id: String,
    pub rustfs_bucket: String,
    pub rustfs_expiration: u32,

    pub minio_accesskey_id: String,
    pub minio_accesskey_secret: String,
    pub minio_endpoint: String,
    pub minio_region_id: String,
    pub minio_bucket: String,
    pub minio_expiration: u32,
}

pub struct CosService;

impl CosService {
    /// Get COS STS credentials based on the configured COS type
    ///
    /// # Arguments
    /// * `config` - Reference to the environment configuration
    /// * `redis_pool` - Reference to the Redis connection pool
    /// * `uid` - User ID for whom the STS credentials are requested
    ///
    /// # Returns
    /// * `AppResult<AwsStsVo>` - Result containing the STS credentials or an error
    ///
    pub async fn get_cos_sts(
        config: &Arc<AwsConfig>,
        redis_pool: &Arc<RedisPool>,
        uid: i64,
    ) -> AppResult<AwsStsVo> {
        let cos_type = config.cos_type.as_str();
        match cos_type {
            "aliyun" => CosService::get_aliyun_sts(config, redis_pool, uid).await,
            "rustfs" => CosService::get_rustfs_sts(config, redis_pool, uid).await,
            "minio" => CosService::get_minio_sts(config, redis_pool, uid).await,
            _ => Err(AppError::ClientError(format!(
                "Unsupported COS type: {}",
                cos_type
            ))),
        }
    }

    /// Get RustFS STS credentials
    ///
    /// # Arguments
    /// * `config` - Reference to the environment configuration
    /// * `redis_pool` - Reference to the Redis connection pool
    /// * `uid` - User ID for whom the STS credentials are requested
    ///
    /// # Returns
    /// * `AppResult<AwsStsVo>` - Result containing the STS credentials or an error
    ///
    pub async fn get_rustfs_sts(
        config: &Arc<AwsConfig>,
        _redis_pool: &Arc<RedisPool>,
        uid: i64,
    ) -> AppResult<AwsStsVo> {
        let _redis_key = format!("{}{}", CACHE_COS_STS, uid);
        Ok(AwsStsVo {
            access_key_id: config.rustfs_accesskey_id.to_owned(),
            access_key_secret: config.rustfs_accesskey_secret.to_owned(),
            security_token: "".to_string(),
            expiration: "3600".to_string(),
            endpoint: config.rustfs_endpoint.to_owned(),
            region: config.rustfs_region_id.to_owned(),
            bucket: config.rustfs_bucket.to_owned(),
        })
    }

    /// Get RustFS STS credentials
    ///
    /// # Arguments
    /// * `config` - Reference to the environment configuration
    /// * `redis_pool` - Reference to the Redis connection pool
    /// * `uid` - User ID for whom the STS credentials are requested
    ///
    /// # Returns
    /// * `AppResult<AwsStsVo>` - Result containing the STS credentials or an error
    ///
    pub async fn get_minio_sts(
        config: &Arc<AwsConfig>,
        _redis_pool: &Arc<RedisPool>,
        uid: i64,
    ) -> AppResult<AwsStsVo> {
        let _redis_key = format!("{}{}", CACHE_COS_STS, uid);
        Ok(AwsStsVo {
            access_key_id: config.minio_accesskey_id.to_owned(),
            access_key_secret: config.minio_accesskey_secret.to_owned(),
            security_token: "".to_string(),
            expiration: "3600".to_string(),
            endpoint: config.minio_endpoint.to_owned(),
            region: config.minio_region_id.to_owned(),
            bucket: config.minio_bucket.to_owned(),
        })
    }

    /// Get Aliyun STS credentials, with caching in Redis
    ///
    /// # Arguments
    /// * `config` - Reference to the environment configuration
    /// * `redis_pool` - Reference to the Redis connection pool
    /// * `uid` - User ID for whom the STS credentials are requested
    ///
    /// # Returns
    /// * `AppResult<AwsStsVo>` - Result containing the STS credentials or an error
    ///
    pub async fn get_aliyun_sts(
        config: &Arc<AwsConfig>,
        redis_pool: &Arc<RedisPool>,
        uid: i64,
    ) -> AppResult<AwsStsVo> {
        let redis_key = format!("{}{}", CACHE_ALIYUN_STS, uid);
        let sts: AwsStsVo = match redis_pool.get::<_, String>(&redis_key).await {
            Ok(Some(t)) => {
                let x = serde_json::from_str(&t).expect("Failed to deserialize AliyunStsVo");
                x
            }
            Ok(None) => {
                let client = StsClient::new(
                    &config.aliyun_accesskey_id,
                    &config.aliyun_accesskey_secret,
                    &config.aliyun_role_arn,
                    &config.aliyun_role_session_name,
                );
                let sts: AwsStsVo = match client.assume_role(config.aliyun_expiration).await {
                    Ok(response) => AwsStsVo {
                        access_key_id: response.credentials.access_key_id,
                        access_key_secret: response.credentials.access_key_secret,
                        security_token: response.credentials.security_token,
                        expiration: response.credentials.expiration,
                        endpoint: config.aliyun_endpoint.to_owned(),
                        region: config.aliyun_region_id.to_owned(),
                        bucket: config.aliyun_bucket.to_owned(),
                    },
                    Err(err) => {
                        return Err(AppError::ClientError(err.to_string()));
                    }
                };
                redis_pool
                    .setex(
                        redis_key,
                        serde_json::to_string(&sts).expect("Failed to serialize AliyunStsVo"),
                        config.aliyun_expiration as u64 - 60,
                    )
                    .await
                    .map_err(|e| AppError::RedisError(e.to_string()))?;
                sts
            }
            Err(err) => return Err(AppError::RedisError(err.to_string())),
        };
        Ok(sts)
    }
}
