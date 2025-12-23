use crate::middleware::token_store::DynTokenStore;

pub const AUTHORIZATION: &str = "Authorization";
pub const BEARER: &str = "Bearer";
pub const BASIC: &str = "Basic";

pub const TOKEN_KEY: &str = ":auth_user:";

// Cache key segments
pub const CACHE_AUTH_UID: &str = ":auth:uid:";
pub const CACHE_AUTH_TOKEN: &str = ":auth:token:";
pub const CACHE_AUTH_REFRESH_TOKEN: &str = ":auth:refresh_token:";
pub const CACHE_ADMIN_PERMS: &str = ":perms:admin:";

// Token expiration (seconds)
pub const EXPIRES_AT: u64 = 60 * 30;
pub const REFRESH_EXPIRES_AT: u64 = 60 * 60 * 24 * 15;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthTokenResult {
    // access token
    pub access_token: String,
    // seconds
    pub expires_at: u64,
    // refresh token
    pub refresh_token: String,
    // seconds
    pub refresh_expires_at: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthModel {
    // user id
    pub uid: i64,
    // space(company/org) id
    pub spid: i64,
    // space(company/org) name
    pub sname: String,
    // mobile number
    pub mobile: String,
    // nickname
    pub nickname: String,
    // username
    pub username: String,
}

/// token_store - A pluggable token store (Redis or in-memory)
/// ignore_urls - URL prefixes that bypass the middleware
/// pms_ignore_urls - Permission system URL prefixes that bypass the middleware
/// prefix - Key prefix/namespace for caching, logging, or identification
pub struct MiddlewareConfig {
    pub token_store: DynTokenStore,
    pub ignore_urls: Vec<String>,
    pub pms_ignore_urls: Vec<String>,
    pub prefix: String,
}
