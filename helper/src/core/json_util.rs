use serde::Deserialize;
use serde_json::Value;

/// Validate and convert JSON Value to a specific type
///
/// # Errors
/// Returns an error string if validation or conversion fails
pub fn validate_and_convert<T>(value: Value) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value::<T>(value)
        .map_err(|e| format!("JSON Validation and conversion failed: {}", e))
}

/// Validate JSON Value against a specific type without conversion
///
/// # Errors
/// Returns an error string if validation fails
pub fn validate_json<T>(value: Value) -> Result<Value, String>
where
    T: for<'de> Deserialize<'de>,
{
    match serde_json::from_value::<T>(value.clone()) {
        Ok(_) => Ok(value),
        Err(e) => Err(format!("JSON Validation failed: {}", e)),
    }
}

/// Parse a JSON string into a specific type
///
/// # Errors
/// Returns an error string if parsing fails
pub fn parse_json<T>(json_str: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str::<T>(json_str).map_err(|e| format!("JSON Parsing failed: {}", e))
}
