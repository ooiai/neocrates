use once_cell::sync::OnceCell;
use std::sync::Arc;

use crate::{
    aws::sts_service::AwsConfig,
    awss3::aws::AwsClient,
    response::error::{AppError, AppResult},
};

pub struct OssConfig {
    pub bucket: String,
    pub region: String,
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
}

static OSS_CONFIG: OnceCell<OssConfig> = OnceCell::new();

impl OssConfig {
    /// Create an OssConfig instance from the provided AwsConfig
    ///
    /// # Arguments
    /// * `cfg` - Reference to the AwsConfig containing configuration details
    ///
    /// # Returns
    /// * `OssConfig` - The constructed OssConfig instance
    ///
    pub fn from_env_config(cfg: &AwsConfig) -> OssConfig {
        match cfg.cos_type.as_str() {
            "aliyun" => OssConfig {
                bucket: cfg.aliyun_bucket.clone(),
                region: cfg.aliyun_region_id.clone(),
                endpoint: cfg.aliyun_endpoint.clone(),
                access_key: cfg.aliyun_accesskey_id.clone(),
                secret_key: cfg.aliyun_accesskey_secret.clone(),
            },
            "rustfs" => OssConfig {
                bucket: cfg.rustfs_bucket.clone(),
                region: cfg.rustfs_region_id.clone(),
                endpoint: cfg.rustfs_endpoint.clone(),
                access_key: cfg.rustfs_accesskey_id.clone(),
                secret_key: cfg.rustfs_accesskey_secret.clone(),
            },
            "minio" => OssConfig {
                bucket: cfg.minio_bucket.clone(),
                region: cfg.minio_region_id.clone(),
                endpoint: cfg.minio_endpoint.clone(),
                access_key: cfg.minio_accesskey_id.clone(),
                secret_key: cfg.minio_accesskey_secret.clone(),
            },
            _ => panic!("Unsupported COS type: {}", cfg.cos_type),
        }
    }
}

pub struct AwsService;

impl AwsService {
    pub fn init_from_env_config(config: &Arc<AwsConfig>) {
        let _ = OSS_CONFIG.set(OssConfig::from_env_config(config.as_ref()));
    }

    /// The download object from aws service
    ///
    /// # Arguments
    /// * `path` - The path where the object is stored
    ///
    /// # Returns
    /// * `AppResult<Vec<u8>>` - Result containing the downloaded data or an error
    ///
    pub async fn download_object(path: &str) -> AppResult<Vec<u8>> {
        let cfg = OSS_CONFIG.get().expect("OSS_CONFIG not initialized");
        let client = match AwsClient::new(
            &cfg.bucket,
            &cfg.region,
            &cfg.endpoint,
            &cfg.access_key,
            &cfg.secret_key,
        )
        .await
        {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("「download_object」Failed to create AWS client: {}", e);
                return Err(AppError::ClientError(e.to_string()));
            }
        };

        let data = match client.get_object(path).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(
                    "「download_object」Failed to download object from AWS: {}",
                    e
                );
                return Err(AppError::ClientError(e.to_string()));
            }
        };
        Ok(data)
    }

    /// The upload object to aws service
    ///
    /// # Arguments
    /// * `path` - The path where the object will be stored
    /// * `data` - The data to be uploaded
    ///
    /// # Returns
    /// * `AppResult<()>` - Result indicating success or failure
    ///
    pub async fn put_object(path: &str, data: Vec<u8>) -> AppResult<()> {
        let cfg = OSS_CONFIG.get().expect("OSS_CONFIG not initialized");
        let client = match AwsClient::new(
            &cfg.bucket,
            &cfg.region,
            &cfg.endpoint,
            &cfg.access_key,
            &cfg.secret_key,
        )
        .await
        {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("「put_object」Failed to create AWS client: {}", e);
                return Err(AppError::ClientError(e.to_string()));
            }
        };

        match client.put_object(path, data).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("「put_object」Failed to upload object to AWS: {}", e);
                return Err(AppError::ClientError(e.to_string()));
            }
        };
        Ok(())
    }
}
