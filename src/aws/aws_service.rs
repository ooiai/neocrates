use once_cell::sync::OnceCell;
use std::{sync::Arc, time::Duration};

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
    pub force_path_style: bool,
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
                force_path_style: false,
            },
            "rustfs" => OssConfig {
                bucket: cfg.rustfs_bucket.clone(),
                region: cfg.rustfs_region_id.clone(),
                endpoint: cfg.rustfs_endpoint.clone(),
                access_key: cfg.rustfs_accesskey_id.clone(),
                secret_key: cfg.rustfs_accesskey_secret.clone(),
                force_path_style: true,
            },
            "minio" => OssConfig {
                bucket: cfg.minio_bucket.clone(),
                region: cfg.minio_region_id.clone(),
                endpoint: cfg.minio_endpoint.clone(),
                access_key: cfg.minio_accesskey_id.clone(),
                secret_key: cfg.minio_accesskey_secret.clone(),
                force_path_style: true,
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
        let client = match Self::build_client(cfg).await {
            Ok(client) => client,
            Err(err) => {
                tracing::error!("「download_object」Failed to create AWS client: {}", err);
                return Err(err);
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
        let client = match Self::build_client(cfg).await {
            Ok(client) => client,
            Err(err) => {
                tracing::error!("「put_object」Failed to create AWS client: {}", err);
                return Err(err);
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

    /// Get a signed URL for accessing an object
    ///
    /// # Arguments
    /// * `path` - The path of the object
    /// * `expires_in` - The expiration time in seconds (default: 3600)
    ///
    /// example: `get_signed_url("path/to/object", 3600).await`
    /// let signed_url = AwsService::get_signed_url("path/to/file.png", 3600).await?;
    ///
    /// # Returns
    /// * `AppResult<String>` - The signed URL or an error
    ///
    pub async fn get_signed_url(path: &str, expires_in: u64) -> AppResult<String> {
        let cfg = OSS_CONFIG.get().expect("OSS_CONFIG not initialized");
        let client = match Self::build_client(cfg).await {
            Ok(client) => client,
            Err(err) => {
                tracing::error!("「get_signed_url」Failed to create AWS client: {}", err);
                return Err(err);
            }
        };

        match client
            .get_presigned_url(path, Duration::from_secs(expires_in.max(1)))
            .await
        {
            Ok(url) => {
                tracing::info!("「get_signed_url」Generated signed URL for path: {}", path);
                Ok(url)
            }
            Err(e) => {
                tracing::error!("「get_signed_url」Failed to generate signed URL: {}", e);
                return Err(AppError::ClientError(e.to_string()));
            }
        }
    }

    /// Get a signed PUT URL for uploading an object.
    pub async fn get_signed_put_url(path: &str, expires_in: u64) -> AppResult<String> {
        let cfg = OSS_CONFIG.get().expect("OSS_CONFIG not initialized");
        let client = match Self::build_client(cfg).await {
            Ok(client) => client,
            Err(err) => {
                tracing::error!("「get_signed_put_url」Failed to create AWS client: {}", err);
                return Err(err);
            }
        };

        match client
            .get_presigned_put_url(path, Duration::from_secs(expires_in.max(1)))
            .await
        {
            Ok(url) => {
                tracing::info!(
                    "「get_signed_put_url」Generated signed PUT URL for path: {}",
                    path
                );
                Ok(url)
            }
            Err(e) => {
                tracing::error!(
                    "「get_signed_put_url」Failed to generate signed PUT URL: {}",
                    e
                );
                Err(AppError::ClientError(e.to_string()))
            }
        }
    }

    /// Download object bytes via signed URL.
    pub async fn download_object_via_signed_url(path: &str, expires_in: u64) -> AppResult<Vec<u8>> {
        let signed_url = Self::get_signed_url(path, expires_in).await?;
        let safe_url = Self::redact_url(&signed_url);
        let response = reqwest::Client::new()
            .get(&signed_url)
            .send()
            .await
            .map_err(|err| {
                AppError::ClientError(format!(
                    "download_object_via_signed_url request failed: url={} err={}",
                    safe_url, err
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::ClientError(format!(
                "download_object_via_signed_url failed: status={} url={} body={}",
                status, safe_url, body
            )));
        }

        let bytes = response.bytes().await.map_err(|err| {
            AppError::ClientError(format!(
                "download_object_via_signed_url read body failed: url={} err={}",
                safe_url, err
            ))
        })?;
        Ok(bytes.to_vec())
    }

    /// Upload object bytes via signed PUT URL.
    pub async fn put_object_via_signed_url(
        path: &str,
        data: Vec<u8>,
        expires_in: u64,
    ) -> AppResult<()> {
        let signed_put_url = Self::get_signed_put_url(path, expires_in).await?;
        let safe_url = Self::redact_url(&signed_put_url);
        let response = reqwest::Client::new()
            .put(&signed_put_url)
            .body(data)
            .send()
            .await
            .map_err(|err| {
                AppError::ClientError(format!(
                    "put_object_via_signed_url request failed: url={} err={}",
                    safe_url, err
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::ClientError(format!(
                "put_object_via_signed_url failed: status={} url={} body={}",
                status, safe_url, body
            )));
        }

        Ok(())
    }

    fn redact_url(url: &str) -> String {
        url.split('?').next().unwrap_or(url).to_string()
    }

    async fn build_client(cfg: &OssConfig) -> AppResult<AwsClient> {
        AwsClient::new_with_options(
            &cfg.bucket,
            &cfg.region,
            &cfg.endpoint,
            &cfg.access_key,
            &cfg.secret_key,
            cfg.force_path_style,
        )
        .await
        .map_err(|e| AppError::ClientError(e.to_string()))
    }
}
