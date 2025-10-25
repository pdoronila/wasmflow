wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "JSON Build Object".to_string(),
            version: "1.0.0".to_string(),
            description: "Build JSON object from lists of keys and values".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "keys".to_string(),
                data_type: DataType::StringListVal,
                description: "List of keys for the JSON object".to_string(),
                required: true,
            },
            PortSpec {
                name: "values".to_string(),
                data_type: DataType::StringListVal,
                description: "List of values (same length as keys)".to_string(),
                required: true,
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "json".to_string(),
                data_type: DataType::String,
                description: "JSON object string".to_string(),
                required: true,
            },
            PortSpec {
                name: "pair_count".to_string(),
                data_type: DataType::U32,
                description: "Number of key-value pairs in the object".to_string(),
                required: true,
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract keys
        let keys_input = inputs
            .iter()
            .find(|(name, _)| name == "keys")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: keys".to_string(),
                input_name: Some("keys".to_string()),
                recovery_hint: Some("Provide a list of keys for the JSON object".to_string()),
            })?;

        let keys = match &keys_input.1 {
            Value::StringListVal(list) => list,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected StringListVal for 'keys', got {:?}", keys_input.1),
                    input_name: Some("keys".to_string()),
                    recovery_hint: Some("Provide a list of strings".to_string()),
                });
            }
        };

        // Extract values
        let values_input = inputs
            .iter()
            .find(|(name, _)| name == "values")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: values".to_string(),
                input_name: Some("values".to_string()),
                recovery_hint: Some("Provide a list of values for the JSON object".to_string()),
            })?;

        let values = match &values_input.1 {
            Value::StringListVal(list) => list,
            _ => {
                return Err(ExecutionError {
                    message: format!(
                        "Expected StringListVal for 'values', got {:?}",
                        values_input.1
                    ),
                    input_name: Some("values".to_string()),
                    recovery_hint: Some("Provide a list of strings".to_string()),
                });
            }
        };

        // Validate lengths match
        if keys.len() != values.len() {
            return Err(ExecutionError {
                message: format!(
                    "Keys and values must have the same length (keys: {}, values: {})",
                    keys.len(),
                    values.len()
                ),
                input_name: None,
                recovery_hint: Some(
                    "Ensure the keys and values lists have the same number of elements"
                        .to_string(),
                ),
            });
        }

        // Build JSON object
        let (json, count) = build_json_object(keys, values);

        Ok(vec![
            ("json".to_string(), Value::StringVal(json)),
            ("pair_count".to_string(), Value::U32Val(count)),
        ])
    }
}

/// Build JSON object from keys and values
fn build_json_object(keys: &[String], values: &[String]) -> (String, u32) {
    if keys.is_empty() {
        return ("{}".to_string(), 0);
    }

    let mut pairs = Vec::new();
    for (key, value) in keys.iter().zip(values.iter()) {
        let escaped_key = escape_json(key);
        let escaped_value = escape_json(value);
        pairs.push(format!("\"{}\":\"{}\"", escaped_key, escaped_value));
    }

    (format!("{{{}}}", pairs.join(",")), keys.len() as u32)
}

/// Escape special characters for JSON strings
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_empty_object() {
        let keys: Vec<String> = vec![];
        let values: Vec<String> = vec![];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, "{}");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_build_single_pair() {
        let keys = vec!["name".to_string()];
        let values = vec!["Alice".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"name":"Alice"}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_build_multiple_pairs() {
        let keys = vec!["name".to_string(), "age".to_string(), "city".to_string()];
        let values = vec!["Bob".to_string(), "30".to_string(), "NYC".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"name":"Bob","age":"30","city":"NYC"}"#);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_escape_quotes() {
        let keys = vec!["message".to_string()];
        let values = vec![r#"He said "hello""#.to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"message":"He said \"hello\""}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_backslash() {
        let keys = vec!["path".to_string()];
        let values = vec![r"C:\Users\test".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"path":"C:\\Users\\test"}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_newline() {
        let keys = vec!["text".to_string()];
        let values = vec!["line1\nline2".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"text":"line1\nline2"}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_tab() {
        let keys = vec!["data".to_string()];
        let values = vec!["col1\tcol2".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"data":"col1\tcol2"}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_carriage_return() {
        let keys = vec!["text".to_string()];
        let values = vec!["line1\rline2".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"text":"line1\rline2"}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_numeric_values() {
        let keys = vec!["count".to_string(), "price".to_string()];
        let values = vec!["42".to_string(), "19.99".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"count":"42","price":"19.99"}"#);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_boolean_values() {
        let keys = vec!["active".to_string()];
        let values = vec!["true".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"active":"true"}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_empty_values() {
        let keys = vec!["name".to_string(), "email".to_string()];
        let values = vec!["".to_string(), "user@example.com".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"name":"","email":"user@example.com"}"#);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_special_chars_in_key() {
        let keys = vec!["user-name".to_string(), "first.last".to_string()];
        let values = vec!["john".to_string(), "jane".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"user-name":"john","first.last":"jane"}"#);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_unicode() {
        let keys = vec!["emoji".to_string()];
        let values = vec!["🚀".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"emoji":"🚀"}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_complex_escaping() {
        let keys = vec!["data".to_string()];
        let values = vec!["\"quoted\"\n\ttab\\slash".to_string()];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(json, r#"{"data":"\\\"quoted\\\"\n\ttab\\\\slash"}"#);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_http_headers() {
        let keys = vec![
            "content-type".to_string(),
            "cache-control".to_string(),
        ];
        let values = vec![
            "application/json".to_string(),
            "no-cache".to_string(),
        ];
        let (json, count) = build_json_object(&keys, &values);
        assert_eq!(
            json,
            r#"{"content-type":"application/json","cache-control":"no-cache"}"#
        );
        assert_eq!(count, 2);
    }

    #[test]
    fn test_many_pairs() {
        let keys: Vec<String> = (0..10).map(|i| format!("key{}", i)).collect();
        let values: Vec<String> = (0..10).map(|i| format!("value{}", i)).collect();
        let (json, count) = build_json_object(&keys, &values);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert_eq!(count, 10);
        // Check a few pairs are present
        assert!(json.contains(r#""key0":"value0""#));
        assert!(json.contains(r#""key9":"value9""#));
    }
}
