use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use hmac::{Hmac, Mac};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HOST, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StsError {
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("STS API error: {code} - {message}")]
    ApiError { code: String, message: String },

    #[error("Signature error: {0}")]
    SignatureError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StsCredential {
    pub tmp_secret_id: String,
    pub tmp_secret_key: String,
    pub token: String,
    pub expiration: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StsResponse {
    #[serde(rename = "Response")]
    pub response: StsResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StsResponseData {
    #[serde(rename = "Credentials")]
    pub credentials: Option<Credentials>,

    #[serde(rename = "RequestId")]
    pub request_id: String,

    #[serde(rename = "Error")]
    pub error: Option<ApiError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    #[serde(rename = "Token")]
    pub token: String,

    #[serde(rename = "TmpSecretId")]
    pub tmp_secret_id: String,

    #[serde(rename = "TmpSecretKey")]
    pub tmp_secret_key: String,

    #[serde(rename = "ExpiredTime")]
    pub expired_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    #[serde(rename = "Code")]
    pub code: String,

    #[serde(rename = "Message")]
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct StsClient {
    pub secret_id: String,
    pub secret_key: String,
    pub endpoint: String,
    pub region: String,
    client: reqwest::Client,
    service: String,
}

impl StsClient {
    pub fn new(
        secret_id: impl Into<String>,
        secret_key: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        let region = region.into();
        StsClient {
            secret_id: secret_id.into(),
            secret_key: secret_key.into(),
            endpoint: "sts.tencentcloudapi.com".to_string(),
            region,
            client: reqwest::Client::new(),
            service: "sts".to_string(),
        }
    }

    pub async fn get_temp_credentials(
        &self,
        name: &str,
        policy: Option<&str>,
        duration_seconds: Option<u32>,
    ) -> Result<StsCredential, StsError> {
        let mut params = HashMap::new();

        // Common parameters for CAM
        params.insert("Action".to_string(), "GetFederationToken".to_string());
        params.insert("Version".to_string(), "2018-08-13".to_string());
        params.insert("Region".to_string(), self.region.clone());
        params.insert("Name".to_string(), name.to_string());

        if let Some(policy_str) = policy {
            params.insert("Policy".to_string(), policy_str.to_string());
        }

        if let Some(duration) = duration_seconds {
            params.insert("DurationSeconds".to_string(), duration.to_string());
        }

        let response = self.send_request(&params).await?;
        let sts_response: StsResponse = serde_json::from_str(&response)?;

        if let Some(error) = sts_response.response.error {
            return Err(StsError::ApiError {
                code: error.code,
                message: error.message,
            });
        }

        let credentials = sts_response
            .response
            .credentials
            .ok_or_else(|| StsError::ApiError {
                code: "NoCredentialsReturned".to_string(),
                message: "No credentials found in the response".to_string(),
            })?;

        // Convert expiration to DateTime<Utc>
        let expiration = Utc
            .timestamp_opt(credentials.expired_time as i64, 0)
            .single()
            .ok_or_else(|| StsError::SignatureError("Invalid expiration timestamp".to_string()))?;

        Ok(StsCredential {
            tmp_secret_id: credentials.tmp_secret_id,
            tmp_secret_key: credentials.tmp_secret_key,
            token: credentials.token,
            expiration,
        })
    }

    async fn send_request(&self, params: &HashMap<String, String>) -> Result<String, StsError> {
        let now = chrono::Utc::now();
        let timestamp = now.timestamp();
        let date = now.format("%Y-%m-%d").to_string();

        let mut final_params = params.clone();
        final_params.insert("Timestamp".to_string(), timestamp.to_string());
        final_params.insert("Nonce".to_string(), rand::random::<u32>().to_string());
        final_params.insert("SecretId".to_string(), self.secret_id.clone());

        let http_request_method = "POST";

        let canonical_params = self.build_canonical_query_string(&final_params);

        let canonical_uri = "/";

        let host = self.endpoint.clone();

        let canonical_request = format!(
            "{}\n{}\n{}\nhost={}\ncontent-type=application/json\n\nhost;content-type\n",
            http_request_method, canonical_uri, canonical_params, host
        );

        let canonical_request_hash = hash_sha256(&canonical_request);

        let string_to_sign = format!(
            "TC3-HMAC-SHA256\n{}\n{}\n{}",
            timestamp, date, canonical_request_hash
        );

        let signature = self.calculate_signature(&date, &string_to_sign)?;

        let authorization = format!(
            "TC3-HMAC-SHA256 Credential={}/{}/{}, SignedHeaders=content-type;host, Signature={}",
            self.secret_id, date, self.service, signature
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            HOST,
            HeaderValue::from_str(&host).expect("Failed to set host header"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&authorization).expect("Failed to set authorization header"),
        );

        let request_body = serde_json::to_string(&final_params)?;

        let url = format!("https://{}{}", host, canonical_uri);
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(request_body)
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }

    fn build_canonical_query_string(&self, params: &HashMap<String, String>) -> String {
        let mut keys: Vec<&String> = params.keys().collect();
        keys.sort();

        let mut canonical_params = String::new();
        for key in keys {
            if !canonical_params.is_empty() {
                canonical_params.push('&');
            }
            canonical_params.push_str(&format!(
                "{}={}",
                url_encode(key),
                url_encode(params.get(key).expect("Failed to get parameter value"))
            ));
        }

        canonical_params
    }

    fn calculate_signature(&self, date: &str, string_to_sign: &str) -> Result<String, StsError> {
        let secret_date = hmac_sha256(format!("TC3{}", self.secret_key).as_bytes(), date)?;

        let secret_service = hmac_sha256(&secret_date, &self.service)?;

        let secret_signing = hmac_sha256(&secret_service, "tc3_request")?;

        let signature = hmac_sha256_hex(&secret_signing, string_to_sign)?;

        Ok(signature)
    }
}

fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char)
            }
            _ => {
                result.push_str(&format!("%{:02X}", b));
            }
        }
    }
    result
}

fn hash_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

fn hmac_sha256(key: &[u8], data: &str) -> Result<Vec<u8>, StsError> {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| StsError::SignatureError(format!("HMAC error: {}", e)))?;

    mac.update(data.as_bytes());

    let result = mac.finalize();
    Ok(result.into_bytes().to_vec())
}

fn hmac_sha256_hex(key: &[u8], data: &str) -> Result<String, StsError> {
    let hash = hmac_sha256(key, data)?;
    Ok(hex::encode(hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_sts() {
        let client = StsClient::new("your_secret_id", "your_secret_key", "ap-guangzhou");

        let policy = r#"{
            "version": "2.0",
            "statement": [
                {
                    "action": [
                        "name/cos:GetObject",
                        "name/cos:PutObject"
                    ],
                    "effect": "allow",
                    "resource": [
                        "qcs::cos:ap-guangzhou:uid/1250000000:examplebucket-1250000000/*"
                    ]
                }
            ]
        }"#;

        let credentials = client
            .get_temp_credentials("example-session", Some(policy), Some(7200))
            .await
            .expect("Failed to get temporary credentials");

        println!("TmpSecretId: {}", credentials.tmp_secret_id);
        println!("TmpSecretKey: {}", credentials.tmp_secret_key);
        println!("Token: {}", credentials.token);
        println!("Expiration: {}", credentials.expiration);
    }
}
