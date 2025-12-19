use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for matching English words.
pub static ENGLISH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z]+$").expect("Failed to compile regex"));
