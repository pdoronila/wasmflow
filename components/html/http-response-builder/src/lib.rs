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
            name: "HTTP Response Builder".to_string(),
            version: "1.0.0".to_string(),
            description: "Builds a complete HTTP response from status, headers, and body".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "status".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "HTTP status code (e.g., 200, 404, 500)".to_string(),
            },
            PortSpec {
                name: "status_message".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "HTTP status message (e.g., 'OK', 'Not Found'). Optional - defaults to standard message for status code".to_string(),
            },
            PortSpec {
                name: "headers".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "HTTP headers as JSON object (e.g., {\"content-type\":\"text/html\"}). Optional - defaults to empty object".to_string(),
            },
            PortSpec {
                name: "body".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Response body content. Optional - defaults to empty string".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "response".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Complete HTTP response string (status line + headers + body)".to_string(),
            },
            PortSpec {
                name: "content_length".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Length of the response body in bytes".to_string(),
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
        // Extract status code (required)
        let status = inputs
            .iter()
            .find(|(name, _)| name == "status")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: status".to_string(),
                input_name: Some("status".to_string()),
                recovery_hint: Some("Connect a status code (200, 404, 500, etc.) to this input".to_string()),
            })?;

        let status_code = match &status.1 {
            Value::U32Val(n) => *n,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected U32 for 'status', got {:?}", status.1),
                    input_name: Some("status".to_string()),
                    recovery_hint: Some("Provide a numeric status code (e.g., 200, 404, 500)".to_string()),
                });
            }
        };

        // Extract status message (optional - will use default if not provided)
        let status_message = if let Some(msg_input) = inputs.iter().find(|(name, _)| name == "status_message") {
            match &msg_input.1 {
                Value::StringVal(s) => s.clone(),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'status_message', got {:?}", msg_input.1),
                        input_name: Some("status_message".to_string()),
                        recovery_hint: Some("Provide a string status message (e.g., 'OK', 'Not Found')".to_string()),
                    });
                }
            }
        } else {
            get_default_status_message(status_code)
        };

        // Extract headers JSON (optional)
        let headers_json = if let Some(headers_input) = inputs.iter().find(|(name, _)| name == "headers") {
            match &headers_input.1 {
                Value::StringVal(s) => s.clone(),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'headers', got {:?}", headers_input.1),
                        input_name: Some("headers".to_string()),
                        recovery_hint: Some("Provide headers as a JSON string (e.g., {{\"content-type\":\"text/html\"}})".to_string()),
                    });
                }
            }
        } else {
            "{}".to_string()
        };

        // Extract body (optional)
        let body = if let Some(body_input) = inputs.iter().find(|(name, _)| name == "body") {
            match &body_input.1 {
                Value::StringVal(s) => s.clone(),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'body', got {:?}", body_input.1),
                        input_name: Some("body".to_string()),
                        recovery_hint: Some("Provide a string for the response body".to_string()),
                    });
                }
            }
        } else {
            String::new()
        };

        // Build the HTTP response
        let (response, content_length) = build_http_response(
            status_code,
            &status_message,
            &headers_json,
            &body,
        ).map_err(|e| ExecutionError {
            message: format!("Failed to build HTTP response: {}", e),
            input_name: None,
            recovery_hint: Some("Check that headers is valid JSON and all inputs are properly formatted".to_string()),
        })?;

        Ok(vec![
            ("response".to_string(), Value::StringVal(response)),
            ("content_length".to_string(), Value::U32Val(content_length)),
        ])
    }
}

// ============================================================================
// HTTP Response Building Logic
// ============================================================================

fn build_http_response(
    status_code: u32,
    status_message: &str,
    headers_json: &str,
    body: &str,
) -> Result<(String, u32), String> {
    let mut response = String::new();

    // Status line: HTTP/1.1 200 OK
    response.push_str(&format!("HTTP/1.1 {} {}\r\n", status_code, status_message));

    // Parse headers from JSON and add them
    let header_lines = parse_headers_from_json(headers_json)?;

    // Check if Content-Length header is already present
    let has_content_length = header_lines.iter()
        .any(|line| line.to_lowercase().starts_with("content-length:"));

    // Add all provided headers
    for header_line in header_lines {
        response.push_str(&header_line);
        response.push_str("\r\n");
    }

    // Add Content-Length if not already present
    let content_length = body.len() as u32;
    if !has_content_length {
        response.push_str(&format!("Content-Length: {}\r\n", content_length));
    }

    // End of headers
    response.push_str("\r\n");

    // Body
    response.push_str(body);

    Ok((response, content_length))
}

/// Parse headers from JSON object string and return as "Key: Value" lines
fn parse_headers_from_json(headers_json: &str) -> Result<Vec<String>, String> {
    let trimmed = headers_json.trim();

    if trimmed.is_empty() || trimmed == "{}" {
        return Ok(Vec::new());
    }

    // Simple JSON object parser (key-value pairs only)
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err("Headers JSON must be an object (start with { and end with })".to_string());
    }

    let content = &trimmed[1..trimmed.len() - 1]; // Remove { and }

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut headers = Vec::new();

    // Split by commas (simple parser - doesn't handle nested objects or escaped commas)
    for pair in content.split(',') {
        let pair = pair.trim();

        // Find the colon separating key and value
        if let Some(colon_pos) = pair.find(':') {
            let key_part = pair[..colon_pos].trim();
            let value_part = pair[colon_pos + 1..].trim();

            // Remove quotes from key and value
            let key = key_part.trim_matches('"');
            let value = value_part.trim_matches('"');

            // Unescape JSON string escapes in value
            let unescaped_value = unescape_json_string(value);

            // Format as HTTP header with proper capitalization
            let header_name = capitalize_header_name(key);
            headers.push(format!("{}: {}", header_name, unescaped_value));
        } else {
            return Err(format!("Invalid header pair in JSON: {}", pair));
        }
    }

    Ok(headers)
}

/// Capitalize header names according to HTTP conventions (e.g., "content-type" -> "Content-Type")
fn capitalize_header_name(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

/// Unescape JSON string escape sequences
fn unescape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Get default status message for common HTTP status codes
fn get_default_status_message(code: u32) -> String {
    match code {
        // 2xx Success
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",

        // 3xx Redirection
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",

        // 4xx Client Error
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        429 => "Too Many Requests",

        // 5xx Server Error
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",

        // Default for unknown codes
        _ => "Unknown",
    }.to_string()
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_200_response() {
        let inputs = vec![
            ("status".to_string(), Value::U32Val(200)),
            ("body".to_string(), Value::StringVal("Hello, World!".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].0, "response");
        if let Value::StringVal(response) = &result[0].1 {
            assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
            assert!(response.contains("Content-Length: 13\r\n"));
            assert!(response.ends_with("\r\n\r\nHello, World!"));
        } else {
            panic!("Expected StringVal for response");
        }

        assert_eq!(result[1].0, "content_length");
        assert_eq!(result[1].1, Value::U32Val(13));
    }

    #[test]
    fn test_404_response_with_custom_message() {
        let inputs = vec![
            ("status".to_string(), Value::U32Val(404)),
            ("status_message".to_string(), Value::StringVal("Page Not Found".to_string())),
            ("body".to_string(), Value::StringVal("<h1>404 - Page Not Found</h1>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(response) = &result[0].1 {
            assert!(response.starts_with("HTTP/1.1 404 Page Not Found\r\n"));
            assert!(response.contains("Content-Length: 29\r\n"));
        }
    }

    #[test]
    fn test_response_with_headers() {
        let inputs = vec![
            ("status".to_string(), Value::U32Val(200)),
            ("headers".to_string(), Value::StringVal("{\"content-type\":\"application/json\",\"cache-control\":\"no-cache\"}".to_string())),
            ("body".to_string(), Value::StringVal("{\"message\":\"success\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(response) = &result[0].1 {
            assert!(response.contains("Content-Type: application/json\r\n"));
            assert!(response.contains("Cache-Control: no-cache\r\n"));
            assert!(response.ends_with("{\"message\":\"success\"}"));
        }
    }

    #[test]
    fn test_default_status_messages() {
        let test_cases = vec![
            (200, "OK"),
            (404, "Not Found"),
            (500, "Internal Server Error"),
            (301, "Moved Permanently"),
        ];

        for (code, expected_msg) in test_cases {
            let inputs = vec![("status".to_string(), Value::U32Val(code))];
            let result = Component::execute(inputs).unwrap();

            if let Value::StringVal(response) = &result[0].1 {
                assert!(response.starts_with(&format!("HTTP/1.1 {} {}\r\n", code, expected_msg)));
            }
        }
    }

    #[test]
    fn test_empty_body() {
        let inputs = vec![("status".to_string(), Value::U32Val(204))];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::U32Val(0)); // content_length should be 0

        if let Value::StringVal(response) = &result[0].1 {
            assert!(response.contains("Content-Length: 0\r\n"));
            assert!(response.ends_with("\r\n\r\n")); // Should end with just headers separator
        }
    }

    #[test]
    fn test_custom_content_length_preserved() {
        let inputs = vec![
            ("status".to_string(), Value::U32Val(200)),
            ("headers".to_string(), Value::StringVal("{\"content-length\":\"999\"}".to_string())),
            ("body".to_string(), Value::StringVal("test".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(response) = &result[0].1 {
            // Should preserve the custom Content-Length and not add another
            assert!(response.contains("Content-Length: 999\r\n"));
            assert_eq!(response.matches("Content-Length").count(), 1);
        }
    }

    #[test]
    fn test_header_capitalization() {
        let inputs = vec![
            ("status".to_string(), Value::U32Val(200)),
            ("headers".to_string(), Value::StringVal("{\"content-type\":\"text/html\",\"x-custom-header\":\"value\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(response) = &result[0].1 {
            assert!(response.contains("Content-Type: text/html\r\n"));
            assert!(response.contains("X-Custom-Header: value\r\n"));
        }
    }

    #[test]
    fn test_missing_status_error() {
        let inputs = vec![("body".to_string(), Value::StringVal("test".to_string()))];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("status".to_string()));
    }

    #[test]
    fn test_invalid_headers_json() {
        let inputs = vec![
            ("status".to_string(), Value::U32Val(200)),
            ("headers".to_string(), Value::StringVal("not valid json".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_escaped_header_values() {
        let inputs = vec![
            ("status".to_string(), Value::U32Val(200)),
            ("headers".to_string(), Value::StringVal("{\"x-test\":\"value with \\\"quotes\\\"\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(response) = &result[0].1 {
            assert!(response.contains("X-Test: value with \"quotes\"\r\n"));
        }
    }
}
