use serde_json::{Number, Value};

/// Normalize all numeric values in a serde_json::Value to a specified number of decimal places.
///
/// # Arguments
/// - `value`: The input serde_json::Value which may contain nested structures.
/// - `decimals`: The number of decimal places to round to.
/// # Returns
/// - A new serde_json::Value with all numeric values rounded to the specified decimal places.
/// # Examples
/// let data = serde_json::json!({
///     "a": 1.23456,
///     "b": [2.34567, 3.45678],
///     "c": {"d": 4.56789}
/// });
/// let normalized = normalize_numbers(data, 2);
/// assert_eq!(normalized, serde_json::json!({
///     "a": 1.23,
///     "b": [2.35, 3.46],
///     "c": {"d": 4.57}
/// }));
///
pub fn normalize_numbers(value: Value, decimals: u32) -> Value {
    match value {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                let factor = 10f64.powi(decimals as i32);
                let rounded = (f * factor).round() / factor;
                Value::Number(Number::from_f64(rounded).unwrap())
            } else {
                Value::Number(n)
            }
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|v| normalize_numbers(v, decimals))
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, normalize_numbers(v, decimals)))
                .collect(),
        ),
        other => other,
    }
}
