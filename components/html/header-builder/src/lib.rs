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
            name: "Header Builder".to_string(),
            version: "1.0.0".to_string(),
            description: "Builds HTTP headers JSON object from individual header values or merges with existing headers".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "base_headers".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Existing headers as JSON object (e.g., {\"content-type\":\"text/html\"})".to_string(),
            },
            PortSpec {
                name: "content_type".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Content-Type header value (e.g., 'application/json')".to_string(),
            },
            PortSpec {
                name: "cache_control".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Cache-Control header value (e.g., 'no-cache')".to_string(),
            },
            PortSpec {
                name: "location".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Location header value for redirects (e.g., '/new-path')".to_string(),
            },
            PortSpec {
                name: "set_cookie".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Set-Cookie header value (e.g., 'session=abc123; HttpOnly')".to_string(),
            },
            PortSpec {
                name: "custom_headers".to_string(),
                data_type: DataType::ListType,
                optional: true,
                description: "List of custom headers in 'Name: Value' format".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "headers_json".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Complete headers as JSON object string".to_string(),
            },
            PortSpec {
                name: "header_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Total number of headers in the result".to_string(),
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
        let mut headers: Vec<(String, String)> = Vec::new();

        // Parse base headers if provided
        if let Some(base_input) = inputs.iter().find(|(name, _)| name == "base_headers") {
            match &base_input.1 {
                Value::StringVal(json_str) => {
                    let parsed_headers = parse_headers_json(json_str).map_err(|e| ExecutionError {
                        message: format!("Failed to parse base_headers: {}", e),
                        input_name: Some("base_headers".to_string()),
                        recovery_hint: Some("Provide valid JSON object (e.g., {{\"key\":\"value\"}})".to_string()),
                    })?;
                    headers.extend(parsed_headers);
                }
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'base_headers', got {:?}", base_input.1),
                        input_name: Some("base_headers".to_string()),
                        recovery_hint: Some("Provide a JSON string".to_string()),
                    });
                }
            }
        }

        // Add individual headers if provided
        if let Some(ct_input) = inputs.iter().find(|(name, _)| name == "content_type") {
            if let Value::StringVal(value) = &ct_input.1 {
                if !value.is_empty() {
                    headers.push(("content-type".to_string(), value.clone()));
                }
            }
        }

        if let Some(cc_input) = inputs.iter().find(|(name, _)| name == "cache_control") {
            if let Value::StringVal(value) = &cc_input.1 {
                if !value.is_empty() {
                    headers.push(("cache-control".to_string(), value.clone()));
                }
            }
        }

        if let Some(loc_input) = inputs.iter().find(|(name, _)| name == "location") {
            if let Value::StringVal(value) = &loc_input.1 {
                if !value.is_empty() {
                    headers.push(("location".to_string(), value.clone()));
                }
            }
        }

        if let Some(cookie_input) = inputs.iter().find(|(name, _)| name == "set_cookie") {
            if let Value::StringVal(value) = &cookie_input.1 {
                if !value.is_empty() {
                    headers.push(("set-cookie".to_string(), value.clone()));
                }
            }
        }

        // Add custom headers if provided
        if let Some(custom_input) = inputs.iter().find(|(name, _)| name == "custom_headers") {
            match &custom_input.1 {
                Value::StringListVal(header_list) => {
                    for header_str in header_list {
                        if let Some(colon_pos) = header_str.find(':') {
                            let key = header_str[..colon_pos].trim().to_lowercase();
                            let value = header_str[colon_pos + 1..].trim().to_string();
                            headers.push((key, value));
                        } else {
                            return Err(ExecutionError {
                                message: format!("Invalid header format: '{}'. Expected 'Name: Value'", header_str),
                                input_name: Some("custom_headers".to_string()),
                                recovery_hint: Some("Use 'Name: Value' format for each header (e.g., 'X-Custom: value')".to_string()),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected StringListVal for 'custom_headers', got {:?}", custom_input.1),
                        input_name: Some("custom_headers".to_string()),
                        recovery_hint: Some("Provide a list of strings in 'Name: Value' format".to_string()),
                    });
                }
            }
        }

        // Build JSON object string
        let headers_json = build_headers_json(&headers);
        let header_count = headers.len() as u32;

        Ok(vec![
            ("headers_json".to_string(), Value::StringVal(headers_json)),
            ("header_count".to_string(), Value::U32Val(header_count)),
        ])
    }
}

// ============================================================================
// Header Parsing and Building Logic
// ============================================================================

/// Parse headers from a JSON object string
fn parse_headers_json(json_str: &str) -> Result<Vec<(String, String)>, String> {
    let trimmed = json_str.trim();

    if trimmed.is_empty() || trimmed == "{}" {
        return Ok(Vec::new());
    }

    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err("Headers JSON must be an object (start with { and end with })".to_string());
    }

    let content = &trimmed[1..trimmed.len() - 1]; // Remove { and }

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut headers = Vec::new();

    // Split by commas (simple parser - doesn't handle nested objects or escaped commas in values)
    for pair in content.split(',') {
        let pair = pair.trim();

        // Find the colon separating key and value
        if let Some(colon_pos) = pair.find(':') {
            let key_part = pair[..colon_pos].trim();
            let value_part = pair[colon_pos + 1..].trim();

            // Remove quotes from key and value
            let key = key_part.trim_matches('"').to_lowercase();
            let value = value_part.trim_matches('"').to_string();

            headers.push((key, value));
        } else {
            return Err(format!("Invalid header pair in JSON: {}", pair));
        }
    }

    Ok(headers)
}

/// Build a JSON object string from header key-value pairs
fn build_headers_json(headers: &[(String, String)]) -> String {
    if headers.is_empty() {
        return "{}".to_string();
    }

    let pairs: Vec<String> = headers
        .iter()
        .map(|(key, value)| {
            let escaped_value = escape_json_string(value);
            format!("\"{}\":\"{}\"", key, escaped_value)
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
    fn test_empty_headers() {
        let inputs = vec![];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].0, "headers_json");
        assert_eq!(result[0].1, Value::StringVal("{}".to_string()));

        assert_eq!(result[1].0, "header_count");
        assert_eq!(result[1].1, Value::U32Val(0));
    }

    #[test]
    fn test_single_content_type() {
        let inputs = vec![
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"content-type\":\"application/json\""));
        } else {
            panic!("Expected StringVal");
        }

        assert_eq!(result[1].1, Value::U32Val(1));
    }

    #[test]
    fn test_multiple_standard_headers() {
        let inputs = vec![
            ("content_type".to_string(), Value::StringVal("text/html".to_string())),
            ("cache_control".to_string(), Value::StringVal("no-cache".to_string())),
            ("location".to_string(), Value::StringVal("/redirect".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"content-type\":\"text/html\""));
            assert!(json.contains("\"cache-control\":\"no-cache\""));
            assert!(json.contains("\"location\":\"/redirect\""));
        }

        assert_eq!(result[1].1, Value::U32Val(3));
    }

    #[test]
    fn test_custom_headers_list() {
        let inputs = vec![
            ("custom_headers".to_string(), Value::StringListVal(vec![
                "X-Custom-Header: CustomValue".to_string(),
                "X-Request-ID: 12345".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"x-custom-header\":\"CustomValue\""));
            assert!(json.contains("\"x-request-id\":\"12345\""));
        }

        assert_eq!(result[1].1, Value::U32Val(2));
    }

    #[test]
    fn test_merge_base_and_new_headers() {
        let inputs = vec![
            ("base_headers".to_string(), Value::StringVal("{\"server\":\"WasmFlow/1.0\"}".to_string())),
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"server\":\"WasmFlow/1.0\""));
            assert!(json.contains("\"content-type\":\"application/json\""));
        }

        assert_eq!(result[1].1, Value::U32Val(2));
    }

    #[test]
    fn test_empty_base_headers() {
        let inputs = vec![
            ("base_headers".to_string(), Value::StringVal("{}".to_string())),
            ("content_type".to_string(), Value::StringVal("text/plain".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"content-type\":\"text/plain\""));
        }

        assert_eq!(result[1].1, Value::U32Val(1));
    }

    #[test]
    fn test_set_cookie_header() {
        let inputs = vec![
            ("set_cookie".to_string(), Value::StringVal("session=abc123; HttpOnly; Secure".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"set-cookie\":\"session=abc123; HttpOnly; Secure\""));
        }
    }

    #[test]
    fn test_escaped_values() {
        let inputs = vec![
            ("custom_headers".to_string(), Value::StringListVal(vec![
                "X-Test: value with \"quotes\"".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\\\"quotes\\\""));
        }
    }

    #[test]
    fn test_header_name_case_normalization() {
        let inputs = vec![
            ("custom_headers".to_string(), Value::StringListVal(vec![
                "Content-Type: text/html".to_string(),
                "CACHE-CONTROL: max-age=3600".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            // Header names should be lowercase in JSON
            assert!(json.contains("\"content-type\":\"text/html\""));
            assert!(json.contains("\"cache-control\":\"max-age=3600\""));
        }
    }

    #[test]
    fn test_invalid_custom_header_format() {
        let inputs = vec![
            ("custom_headers".to_string(), Value::StringListVal(vec![
                "InvalidHeaderNoColon".to_string(),
            ])),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid header format"));
        assert_eq!(err.input_name, Some("custom_headers".to_string()));
    }

    #[test]
    fn test_invalid_base_headers_json() {
        let inputs = vec![
            ("base_headers".to_string(), Value::StringVal("not valid json".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_string_values_ignored() {
        let inputs = vec![
            ("content_type".to_string(), Value::StringVal("".to_string())),
            ("cache_control".to_string(), Value::StringVal("no-cache".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        // Only cache-control should be included (content-type is empty)
        assert_eq!(result[1].1, Value::U32Val(1));

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"cache-control\":\"no-cache\""));
            assert!(!json.contains("content-type"));
        }
    }

    #[test]
    fn test_all_headers_combined() {
        let inputs = vec![
            ("base_headers".to_string(), Value::StringVal("{\"server\":\"WasmFlow\"}".to_string())),
            ("content_type".to_string(), Value::StringVal("application/json".to_string())),
            ("cache_control".to_string(), Value::StringVal("no-store".to_string())),
            ("location".to_string(), Value::StringVal("/api/v2".to_string())),
            ("set_cookie".to_string(), Value::StringVal("token=xyz".to_string())),
            ("custom_headers".to_string(), Value::StringListVal(vec![
                "X-API-Version: 2.0".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::U32Val(6));

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"server\":\"WasmFlow\""));
            assert!(json.contains("\"content-type\":\"application/json\""));
            assert!(json.contains("\"cache-control\":\"no-store\""));
            assert!(json.contains("\"location\":\"/api/v2\""));
            assert!(json.contains("\"set-cookie\":\"token=xyz\""));
            assert!(json.contains("\"x-api-version\":\"2.0\""));
        }
    }
}
