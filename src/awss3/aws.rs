use std::time::Duration;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    Client,
    config::{Builder as S3ConfigBuilder, Credentials, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
};

pub struct AwsClient {
    client: Client,
    bucket: String,
}

impl AwsClient {
    pub async fn new_with_options(
        bucket: &str,
        region: &str,
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        force_path_style: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let region_provider = RegionProviderChain::first_try(Region::new(region.to_owned()));
        let config_loader = aws_config::from_env()
            .region(region_provider)
            .endpoint_url(endpoint)
            .credentials_provider(Credentials::new(access_key, secret_key, None, None, "oss"));

        let shared_config = config_loader.load().await;
        let s3_config = S3ConfigBuilder::from(&shared_config)
            .force_path_style(force_path_style)
            .build();
        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: bucket.to_owned(),
        })
    }

    pub async fn new(
        bucket: &str,
        region: &str,
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_with_options(bucket, region, endpoint, access_key, secret_key, false).await
    }

    ///
    /// Put an object into the bucket.
    ///
    pub async fn put_object(
        &self,
        key: &str,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .send()
            .await?;
        Ok(())
    }

    ///
    /// Get an object from the bucket.
    ///
    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = resp.body.collect().await?.into_bytes().to_vec();
        Ok(data)
    }

    ///
    /// Get a presigned URL for an object in the bucket.
    ///
    pub async fn get_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let presign_config = PresigningConfig::expires_in(expires_in)?;
        let presigned_req = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presign_config)
            .await?;
        Ok(presigned_req.uri().to_string())
    }

    ///
    /// Get a presigned PUT URL for uploading an object to the bucket.
    ///
    pub async fn get_presigned_put_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let presign_config = PresigningConfig::expires_in(expires_in)?;
        let presigned_req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presign_config)
            .await?;
        Ok(presigned_req.uri().to_string())
    }

    ///
    /// Get object metadata
    ///
    pub async fn head_object(
        &self,
        key: &str,
    ) -> Result<aws_sdk_s3::operation::head_object::HeadObjectOutput, Box<dyn std::error::Error>>
    {
        let resp = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(resp)
    }

    ///
    /// Delete the Object
    ///
    pub async fn delete_object(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    ///
    /// List the Objects
    ///
    pub async fn list_objects(
        &self,
        prefix: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut req = self.client.list_objects_v2().bucket(&self.bucket);
        if let Some(p) = prefix {
            req = req.prefix(p);
        }
        let resp = req.send().await?;
        let keys = resp
            .contents()
            .iter()
            .filter_map(|obj| obj.key().map(|k| k.to_string()))
            .collect();
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_object_from_oss() {
        let bucket = "xxxxxx";
        let key = "datasets/5f05a11ed5d5d6cbe2110b1c3aa8ba41.png";
        let region = "cn-xxxx";
        let endpoint = "https://xxxxxx.aliyuncs.com";
        let access_key = "LTAI5tQUqasxxxxxx";
        let secret_key = "vTmRciinHVwxxxxxxxx";

        let client = AwsClient::new(&bucket, &region, &endpoint, &access_key, &secret_key)
            .await
            .expect("Failed to create AwsClient");

        let data = client
            .get_object(&key)
            .await
            .expect("Failed to get object from OSS");
        println!("{:?}", data);

        let url = client
            .get_presigned_url(key, std::time::Duration::from_secs(600))
            .await
            .expect("Failed to generate presigned URL");

        println!("Presigned URL: {}", url);

        // assert!(!data.is_empty(), "Downloaded object is empty");
    }
}
