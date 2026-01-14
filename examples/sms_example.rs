//! SMS captcha service usage example
//!
//! This example demonstrates how to:
//! - Choose SMS provider (Aliyun / Tencent)
//! - Send a captcha (and store it in Redis)
//! - Validate captcha from Redis
//!
//! Run with:
//! - Aliyun (recommended for a quick start; set `SMS_PROVIDER=aliyun`):
//!   cargo run --example sms_example --features full
//!
//! - Tencent (set `SMS_PROVIDER=tencent`):
//!   cargo run --example sms_example --features full
//!
//! Environment variables:
//! - REDIS_URL (optional): default "redis://127.0.0.1:6379"
//! - SMS_PROVIDER: "aliyun" | "tencent" (default: "aliyun")
//!
//! Aliyun variables (required if SMS_PROVIDER=aliyun):
//! - ALIYUN_SMS_ACCESS_KEY_ID
//! - ALIYUN_SMS_ACCESS_KEY_SECRET
//! - ALIYUN_SMS_SIGN_NAME
//! - ALIYUN_SMS_TEMPLATE_CODE
//!
//! Tencent variables (required if SMS_PROVIDER=tencent):
//! - TENCENT_SMS_SECRET_ID
//! - TENCENT_SMS_SECRET_KEY
//! - TENCENT_SMS_APP_ID
//! - TENCENT_SMS_REGION (optional): "ap-beijing" | "ap-nanjing" | "ap-guangzhou" | other string, default "ap-beijing"
//! - TENCENT_SMS_SIGN_NAME
//! - TENCENT_SMS_TEMPLATE_ID
//!
//! Behavior notes:
//! - If you want to test without real SMS sending, set `debug: true` in `SmsConfig` below.
//!   In debug mode, the captcha is ONLY stored in Redis (no SMS request is made).
//! - Phone format:
//!   - Aliyun: typically expects mainland China numbers like "13800138000"
//!   - Tencent: this service wrapper auto-prefixes "+86" if you didn't include "+"

use std::{env, sync::Arc};

use neocrates::rediscache::{RedisConfig, RedisPool};
use neocrates::sms::sms_service::{
    AliyunSmsConfig, SmsConfig, SmsProviderConfig, SmsService, TencentSmsConfig,
};
use neocrates::sms::tencent::Region;

fn must_get_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        eprintln!("Missing required env var: {}", key);
        std::process::exit(1);
    })
}

fn parse_region(s: &str) -> Region {
    match s {
        "ap-beijing" => Region::Beijing,
        "ap-nanjing" => Region::Nanjing,
        "ap-guangzhou" => Region::Guangzhou,
        other => Region::Other(other.to_string()),
    }
}

#[neocrates::tokio::main]
async fn main() {
    // ---------- Redis ----------
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    println!("Connecting to Redis at: {}", redis_url);

    let redis_config = RedisConfig {
        url: redis_url,
        max_size: 10,
        min_idle: Some(1),
        connection_timeout: std::time::Duration::from_secs(5),
        idle_timeout: Some(std::time::Duration::from_secs(600)),
        max_lifetime: Some(std::time::Duration::from_secs(3600)),
    };

    let redis_pool = match RedisPool::new(redis_config).await {
        Ok(pool) => Arc::new(pool),
        Err(e) => {
            eprintln!("Failed to connect to Redis: {}", e);
            eprintln!("Please make sure Redis is running. Example: redis-server");
            std::process::exit(1);
        }
    };

    // ---------- Provider selection ----------
    let provider = env::var("SMS_PROVIDER").unwrap_or_else(|_| "aliyun".to_string());

    // Set to true to avoid real SMS calls (only store captcha in Redis).
    let debug = env::var("SMS_DEBUG")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let sms_config = match provider.as_str() {
        "aliyun" => {
            let aliyun = AliyunSmsConfig {
                access_key_id: must_get_env("ALIYUN_SMS_ACCESS_KEY_ID"),
                access_key_secret: must_get_env("ALIYUN_SMS_ACCESS_KEY_SECRET"),
                sign_name: must_get_env("ALIYUN_SMS_SIGN_NAME"),
                template_code: must_get_env("ALIYUN_SMS_TEMPLATE_CODE"),
            };

            SmsConfig {
                debug,
                provider: SmsProviderConfig::Aliyun(aliyun),
            }
        }
        "tencent" => {
            let region_str =
                env::var("TENCENT_SMS_REGION").unwrap_or_else(|_| "ap-beijing".to_string());

            let tencent = TencentSmsConfig {
                secret_id: must_get_env("TENCENT_SMS_SECRET_ID"),
                secret_key: must_get_env("TENCENT_SMS_SECRET_KEY"),
                sms_app_id: must_get_env("TENCENT_SMS_APP_ID"),
                region: parse_region(&region_str),
                sign_name: must_get_env("TENCENT_SMS_SIGN_NAME"),
                template_id: must_get_env("TENCENT_SMS_TEMPLATE_ID"),
            };

            SmsConfig {
                debug,
                provider: SmsProviderConfig::Tencent(tencent),
            }
        }
        other => {
            eprintln!(
                "Unsupported SMS_PROVIDER: {} (expected: aliyun|tencent)",
                other
            );
            std::process::exit(1);
        }
    };

    let sms_config = Arc::new(sms_config);

    // ---------- Business parameters ----------
    // Redis key prefix for captcha codes
    let redis_key_prefix = "captcha:sms:";

    // A simple China mainland mobile regex (11 digits, starts with 1)
    // You can replace this with your project's MOBILE_REGEX constant if you have one.
    let mobile_regex = regex::Regex::new(r"^1\d{10}$").expect("invalid mobile regex");

    // Target mobile number (for demo)
    // - In real projects you receive this from user input.
    let mobile = env::var("MOBILE").unwrap_or_else(|_| "13800138000".to_string());

    println!("\n=== SMS captcha demo ===");
    println!("provider: {}", provider);
    println!("debug: {}", debug);
    println!("mobile: {}", mobile);
    println!("redis_key_prefix: {}", redis_key_prefix);

    // ---------- 1) Send captcha ----------
    // This will:
    // - validate mobile by regex
    // - generate a 6-digit code
    // - send SMS via selected provider (unless debug=true)
    // - store code in Redis on success (always stores in debug mode)
    let send_res = SmsService::send_captcha(
        &sms_config,
        &redis_pool,
        &mobile,
        redis_key_prefix,
        &mobile_regex,
    )
    .await;

    match send_res {
        Ok(_) => println!("send_captcha: OK (code stored in Redis)"),
        Err(e) => {
            eprintln!("send_captcha: FAILED: {}", e);
            eprintln!("If you are testing locally, consider setting SMS_DEBUG=true.");
            std::process::exit(1);
        }
    }

    // ---------- 2) Read code back from Redis (demo only) ----------
    // In production you should NOT read it back; the user receives it via SMS.
    let stored = SmsService::get_captcha_code(&redis_pool, &mobile, redis_key_prefix)
        .await
        .unwrap_or(None);

    let Some(code) = stored else {
        eprintln!("No captcha found in Redis (unexpected).");
        std::process::exit(1);
    };

    println!("(demo) captcha read from Redis: {}", code);

    // ---------- 3) Validate captcha ----------
    // delete=true means: once validated successfully, remove it from Redis (one-time use).
    let valid_res =
        SmsService::valid_auth_captcha(&redis_pool, &mobile, &code, redis_key_prefix, true).await;

    match valid_res {
        Ok(_) => println!("valid_auth_captcha: OK (deleted from Redis)"),
        Err(e) => {
            eprintln!("valid_auth_captcha: FAILED: {}", e);
            std::process::exit(1);
        }
    }

    // ---------- 4) Validate again (should fail due to deletion) ----------
    let valid_again =
        SmsService::valid_auth_captcha(&redis_pool, &mobile, &code, redis_key_prefix, true).await;

    println!(
        "valid_auth_captcha (again, expected to fail): {}",
        match valid_again {
            Ok(_) => "unexpected OK".to_string(),
            Err(e) => format!("error: {}", e),
        }
    );

    println!("\nDone.");
}
