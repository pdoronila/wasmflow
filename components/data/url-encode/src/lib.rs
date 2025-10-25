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
            name: "URL Encode".to_string(),
            version: "1.0.0".to_string(),
            description: "Encode text to URL-safe format (percent-encoding)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "text".to_string(),
                data_type: DataType::String,
                description: "Text to URL-encode".to_string(),
                required: true,
            },
            PortSpec {
                name: "encode_spaces_as_plus".to_string(),
                data_type: DataType::Bool,
                description: "Encode spaces as + instead of %20 (default: false)".to_string(),
                required: false,
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "encoded".to_string(),
                data_type: DataType::String,
                description: "URL-encoded text".to_string(),
                required: true,
            },
            PortSpec {
                name: "encode_count".to_string(),
                data_type: DataType::U32,
                description: "Number of characters encoded".to_string(),
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
                recovery_hint: Some("Provide text to URL-encode".to_string()),
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

        // Extract optional encode_spaces_as_plus (default: false)
        let encode_plus = inputs
            .iter()
            .find(|(name, _)| name == "encode_spaces_as_plus")
            .and_then(|(_, v)| match v {
                Value::BoolVal(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);

        // Encode URL
        let (encoded, count) = url_encode(input_text, encode_plus);

        Ok(vec![
            ("encoded".to_string(), Value::StringVal(encoded)),
            ("encode_count".to_string(), Value::U32Val(count)),
        ])
    }
}

/// URL-encode a string
///
/// Encodes all characters except:
/// - Alphanumeric: A-Z, a-z, 0-9
/// - Unreserved: - _ . ~
fn url_encode(text: &str, encode_spaces_as_plus: bool) -> (String, u32) {
    let mut encoded = String::with_capacity(text.len() * 2);
    let mut encode_count = 0u32;

    for ch in text.chars() {
        match ch {
            ' ' if encode_spaces_as_plus => {
                encoded.push('+');
                encode_count += 1;
            }
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                // Unreserved characters - don't encode
                encoded.push(ch);
            }
            _ => {
                // Encode everything else as %XX
                let bytes = ch.to_string().as_bytes().to_vec();
                for byte in bytes {
                    encoded.push_str(&format!("%{:02X}", byte));
                    encode_count += 1;
                }
            }
        }
    }

    (encoded, encode_count)
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple() {
        let (encoded, count) = url_encode("hello", false);
        assert_eq!(encoded, "hello");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_encode_space_as_percent() {
        let (encoded, count) = url_encode("hello world", false);
        assert_eq!(encoded, "hello%20world");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_encode_space_as_plus() {
        let (encoded, count) = url_encode("hello world", true);
        assert_eq!(encoded, "hello+world");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_encode_special_characters() {
        let (encoded, count) = url_encode("name=John&age=30", false);
        assert_eq!(encoded, "name%3DJohn%26age%3D30");
        assert_eq!(count, 4);
    }

    #[test]
    fn test_encode_unreserved_characters() {
        let (encoded, count) = url_encode("test-file_name.txt~", false);
        assert_eq!(encoded, "test-file_name.txt~");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_encode_alphanumeric() {
        let (encoded, count) = url_encode("abc123XYZ", false);
        assert_eq!(encoded, "abc123XYZ");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_encode_mixed() {
        let (encoded, count) = url_encode("search=hello world&page=1", false);
        assert_eq!(encoded, "search%3Dhello%20world%26page%3D1");
        assert_eq!(count, 5); // =, space, &, =, =
    }

    #[test]
    fn test_encode_empty_string() {
        let (encoded, count) = url_encode("", false);
        assert_eq!(encoded, "");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_encode_symbols() {
        let (encoded, count) = url_encode("!@#$%^&*()", false);
        assert_eq!(encoded, "%21%40%23%24%25%5E%26%2A%28%29");
        assert_eq!(count, 10);
    }

    #[test]
    fn test_encode_slash() {
        let (encoded, count) = url_encode("/api/users/123", false);
        assert_eq!(encoded, "%2Fapi%2Fusers%2F123");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_encode_question_mark() {
        let (encoded, count) = url_encode("query?param=value", false);
        assert_eq!(encoded, "query%3Fparam%3Dvalue");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_encode_unicode() {
        let (encoded, count) = url_encode("hello✓", false);
        assert_eq!(encoded, "hello%E2%9C%93");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_encode_emoji() {
        let (encoded, count) = url_encode("test🚀", false);
        assert_eq!(encoded, "test%F0%9F%9A%80");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_encode_query_string() {
        let (encoded, count) = url_encode("hello world", true);
        assert_eq!(encoded, "hello+world");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_encode_email() {
        let (encoded, count) = url_encode("user@example.com", false);
        assert_eq!(encoded, "user%40example.com");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_encode_multiple_spaces() {
        let (encoded, count) = url_encode("one  two   three", false);
        assert_eq!(encoded, "one%20%20two%20%20%20three");
        assert_eq!(count, 5);
    }

    #[test]
    fn test_encode_brackets() {
        let (encoded, count) = url_encode("data[key]", false);
        assert_eq!(encoded, "data%5Bkey%5D");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_encode_quotes() {
        let (encoded, count) = url_encode(r#"value="test""#, false);
        assert_eq!(encoded, "value%3D%22test%22");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_encode_complex_query() {
        let (encoded, count) = url_encode("name=John Doe&email=john@example.com", true);
        assert_eq!(encoded, "name%3DJohn+Doe%26email%3Djohn%40example.com");
        assert_eq!(count, 5); // =, space, &, =, @
    }
}
