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
            name: "Parse Key-Value Pairs".to_string(),
            version: "1.0.0".to_string(),
            description: "Parse delimiter-separated key=value pairs (e.g., cookies, query strings, .env files)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "text".to_string(),
                data_type: DataType::String,
                description: "Text containing key-value pairs to parse".to_string(),
                required: true,
            },
            PortSpec {
                name: "pair_separator".to_string(),
                data_type: DataType::String,
                description: "Separator between pairs (default: ';' for cookies)".to_string(),
                required: false,
            },
            PortSpec {
                name: "key_value_separator".to_string(),
                data_type: DataType::String,
                description: "Separator between key and value (default: '=')".to_string(),
                required: false,
            },
            PortSpec {
                name: "trim_whitespace".to_string(),
                data_type: DataType::Bool,
                description: "Trim whitespace from keys and values (default: true)".to_string(),
                required: false,
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "keys".to_string(),
                data_type: DataType::StringListVal,
                description: "List of keys extracted".to_string(),
                required: true,
            },
            PortSpec {
                name: "values".to_string(),
                data_type: DataType::StringListVal,
                description: "List of values extracted (same length as keys)".to_string(),
                required: true,
            },
            PortSpec {
                name: "pair_count".to_string(),
                data_type: DataType::U32,
                description: "Number of key-value pairs parsed".to_string(),
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
        let text = inputs
            .iter()
            .find(|(name, _)| name == "text")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: text".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide text containing key-value pairs".to_string()),
            })?;

        let input_text = match &text.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'text', got {:?}", text.1),
                    input_name: Some("text".to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        };

        // Extract optional parameters
        let pair_sep = inputs
            .iter()
            .find(|(name, _)| name == "pair_separator")
            .and_then(|(_, v)| match v {
                Value::StringVal(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or(";");

        let kv_sep = inputs
            .iter()
            .find(|(name, _)| name == "key_value_separator")
            .and_then(|(_, v)| match v {
                Value::StringVal(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("=");

        let trim = inputs
            .iter()
            .find(|(name, _)| name == "trim_whitespace")
            .and_then(|(_, v)| match v {
                Value::BoolVal(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(true);

        // Parse key-value pairs
        let (keys, values, count) = parse_key_value_pairs(input_text, pair_sep, kv_sep, trim);

        Ok(vec![
            ("keys".to_string(), Value::StringListVal(keys)),
            ("values".to_string(), Value::StringListVal(values)),
            ("pair_count".to_string(), Value::U32Val(count)),
        ])
    }
}

/// Parse key-value pairs from text
fn parse_key_value_pairs(
    text: &str,
    pair_separator: &str,
    kv_separator: &str,
    trim_whitespace: bool,
) -> (Vec<String>, Vec<String>, u32) {
    if text.trim().is_empty() {
        return (vec![], vec![], 0);
    }

    let mut keys = Vec::new();
    let mut values = Vec::new();

    // Split by pair separator
    for pair in text.split(pair_separator) {
        let pair = if trim_whitespace {
            pair.trim()
        } else {
            pair
        };

        if pair.is_empty() {
            continue;
        }

        // Split by key-value separator (only first occurrence)
        if let Some(eq_pos) = pair.find(kv_separator) {
            let key = &pair[..eq_pos];
            let value = &pair[eq_pos + kv_separator.len()..];

            let key_final = if trim_whitespace {
                key.trim().to_string()
            } else {
                key.to_string()
            };

            let value_final = if trim_whitespace {
                value.trim().to_string()
            } else {
                value.to_string()
            };

            // Only add if key is not empty
            if !key_final.is_empty() {
                keys.push(key_final);
                values.push(value_final);
            }
        } else {
            // No separator found - treat as key with empty value
            let key_final = if trim_whitespace {
                pair.trim().to_string()
            } else {
                pair.to_string()
            };

            if !key_final.is_empty() {
                keys.push(key_final);
                values.push(String::new());
            }
        }
    }

    (keys, values, keys.len() as u32)
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cookies() {
        let (keys, values, count) = parse_key_value_pairs("session=abc123; user=alice", ";", "=", true);
        assert_eq!(keys, vec!["session", "user"]);
        assert_eq!(values, vec!["abc123", "alice"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_query_string() {
        let (keys, values, count) = parse_key_value_pairs("name=John&age=30&city=NYC", "&", "=", true);
        assert_eq!(keys, vec!["name", "age", "city"]);
        assert_eq!(values, vec!["John", "30", "NYC"]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_env_file() {
        let text = "DB_HOST=localhost\nDB_PORT=5432\nDB_NAME=myapp";
        let (keys, values, count) = parse_key_value_pairs(text, "\n", "=", true);
        assert_eq!(keys, vec!["DB_HOST", "DB_PORT", "DB_NAME"]);
        assert_eq!(values, vec!["localhost", "5432", "myapp"]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let (keys, values, count) = parse_key_value_pairs("key1 = value1 ; key2 = value2", ";", "=", true);
        assert_eq!(keys, vec!["key1", "key2"]);
        assert_eq!(values, vec!["value1", "value2"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_without_trim() {
        let (keys, values, count) = parse_key_value_pairs("key1 = value1 ", ";", "=", false);
        assert_eq!(keys, vec!["key1 "]);
        assert_eq!(values, vec![" value1 "]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_empty_values() {
        let (keys, values, count) = parse_key_value_pairs("key1=;key2=value2", ";", "=", true);
        assert_eq!(keys, vec!["key1", "key2"]);
        assert_eq!(values, vec!["", "value2"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_no_separator() {
        let (keys, values, count) = parse_key_value_pairs("flag1;flag2;flag3", ";", "=", true);
        assert_eq!(keys, vec!["flag1", "flag2", "flag3"]);
        assert_eq!(values, vec!["", "", ""]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_empty_string() {
        let (keys, values, count) = parse_key_value_pairs("", ";", "=", true);
        assert_eq!(keys.len(), 0);
        assert_eq!(values.len(), 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let (keys, values, count) = parse_key_value_pairs("   ", ";", "=", true);
        assert_eq!(keys.len(), 0);
        assert_eq!(values.len(), 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_parse_trailing_separator() {
        let (keys, values, count) = parse_key_value_pairs("key1=value1;key2=value2;", ";", "=", true);
        assert_eq!(keys, vec!["key1", "key2"]);
        assert_eq!(values, vec!["value1", "value2"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_multiple_separators() {
        let (keys, values, count) = parse_key_value_pairs("key1=value1;;key2=value2", ";", "=", true);
        assert_eq!(keys, vec!["key1", "key2"]);
        assert_eq!(values, vec!["value1", "value2"]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_value_with_separator() {
        let (keys, values, count) = parse_key_value_pairs("key=value=extra", ";", "=", true);
        assert_eq!(keys, vec!["key"]);
        assert_eq!(values, vec!["value=extra"]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_empty_key() {
        let (keys, values, count) = parse_key_value_pairs("=value;key2=value2", ";", "=", true);
        assert_eq!(keys, vec!["key2"]);
        assert_eq!(values, vec!["value2"]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_parse_custom_separators() {
        let (keys, values, count) = parse_key_value_pairs("a:1|b:2|c:3", "|", ":", true);
        assert_eq!(keys, vec!["a", "b", "c"]);
        assert_eq!(values, vec!["1", "2", "3"]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_csv_headers() {
        let (keys, values, count) = parse_key_value_pairs("name,age,city", ",", "=", true);
        assert_eq!(keys, vec!["name", "age", "city"]);
        assert_eq!(values, vec!["", "", ""]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_complex_query() {
        let text = "search=hello world&category=news&page=1";
        let (keys, values, count) = parse_key_value_pairs(text, "&", "=", true);
        assert_eq!(keys, vec!["search", "category", "page"]);
        assert_eq!(values, vec!["hello world", "news", "1"]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_multiline_env() {
        let text = "KEY1=value1\nKEY2=value2\nKEY3=value3";
        let (keys, values, count) = parse_key_value_pairs(text, "\n", "=", true);
        assert_eq!(keys, vec!["KEY1", "KEY2", "KEY3"]);
        assert_eq!(values, vec!["value1", "value2", "value3"]);
        assert_eq!(count, 3);
    }
}
