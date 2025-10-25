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
            name: "JSON Parse Flat Object".to_string(),
            version: "1.0.0".to_string(),
            description: "Parse flat JSON object into lists of keys and values".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "json".to_string(),
            data_type: DataType::String,
            description: "JSON object string (flat, no nested objects)".to_string(),
            required: true,
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "keys".to_string(),
                data_type: DataType::StringListVal,
                description: "List of keys from the JSON object".to_string(),
                required: true,
            },
            PortSpec {
                name: "values".to_string(),
                data_type: DataType::StringListVal,
                description: "List of values (same length as keys)".to_string(),
                required: true,
            },
            PortSpec {
                name: "pair_count".to_string(),
                data_type: DataType::U32,
                description: "Number of key-value pairs".to_string(),
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
        // Extract required input
        let json_input = inputs
            .iter()
            .find(|(name, _)| name == "json")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: json".to_string(),
                input_name: Some("json".to_string()),
                recovery_hint: Some("Provide a JSON object string to parse".to_string()),
            })?;

        let json_str = match &json_input.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'json', got {:?}", json_input.1),
                    input_name: Some("json".to_string()),
                    recovery_hint: Some("Provide a JSON string".to_string()),
                });
            }
        };

        // Parse JSON object
        let (keys, values, count) = parse_flat_json_object(json_str)?;

        Ok(vec![
            ("keys".to_string(), Value::StringListVal(keys)),
            ("values".to_string(), Value::StringListVal(values)),
            ("pair_count".to_string(), Value::U32Val(count)),
        ])
    }
}

/// Parse flat JSON object into key-value pairs
///
/// Supports only flat objects with string values.
/// Does not support nested objects or arrays.
fn parse_flat_json_object(json: &str) -> Result<(Vec<String>, Vec<String>, u32), ExecutionError> {
    let trimmed = json.trim();

    // Check for empty object
    if trimmed == "{}" {
        return Ok((vec![], vec![], 0));
    }

    // Remove outer braces
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(ExecutionError {
            message: "Invalid JSON: must start with { and end with }".to_string(),
            input_name: Some("json".to_string()),
            recovery_hint: Some("Provide a valid JSON object (e.g., {\"key\":\"value\"})".to_string()),
        });
    }

    let content = &trimmed[1..trimmed.len() - 1].trim();

    if content.is_empty() {
        return Ok((vec![], vec![], 0));
    }

    let mut keys = Vec::new();
    let mut values = Vec::new();

    // Split by commas (simple approach - doesn't handle commas in strings)
    // For a simple flat object parser, this is acceptable
    for pair in content.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        // Find the colon separator
        let colon_pos = pair.find(':').ok_or_else(|| ExecutionError {
            message: format!("Invalid JSON pair (no colon): {}", pair),
            input_name: Some("json".to_string()),
            recovery_hint: Some("Each pair must be in format \"key\":\"value\"".to_string()),
        })?;

        let key_part = pair[..colon_pos].trim();
        let value_part = pair[colon_pos + 1..].trim();

        // Remove quotes from key
        let key = if key_part.starts_with('"') && key_part.ends_with('"') {
            unescape_json_string(&key_part[1..key_part.len() - 1])
        } else {
            return Err(ExecutionError {
                message: format!("Invalid JSON key (must be quoted): {}", key_part),
                input_name: Some("json".to_string()),
                recovery_hint: Some("Keys must be quoted strings".to_string()),
            });
        };

        // Remove quotes from value
        let value = if value_part.starts_with('"') && value_part.ends_with('"') {
            unescape_json_string(&value_part[1..value_part.len() - 1])
        } else {
            // Support unquoted values (numbers, booleans, null)
            value_part.to_string()
        };

        keys.push(key);
        values.push(value);
    }

    Ok((keys, values, keys.len() as u32))
}

/// Unescape JSON string escapes
fn unescape_json_string(s: &str) -> String {
    s.replace(r#"\""#, "\"")
        .replace(r"\\", "\\")
        .replace(r"\n", "\n")
        .replace(r"\r", "\r")
        .replace(r"\t", "\t")
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_object() {
        let (keys, values, count) = parse_flat_json_object("{}").unwrap();
        assert_eq!(keys.len(), 0);
        assert_eq!(values.len(), 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_parse_single_pair() {
        let (keys, values, count) = parse_flat_json_object(r#"{"name":"Alice"}"#).unwrap();
        assert_eq!(keys, vec!["name"]);
        assert_eq!(values, vec!["Alice"]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_multiple_pairs() {
        let (keys, values, count) =
            parse_flat_json_object(r#"{"name":"Bob","age":"30","city":"NYC"}"#).unwrap();
        assert_eq!(keys, vec!["name", "age", "city"]);
        assert_eq!(values, vec!["Bob", "30", "NYC"]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let (keys, values, count) =
            parse_flat_json_object(r#"{ "key1" : "value1" , "key2" : "value2" }"#).unwrap();
        assert_eq!(keys, vec!["key1", "key2"]);
        assert_eq!(values, vec!["value1", "value2"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_numeric_values() {
        let (keys, values, count) = parse_flat_json_object(r#"{"count":42,"price":19.99}"#).unwrap();
        assert_eq!(keys, vec!["count", "price"]);
        assert_eq!(values, vec!["42", "19.99"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_boolean_values() {
        let (keys, values, count) = parse_flat_json_object(r#"{"active":true,"disabled":false}"#).unwrap();
        assert_eq!(keys, vec!["active", "disabled"]);
        assert_eq!(values, vec!["true", "false"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_null_value() {
        let (keys, values, count) = parse_flat_json_object(r#"{"data":null}"#).unwrap();
        assert_eq!(keys, vec!["data"]);
        assert_eq!(values, vec!["null"]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_empty_string_value() {
        let (keys, values, count) = parse_flat_json_object(r#"{"name":""}"#).unwrap();
        assert_eq!(keys, vec!["name"]);
        assert_eq!(values, vec![""]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_escaped_quotes() {
        let (keys, values, count) = parse_flat_json_object(r#"{"message":"He said \"hello\""}"#).unwrap();
        assert_eq!(keys, vec!["message"]);
        assert_eq!(values, vec!["He said \"hello\""]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_escaped_newline() {
        let (keys, values, count) = parse_flat_json_object(r#"{"text":"line1\nline2"}"#).unwrap();
        assert_eq!(keys, vec!["text"]);
        assert_eq!(values, vec!["line1\nline2"]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_special_chars_in_key() {
        let (keys, values, count) =
            parse_flat_json_object(r#"{"user-name":"john","first.last":"jane"}"#).unwrap();
        assert_eq!(keys, vec!["user-name", "first.last"]);
        assert_eq!(values, vec!["john", "jane"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_invalid_no_braces() {
        let result = parse_flat_json_object(r#"name":"Alice""#);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("must start with"));
    }

    #[test]
    fn test_parse_invalid_no_quotes_on_key() {
        let result = parse_flat_json_object(r#"{name:"Alice"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("must be quoted"));
    }

    #[test]
    fn test_parse_invalid_no_colon() {
        let result = parse_flat_json_object(r#"{"name""Alice"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("no colon"));
    }

    #[test]
    fn test_parse_multiline_formatting() {
        let json = r#"{
            "name": "Alice",
            "age": "25"
        }"#;
        let (keys, values, count) = parse_flat_json_object(json).unwrap();
        assert_eq!(keys, vec!["name", "age"]);
        assert_eq!(values, vec!["Alice", "25"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_unicode() {
        let (keys, values, count) = parse_flat_json_object(r#"{"emoji":"🚀"}"#).unwrap();
        assert_eq!(keys, vec!["emoji"]);
        assert_eq!(values, vec!["🚀"]);
        assert_eq!(count, 1);
    }
}
