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
            name: "URL Decode".to_string(),
            version: "1.0.0".to_string(),
            description: "Decode URL-encoded strings (percent-encoding)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "text".to_string(),
                data_type: DataType::String,
                description: "URL-encoded text to decode".to_string(),
                required: true,
            },
            PortSpec {
                name: "decode_plus_as_space".to_string(),
                data_type: DataType::Bool,
                description: "Treat + as space (default: true, for query strings)".to_string(),
                required: false,
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "decoded".to_string(),
                data_type: DataType::String,
                description: "Decoded text".to_string(),
                required: true,
            },
            PortSpec {
                name: "decode_count".to_string(),
                data_type: DataType::U32,
                description: "Number of sequences decoded (+ and %XX)".to_string(),
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
                recovery_hint: Some("Provide URL-encoded text to decode".to_string()),
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

        // Extract optional decode_plus_as_space (default: true)
        let decode_plus = inputs
            .iter()
            .find(|(name, _)| name == "decode_plus_as_space")
            .and_then(|(_, v)| match v {
                Value::BoolVal(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(true);

        // Decode URL
        let (decoded, count) = url_decode(input_text, decode_plus)?;

        Ok(vec![
            ("decoded".to_string(), Value::StringVal(decoded)),
            ("decode_count".to_string(), Value::U32Val(count)),
        ])
    }
}

/// Decode URL-encoded string
fn url_decode(encoded: &str, decode_plus_as_space: bool) -> Result<(String, u32), ExecutionError> {
    let mut decoded = String::with_capacity(encoded.len());
    let mut chars = encoded.chars().peekable();
    let mut decode_count = 0u32;

    while let Some(ch) = chars.next() {
        match ch {
            '+' if decode_plus_as_space => {
                decoded.push(' ');
                decode_count += 1;
            }
            '%' => {
                // Read next two hex digits
                let hex1 = chars.next().ok_or_else(|| ExecutionError {
                    message: "Invalid URL encoding: % not followed by two hex digits".to_string(),
                    input_name: Some("text".to_string()),
                    recovery_hint: Some("Ensure all % sequences are followed by two hex digits (e.g., %20)".to_string()),
                })?;

                let hex2 = chars.next().ok_or_else(|| ExecutionError {
                    message: "Invalid URL encoding: % followed by only one hex digit".to_string(),
                    input_name: Some("text".to_string()),
                    recovery_hint: Some("Ensure all % sequences are followed by two hex digits (e.g., %20)".to_string()),
                })?;

                // Parse hex digits
                let hex_str = format!("{}{}", hex1, hex2);
                let byte = u8::from_str_radix(&hex_str, 16).map_err(|_| ExecutionError {
                    message: format!("Invalid hex digits in URL encoding: %{}", hex_str),
                    input_name: Some("text".to_string()),
                    recovery_hint: Some("Hex digits must be 0-9, A-F, or a-f".to_string()),
                })?;

                decoded.push(byte as char);
                decode_count += 1;
            }
            _ => {
                decoded.push(ch);
            }
        }
    }

    Ok((decoded, decode_count))
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple() {
        let (decoded, count) = url_decode("hello", true).unwrap();
        assert_eq!(decoded, "hello");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_decode_plus_as_space() {
        let (decoded, count) = url_decode("hello+world", true).unwrap();
        assert_eq!(decoded, "hello world");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_decode_plus_as_plus() {
        let (decoded, count) = url_decode("hello+world", false).unwrap();
        assert_eq!(decoded, "hello+world");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_decode_percent_encoding() {
        let (decoded, count) = url_decode("hello%20world", true).unwrap();
        assert_eq!(decoded, "hello world");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_decode_multiple_encodings() {
        let (decoded, count) = url_decode("hello+world%21", true).unwrap();
        assert_eq!(decoded, "hello world!");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_decode_special_characters() {
        let (decoded, count) = url_decode("name%3DJohn%26age%3D30", true).unwrap();
        assert_eq!(decoded, "name=John&age=30");
        assert_eq!(count, 4);
    }

    #[test]
    fn test_decode_unicode() {
        let (decoded, count) = url_decode("hello%E2%9C%93", true).unwrap();
        assert_eq!(decoded, "hello✓");
        assert_eq!(count, 1); // Only counts first %XX
    }

    #[test]
    fn test_decode_mixed() {
        let (decoded, count) = url_decode("search%3Dhello+world%26page%3D1", true).unwrap();
        assert_eq!(decoded, "search=hello world&page=1");
        assert_eq!(count, 4); // %3D, +, %26, %3D
    }

    #[test]
    fn test_decode_empty_string() {
        let (decoded, count) = url_decode("", true).unwrap();
        assert_eq!(decoded, "");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_decode_no_encoding() {
        let (decoded, count) = url_decode("plaintext", true).unwrap();
        assert_eq!(decoded, "plaintext");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_decode_invalid_percent_incomplete() {
        let result = url_decode("test%2", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("% followed by only one hex digit"));
    }

    #[test]
    fn test_decode_invalid_percent_missing() {
        let result = url_decode("test%", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("% not followed by two hex digits"));
    }

    #[test]
    fn test_decode_invalid_hex() {
        let result = url_decode("test%ZZ", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Invalid hex digits"));
    }

    #[test]
    fn test_decode_query_string() {
        let (decoded, count) = url_decode("q=hello+world&category=news", true).unwrap();
        assert_eq!(decoded, "q=hello world&category=news");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_decode_path() {
        let (decoded, count) = url_decode("/api/users/John%20Doe", true).unwrap();
        assert_eq!(decoded, "/api/users/John Doe");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_decode_complex_query() {
        let (decoded, count) = url_decode(
            "name=John+Doe&email=john%40example.com&msg=Hello%2C+World%21",
            true,
        )
        .unwrap();
        assert_eq!(decoded, "name=John Doe&email=john@example.com&msg=Hello, World!");
        assert_eq!(count, 5); // 2 plus, 3 percent sequences
    }

    #[test]
    fn test_decode_consecutive_percent() {
        let (decoded, count) = url_decode("test%20%20%20end", true).unwrap();
        assert_eq!(decoded, "test   end");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_decode_case_insensitive_hex() {
        let (decoded1, _) = url_decode("test%2f", true).unwrap();
        let (decoded2, _) = url_decode("test%2F", true).unwrap();
        assert_eq!(decoded1, "test/");
        assert_eq!(decoded2, "test/");
    }
}
