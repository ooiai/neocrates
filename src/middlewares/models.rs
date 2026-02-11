use crate::middlewares::token_store::DynTokenStore;

pub const AUTHORIZATION: &str = "Authorization";
pub const BEARER: &str = "Bearer";
pub const BASIC: &str = "Basic";

pub const TOKEN_KEY: &str = ":auth_user:";

// Cache key segments
pub const CACHE_USER_INFO: &str = ":userinfo:uid:";
pub const CACHE_AUTH_UID: &str = ":auth:uid:";
pub const CACHE_AUTH_TOKEN: &str = ":auth:token:";
pub const CACHE_AUTH_REFRESH_TOKEN: &str = ":auth:refresh_token:";
pub const CACHE_ADMIN_PERMS: &str = ":perms:admin:";

pub const CACHE_AUTH_FP_UID: &str = ":auth:fp:uid:";
pub const CACHE_AUTH_UID_FP: &str = ":auth:uid:fp:";

// set role permission cache key
pub const CACHE_PERMS_RID: &str = ":perms:roleid:";
// set role menus cache key
pub const CACHE_MENUS_RID: &str = ":menus:roleid:";

// Token expiration (seconds)
// pub const EXPIRES_AT: u64 = 60 * 30;
// pub const REFRESH_EXPIRES_AT: u64 = 60 * 60 * 24 * 15;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
pub struct AuthModel {
    // user id
    pub uid: i64,
    // mobile number
    #[serde(default)]
    pub mobile: String,
    // nickname
    #[serde(default)]
    pub nickname: String,
    // username
    #[serde(default)]
    pub username: String,
    // tenant id
    pub tid: i64,
    // tenant name
    #[serde(default)]
    pub tname: String,
    // space(company/org/org_unit_id) id
    pub ouid: i64,
    // space(company/org/org_unit_id) name
    #[serde(default)]
    pub ouname: String,
    // role ids
    #[serde(default)]
    pub rids: Vec<i64>,
    // pms ids
    #[serde(default)]
    pub pmsids: Vec<i64>,
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
    pub auth_basics: Vec<String>,
}
