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
//! neocrates = { version = "0.1", features = ["aws", "database"] }
//! ```

#![doc(html_root_url = "https://docs.rs/neocrates/0.1.0")]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// Re-export commonly used dependencies
pub use anyhow;
pub use async_trait;
pub use axum;
pub use bon;
pub use chrono;
pub use dashmap;
pub use indexmap;
pub use lazy_static;
pub use log;
pub use once_cell;
pub use rand;
pub use regex;
pub use serde;
pub use serde_json;
pub use thiserror;
pub use tokio;
pub use tower;
pub use tower_http;
pub use tracing;
pub use url;
pub use uuid;

// Optional feature re-exports
#[cfg(feature = "aws")]
pub use aws_config;
#[cfg(feature = "aws")]
pub use aws_sdk_s3;
#[cfg(feature = "aws")]
pub use aws_sdk_sts;
#[cfg(feature = "aws")]
pub use aws_types;

#[cfg(feature = "database")]
pub use deadpool;
#[cfg(feature = "database")]
pub use deadpool_diesel;
#[cfg(feature = "database")]
pub use diesel;
#[cfg(feature = "database")]
pub use diesel_migrations;

#[cfg(feature = "redis")]
pub use bb8;
#[cfg(feature = "redis")]
pub use bb8_redis;
#[cfg(feature = "redis")]
pub use moka;
#[cfg(feature = "redis")]
pub use redis;

#[cfg(feature = "crypto")]
pub use argon2;
#[cfg(feature = "crypto")]
pub use hmac;
#[cfg(feature = "crypto")]
pub use openssl;
#[cfg(feature = "crypto")]
pub use ring;
#[cfg(feature = "crypto")]
pub use sha2;

/// Module for common helper functions and utilities
pub mod helpers {
    //! Common helper functions and utilities
}

/// Module for response handling and formatting
pub mod responses {
    //! Response handling and formatting utilities
}

/// Module for logging utilities
pub mod logging {
    //! Logging utilities and configuration
}

/// Module for validation utilities
pub mod validation {
    //! Validation utilities and custom validators
}

/// Module for error handling
pub mod errors {
    //! Custom error types and error handling utilities
}

// Placeholder modules that can be expanded based on your actual implementation
// You can replace these with actual implementations from your workspace crates

/// Re-export helper functionality
#[cfg(feature = "full")]
pub use crate::helpers::*;

/// Re-export response functionality
#[cfg(feature = "full")]
pub use crate::responses::*;

/// Re-export logging functionality
#[cfg(feature = "full")]
pub use crate::logging::*;

/// Re-export validation functionality
#[cfg(feature = "full")]
pub use crate::validation::*;

/// Re-export error handling functionality
#[cfg(feature = "full")]
pub use crate::errors::*;
