wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

// ============================================================================
// Metadata Interface
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Body Parser".to_string(),
            version: "1.0.0".to_string(),
            description: "Parses HTTP request body based on Content-Type (JSON, form data, plain text)".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "body".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Request body content".to_string(),
            },
            PortSpec {
                name: "content_type".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Content-Type header (e.g., 'application/json', 'application/x-www-form-urlencoded')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "parsed".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Parsed data as JSON object (for JSON/form data) or raw text".to_string(),
            },
            PortSpec {
                name: "body_type".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Detected body type: 'json', 'form', 'text', or 'binary'".to_string(),
            },
            PortSpec {
                name: "is_valid".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if body was successfully parsed according to Content-Type".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}


// ============================================================================
// Execution Interface
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract body
        let body = inputs
            .iter()
            .find(|(name, _)| name == "body")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: body".to_string(),
                input_name: Some("body".to_string()),
                recovery_hint: Some("Connect request body to this input".to_string()),
            })?;

        let body_str = match &body.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'body', got {:?}", body.1),
                    input_name: Some("body".to_string()),
                    recovery_hint: Some("Provide body as string".to_string()),
                });
            }
        };

        // Extract content_type (optional)
        let content_type = if let Some(ct_input) = inputs.iter().find(|(name, _)| name == "content_type") {
            match &ct_input.1 {
                Value::StringVal(s) => s.clone(),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'content_type', got {:?}", ct_input.1),
                        input_name: Some("content_type".to_string()),
                        recovery_hint: Some("Provide Content-Type as string".to_string()),
                    });
                }
            }
        } else {
            String::new()
        };

        // Parse body based on content type
        let (parsed, body_type, is_valid) = parse_body(body_str, &content_type);

        Ok(vec![
            ("parsed".to_string(), Value::StringVal(parsed)),
            ("body_type".to_string(), Value::StringVal(body_type)),
            ("is_valid".to_string(), Value::BoolVal(is_valid)),
        ])
    }
}

// ============================================================================
// Body Parsing Logic
// ============================================================================

/// Parse body based on Content-Type
/// Returns: (parsed_data, body_type, is_valid)
fn parse_body(body: &str, content_type: &str) -> (String, String, bool) {
    // Normalize content type (lowercase, extract base type)
    let content_type_lower = content_type.to_lowercase();
    let base_type = content_type_lower.split(';').next().unwrap_or("").trim();

    match base_type {
        "application/json" => parse_json_body(body),
        "application/x-www-form-urlencoded" => parse_form_body(body),
        "text/plain" | "text/html" | "text/css" | "text/javascript" => {
            // Return as-is for text types
            (body.to_string(), "text".to_string(), true)
        }
        "" => {
            // No Content-Type provided - try to detect
            if body.trim().starts_with('{') || body.trim().starts_with('[') {
                // Looks like JSON
                parse_json_body(body)
            } else if body.contains('=') && (body.contains('&') || !body.contains(' ')) {
                // Looks like form data
                parse_form_body(body)
            } else {
                // Treat as plain text
                (body.to_string(), "text".to_string(), true)
            }
        }
        _ => {
            // Unknown/binary content type
            (body.to_string(), "binary".to_string(), true)
        }
    }
}

/// Parse JSON body
fn parse_json_body(body: &str) -> (String, String, bool) {
    // Simple JSON validation - check if it looks like valid JSON
    let trimmed = body.trim();

    if trimmed.is_empty() {
        return ("{}".to_string(), "json".to_string(), true);
    }

    // Basic validation: must start with { or [
    let is_valid = (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'));

    if is_valid {
        (body.to_string(), "json".to_string(), true)
    } else {
        // Invalid JSON - return as-is but mark invalid
        (body.to_string(), "json".to_string(), false)
    }
}

/// Parse form-urlencoded body
fn parse_form_body(body: &str) -> (String, String, bool) {
    let mut params = Vec::new();

    if body.is_empty() {
        return ("{}".to_string(), "form".to_string(), true);
    }

    // Split by '&' to get individual parameters
    for pair in body.split('&') {
        if pair.is_empty() {
            continue;
        }

        // Split by '=' to get key and value
        if let Some(equals_pos) = pair.find('=') {
            let key = &pair[..equals_pos];
            let value = &pair[equals_pos + 1..];

            // URL decode both key and value
            let decoded_key = url_decode(key);
            let decoded_value = url_decode(value);

            params.push((decoded_key, decoded_value));
        } else {
            // No '=' found - treat as key with empty value
            let decoded_key = url_decode(pair);
            params.push((decoded_key, String::new()));
        }
    }

    // Build JSON object
    let json = build_params_json(&params);

    (json, "form".to_string(), true)
}

/// URL decode a string (decode %XX sequences and + as space)
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                let hex1 = chars.next();
                let hex2 = chars.next();

                if let (Some(h1), Some(h2)) = (hex1, hex2) {
                    if let Some(decoded) = decode_hex_pair(h1, h2) {
                        result.push(decoded);
                    } else {
                        result.push('%');
                        result.push(h1);
                        result.push(h2);
                    }
                } else {
                    result.push('%');
                    if let Some(h1) = hex1 {
                        result.push(h1);
                    }
                }
            }
            _ => result.push(c),
        }
    }

    result
}

/// Decode a hex pair
fn decode_hex_pair(h1: char, h2: char) -> Option<char> {
    let hex_str = format!("{}{}", h1, h2);
    u8::from_str_radix(&hex_str, 16)
        .ok()
        .and_then(|byte| {
            if byte.is_ascii() {
                Some(byte as char)
            } else {
                None
            }
        })
}

/// Build JSON object from parameter key-value pairs
fn build_params_json(params: &[(String, String)]) -> String {
    if params.is_empty() {
        return "{}".to_string();
    }

    let pairs: Vec<String> = params
        .iter()
        .map(|(key, value)| {
            let escaped_key = escape_json_string(key);
            let escaped_value = escape_json_string(value);
            format!("\"{}\":\"{}\"", escaped_key, escaped_value)
        })
        .collect();

    format!("{{{}}}", pairs.join(","))
}

/// Escape special characters for JSON string values
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }

    result
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_body() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("{\"name\":\"John\",\"age\":30}".to_string())),
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("{\"name\":\"John\",\"age\":30}".to_string())); // parsed
        assert_eq!(result[1].1, Value::StringVal("json".to_string())); // body_type
        assert_eq!(result[2].1, Value::BoolVal(true)); // is_valid
    }

    #[test]
    fn test_form_urlencoded_body() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("name=John&age=30&city=New+York".to_string())),
            ("content_type".to_string(), Value::StringVal("application/x-www-form-urlencoded".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::StringVal("form".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(true));

        if let Value::StringVal(parsed) = &result[0].1 {
            assert!(parsed.contains("\"name\":\"John\""));
            assert!(parsed.contains("\"age\":\"30\""));
            assert!(parsed.contains("\"city\":\"New York\"")); // + decoded to space
        }
    }

    #[test]
    fn test_plain_text_body() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("This is plain text".to_string())),
            ("content_type".to_string(), Value::StringVal("text/plain".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("This is plain text".to_string()));
        assert_eq!(result[1].1, Value::StringVal("text".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(true));
    }

    #[test]
    fn test_empty_body() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("".to_string())),
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("{}".to_string())); // Empty JSON
        assert_eq!(result[1].1, Value::StringVal("json".to_string()));
    }

    #[test]
    fn test_invalid_json() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("{invalid json".to_string())),
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::StringVal("json".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(false)); // is_valid = false
    }

    #[test]
    fn test_json_array() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("[1,2,3]".to_string())),
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("[1,2,3]".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(true)); // Valid JSON array
    }

    #[test]
    fn test_auto_detect_json() {
        // No Content-Type provided, but body looks like JSON
        let inputs = vec![
            ("body".to_string(), Value::StringVal("{\"auto\":\"detected\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::StringVal("json".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(true));
    }

    #[test]
    fn test_auto_detect_form() {
        // No Content-Type, but body looks like form data
        let inputs = vec![
            ("body".to_string(), Value::StringVal("key1=value1&key2=value2".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::StringVal("form".to_string()));
    }

    #[test]
    fn test_auto_detect_text() {
        // No Content-Type, doesn't look like JSON or form
        let inputs = vec![
            ("body".to_string(), Value::StringVal("Just some random text".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::StringVal("text".to_string()));
    }

    #[test]
    fn test_content_type_with_charset() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("{\"test\":true}".to_string())),
            ("content_type".to_string(), Value::StringVal("application/json; charset=utf-8".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::StringVal("json".to_string())); // Should still detect as JSON
    }

    #[test]
    fn test_url_encoded_form_values() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("email=user%40example.com&message=Hello%20World".to_string())),
            ("content_type".to_string(), Value::StringVal("application/x-www-form-urlencoded".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(parsed) = &result[0].1 {
            assert!(parsed.contains("\"email\":\"user@example.com\""));
            assert!(parsed.contains("\"message\":\"Hello World\""));
        }
    }

    #[test]
    fn test_binary_content_type() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("BINARY_DATA".to_string())),
            ("content_type".to_string(), Value::StringVal("application/octet-stream".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::StringVal("binary".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(true));
    }

    #[test]
    fn test_missing_body_input() {
        let inputs = vec![
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("body".to_string()));
    }

    #[test]
    fn test_form_with_empty_values() {
        let inputs = vec![
            ("body".to_string(), Value::StringVal("field1=&field2=value2".to_string())),
            ("content_type".to_string(), Value::StringVal("application/x-www-form-urlencoded".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(parsed) = &result[0].1 {
            assert!(parsed.contains("\"field1\":\"\""));
            assert!(parsed.contains("\"field2\":\"value2\""));
        }
    }

    #[test]
    fn test_complex_json() {
        let json = "{\"user\":{\"name\":\"Alice\",\"roles\":[\"admin\",\"user\"]}}";
        let inputs = vec![
            ("body".to_string(), Value::StringVal(json.to_string())),
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal(json.to_string()));
        assert_eq!(result[2].1, Value::BoolVal(true));
    }
}
