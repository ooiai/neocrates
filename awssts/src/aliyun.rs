use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

// Constants
const STS_SIGN_VERSION: &str = "1.0";
const STS_API_VERSION: &str = "2015-04-01";
const STS_HOST: &str = "https://sts.aliyuncs.com/";
const RESP_BODY_FORMAT: &str = "JSON";
const PERCENT_ENCODE: &str = "%2F";
const HTTP_GET: &str = "GET";

#[derive(Error, Debug)]
pub enum StsError {
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error(
        "STS API error: StatusCode={status_code}, ErrorCode={code}, ErrorMessage={message}, RequestId={request_id}"
    )]
    ServiceError {
        status_code: u16,
        code: String,
        message: String,
        request_id: String,
        host_id: Option<String>,
        raw_message: String,
    },

    #[error("Signature error: {0}")]
    SignatureError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    #[serde(rename = "AccessKeyId")]
    pub access_key_id: String,

    #[serde(rename = "AccessKeySecret")]
    pub access_key_secret: String,

    #[serde(rename = "SecurityToken")]
    pub security_token: String,

    #[serde(rename = "Expiration")]
    pub expiration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumedRoleUser {
    #[serde(rename = "Arn")]
    pub arn: String,

    #[serde(rename = "AssumedRoleId")]
    pub assumed_role_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(rename = "Credentials")]
    pub credentials: Credentials,

    #[serde(rename = "AssumedRoleUser")]
    pub assumed_role_user: AssumedRoleUser,

    #[serde(rename = "RequestId")]
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "Code")]
    pub code: String,

    #[serde(rename = "Message")]
    pub message: String,

    #[serde(rename = "RequestId")]
    pub request_id: String,

    #[serde(rename = "HostId")]
    pub host_id: Option<String>,
}

pub struct StsClient {
    access_key_id: String,
    access_key_secret: String,
    role_arn: String,
    session_name: String,
}

impl StsClient {
    pub fn new(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        role_arn: impl Into<String>,
        session_name: impl Into<String>,
    ) -> Self {
        Self {
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            role_arn: role_arn.into(),
            session_name: session_name.into(),
        }
    }

    pub async fn assume_role(&self, expired_time_seconds: u32) -> Result<Response, StsError> {
        let url = self.generate_signed_url(expired_time_seconds)?;
        let (body, status) = self.send_request(&url).await?;
        self.handle_response(body, status)
    }

    // Private functions
    fn generate_signed_url(&self, expired_time_seconds: u32) -> Result<String, StsError> {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = Uuid::new_v4().to_string();
        let expired_time_str = expired_time_seconds.to_string();

        // Build query string
        let mut query_params = Vec::new();
        query_params.push(("SignatureVersion", STS_SIGN_VERSION));
        query_params.push(("Format", RESP_BODY_FORMAT));
        query_params.push(("Timestamp", &timestamp));
        query_params.push(("RoleArn", &self.role_arn));
        query_params.push(("RoleSessionName", &self.session_name));
        query_params.push(("AccessKeyId", &self.access_key_id));
        query_params.push(("SignatureMethod", "HMAC-SHA1"));
        query_params.push(("Version", STS_API_VERSION));
        query_params.push(("Action", "AssumeRole"));
        query_params.push(("SignatureNonce", &nonce));
        query_params.push(("DurationSeconds", &expired_time_str));

        // Sort query parameters by name
        query_params.sort_by(|a, b| a.0.cmp(b.0));

        // Build the canonical query string
        let canonical_query_string: String = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
            .collect::<Vec<String>>()
            .join("&");

        // Create string to sign
        let string_to_sign = format!(
            "{}&{}&{}",
            HTTP_GET,
            PERCENT_ENCODE,
            url_encode(&canonical_query_string)
        );

        // Generate signature
        let key = format!("{}&", self.access_key_secret);
        let signature = sign_string_hmac_sha1(&string_to_sign, &key)?;

        // Build the final URL
        let assume_url = format!(
            "{}?{}&Signature={}",
            STS_HOST,
            canonical_query_string,
            url_encode(&signature)
        );

        Ok(assume_url)
    }

    async fn send_request(&self, url: &str) -> Result<(Vec<u8>, StatusCode), StsError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(true)
            .build()?;

        let resp = client.get(url).send().await?;
        let status = resp.status();
        let body = resp.bytes().await?.to_vec();

        Ok((body, status))
    }

    fn handle_response(
        &self,
        response_body: Vec<u8>,
        status_code: StatusCode,
    ) -> Result<Response, StsError> {
        if !status_code.is_success() {
            let raw_message = String::from_utf8_lossy(&response_body).to_string();

            let error_response: ErrorResponse = match serde_json::from_slice(&response_body) {
                Ok(err) => err,
                Err(_) => {
                    return Err(StsError::ServiceError {
                        status_code: status_code.as_u16(),
                        code: "UnknownError".to_string(),
                        message: "Failed to parse error response".to_string(),
                        request_id: "".to_string(),
                        host_id: None,
                        raw_message,
                    });
                }
            };

            return Err(StsError::ServiceError {
                status_code: status_code.as_u16(),
                code: error_response.code,
                message: error_response.message,
                request_id: error_response.request_id,
                host_id: error_response.host_id,
                raw_message,
            });
        }

        let response: Response = serde_json::from_slice(&response_body)?;
        Ok(response)
    }
}

// Utility functions
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

fn sign_string_hmac_sha1(string_to_sign: &str, key: &str) -> Result<String, StsError> {
    type HmacSha1 = Hmac<Sha1>;

    let mut mac = HmacSha1::new_from_slice(key.as_bytes())
        .map_err(|e| StsError::SignatureError(format!("HMAC error: {}", e)))?;

    mac.update(string_to_sign.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    Ok(STANDARD.encode(code_bytes))
}

// Helper functions to convert between time formats
pub fn parse_iso8601_to_datetime(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    let dt = DateTime::parse_from_rfc3339(s)?;
    Ok(dt.with_timezone(&Utc))
}

pub fn format_datetime_to_iso8601(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_assume_role() {
        let client = StsClient::new(
            "LTAI5tQUqaxxxxx",
            "vTmRciinHVwxxxxx",
            "acs:ram::1405xxxx63:role/smartrxx",
            "smartxx",
        );

        match client.assume_role(3600).await {
            Ok(response) => {
                println!("AccessKeyId: {}", response.credentials.access_key_id);
                println!("SecurityToken: {}", response.credentials.security_token);
                println!("Expiration: {}", response.credentials.expiration);
                println!("AssumedRoleUser ARN: {}", response.assumed_role_user.arn);

                // Convert expiration string to DateTime
                match parse_iso8601_to_datetime(&response.credentials.expiration) {
                    Ok(expiration_time) => {
                        println!("Expiration as DateTime: {}", expiration_time);
                    }
                    Err(e) => {
                        println!("Failed to parse expiration time: {}", e);
                    }
                }
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
