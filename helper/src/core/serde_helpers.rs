use core::fmt;
use std::str::FromStr;

use chrono::Local;
use regex::Regex;
use serde::{
    Deserialize, Deserializer, Serializer,
    de::{self, DeserializeOwned},
};
use serde_json::Value;
use std::fmt::Debug;

use super::{hashid, snowflake::generate_snowflake_id};

pub const PAGE_SIZES: [i64; 7] = [10, 20, 30, 40, 50, 100, 200];
pub const DEFAULT_PAGE_SIZE: i64 = 10;
pub const ORDER_VALUES: [&str; 3] = ["asc", "desc", "hidden"];
pub const DEFAULT_ORDER: &str = "desc";
pub const MIN_PAGE_NUMBER: i64 = 1;
pub const MAX_PAGE_NUMBER: i64 = 1000;

///
/// 解析 i64 类型
///
pub fn deserialize_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value {
        Value::Number(num) => num
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom("Invalid number")),
        Value::String(s) => hashid::decode_i64(s.as_str())
            .to_string()
            .parse::<i64>()
            .map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom("Expected a number or string")),
    }
}

///
/// 解析 Vec<i64> 类型
///
pub fn deserialize_vec_i64<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                let parsed_id = match item {
                    Value::Number(num) => num
                        .as_i64()
                        .ok_or_else(|| serde::de::Error::custom("Invalid number"))?,
                    Value::String(s) => hashid::decode_i64(s.as_str())
                        .to_string()
                        .parse::<i64>()
                        .map_err(serde::de::Error::custom)?,
                    _ => {
                        return Err(serde::de::Error::custom(
                            "Expected a number or string in array",
                        ));
                    }
                };
                result.push(parsed_id);
            }
            Ok(result)
        }
        _ => Err(serde::de::Error::custom("Expected an array")),
    }
}

///
/// 解析 option i64 类型
///
pub fn deserialize_option_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    // tracing::warn!("deserialize_option_i64 value:{:?}", value);
    match value {
        Value::Null => Ok(None),
        Value::Number(num) => {
            // Try to extract the i64 value from the number
            num.as_i64()
                .map(Some)
                .ok_or_else(|| serde::de::Error::custom("Invalid number"))
        }
        Value::String(s) => {
            if s.is_empty() {
                return Ok(None);
            }
            // Decode the string to u64, then convert to i64
            let decoded = hashid::decode_i64(s.as_str())
                .to_string()
                .parse::<i64>()
                .map_err(|_| serde::de::Error::custom("Failed to decode string"))?;

            // Try to convert u64 to i64
            i64::try_from(decoded)
                .map(Some)
                .map_err(|_| serde::de::Error::custom("Decoded value is out of range for i64"))
        }
        _ => Err(serde::de::Error::custom(
            "Expected a null, number, or string",
        )),
    }
}

/// Any 类型转 option i64
/// 支持 null, number, string
///
pub fn deserialize_option_any_to_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let val = Option::<Value>::deserialize(deserializer)?;
    match val {
        None => Ok(None),
        Some(Value::Number(n)) => n
            .as_i64()
            .map(Some)
            .ok_or_else(|| Error::custom("Invalid number")),
        Some(Value::String(s)) => {
            let s = s.trim();
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i64>().map(Some).map_err(Error::custom)
            }
        }
        Some(other) => Err(Error::custom(format!("Unexpected type: {:?}", other))),
    }
}

/// Any 类型转 option f64
/// 支持 null, number, string
pub fn deserialize_option_any_to_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let val = Option::<Value>::deserialize(deserializer)?;
    match val {
        None => Ok(None),
        Some(Value::Number(n)) => n
            .as_f64()
            .map(Some)
            .ok_or_else(|| Error::custom("Invalid number")),
        Some(Value::String(s)) => {
            let s = s.trim();
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<f64>().map(Some).map_err(Error::custom)
            }
        }
        Some(other) => Err(Error::custom(format!("Unexpected type: {:?}", other))),
    }
}

///
/// 序列化 i64 类型
///
pub fn serialize_i64<S>(x: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encode_u64 = hashid::encode_i64(*x);
    serializer.serialize_str(&encode_u64)
}

///
/// 序列化 option i64 类型
///
pub fn serialize_option_i64<S>(x: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(value) => {
            // tracing::error!("serialize_option_i64 value: {}", value);
            let encode_u64 = hashid::encode_i64(*value);
            // tracing::error!("serialize_option_i64: {}", encode_u64);
            serializer.serialize_str(&encode_u64)
        }
        None => serializer.serialize_none(),
    }
}

///
/// 生成 snowflake id
///
pub fn snowflake_id() -> Option<i64> {
    Some(generate_snowflake_id())
}

///
/// 默认pid值为-1
///
pub fn default_pid() -> i64 {
    -1
}

///
/// 默认pid值为-1
///
pub fn default_option_pid() -> Option<i64> {
    Some(-1)
}

///
/// 默认当前时间
///
pub fn now_datetime() -> Option<chrono::NaiveDateTime> {
    Some(Local::now().naive_local())
}

///
/// 默认当前页current
///
pub fn current() -> Option<i64> {
    Some(1)
}

///
/// 默认当前页zie
///
pub fn size() -> Option<i64> {
    Some(10)
}

///
/// 字符串转 i16
///
pub fn string_to_i16<'de, D>(deserializer: D) -> Result<i16, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

///
/// 字符串转 i16 option
///
pub fn string_to_i16_option<'de, D>(deserializer: D) -> Result<Option<i16>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
    }
}

///
/// 空字符串转 None
///
// pub fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s: Option<String> = Option::deserialize(deserializer)?;
//     Ok(s.filter(|val| !val.is_empty()))
// }

pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

///
/// 验证并规范化分页大小
/// 如果size不在允许范围内，返回默认值
///
pub fn normalize_page_size<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let size: Option<i64> = Option::deserialize(deserializer)?;
    Ok(size.map(|s| {
        if PAGE_SIZES.contains(&s) {
            s
        } else {
            DEFAULT_PAGE_SIZE
        }
    }))
}

///
/// 验证分页大小
///
pub fn validate_page_size(size: &Option<i64>) -> Result<(), validator::ValidationError> {
    if let Some(s) = size {
        if !PAGE_SIZES.contains(s) {
            return Err(validator::ValidationError::new("invalid_page_size"));
        }
    }
    Ok(())
}

///
/// 验证并规范化排序方式
/// 如果order不在允许范围内，返回默认值
///
pub fn normalize_order<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let order: Option<String> = Option::deserialize(deserializer)?;
    Ok(order.map(|s| {
        let lowercase_order = s.to_lowercase();
        if ORDER_VALUES.contains(&lowercase_order.as_str()) {
            lowercase_order
        } else {
            DEFAULT_ORDER.to_string()
        }
    }))
}

///
/// 验证并规范化当前页码
/// 如果current小于1，返回1
/// 如果current大于500，返回500
///
pub fn normalize_current<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let current: Option<i64> = Option::deserialize(deserializer)?;
    Ok(current.map(|c| {
        if c < MIN_PAGE_NUMBER {
            MIN_PAGE_NUMBER
        } else if c > MAX_PAGE_NUMBER {
            // MAX_PAGE_NUMBER
            MIN_PAGE_NUMBER
        } else {
            c
        }
    }))
}

///
/// 验证并清理搜索键
/// 只允许指定的搜索键列表，其他返回None
///
pub fn normalize_search_key_with<'de, D>(
    allowed_keys: &'static [&'static str],
) -> impl Fn(D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    move |deserializer| {
        let key: Option<String> = Option::deserialize(deserializer)?;
        Ok(key.and_then(|k| {
            if allowed_keys.contains(&k.as_str()) {
                Some(k)
            } else {
                None
            }
        }))
    }
}

///
/// 验证并清理搜索值
///
pub fn normalize_search_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    Ok(value.and_then(|v| {
        let cleaned = v.trim();
        if cleaned.is_empty() {
            None
        } else {
            let cleaned: String = cleaned
                .chars()
                .filter(|c| {
                    c.is_alphanumeric()
                        || c.is_whitespace()
                        || *c == '-'
                        || *c == '_'
                        || *c == '.'
                        || *c == '@'
                })
                .take(100)
                .collect();

            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            }
        }
    }))
}

///
/// 验证并清理英文字符串
///
pub fn validate_english(input: &str) -> Result<(), validator::ValidationError> {
    let re = Regex::new(r"^[a-zA-Z]+$").expect("Failed to compile regex");
    if !re.is_match(input) {
        return Err(validator::ValidationError::new("is_not_english"));
    }
    Ok(())
}

///
/// 验证并清理英文数字字符串
/// 必须英文开始
///
pub fn validate_english_number(input: &str) -> Result<(), validator::ValidationError> {
    let re = Regex::new(r"^[A-Za-z][A-Za-z0-9]*$").expect("Failed to compile regex");
    if !re.is_match(input) {
        return Err(validator::ValidationError::new("is_not_english_number"));
    }
    Ok(())
}

/// Generic validator that verifies the incoming JSON `Value` can be deserialized into `T`,
/// and returns the `Value`.
pub fn validate_json<'de, D, T>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Debug,
{
    // Deserialize into a Value first
    let value = Value::deserialize(deserializer)?;

    // Try to deserialize the Value into T for validation
    match serde_json::from_value::<T>(value.clone()) {
        Ok(_) => Ok(value),
        Err(e) => Err(serde::de::Error::custom(format!(
            "JSON validation failed: {}",
            e
        ))),
    }
}

/// Deserialize a flexible JSON value.
///
pub fn deserialize_flexible_json<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Value>::deserialize(deserializer)?;
    Ok(match opt {
        Some(Value::String(s)) => serde_json::from_str(&s).ok(),
        other => other,
    })
}

#[cfg(test)]
mod tests {
    use crate::core::hashid::{decode_i64, encode_i64};

    #[test]
    fn test_encode() {
        let n: i64 = 594031369676525600;
        let value = encode_i64(n);
        println!("Encoded value: {}", value);
    }

    #[test]
    fn test_decode() {
        let n: &str = "H8Q8WT584400";
        let value = decode_i64(n);
        println!("Decoded value: {}", value);
    }
}
