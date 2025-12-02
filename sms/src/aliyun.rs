//! # aliyun SMS
//!
//! # Overview
//! Aliyun SMS is a cloud-based service provided by Alibaba Cloud that allows users to send SMS messages programmatically.
//!
//! This module provides functionality to send SMS messages using the Aliyun SMS service.
//!

use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{SecondsFormat, Utc};
use ring::hmac;
use std::collections::HashMap;

/// The version of the SMS API. Currently a fixed value `2017-05-25`.
const SMS_VERSION: &str = "2017-05-25";

/// The version of the signature. Currently a fixed value `1.0`.
const SIGNATURE_VERSION: &str = "1.0";

/// The method used for signing requests. Currently a fixed value `HMAC-SHA1`.
const SIGNATURE_METHOD: &str = "HMAC-SHA1";

/// The format of the response data. You can choose either `JSON` or `XML`. The default is `XML`.
const FORMAT: &str = "json";

/// aliyun sms
pub struct Aliyun<'a> {
    access_key_id: &'a str,
    access_secret: &'a str,
}

impl<'a> Aliyun<'a> {
    /// init access key
    ///
    /// ```rust,no_run
    /// use sms::aliyun::Aliyun;
    ///
    /// let aliyun = Aliyun::new("xxxx", "xxxx");
    ///
    /// ```
    pub fn new(access_key_id: &'a str, access_secret: &'a str) -> Self {
        Self {
            access_key_id,
            access_secret,
        }
    }

    /// send_sms
    ///
    /// ```rust,no_run
    /// use sms::aliyun::Aliyun;
    /// use rand::prelude::*;
    ///
    /// let aliyun = Aliyun::new("xxxx", "xxxx");
    ///
    /// let mut rng = rand::thread_rng();
    /// let code = format!(
    ///     r#"{{"code":"{}","product":"EchoLi"}}"#,
    ///     rng.gen_range(1000..=9999)
    /// );
    ///
    /// let resp = aliyun
    ///     .send_sms("18888888888", "登录验证", "SMS_123456", code.as_str())
    ///     .await
    ///     .unwrap();
    ///
    /// println!("{:?}", resp);
    /// ```
    pub async fn send_sms(
        &self,
        phone_numbers: &'a str,
        sign_name: &'a str,
        template_code: &'a str,
        template_param: &'a str,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut params = HashMap::new();

        params.insert("PhoneNumbers", phone_numbers);
        params.insert("SignName", sign_name);
        params.insert("TemplateCode", template_code);
        params.insert("RegionId", "cn-hangzhou");
        params.insert("TemplateParam", template_param);
        params.insert("Action", "SendSms");
        params.insert("Version", SMS_VERSION);

        let canonicalize_query_string = self.canonicalize_query_string(&params);

        let signature = self.signature(
            format!(
                "GET&%2F&{}",
                urlencoding::encode(&canonicalize_query_string)
            )
            .as_bytes(),
        );

        let url = format!(
            "https://dysmsapi.aliyuncs.com/?{}&Signature={}",
            canonicalize_query_string, signature
        );

        let resp = reqwest::get(url)
            .await?
            .json::<HashMap<String, String>>()
            .await?;

        Ok(resp)
    }

    /// Build the canonicalized query string
    ///
    /// link: https://help.aliyun.com/document_detail/315526.html#sectiondiv-y9b-x9s-wvp
    fn canonicalize_query_string(&self, params: &HashMap<&str, &'a str>) -> String {
        let now = Utc::now();

        let signature_nonce = now.timestamp_micros().to_string();
        let timestamp = now.to_rfc3339_opts(SecondsFormat::Secs, true);

        let mut all_params = HashMap::new();

        all_params.insert("AccessKeyId", self.access_key_id);
        all_params.insert("Format", FORMAT);
        all_params.insert("SignatureMethod", SIGNATURE_METHOD);
        all_params.insert("SignatureNonce", signature_nonce.as_str());
        all_params.insert("SignatureVersion", SIGNATURE_VERSION);
        all_params.insert("Timestamp", timestamp.as_str());

        params.iter().for_each(|(&k, &v)| {
            all_params.insert(k, v);
        });

        let mut vec_arams: Vec<String> = all_params
            .iter()
            .map(|(&k, &v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect();

        vec_arams.sort();

        vec_arams.join("&")
    }

    /// Build the signature
    ///
    fn signature(&self, string_to_sign: &[u8]) -> String {
        let key = hmac::Key::new(
            hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            format!("{}&", self.access_secret).as_bytes(),
        );

        let sign = hmac::sign(&key, string_to_sign);

        STANDARD.encode(sign.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[tokio::test]
    async fn test_send_sms() {
        let aliyun = Aliyun::new("LTAI5t9WtXXXXXXX", "63HhssAIfRNPXXXXXXXX");

        let mut rng = rand::rng();
        let code = format!(
            r#"{{"code":"{}","product":"EchoLi"}}"#,
            rng.random_range(1000..=9999)
        );

        let resp = aliyun
            .send_sms("191xxxxxxxx", "xxxxx", "SMS_469xxxxx", code.as_str())
            .await
            .expect("Failed to send SMS");

        assert_eq!(resp.get(&"Code".to_string()), Some(&"OK".to_string()));
    }
}
