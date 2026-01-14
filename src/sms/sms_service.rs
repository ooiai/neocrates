use std::{collections::HashMap, sync::Arc};

use crate::rediscache::RedisPool;
use crate::response::error::{AppError, AppResult};
use crate::sms::aliyun::Aliyun;
use crate::sms::tencent::{Region, Tencent};

/// 发送验证码所需的短信模板变量。
///
/// 目前只包含 `code`，如果以后扩展模板参数，可以在这里增加字段并调整序列化逻辑。
#[derive(Debug, Clone)]
pub struct CaptchaTemplate {
    pub code: String,
}

impl CaptchaTemplate {
    pub fn to_aliyun_template_param_json(&self) -> String {
        // Aliyun 的 TemplateParam 是 JSON 字符串，例如：{"code":"123456"}
        format!(r#"{{"code":"{}"}}"#, self.code)
    }

    pub fn to_tencent_template_param_vec(&self) -> Vec<String> {
        // Tencent 的 TemplateParamSet 是数组，按模板参数顺序传递
        vec![self.code.clone()]
    }
}

/// 可扩展的短信提供商配置。
///
/// - `Aliyun`: 走阿里云短信
/// - `Tencent`: 走腾讯云短信
#[derive(Debug, Clone)]
pub enum SmsProviderConfig {
    Aliyun(AliyunSmsConfig),
    Tencent(TencentSmsConfig),
}

/// 阿里云短信配置（SendSms）。
#[derive(Debug, Clone)]
pub struct AliyunSmsConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub sign_name: String,
    pub template_code: String,
}

/// 腾讯云短信配置（SendSms）。
#[derive(Debug, Clone)]
pub struct TencentSmsConfig {
    pub secret_id: String,
    pub secret_key: String,
    pub sms_app_id: String,
    pub region: Region,
    pub sign_name: String,
    pub template_id: String,
}

/// SmsService 运行配置。
///
/// `provider` 决定使用哪个短信服务商；
/// `debug` 为 true 时不发短信，只把验证码写入 Redis（便于联调/测试）。
#[derive(Debug, Clone)]
pub struct SmsConfig {
    pub debug: bool,
    pub provider: SmsProviderConfig,
}

/// 发送结果（便于日志/调用方排查）。
#[derive(Debug, Clone)]
pub struct SmsSendResult {
    pub provider: &'static str,
    pub request_id: Option<String>,
    pub raw_code: Option<String>,
    pub raw_message: Option<String>,
}

/// 验证码短信服务
pub struct SmsService;

impl SmsService {
    /// Send a captcha to the given mobile number.
    ///
    /// - `redis_key_prefix`: Redis key 前缀（会拼接手机号）
    /// - `mobile_regex`: 由调用方注入的手机号正则（避免对某个固定常量/模块路径的耦合）
    ///
    /// 行为：
    /// 1. 校验手机号
    /// 2. 生成 6 位验证码
    /// 3. debug 模式：只存 Redis，不发短信
    /// 4. 正常模式：发短信成功后存 Redis；失败则返回错误
    pub async fn send_captcha(
        config: &Arc<SmsConfig>,
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        redis_key_prefix: &str,
        mobile_regex: &regex::Regex,
    ) -> AppResult<()> {
        Self::send_captcha_with_options(
            config,
            redis_pool,
            mobile,
            redis_key_prefix,
            mobile_regex,
            60 * 5,
            true,
        )
        .await
        .map(|_| ())
    }

    /// Send a captcha with options.
    ///
    /// - `expire_seconds`: Redis 过期秒数
    /// - `delete_on_mismatch`: 验证码校验失败时是否删除（与 `valid_auth_captcha` 对齐）
    pub async fn send_captcha_with_options(
        config: &Arc<SmsConfig>,
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        redis_key_prefix: &str,
        mobile_regex: &regex::Regex,
        expire_seconds: u64,
        _delete_on_mismatch: bool,
    ) -> AppResult<SmsSendResult> {
        if !mobile_regex.is_match(mobile) {
            return Err(AppError::ClientError("手机号码格式不正确".to_string()));
        }

        let code_num: u32 = rand::random::<u32>() % 900000 + 100000;
        let template = CaptchaTemplate {
            code: code_num.to_string(),
        };

        tracing::info!(
            "「send_captcha」 mobile: {}, code: {}",
            mobile,
            template.code
        );

        // debug 模式：不发短信，只入库
        if config.debug {
            Self::store_captcha_code_with_options(
                redis_pool,
                mobile,
                code_num,
                expire_seconds,
                redis_key_prefix,
            )
            .await?;

            tracing::warn!("「send_captcha」 Debug mode: SMS not sent, code stored in Redis");

            return Ok(SmsSendResult {
                provider: "debug",
                request_id: None,
                raw_code: Some("OK".to_string()),
                raw_message: Some("debug mode".to_string()),
            });
        }

        let send_result = Self::send_via_provider(config, mobile, &template).await?;

        // 只有发送成功才入 Redis（避免用户收不到但能用验证码登录）
        Self::store_captcha_code_with_options(
            redis_pool,
            mobile,
            code_num,
            expire_seconds,
            redis_key_prefix,
        )
        .await?;

        tracing::info!("「send_captcha」 SMS sent and code stored successfully");
        Ok(send_result)
    }

    async fn send_via_provider(
        config: &Arc<SmsConfig>,
        mobile: &str,
        template: &CaptchaTemplate,
    ) -> AppResult<SmsSendResult> {
        match &config.provider {
            SmsProviderConfig::Aliyun(aliyun_cfg) => {
                let aliyun = Aliyun::new(&aliyun_cfg.access_key_id, &aliyun_cfg.access_key_secret);

                let resp: HashMap<String, String> = aliyun
                    .send_sms(
                        mobile,
                        &aliyun_cfg.sign_name,
                        &aliyun_cfg.template_code,
                        &template.to_aliyun_template_param_json(),
                    )
                    .await
                    .map_err(|e| AppError::ClientError(format!("短信发送失败(Aliyun): {}", e)))?;

                // Aliyun 成功一般是 Code=OK
                match resp.get("Code").map(|s| s.as_str()) {
                    Some("OK") => Ok(SmsSendResult {
                        provider: "aliyun",
                        request_id: resp.get("RequestId").cloned(),
                        raw_code: resp.get("Code").cloned(),
                        raw_message: resp.get("Message").cloned(),
                    }),
                    _ => Err(AppError::ClientError(format!(
                        "发送短信失败(Aliyun): {}",
                        resp.get("Message")
                            .cloned()
                            .unwrap_or_else(|| "Unknown error".to_string())
                    ))),
                }
            }
            SmsProviderConfig::Tencent(tencent_cfg) => {
                let tencent = Tencent::new(
                    tencent_cfg.secret_id.clone(),
                    tencent_cfg.secret_key.clone(),
                    tencent_cfg.sms_app_id.clone(),
                );

                // Tencent phone number 需要带国家码（例如 +86xxxxxxxxxxx）
                // 这里保持最小侵入：如果调用方没带 +，默认按 +86 拼接。
                let phone = if mobile.starts_with('+') {
                    mobile.to_string()
                } else {
                    format!("+86{}", mobile)
                };

                let params = template
                    .to_tencent_template_param_vec()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                let params_ref = params.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

                let resp = tencent
                    .send_sms(
                        tencent_cfg.region.clone(),
                        &tencent_cfg.sign_name,
                        vec![phone.as_str()],
                        tencent_cfg.template_id.clone(),
                        params_ref,
                    )
                    .await
                    .map_err(|e| AppError::ClientError(format!("短信发送失败(Tencent): {}", e)))?;

                // 腾讯云返回结构：
                // resp.response.send_status_set[0].code == "Ok" 表示成功
                let status = resp
                    .response
                    .send_status_set
                    .get(0)
                    .cloned()
                    .ok_or_else(|| {
                        AppError::ClientError("发送短信失败(Tencent): empty response".to_string())
                    })?;

                if status.code.eq_ignore_ascii_case("Ok") {
                    Ok(SmsSendResult {
                        provider: "tencent",
                        request_id: Some(resp.response.request_id),
                        raw_code: Some(status.code),
                        raw_message: Some(status.message),
                    })
                } else {
                    Err(AppError::ClientError(format!(
                        "发送短信失败(Tencent): {}",
                        status.message
                    )))
                }
            }
        }
    }

    /// Validate authentication captcha
    pub async fn valid_auth_captcha(
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        captcha: &str,
        redis_key_prefix: &str,
        delete: bool,
    ) -> AppResult<()> {
        let code = Self::get_captcha_code(redis_pool, mobile, redis_key_prefix).await?;
        match code {
            Some(code) => {
                if code != captcha {
                    // remove captcha code from redis
                    Self::delete_captcha_code(redis_pool, mobile, redis_key_prefix).await?;
                    tracing::warn!(
                        "「valid_auth_captcha」 failed mobile:{}, captcha:{}",
                        mobile,
                        captcha
                    );
                    Err(AppError::ClientError("验证码错误".to_string()))
                } else {
                    if delete {
                        // remove captcha code from redis
                        Self::delete_captcha_code(redis_pool, mobile, redis_key_prefix).await?;
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

    /// Store captcha code in Redis (default 5 minutes)
    pub async fn store_captcha_code(
        redis_pool: &Arc<RedisPool>,
        mobile: &str,
        code: u32,
        redis_key_prefix: &str,
    ) -> AppResult<()> {
        Self::store_captcha_code_with_options(redis_pool, mobile, code, 60 * 5, redis_key_prefix)
            .await
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
        redis_key_prefix: &str,
    ) -> AppResult<Option<String>> {
        let key = format!("{}{}", redis_key_prefix, mobile);

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
        redis_key_prefix: &str,
    ) -> AppResult<()> {
        let key = format!("{}{}", redis_key_prefix, mobile);

        redis_pool
            .del(&key)
            .await
            .map_err(|e| AppError::RedisError(e.to_string()))?;

        tracing::info!("「delete_captcha_code」 验证码已删除: mobile={}", mobile);
        Ok(())
    }
}
