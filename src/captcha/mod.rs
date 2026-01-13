//! Captcha module for generating and validating various types of captchas
//!
//! This module provides captcha services including:
//! - Slider captcha
//! - Numeric captcha
//! - Alphanumeric captcha
//! - Image captcha

pub mod captcha_service;

pub use captcha_service::{CaptchaData, CaptchaService, CaptchaType};
