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
            name: "JSON Response Builder".to_string(),
            version: "1.0.0".to_string(),
            description: "Builds complete JSON API HTTP responses with proper headers and status".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "data".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "JSON data as string (e.g., '{\"message\":\"success\"}')".to_string(),
            },
            PortSpec {
                name: "status".to_string(),
                data_type: DataType::U32Type,
                optional: true,
                description: "HTTP status code (default: 200)".to_string(),
            },
            PortSpec {
                name: "additional_headers".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Additional headers as JSON object to merge (e.g., '{\"x-api-version\":\"1.0\"}')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "status".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "HTTP status code".to_string(),
            },
            PortSpec {
                name: "headers".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Complete headers as JSON object with Content-Type and Content-Length".to_string(),
            },
            PortSpec {
                name: "body".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "JSON response body".to_string(),
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
        // Extract data (required)
        let data = inputs
            .iter()
            .find(|(name, _)| name == "data")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: data".to_string(),
                input_name: Some("data".to_string()),
                recovery_hint: Some("Connect JSON data string to this input".to_string()),
            })?;

        let json_data = match &data.1 {
            Value::StringVal(s) => s.clone(),
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'data', got {:?}", data.1),
                    input_name: Some("data".to_string()),
                    recovery_hint: Some("Provide a JSON string".to_string()),
                });
            }
        };

        // Extract status (optional, default 200)
        let status_code = if let Some(status_input) = inputs.iter().find(|(name, _)| name == "status") {
            match &status_input.1 {
                Value::U32Val(n) => *n,
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected U32 for 'status', got {:?}", status_input.1),
                        input_name: Some("status".to_string()),
                        recovery_hint: Some("Provide a numeric status code (e.g., 200, 404)".to_string()),
                    });
                }
            }
        } else {
            200
        };

        // Extract additional_headers (optional)
        let additional_headers = if let Some(headers_input) = inputs.iter().find(|(name, _)| name == "additional_headers") {
            match &headers_input.1 {
                Value::StringVal(s) => s.clone(),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'additional_headers', got {:?}", headers_input.1),
                        input_name: Some("additional_headers".to_string()),
                        recovery_hint: Some("Provide headers as JSON object string".to_string()),
                    });
                }
            }
        } else {
            "{}".to_string()
        };

        // Build response headers
        let headers = build_json_headers(&json_data, &additional_headers)
            .map_err(|e| ExecutionError {
                message: format!("Failed to build headers: {}", e),
                input_name: Some("additional_headers".to_string()),
                recovery_hint: Some("Ensure additional_headers is valid JSON object".to_string()),
            })?;

        Ok(vec![
            ("status".to_string(), Value::U32Val(status_code)),
            ("headers".to_string(), Value::StringVal(headers)),
            ("body".to_string(), Value::StringVal(json_data)),
        ])
    }
}

// ============================================================================
// JSON Response Building Logic
// ============================================================================

/// Build headers for JSON response
fn build_json_headers(json_data: &str, additional_headers: &str) -> Result<String, String> {
    let mut header_pairs = Vec::new();

    // Add Content-Type for JSON
    header_pairs.push(("content-type".to_string(), "application/json; charset=utf-8".to_string()));

    // Add Content-Length
    let content_length = json_data.len();
    header_pairs.push(("content-length".to_string(), content_length.to_string()));

    // Parse and add additional headers
    let extra_headers = parse_headers_json(additional_headers)?;
    for (key, value) in extra_headers {
        // Don't override Content-Type or Content-Length
        if key != "content-type" && key != "content-length" {
            header_pairs.push((key, value));
        }
    }

    // Build JSON object
    let pairs: Vec<String> = header_pairs
        .iter()
        .map(|(key, value)| {
            let escaped_value = escape_json_string(value);
            format!("\"{}\":\"{}\"", key, escaped_value)
        })
        .collect();

    Ok(format!("{{{}}}", pairs.join(",")))
}

/// Parse headers from JSON object string
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

    for pair in content.split(',') {
        let pair = pair.trim();

        if let Some(colon_pos) = pair.find(':') {
            let key_part = pair[..colon_pos].trim();
            let value_part = pair[colon_pos + 1..].trim();

            let key = key_part.trim_matches('"').to_lowercase();
            let value = value_part.trim_matches('"').to_string();

            headers.push((key, value));
        } else {
            return Err(format!("Invalid header pair in JSON: {}", pair));
        }
    }

    Ok(headers)
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
    fn test_simple_json_response() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{\"message\":\"success\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::U32Val(200)); // status
        assert_eq!(result[2].1, Value::StringVal("{\"message\":\"success\"}".to_string())); // body

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"content-type\":\"application/json; charset=utf-8\""));
            assert!(headers.contains("\"content-length\":\"21\"")); // Length of JSON
        } else {
            panic!("Expected StringVal for headers");
        }
    }

    #[test]
    fn test_custom_status() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{\"error\":\"not found\"}".to_string())),
            ("status".to_string(), Value::U32Val(404)),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::U32Val(404));
    }

    #[test]
    fn test_with_additional_headers() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{\"data\":[]}".to_string())),
            ("additional_headers".to_string(), Value::StringVal("{\"x-api-version\":\"1.0\",\"x-request-id\":\"abc123\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"content-type\":\"application/json; charset=utf-8\""));
            assert!(headers.contains("\"x-api-version\":\"1.0\""));
            assert!(headers.contains("\"x-request-id\":\"abc123\""));
        }
    }

    #[test]
    fn test_created_status() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{\"id\":123}".to_string())),
            ("status".to_string(), Value::U32Val(201)),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::U32Val(201)); // 201 Created
    }

    #[test]
    fn test_server_error_status() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{\"error\":\"internal error\"}".to_string())),
            ("status".to_string(), Value::U32Val(500)),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::U32Val(500));
    }

    #[test]
    fn test_empty_json_object() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[2].1, Value::StringVal("{}".to_string()));

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"content-length\":\"2\""));
        }
    }

    #[test]
    fn test_empty_json_array() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("[]".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[2].1, Value::StringVal("[]".to_string()));
    }

    #[test]
    fn test_complex_json() {
        let json = "{\"users\":[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"}]}";
        let inputs = vec![
            ("data".to_string(), Value::StringVal(json.to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[2].1, Value::StringVal(json.to_string()));

        if let Value::StringVal(headers) = &result[1].1 {
            let expected_length = json.len();
            assert!(headers.contains(&format!("\"content-length\":\"{}\"", expected_length)));
        }
    }

    #[test]
    fn test_additional_headers_dont_override_content_type() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{}".to_string())),
            ("additional_headers".to_string(), Value::StringVal("{\"content-type\":\"text/plain\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            // Should keep application/json, not allow override
            assert!(headers.contains("\"content-type\":\"application/json; charset=utf-8\""));
            assert!(!headers.contains("text/plain"));
        }
    }

    #[test]
    fn test_additional_headers_dont_override_content_length() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{}".to_string())),
            ("additional_headers".to_string(), Value::StringVal("{\"content-length\":\"999\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            // Should use actual length, not override
            assert!(headers.contains("\"content-length\":\"2\""));
            assert!(!headers.contains("999"));
        }
    }

    #[test]
    fn test_missing_data_input() {
        let inputs = vec![
            ("status".to_string(), Value::U32Val(200)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("data".to_string()));
    }

    #[test]
    fn test_invalid_additional_headers() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{}".to_string())),
            ("additional_headers".to_string(), Value::StringVal("not valid json".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_in_json() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{\"message\":\"Hello 世界\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[2].1, Value::StringVal("{\"message\":\"Hello 世界\"}".to_string()));
    }

    #[test]
    fn test_no_cache_header() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{\"timestamp\":1234567890}".to_string())),
            ("additional_headers".to_string(), Value::StringVal("{\"cache-control\":\"no-cache\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"cache-control\":\"no-cache\""));
        }
    }

    #[test]
    fn test_cors_headers() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{}".to_string())),
            ("additional_headers".to_string(), Value::StringVal("{\"access-control-allow-origin\":\"*\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"access-control-allow-origin\":\"*\""));
        }
    }
}
