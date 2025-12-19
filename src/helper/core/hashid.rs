pub fn encode_u64(n: u64) -> String {
    let mut buf = Vec::with_capacity(13);
    crockford::encode_into(n, &mut buf);
    let result = std::str::from_utf8(&buf).expect("Failed to convert bytes to string");
    result.to_string()
}

pub fn encode_i64(n: i64) -> String {
    let mut buf = Vec::with_capacity(13);
    let n = n as u64;
    crockford::encode_into(n, &mut buf);
    let result = std::str::from_utf8(&buf).expect("Failed to convert bytes to string");
    result.to_string()
}

pub fn decode_u64(s: &str) -> u64 {
    crockford::decode(s).expect("Failed to decode string")
}

pub fn decode_i64(s: &str) -> i64 {
    let n = decode_u64(s);
    n as i64
}

// test
#[cfg(test)]
mod tests {
    use crate::core::{
        hashid::{decode_i64, encode_i64},
        snowflake::generate_snowflake_id,
    };

    #[test]
    fn test_encode_decode() {
        let n = -1;
        let encoded = encode_i64(n);
        println!("encoded:{}", encoded);
        let decoded = decode_i64(&encoded);
        println!("decoded:{}", decoded);
        assert_eq!(n, decoded);
    }

    #[test]
    fn test_snowflake_encode_decode() {
        let n = generate_snowflake_id();
        let encoded = encode_i64(n);
        println!("encoded:{}", encoded);
        let decoded = decode_i64(&encoded);
        println!("decoded:{}", decoded);
        assert_eq!(n, decoded);
    }
}
