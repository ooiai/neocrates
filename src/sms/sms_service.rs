use std::{collections::HashMap, sync::Arc};

use crate::rediscache::RedisPool;
use crate::response::error::{AppError, AppResult};
use crate::sms::aliyun::Aliyun;

/// SmsService 所需的配置（由调用方组装传入，避免依赖外部 EnvConfig）
pub struct SmsConfig {
    pub debug: bool,
    pub aliyun_sms_accesskey_id: String,
    pub aliyun_sms_accesskey_secret: String,
    pub aliyun_sms_signname: String,
    pub aliyun_sms_template_code: String,
}

pub struct SmsService;

impl SmsService {
    ///
    /// Send a captcha to the given mobile number.
    ///
    pub async fn send_captcha(
        config: &Arc<SmsConfig>,
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        redis_key: &str,
        mobile_regex: &regex::Regex,
    ) -> AppResult<()> {
        if !mobile_regex.is_match(mobile) {
            return Err(AppError::ClientError("手机号码格式不正确".to_string()));
        }

        let code_num: u32 = rand::random::<u32>() % 900000 + 100000;
        let code = format!(r#"{{"code":"{}"}}"#, code_num);

        tracing::info!(
            "「send_signin_captcha」 mobile: {}, code: {}",
            mobile,
            code_num
        );

        if config.debug {
            Self::store_captcha_code(redis_pool, mobile, code_num, redis_key).await?;
            tracing::warn!(
                "「send_signin_captcha」 Debug mode: SMS not sent, code stored in Redis"
            );
            return Ok(());
        }

        let aliyun = Aliyun::new(
            &config.aliyun_sms_accesskey_id,
            &config.aliyun_sms_accesskey_secret,
        );

        let resp: HashMap<String, String> = aliyun
            .send_sms(
                mobile,
                &config.aliyun_sms_signname,
                &config.aliyun_sms_template_code,
                code.as_str(),
            )
            .await
            .map_err(|e| AppError::ClientError(format!("短信发送失败: {}", e)))?;

        match resp.get("Code") {
            Some(code) if code == "OK" => {
                Self::store_captcha_code(redis_pool, mobile, code_num, redis_key).await?;
                tracing::info!("「send_signin_captcha」 SMS sent and code stored successfully");
                Ok(())
            }
            _ => Err(AppError::ClientError(format!(
                "发送短信失败: {}",
                resp.get("Message").unwrap_or(&"Unknown error".to_string())
            ))),
        }
    }

    /// Validate authentication captcha
    pub async fn valid_auth_captcha(
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        captcha: &str,
        redis_key: &str,
        delete: bool,
    ) -> AppResult<()> {
        let code = Self::get_captcha_code(redis_pool, mobile, redis_key).await?;
        match code {
            Some(code) => {
                if code != captcha {
                    // remove captcha code from redis
                    Self::delete_captcha_code(redis_pool, mobile, redis_key).await?;
                    tracing::warn!(
                        "「valid_auth_captcha」 failed mobile:{}, captcha:{}",
                        mobile,
                        captcha
                    );
                    Err(AppError::ClientError("验证码错误".to_string()))
                } else {
                    if delete {
                        // remove captcha code from redis
                        Self::delete_captcha_code(redis_pool, mobile, redis_key).await?;
                    }
                    tracing::info!(
                        "「valid_auth_captcha」 success mobile:{} captcha:{}",
                        mobile,
                        captcha
                    );
                    Ok(())
                }
            }
            None => Err(AppError::ClientError("验证码已过期".to_string())),
        }
    }

    /// Store captcha code in Redis
    pub async fn store_captcha_code(
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        code: u32,
        redis_key: &str,
    ) -> AppResult<()> {
        Self::store_captcha_code_with_options(redis_pool, mobile, code, 60 * 5, redis_key).await
    }

    /// Store captcha code in Redis with options
    pub async fn store_captcha_code_with_options(
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        code: u32,
        expire_seconds: u64,
        key_prefix: &str,
    ) -> AppResult<()> {
        let key = format!("{}{}", key_prefix, mobile);
        let value = code.to_string();

        redis_pool
            .setex(&key, &value, expire_seconds)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        tracing::info!(
            "「store_captcha_code」 验证码已存储: key={}, expire_seconds={}",
            key,
            expire_seconds
        );
        Ok(())
    }

    /// Get captcha code from Redis
    pub async fn get_captcha_code(
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        redis_key: &str,
    ) -> AppResult<Option<String>> {
        let key = format!("{}{}", redis_key, mobile);

        match redis_pool.get(&key).await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::RedisError(e.to_string())),
        }
    }

    /// Delete captcha code from Redis
    pub async fn delete_captcha_code(
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        redis_key: &str,
    ) -> AppResult<()> {
        let key = format!("{}{}", redis_key, mobile);

        redis_pool
            .del(&key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        tracing::info!("「delete_captcha_code」 验证码已删除: mobile={}", mobile);
        Ok(())
    }
}
