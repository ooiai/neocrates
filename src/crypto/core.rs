use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use anyhow::Error;
use base64::{Engine as _, engine::general_purpose};
use hex::encode;
use rand::Rng;
use tracing::warn;

pub struct Crypto;

impl Crypto {
    ///
    /// MD5 hash function.
    ///
    /// # Arguments
    /// * `data` - The data to hash.
    ///
    /// # Returns
    /// * `String` - The MD5 hash of the data.
    pub fn md5_string(data: &str) -> String {
        let digest = md5::compute(data);
        format!("{:x}", digest)
    }

    /// Hashes a password using Argon2id (latest recommended practice).
    ///
    /// # Arguments
    /// * `password` - The plaintext password to hash.
    ///
    /// # Returns
    /// * `Ok(String)` - On success, returns the PHC format hash string.
    /// * `Err(password_hash::Error)` - On failure, returns an error.
    pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
        // SaltString uses a cryptographically secure RNG (OsRng) to automatically generate a salt.
        let salt = SaltString::generate(&mut OsRng);

        // Argon2::default() uses the recommended secure parameters.
        let argon2 = Argon2::default();

        // Perform the hash calculation.
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(password_hash)
    }

    /// Verifies if a password matches a given hash.
    pub fn verify_password(password: &str, hash: &str) -> bool {
        // Parse the hash string into a PasswordHash struct.
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(hash) => hash,
            Err(_) => return false, // If the hash format is invalid, return false immediately.
        };

        // Verify the password against the parsed hash.
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    pub fn generate_basic_auth_key(key: &str) -> String {
        let first_encode = general_purpose::STANDARD.encode(key.as_bytes());
        general_purpose::STANDARD.encode(first_encode.as_bytes())
    }

    pub fn decode_basic_auth_key(encoded_key: &str) -> Result<String, Error> {
        warn!(
            "...「decode_basic_auth_key」encoded_key: {} ...",
            encoded_key
        );
        let first_decode = general_purpose::STANDARD.decode(encoded_key)?;
        let second_decode = general_purpose::STANDARD.decode(&first_decode)?;
        String::from_utf8(second_decode).map_err(Error::from)
    }

    // zstd_compress
    pub fn zstd_compress(data: &[u8]) -> Result<Vec<u8>, Error> {
        let compressed = zstd::stream::encode_all(data, 0)?;
        Ok(compressed)
    }

    // Generate a random 32-character AES key in hexadecimal format.
    pub fn generate_aes_key() -> String {
        let mut key = [0u8; 32];
        rand::rng().fill(&mut key);
        let hex_string = encode(&key);
        // Ensure the string length is 32
        let hex_string = if hex_string.len() >= 32 {
            hex_string[..32].to_string()
        } else {
            hex_string
        };
        hex_string
    }
}

// fn main() {
//     let my_password = "a-very-secure-password-123";

//     // 1. Hash the password.
//     let hashed_password = match Crypto::hash_password(my_password) {
//         Ok(h) => {
//             println!("Password hashed successfully: {}", h);
//             h
//         }
//         Err(e) => {
//             eprintln!("Password hashing failed: {}", e);
//             return;
//         }
//     };

//     // 2. Verify the correct password.
//     let is_valid = Crypto::verify_password(my_password, &hashed_password);
//     println!("Verification result for correct password: {}", is_valid); // Should print true

//     // 3. Verify an incorrect password.
//     let is_valid_wrong = Crypto::verify_password("wrong-password", &hashed_password);
//     println!(
//         "Verification result for incorrect password: {}",
//         is_valid_wrong
//     ); // Should print false
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_generate_basic_auth_key() {
        let key = "topedu::auth";
        let encoded_key = Crypto::generate_basic_auth_key(key);
        println!("{}", encoded_key);
        // assert_eq!(encoded_key, "dGVzdF9rZXk="); // "test_key" encoded in base64
    }

    #[test]
    fn test_decode_basic_auth_key() {
        let key = "topedu::auth";
        let encoded_key = Crypto::generate_basic_auth_key(key);
        let decoded_key =
            Crypto::decode_basic_auth_key(&encoded_key).expect("Failed to decode key");
        println!("encoded_key: {}", encoded_key);
        println!("decoded_key: {}", decoded_key);
    }

    #[test]
    fn test_new() {
        assert!(Crypto::generate_basic_auth_key("test").len() > 0);
    }

    #[test]
    fn test_ases_generate_aes_key() {
        let key = Crypto::generate_aes_key();
        println!("the aes_key :{}", key)
    }

    #[test]
    fn test_md5_string() {
        let data = "hello world";
        let md5 = Crypto::md5_string(data);
        println!("md5: {}", md5);
    }
}
