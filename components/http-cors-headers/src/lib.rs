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
            name: "HTTP CORS Headers Builder".to_string(),
            version: "1.0.0".to_string(),
            description: "Build CORS (Cross-Origin Resource Sharing) headers for HTTP responses".to_string(),
            author: "WasmFlow HTTP Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "origin".to_string(),
                data_type: DataType::String,
                description: "Allowed origin (* for all, or specific domain like 'https://example.com')".to_string(),
                required: false,
            },
            PortSpec {
                name: "methods".to_string(),
                data_type: DataType::String,
                description: "Allowed HTTP methods (comma-separated, e.g., 'GET, POST, PUT, DELETE')".to_string(),
                required: false,
            },
            PortSpec {
                name: "headers".to_string(),
                data_type: DataType::String,
                description: "Allowed request headers (comma-separated, e.g., 'Content-Type, Authorization')".to_string(),
                required: false,
            },
            PortSpec {
                name: "credentials".to_string(),
                data_type: DataType::Bool,
                description: "Allow credentials (cookies, authorization headers). Default: false".to_string(),
                required: false,
            },
            PortSpec {
                name: "max_age".to_string(),
                data_type: DataType::U32,
                description: "Preflight cache duration in seconds (e.g., 3600 for 1 hour)".to_string(),
                required: false,
            },
            PortSpec {
                name: "expose_headers".to_string(),
                data_type: DataType::String,
                description: "Headers to expose to client (comma-separated, e.g., 'X-Request-ID')".to_string(),
                required: false,
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "headers_json".to_string(),
                data_type: DataType::String,
                description: "CORS headers as JSON object (lowercase keys)".to_string(),
                required: true,
            },
            PortSpec {
                name: "header_count".to_string(),
                data_type: DataType::U32,
                description: "Number of CORS headers added".to_string(),
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
        // Extract optional inputs
        let origin = extract_optional_string(&inputs, "origin");
        let methods = extract_optional_string(&inputs, "methods");
        let headers = extract_optional_string(&inputs, "headers");
        let credentials = extract_optional_bool(&inputs, "credentials");
        let max_age = extract_optional_u32(&inputs, "max_age");
        let expose_headers = extract_optional_string(&inputs, "expose_headers");

        // Build CORS headers JSON
        let (headers_json, count) = build_cors_headers(
            origin.as_deref(),
            methods.as_deref(),
            headers.as_deref(),
            credentials,
            max_age,
            expose_headers.as_deref(),
        );

        Ok(vec![
            ("headers_json".to_string(), Value::StringVal(headers_json)),
            ("header_count".to_string(), Value::U32Val(count)),
        ])
    }
}

fn build_cors_headers(
    origin: Option<&str>,
    methods: Option<&str>,
    headers: Option<&str>,
    credentials: Option<bool>,
    max_age: Option<u32>,
    expose_headers: Option<&str>,
) -> (String, u32) {
    let mut parts = Vec::new();
    let mut count = 0u32;

    // Access-Control-Allow-Origin
    if let Some(o) = origin {
        if !o.is_empty() {
            parts.push(format!("\"access-control-allow-origin\":\"{}\"", escape_json(o)));
            count += 1;
        }
    }

    // Access-Control-Allow-Methods
    if let Some(m) = methods {
        if !m.is_empty() {
            parts.push(format!("\"access-control-allow-methods\":\"{}\"", escape_json(m)));
            count += 1;
        }
    }

    // Access-Control-Allow-Headers
    if let Some(h) = headers {
        if !h.is_empty() {
            parts.push(format!("\"access-control-allow-headers\":\"{}\"", escape_json(h)));
            count += 1;
        }
    }

    // Access-Control-Allow-Credentials
    if let Some(c) = credentials {
        if c {
            parts.push("\"access-control-allow-credentials\":\"true\"".to_string());
            count += 1;
        }
    }

    // Access-Control-Max-Age
    if let Some(age) = max_age {
        parts.push(format!("\"access-control-max-age\":\"{}\"", age));
        count += 1;
    }

    // Access-Control-Expose-Headers
    if let Some(eh) = expose_headers {
        if !eh.is_empty() {
            parts.push(format!("\"access-control-expose-headers\":\"{}\"", escape_json(eh)));
            count += 1;
        }
    }

    if parts.is_empty() {
        return ("{}".to_string(), 0);
    }

    (format!("{{{}}}", parts.join(",")), count)
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn extract_optional_string(inputs: &[(String, Value)], name: &str) -> Option<String> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| match v {
            Value::StringVal(s) => Some(s.clone()),
            _ => None,
        })
}

fn extract_optional_bool(inputs: &[(String, Value)], name: &str) -> Option<bool> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| match v {
            Value::BoolVal(b) => Some(*b),
            _ => None,
        })
}

fn extract_optional_u32(inputs: &[(String, Value)], name: &str) -> Option<u32> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| match v {
            Value::U32Val(n) => Some(*n),
            _ => None,
        })
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_all_origins() {
        let (json, count) = build_cors_headers(Some("*"), None, None, None, None, None);
        assert_eq!(count, 1);
        assert!(json.contains("\"access-control-allow-origin\":\"*\""));
    }

    #[test]
    fn test_specific_origin() {
        let (json, count) = build_cors_headers(
            Some("https://example.com"),
            None,
            None,
            None,
            None,
            None,
        );
        assert_eq!(count, 1);
        assert!(json.contains("\"access-control-allow-origin\":\"https://example.com\""));
    }

    #[test]
    fn test_allowed_methods() {
        let (json, count) = build_cors_headers(
            None,
            Some("GET, POST, PUT, DELETE"),
            None,
            None,
            None,
            None,
        );
        assert_eq!(count, 1);
        assert!(json.contains("\"access-control-allow-methods\":\"GET, POST, PUT, DELETE\""));
    }

    #[test]
    fn test_allowed_headers() {
        let (json, count) = build_cors_headers(
            None,
            None,
            Some("Content-Type, Authorization"),
            None,
            None,
            None,
        );
        assert_eq!(count, 1);
        assert!(json.contains("\"access-control-allow-headers\":\"Content-Type, Authorization\""));
    }

    #[test]
    fn test_credentials_true() {
        let (json, count) = build_cors_headers(None, None, None, Some(true), None, None);
        assert_eq!(count, 1);
        assert!(json.contains("\"access-control-allow-credentials\":\"true\""));
    }

    #[test]
    fn test_credentials_false() {
        let (json, count) = build_cors_headers(None, None, None, Some(false), None, None);
        assert_eq!(count, 0);
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_max_age() {
        let (json, count) = build_cors_headers(None, None, None, None, Some(3600), None);
        assert_eq!(count, 1);
        assert!(json.contains("\"access-control-max-age\":\"3600\""));
    }

    #[test]
    fn test_expose_headers() {
        let (json, count) = build_cors_headers(
            None,
            None,
            None,
            None,
            None,
            Some("X-Request-ID, X-Custom-Header"),
        );
        assert_eq!(count, 1);
        assert!(json.contains("\"access-control-expose-headers\":\"X-Request-ID, X-Custom-Header\""));
    }

    #[test]
    fn test_all_cors_headers() {
        let (json, count) = build_cors_headers(
            Some("https://example.com"),
            Some("GET, POST"),
            Some("Content-Type"),
            Some(true),
            Some(7200),
            Some("X-Custom"),
        );
        assert_eq!(count, 6);
        assert!(json.contains("\"access-control-allow-origin\":\"https://example.com\""));
        assert!(json.contains("\"access-control-allow-methods\":\"GET, POST\""));
        assert!(json.contains("\"access-control-allow-headers\":\"Content-Type\""));
        assert!(json.contains("\"access-control-allow-credentials\":\"true\""));
        assert!(json.contains("\"access-control-max-age\":\"7200\""));
        assert!(json.contains("\"access-control-expose-headers\":\"X-Custom\""));
    }

    #[test]
    fn test_empty_inputs() {
        let (json, count) = build_cors_headers(None, None, None, None, None, None);
        assert_eq!(count, 0);
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_empty_strings_ignored() {
        let (json, count) = build_cors_headers(Some(""), Some(""), Some(""), None, None, Some(""));
        assert_eq!(count, 0);
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_json_escaping() {
        let (json, _) = build_cors_headers(
            Some("https://example.com/path?query=\"value\""),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(json.contains("\\\"value\\\""));
    }

    #[test]
    fn test_wildcard_with_credentials_warning() {
        // Note: In real CORS, origin="*" with credentials=true is invalid
        // But this component just builds headers, validation is separate
        let (json, count) = build_cors_headers(Some("*"), None, None, Some(true), None, None);
        assert_eq!(count, 2);
        assert!(json.contains("\"access-control-allow-origin\":\"*\""));
        assert!(json.contains("\"access-control-allow-credentials\":\"true\""));
    }

    #[test]
    fn test_multiple_methods() {
        let (json, count) = build_cors_headers(
            None,
            Some("GET, POST, PUT, DELETE, PATCH, OPTIONS"),
            None,
            None,
            None,
            None,
        );
        assert_eq!(count, 1);
        assert!(json.contains("GET, POST, PUT, DELETE, PATCH, OPTIONS"));
    }

    #[test]
    fn test_max_age_zero() {
        let (json, count) = build_cors_headers(None, None, None, None, Some(0), None);
        assert_eq!(count, 1);
        assert!(json.contains("\"access-control-max-age\":\"0\""));
    }

    #[test]
    fn test_preflight_scenario() {
        // Typical preflight response headers
        let (json, count) = build_cors_headers(
            Some("https://app.example.com"),
            Some("GET, POST, PUT, DELETE"),
            Some("Content-Type, Authorization, X-API-Key"),
            Some(true),
            Some(86400), // 24 hours
            None,
        );
        assert_eq!(count, 5);
        assert!(json.contains("\"access-control-allow-origin\":\"https://app.example.com\""));
        assert!(json.contains("\"access-control-allow-methods\":\"GET, POST, PUT, DELETE\""));
        assert!(json.contains("\"access-control-allow-headers\":\"Content-Type, Authorization, X-API-Key\""));
        assert!(json.contains("\"access-control-allow-credentials\":\"true\""));
        assert!(json.contains("\"access-control-max-age\":\"86400\""));
    }
}
