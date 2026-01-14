use once_cell::sync::Lazy;
use rand::prelude::*;
use regex::Regex;

pub struct Utils;

// ==================== Common Validators ====================
//
// Notes:
// - These validators are intended for common application validation, not for strict telecom compliance.
// - Mainland China mobile numbers change over time; keep regex updated if your business needs stricter rules.

static CN_MOBILE_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Mainland China mobile (simple): 11 digits, starts with 1, second digit 3-9
    // Examples: 13800138000
    Regex::new(r"^1[3-9]\d{9}$").expect("Failed to compile CN_MOBILE_REGEX")
});

static CN_LANDLINE_REGEX: Lazy<Regex> = Lazy::new(|| {
    // China landline (simple):
    // - With area code: 0xx-xxxxxxx / 0xxx-xxxxxxxx
    // - Without area code: xxxxxxx / xxxxxxxx
    // - Optional extension: -xxxx (1-6 digits)
    // Examples:
    // - 010-88886666
    // - 02088886666
    // - 0571-88886666-123
    // - 88886666
    Regex::new(r"^(?:(?:0\d{2,3}-?)?\d{7,8})(?:-\d{1,6})?$")
        .expect("Failed to compile CN_LANDLINE_REGEX")
});

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Practical email regex (not fully RFC 5322, but good for most cases)
    // - local part: letters/digits and common symbols
    // - domain: labels separated by dots, TLD length >= 2
    Regex::new(r"^[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]+@[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?(?:\.[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)+$")
        .expect("Failed to compile EMAIL_REGEX")
});

impl Utils {
    /// Generate a random token using UUIDv4.
    pub fn generate_token() -> String {
        let uuid = uuid::Uuid::new_v4();
        uuid.to_string()
    }

    /// Validate mainland China mobile number (common rule).
    ///
    /// Examples:
    /// - valid: "13800138000"
    /// - invalid: "12800138000", "1380013800", "+8613800138000"
    pub fn is_cn_mobile(phone: &str) -> bool {
        CN_MOBILE_REGEX.is_match(phone.trim())
    }

    /// Validate China landline number (common rule).
    ///
    /// Examples:
    /// - valid: "010-88886666", "02088886666", "0571-88886666-123", "88886666"
    /// - invalid: "010-8888666", "0a0-88886666"
    pub fn is_cn_landline(phone: &str) -> bool {
        CN_LANDLINE_REGEX.is_match(phone.trim())
    }

    /// Validate email address (practical rule).
    ///
    /// Examples:
    /// - valid: "user@example.com"
    /// - invalid: "user@", "@example.com", "user@example"
    pub fn is_email(email: &str) -> bool {
        EMAIL_REGEX.is_match(email.trim())
    }

    /// Validate "phone-like" input: either mainland mobile or landline.
    pub fn is_cn_phone(phone: &str) -> bool {
        let p = phone.trim();
        Self::is_cn_mobile(p) || Self::is_cn_landline(p)
    }

    // Mask phone numbers differently based on input length
    // 11-digit number: 138****1234
    // 10-digit number: 138***1234
    // 7-digit number: 13***1234
    // Other lengths: no masking
    pub fn mask_phone_number(phone: &str) -> String {
        let len = phone.len();
        match len {
            11 => {
                let mut masked_phone = phone.to_string();
                masked_phone.replace_range(3..7, "****");
                masked_phone
            }
            10 => {
                let mut masked_phone = phone.to_string();
                masked_phone.replace_range(3..6, "***");
                masked_phone
            }
            7 => {
                let mut masked_phone = phone.to_string();
                masked_phone.replace_range(2..5, "***");
                masked_phone
            }
            _ => phone.to_string(), // For other lengths, do not mask
        }
    }

    // Generate a random username
    // pub fn generate_username() -> String {
    //     let mut rng = rand::thread_rng();
    //     let username_length = rng.gen_range(6..=12);
    //     rng.sample_iter(&Alphanumeric)
    //         .take(username_length)
    //         .map(char::from)
    //         .collect()
    // }
    //

    /// Select a name using weighted randomness
    ///
    /// # Parameters
    /// - `names`: list of names
    /// - `weights`: list of weights (one-to-one with names)
    ///
    /// fn main() {
    //     let names = vec![
    //         "Alice".to_string(),
    //         "Bob".to_string(),
    //         "Charlie".to_string(),
    //     ];
    //     let weights = vec![1, 3, 6];

    //     if let Some(name) = utils::weighted_random::weighted_random_name(&names, &weights) {
    //         println!("Weighted randomly selected name: {}", name);
    //     }
    // }
    /// # Returns
    /// - `Option<String>`: randomly selected name
    pub fn weighted_random_name(names: &[String], weights: &[usize]) -> Option<String> {
        if names.is_empty() || names.len() != weights.len() {
            return None;
        }
        let total: usize = weights.iter().sum();
        if total == 0 {
            return None;
        }
        let mut rng = rand::rng();
        let mut target = rng.random_range(0..total);
        for (name, &weight) in names.iter().zip(weights.iter()) {
            if target < weight {
                return Some(name.clone());
            }
            target -= weight;
        }
        None
    }

    /// Randomly pick a name from the list
    ///
    /// # Parameters
    /// - `names`: slice of names
    ///
    /// # Returns
    /// - `Option<String>`: randomly selected name, or None if the list is empty
    pub fn random_name(names: &[String]) -> Option<String> {
        // Create a thread-local RNG
        let mut rng = rand::rng();
        // Choose a random element and clone as String
        names.choose(&mut rng).cloned()
    }

    /// Parse string into usize; return default if parsing fails
    //
    /// # Parameters
    pub fn to_usize_or(s: &str, default: usize) -> usize {
        s.trim().parse::<usize>().unwrap_or(default)
    }
}
