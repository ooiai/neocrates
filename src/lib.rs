#![doc(html_root_url = "https://docs.rs/neocrates/0.1.2")]
#![allow(missing_docs)]
#![allow(rustdoc::missing_crate_level_docs)]

//! # Neocrates
//!
//! A comprehensive Rust toolkit targeting Web, AWS, Database, Redis, and Cryptography scenarios.
//!
//! Modules and their dependencies are gated by features for tree-shaking. The middleware uses a pluggable TokenStore and only requires the "web" feature. When the "redis" feature is enabled, you can provide a Redis-backed store; otherwise an in-memory store is used by default.
//!
//! ## Feature Groups
//!
//! - web: Web capabilities (Axum/Tower/Hyper, HTTP, URL handling, common middleware and response utilities)
//! - aws: AWS capabilities (aws-config, aws-types, S3, STS). Note: `aws` includes shared S3/STS dependencies
//!   - awss3: S3-only (requires aws-sdk-s3 + shared aws-config/types)
//!   - awssts: STS-only (requires aws-sdk-sts + shared aws-config/types)
//! - database: Database and Diesel capabilities (diesel, deadpool/connection pooling, migrations)
//! - redis: Redis and caching (redis, bb8, bb8-redis, moka)
//! - crypto: Cryptography and hashing (argon2, hmac, ring, sha2)
//! - sms: SMS-related modules (if they depend on HTTP, enable together with "web")
//! - full: Enable all features
//!
//! Note: Modules are compiled only when their feature is enabled; related dependencies are marked optional in Cargo.toml and aggregated via
//! the `[features]` section. See the example below.
//!
//! ## Cargo.toml Example
//!
//! ```toml
//! [features]
//! default = []
//! # Aggregated features
//! web = ["dep:axum", "dep:tower", "dep:tower-http", "dep:hyper", "dep:reqwest", "dep:url", "dep:urlencoding"]
//! aws = ["awss3", "awssts", "dep:aws-config", "dep:aws-types"]
//! awss3 = ["dep:aws-sdk-s3", "dep:aws-config", "dep:aws-types"]
//! awssts = ["dep:aws-sdk-sts", "dep:aws-config", "dep:aws-types"]
//! diesel = ["dep:diesel", "dep:deadpool", "dep:deadpool-diesel", "dep:diesel_migrations"]
//! redis = ["dep:redis", "dep:bb8", "dep:bb8-redis", "dep:moka"]
//! crypto = ["dep:argon2", "dep:hmac", "dep:ring", "dep:sha2"]
//! sms = [] # If HTTP is needed, enable together with "web"
//! full = ["web", "aws", "awss3", "awssts", "diesel", "redis", "crypto", "sms"]
//!
//! [dependencies]
//! # Mark related dependencies as optional
//! axum = { version = "0.8", features = ["macros"], optional = true }
//! tower = { version = "0.5", optional = true }
//! tower-http = { version = "0.6", optional = true }
//! hyper = { version = "1.6", features = ["full"], optional = true }
//! reqwest = { version = "0.12", features = ["gzip", "json"], optional = true }
//! url = { version = "2.5.4", optional = true }
//! urlencoding = { version = "2.1.3", optional = true }
//!
//! aws-config = { version = "1.1.7", features = ["behavior-version-latest"], optional = true }
//! aws-sdk-s3 = { version = "1.83.0", optional = true }
//! aws-sdk-sts = { version = "1.66.0", optional = true }
//! aws-types = { version = "1.3.7", optional = true }
//!
//! diesel = { version = "2", features = ["chrono", "serde_json"], optional = true }
//! deadpool = { version = "0.12", optional = true }
//! deadpool-diesel = { version = "0.6", features = ["postgres"], optional = true }
//! diesel_migrations = { version = "2", optional = true }
//!
//! redis = { version = "0.32", optional = true }
//! bb8 = { version = "0.9", optional = true }
//! bb8-redis = { version = "0.24", optional = true }
//! moka = { version = "0.12", features = ["future"], optional = true }
//!
//! argon2 = { version = "0.5", optional = true }
//! hmac = { version = "0.12", optional = true }
//! sha2 = { version = "0.10", optional = true }
//! ring = { version = "0.17.14", optional = true }
//! ```
//!
//! The above is only an example; align versions with your actual dependencies.

// =========================
// Core re-exports (always)
// =========================

pub use anyhow;
pub use async_trait;
pub use bon;
pub use chrono;
pub use dashmap;
pub use indexmap;
pub use lazy_static;
pub use log;
pub use once_cell;
pub use rand;
pub use regex;
pub use schemars;
pub use serde;
pub use serde_json;
pub use thiserror;
pub use tokio;
pub use tracing;
pub use uuid;
pub use validator;

// =========================
// Web re-exports (feature)
// =========================

#[cfg(any(feature = "web", feature = "full"))]
pub use axum;
#[cfg(any(feature = "web", feature = "full"))]
pub use hyper;
#[cfg(any(feature = "web", feature = "full"))]
pub use reqwest;
#[cfg(any(feature = "web", feature = "full"))]
pub use tower;
#[cfg(any(feature = "web", feature = "full"))]
pub use tower_http;
#[cfg(any(feature = "web", feature = "full"))]
pub use url;
#[cfg(any(feature = "web", feature = "full"))]
pub use urlencoding;

// =========================
// AWS re-exports (feature)
// =========================

// Shared AWS config/types are available under any AWS sub-feature or aws/full
#[cfg(any(
    feature = "aws",
    feature = "awss3",
    feature = "awssts",
    feature = "full"
))]
pub use aws_config;
#[cfg(any(
    feature = "aws",
    feature = "awss3",
    feature = "awssts",
    feature = "full"
))]
pub use aws_types;

// S3 is available only under awss3 or aws/full
#[cfg(any(feature = "aws", feature = "awss3", feature = "full"))]
pub use aws_sdk_s3;

// STS is available only under awssts or aws/full
#[cfg(any(feature = "aws", feature = "awssts", feature = "full"))]
pub use aws_sdk_sts;

// =============================
// Database diesel re-exports (feature)
// =============================

#[cfg(any(feature = "diesel", feature = "full"))]
pub use deadpool;
#[cfg(any(feature = "diesel", feature = "full"))]
pub use deadpool_diesel;
#[cfg(any(feature = "diesel", feature = "full"))]
pub use diesel;
#[cfg(any(feature = "diesel", feature = "full"))]
pub use diesel_migrations;

// =========================
// Redis re-exports (feature)
// =========================

#[cfg(any(feature = "redis", feature = "full"))]
pub use bb8;
#[cfg(any(feature = "redis", feature = "full"))]
pub use bb8_redis;
#[cfg(any(feature = "redis", feature = "full"))]
pub use moka;
#[cfg(any(feature = "redis", feature = "full"))]
pub use redis;

// ==========================
// Crypto re-exports (feature)
// ==========================

#[cfg(any(feature = "crypto", feature = "full"))]
pub use argon2;
#[cfg(any(feature = "crypto", feature = "full"))]
pub use hmac;
#[cfg(any(feature = "crypto", feature = "full"))]
pub use ring;
#[cfg(any(feature = "crypto", feature = "full"))]
pub use sha2;

// ==================
// Module declarations
// ==================

// Core and common modules (always available)
pub mod helper;
#[cfg(any(feature = "logger", feature = "full"))]
pub mod logger;

// Web modules
#[cfg(any(feature = "web", feature = "full"))]
pub mod middlewares;
#[cfg(any(feature = "web", feature = "full"))]
pub mod response;

// AWS
#[cfg(any(feature = "aws", feature = "awssts", feature = "full"))]
pub mod aws;
#[cfg(any(feature = "aws", feature = "awss3", feature = "full"))]
pub mod awss3;
#[cfg(any(feature = "aws", feature = "awssts", feature = "full"))]
pub mod awssts;

// Database
#[cfg(any(feature = "diesel", feature = "full"))]
pub mod dieselhelper;

// Redis
#[cfg(any(feature = "redis", feature = "full"))]
pub mod rediscache;

// Crypto
#[cfg(any(feature = "crypto", feature = "full"))]
pub mod crypto;

// SMS (if it depends on HTTP/network requests, enable together with "web")
#[cfg(any(feature = "sms", feature = "full"))]
pub mod sms;
