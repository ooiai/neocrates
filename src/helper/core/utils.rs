use rand::prelude::*;

pub struct Utils;

impl Utils {
    /// Generate a random token using UUIDv4.
    pub fn generate_token() -> String {
        let uuid = uuid::Uuid::new_v4();
        uuid.to_string()
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
