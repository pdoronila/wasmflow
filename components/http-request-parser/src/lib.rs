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
            name: "HTTP Request Parser".to_string(),
            version: "1.0.0".to_string(),
            description: "Parses raw HTTP request into structured components (method, path, headers, body)".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "raw_request".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Raw HTTP request string (e.g., from network socket)".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "method".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "HTTP method (GET, POST, PUT, DELETE, etc.)".to_string(),
            },
            PortSpec {
                name: "path".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Request path (e.g., /api/users)".to_string(),
            },
            PortSpec {
                name: "version".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "HTTP version (e.g., HTTP/1.1)".to_string(),
            },
            PortSpec {
                name: "headers".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "HTTP headers as JSON object (e.g., {\"content-type\":\"application/json\"})".to_string(),
            },
            PortSpec {
                name: "body".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Request body content (empty string if no body)".to_string(),
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
        // Extract raw_request input
        let raw_request = inputs
            .iter()
            .find(|(name, _)| name == "raw_request")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: raw_request".to_string(),
                input_name: Some("raw_request".to_string()),
                recovery_hint: Some("Connect an HTTP request string to this input".to_string()),
            })?;

        let request_text = match &raw_request.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'raw_request', got {:?}", raw_request.1),
                    input_name: Some("raw_request".to_string()),
                    recovery_hint: Some("Provide a string value containing the HTTP request".to_string()),
                });
            }
        };

        // Parse the HTTP request
        let (method, path, version, headers_json, body) = parse_http_request(request_text)
            .map_err(|e| ExecutionError {
                message: format!("Failed to parse HTTP request: {}", e),
                input_name: Some("raw_request".to_string()),
                recovery_hint: Some("Ensure the request follows HTTP/1.1 format: 'METHOD /path HTTP/1.1\\r\\nHeader: Value\\r\\n\\r\\nBody'".to_string()),
            })?;

        Ok(vec![
            ("method".to_string(), Value::StringVal(method)),
            ("path".to_string(), Value::StringVal(path)),
            ("version".to_string(), Value::StringVal(version)),
            ("headers".to_string(), Value::StringVal(headers_json)),
            ("body".to_string(), Value::StringVal(body)),
        ])
    }
}

// ============================================================================
// HTTP Parsing Logic
// ============================================================================

fn parse_http_request(request: &str) -> Result<(String, String, String, String, String), String> {
    // Split request into headers section and body
    let parts: Vec<&str> = request.splitn(2, "\r\n\r\n").collect();

    if parts.is_empty() {
        return Err("Empty request".to_string());
    }

    let headers_section = parts[0];
    let body = parts.get(1).unwrap_or(&"").to_string();

    // Split headers section into lines
    let lines: Vec<&str> = headers_section.split("\r\n").collect();

    if lines.is_empty() {
        return Err("No request line found".to_string());
    }

    // Parse request line (first line)
    let request_line = lines[0];
    let request_parts: Vec<&str> = request_line.split_whitespace().collect();

    if request_parts.len() != 3 {
        return Err(format!(
            "Invalid request line format. Expected 'METHOD PATH VERSION', got: '{}'",
            request_line
        ));
    }

    let method = request_parts[0].to_string();
    let path = request_parts[1].to_string();
    let version = request_parts[2].to_string();

    // Parse headers (remaining lines) and build JSON manually
    let mut header_pairs = Vec::new();

    for line in &lines[1..] {
        if line.is_empty() {
            continue;
        }

        // Split header line by first colon
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim();

            // Escape special characters in the value for JSON
            let escaped_value = escape_json_string(value);
            header_pairs.push(format!("\"{}\":\"{}\"", key, escaped_value));
        }
        // Skip malformed header lines
    }

    // Build JSON object string manually
    let headers_json = format!("{{{}}}", header_pairs.join(","));

    Ok((method, path, version, headers_json, body))
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
    fn test_simple_get_request() {
        let request = "GET /api/users HTTP/1.1\r\nHost: example.com\r\nUser-Agent: TestClient/1.0\r\n\r\n";

        let inputs = vec![("raw_request".to_string(), Value::StringVal(request.to_string()))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 5);

        assert_eq!(result[0].0, "method");
        assert_eq!(result[0].1, Value::StringVal("GET".to_string()));

        assert_eq!(result[1].0, "path");
        assert_eq!(result[1].1, Value::StringVal("/api/users".to_string()));

        assert_eq!(result[2].0, "version");
        assert_eq!(result[2].1, Value::StringVal("HTTP/1.1".to_string()));

        assert_eq!(result[3].0, "headers");
        if let Value::StringVal(headers) = &result[3].1 {
            assert!(headers.contains("host"));
            assert!(headers.contains("example.com"));
            assert!(headers.contains("user-agent"));
        } else {
            panic!("Expected StringVal for headers");
        }

        assert_eq!(result[4].0, "body");
        assert_eq!(result[4].1, Value::StringVal("".to_string()));
    }

    #[test]
    fn test_post_request_with_body() {
        let request = "POST /api/users HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 27\r\n\r\n{\"name\":\"John\",\"age\":30}";

        let inputs = vec![("raw_request".to_string(), Value::StringVal(request.to_string()))];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("POST".to_string()));
        assert_eq!(result[1].1, Value::StringVal("/api/users".to_string()));
        assert_eq!(result[4].1, Value::StringVal("{\"name\":\"John\",\"age\":30}".to_string()));
    }

    #[test]
    fn test_request_with_query_string() {
        let request = "GET /search?q=rust&page=2 HTTP/1.1\r\nHost: example.com\r\n\r\n";

        let inputs = vec![("raw_request".to_string(), Value::StringVal(request.to_string()))];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::StringVal("/search?q=rust&page=2".to_string()));
    }

    #[test]
    fn test_multiple_headers() {
        let request = "GET / HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\nAccept-Encoding: gzip\r\nConnection: keep-alive\r\n\r\n";

        let inputs = vec![("raw_request".to_string(), Value::StringVal(request.to_string()))];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[3].1 {
            // Verify JSON structure and values
            assert!(headers.contains("\"host\":\"example.com\""));
            assert!(headers.contains("\"accept\":\"*/*\""));
            assert!(headers.contains("\"accept-encoding\":\"gzip\""));
            assert!(headers.contains("\"connection\":\"keep-alive\""));
        } else {
            panic!("Expected StringVal for headers");
        }
    }

    #[test]
    fn test_case_insensitive_headers() {
        let request = "GET / HTTP/1.1\r\nContent-Type: text/html\r\nCONTENT-LENGTH: 100\r\n\r\n";

        let inputs = vec![("raw_request".to_string(), Value::StringVal(request.to_string()))];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[3].1 {
            // Headers should be lowercase in the JSON
            assert!(headers.contains("\"content-type\":\"text/html\""));
            assert!(headers.contains("\"content-length\":\"100\""));
        } else {
            panic!("Expected StringVal for headers");
        }
    }

    #[test]
    fn test_invalid_request_line() {
        let request = "INVALID REQUEST\r\n\r\n";

        let inputs = vec![("raw_request".to_string(), Value::StringVal(request.to_string()))];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid request line"));
    }

    #[test]
    fn test_empty_request() {
        let request = "";

        let inputs = vec![("raw_request".to_string(), Value::StringVal(request.to_string()))];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_input() {
        let inputs = vec![];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("raw_request".to_string()));
    }
}
