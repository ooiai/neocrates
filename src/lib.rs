//! # Neocrates
//!
//! A comprehensive Rust library providing various utilities and helpers for web development,
//! AWS integration, database operations, caching, and more.
//!
//! ## Features
//!
//! - **Web Framework**: Built on top of Axum with middleware support
//! - **AWS Integration**: S3, STS, and other AWS services
//! - **Database Helpers**: Diesel integration with connection pooling
//! - **Caching**: Redis support with connection pooling
//! - **Utilities**: Logging, response handling, validation, and more
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! neocrates = "0.1"
//! ```
//!
//! ## Feature Flags
//!
//! This crate uses feature flags to enable optional functionality:
//!
//! - `aws`: AWS service integrations (S3, STS, etc.)
//! - `database`: Database helpers and Diesel integration
//! - `redis`: Redis caching support
//! - `crypto`: Cryptographic utilities
//! - `full`: Enable all features
//!
//! Example with specific features:
//! ```toml
//! [dependencies]
//! neocrates = { version = "0.1" }
//! ```

#![doc(html_root_url = "https://docs.rs/neocrates/0.1.0")]
#![allow(missing_docs)]
#![allow(rustdoc::missing_crate_level_docs)]

// Re-export commonly used dependencies
pub use anyhow;
pub use argon2;
pub use async_trait;
pub use aws_config;
pub use aws_sdk_s3;
pub use aws_sdk_sts;
pub use aws_types;
pub use axum;
pub use bb8;
pub use bb8_redis;
pub use bon;
pub use chrono;
pub use dashmap;
pub use deadpool;
pub use deadpool_diesel;
pub use diesel;
pub use diesel_migrations;
pub use hmac;
pub use hyper;
pub use indexmap;
pub use lazy_static;
pub use log;
pub use moka;
pub use once_cell;
pub use rand;
pub use redis;
pub use regex;
pub use reqwest;
pub use ring;
pub use schemars;
pub use serde;
pub use serde_json;
pub use sha2;
pub use thiserror;
pub use tokio;
pub use tower;
pub use tower_http;
pub use tracing;
pub use url;
pub use urlencoding;
pub use uuid;
pub use validator;

// mod exports
pub mod awss3;
pub mod awssts;
pub mod crypto;
pub mod dieselhelper;
pub mod helper;
pub mod logger;
pub mod middleware;
pub mod rediscache;
pub mod response;
pub mod sms;
