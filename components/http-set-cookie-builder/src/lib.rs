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
            name: "HTTP Set-Cookie Builder".to_string(),
            version: "1.0.0".to_string(),
            description: "Build Set-Cookie header values with all cookie attributes".to_string(),
            author: "WasmFlow HTTP Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "name".to_string(),
                data_type: DataType::String,
                description: "Cookie name (required)".to_string(),
                required: true,
            },
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::String,
                description: "Cookie value (required)".to_string(),
                required: true,
            },
            PortSpec {
                name: "expires".to_string(),
                data_type: DataType::String,
                description: "Expiration date (RFC 2822 format, e.g., 'Wed, 21 Oct 2025 07:28:00 GMT')".to_string(),
                required: false,
            },
            PortSpec {
                name: "max_age".to_string(),
                data_type: DataType::U32,
                description: "Max age in seconds (e.g., 3600 for 1 hour)".to_string(),
                required: false,
            },
            PortSpec {
                name: "domain".to_string(),
                data_type: DataType::String,
                description: "Domain scope for the cookie (e.g., '.example.com')".to_string(),
                required: false,
            },
            PortSpec {
                name: "path".to_string(),
                data_type: DataType::String,
                description: "Path scope for the cookie (default: '/')".to_string(),
                required: false,
            },
            PortSpec {
                name: "secure".to_string(),
                data_type: DataType::Bool,
                description: "Send cookie only over HTTPS (default: false)".to_string(),
                required: false,
            },
            PortSpec {
                name: "http_only".to_string(),
                data_type: DataType::Bool,
                description: "Prevent JavaScript access to cookie (default: false)".to_string(),
                required: false,
            },
            PortSpec {
                name: "same_site".to_string(),
                data_type: DataType::String,
                description: "SameSite attribute: 'Strict', 'Lax', or 'None' (default: none)".to_string(),
                required: false,
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "set_cookie".to_string(),
                data_type: DataType::String,
                description: "Complete Set-Cookie header value".to_string(),
                required: true,
            },
            PortSpec {
                name: "attribute_count".to_string(),
                data_type: DataType::U32,
                description: "Number of attributes set (including name=value)".to_string(),
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
        // Extract required inputs
        let name = extract_required_string(&inputs, "name")?;
        let value = extract_required_string(&inputs, "value")?;

        // Validate cookie name
        if name.trim().is_empty() {
            return Err(ExecutionError {
                message: "Cookie name cannot be empty".to_string(),
                input_name: Some("name".to_string()),
                recovery_hint: Some("Provide a valid cookie name".to_string()),
            });
        }

        // Extract optional inputs
        let expires = extract_optional_string(&inputs, "expires");
        let max_age = extract_optional_u32(&inputs, "max_age");
        let domain = extract_optional_string(&inputs, "domain");
        let path = extract_optional_string(&inputs, "path");
        let secure = extract_optional_bool(&inputs, "secure");
        let http_only = extract_optional_bool(&inputs, "http_only");
        let same_site = extract_optional_string(&inputs, "same_site");

        // Validate SameSite if provided
        if let Some(ref ss) = same_site {
            if !ss.is_empty()
                && ss != "Strict"
                && ss != "Lax"
                && ss != "None"
            {
                return Err(ExecutionError {
                    message: format!(
                        "Invalid SameSite value: '{}'. Must be 'Strict', 'Lax', or 'None'",
                        ss
                    ),
                    input_name: Some("same_site".to_string()),
                    recovery_hint: Some(
                        "Use 'Strict', 'Lax', or 'None' (case-sensitive)".to_string(),
                    ),
                });
            }
        }

        // Build Set-Cookie value
        let (set_cookie, count) = build_set_cookie(
            &name,
            &value,
            expires.as_deref(),
            max_age,
            domain.as_deref(),
            path.as_deref(),
            secure,
            http_only,
            same_site.as_deref(),
        );

        Ok(vec![
            ("set_cookie".to_string(), Value::StringVal(set_cookie)),
            ("attribute_count".to_string(), Value::U32Val(count)),
        ])
    }
}

fn build_set_cookie(
    name: &str,
    value: &str,
    expires: Option<&str>,
    max_age: Option<u32>,
    domain: Option<&str>,
    path: Option<&str>,
    secure: Option<bool>,
    http_only: Option<bool>,
    same_site: Option<&str>,
) -> (String, u32) {
    let mut parts = Vec::new();
    let mut count = 1u32; // name=value counts as 1

    // name=value (required)
    parts.push(format!("{}={}", name, value));

    // Expires
    if let Some(exp) = expires {
        if !exp.is_empty() {
            parts.push(format!("Expires={}", exp));
            count += 1;
        }
    }

    // Max-Age
    if let Some(age) = max_age {
        parts.push(format!("Max-Age={}", age));
        count += 1;
    }

    // Domain
    if let Some(dom) = domain {
        if !dom.is_empty() {
            parts.push(format!("Domain={}", dom));
            count += 1;
        }
    }

    // Path
    if let Some(p) = path {
        if !p.is_empty() {
            parts.push(format!("Path={}", p));
            count += 1;
        }
    }

    // Secure (flag)
    if let Some(true) = secure {
        parts.push("Secure".to_string());
        count += 1;
    }

    // HttpOnly (flag)
    if let Some(true) = http_only {
        parts.push("HttpOnly".to_string());
        count += 1;
    }

    // SameSite
    if let Some(ss) = same_site {
        if !ss.is_empty() {
            parts.push(format!("SameSite={}", ss));
            count += 1;
        }
    }

    (parts.join("; "), count)
}

fn extract_required_string(
    inputs: &[(String, Value)],
    name: &str,
) -> Result<String, ExecutionError> {
    let input = inputs.iter().find(|(n, _)| n == name).ok_or_else(|| {
        ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(format!("Provide a value for '{}'", name)),
        }
    })?;

    match &input.1 {
        Value::StringVal(s) => Ok(s.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected string for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a string value".to_string()),
        }),
    }
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
    fn test_simple_cookie() {
        let (cookie, count) = build_set_cookie("session", "abc123", None, None, None, None, None, None, None);
        assert_eq!(count, 1);
        assert_eq!(cookie, "session=abc123");
    }

    #[test]
    fn test_cookie_with_expires() {
        let (cookie, count) = build_set_cookie(
            "session",
            "abc123",
            Some("Wed, 21 Oct 2025 07:28:00 GMT"),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; Expires=Wed, 21 Oct 2025 07:28:00 GMT");
    }

    #[test]
    fn test_cookie_with_max_age() {
        let (cookie, count) = build_set_cookie("session", "abc123", None, Some(3600), None, None, None, None, None);
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; Max-Age=3600");
    }

    #[test]
    fn test_cookie_with_domain() {
        let (cookie, count) = build_set_cookie(
            "session",
            "abc123",
            None,
            None,
            Some(".example.com"),
            None,
            None,
            None,
            None,
        );
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; Domain=.example.com");
    }

    #[test]
    fn test_cookie_with_path() {
        let (cookie, count) = build_set_cookie(
            "session",
            "abc123",
            None,
            None,
            None,
            Some("/app"),
            None,
            None,
            None,
        );
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; Path=/app");
    }

    #[test]
    fn test_cookie_with_secure() {
        let (cookie, count) = build_set_cookie("session", "abc123", None, None, None, None, Some(true), None, None);
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; Secure");
    }

    #[test]
    fn test_cookie_with_httponly() {
        let (cookie, count) = build_set_cookie("session", "abc123", None, None, None, None, None, Some(true), None);
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; HttpOnly");
    }

    #[test]
    fn test_cookie_with_samesite_strict() {
        let (cookie, count) = build_set_cookie(
            "session",
            "abc123",
            None,
            None,
            None,
            None,
            None,
            None,
            Some("Strict"),
        );
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; SameSite=Strict");
    }

    #[test]
    fn test_cookie_with_samesite_lax() {
        let (cookie, count) = build_set_cookie(
            "session",
            "abc123",
            None,
            None,
            None,
            None,
            None,
            None,
            Some("Lax"),
        );
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; SameSite=Lax");
    }

    #[test]
    fn test_cookie_with_samesite_none() {
        let (cookie, count) = build_set_cookie(
            "session",
            "abc123",
            None,
            None,
            None,
            None,
            None,
            None,
            Some("None"),
        );
        assert_eq!(count, 2);
        assert_eq!(cookie, "session=abc123; SameSite=None");
    }

    #[test]
    fn test_full_cookie_all_attributes() {
        let (cookie, count) = build_set_cookie(
            "session",
            "abc123",
            Some("Wed, 21 Oct 2025 07:28:00 GMT"),
            Some(3600),
            Some(".example.com"),
            Some("/"),
            Some(true),
            Some(true),
            Some("Strict"),
        );
        assert_eq!(count, 9);
        assert!(cookie.contains("session=abc123"));
        assert!(cookie.contains("Expires=Wed, 21 Oct 2025 07:28:00 GMT"));
        assert!(cookie.contains("Max-Age=3600"));
        assert!(cookie.contains("Domain=.example.com"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Strict"));
    }

    #[test]
    fn test_secure_httponly_cookie() {
        let (cookie, count) = build_set_cookie(
            "token",
            "xyz789",
            None,
            None,
            None,
            Some("/"),
            Some(true),
            Some(true),
            Some("Strict"),
        );
        assert_eq!(count, 5);
        assert_eq!(cookie, "token=xyz789; Path=/; Secure; HttpOnly; SameSite=Strict");
    }

    #[test]
    fn test_session_cookie_no_expiry() {
        // Session cookie (no Expires or Max-Age)
        let (cookie, count) = build_set_cookie(
            "session",
            "temp123",
            None,
            None,
            None,
            Some("/"),
            Some(true),
            Some(true),
            None,
        );
        assert_eq!(count, 4);
        assert!(!cookie.contains("Expires"));
        assert!(!cookie.contains("Max-Age"));
    }

    #[test]
    fn test_cookie_with_empty_value() {
        let (cookie, count) = build_set_cookie("logout", "", None, None, None, None, None, None, None);
        assert_eq!(count, 1);
        assert_eq!(cookie, "logout=");
    }

    #[test]
    fn test_delete_cookie() {
        // Delete cookie by setting Max-Age=0
        let (cookie, count) = build_set_cookie(
            "session",
            "",
            Some("Thu, 01 Jan 1970 00:00:00 GMT"),
            Some(0),
            None,
            Some("/"),
            None,
            None,
            None,
        );
        assert_eq!(count, 4);
        assert!(cookie.contains("Max-Age=0"));
        assert!(cookie.contains("Expires=Thu, 01 Jan 1970 00:00:00 GMT"));
    }

    #[test]
    fn test_empty_string_attributes_ignored() {
        let (cookie, count) = build_set_cookie(
            "test",
            "value",
            Some(""),
            None,
            Some(""),
            Some(""),
            None,
            None,
            Some(""),
        );
        assert_eq!(count, 1);
        assert_eq!(cookie, "test=value");
    }

    #[test]
    fn test_flags_false_not_included() {
        let (cookie, count) = build_set_cookie(
            "test",
            "value",
            None,
            None,
            None,
            None,
            Some(false),
            Some(false),
            None,
        );
        assert_eq!(count, 1);
        assert_eq!(cookie, "test=value");
        assert!(!cookie.contains("Secure"));
        assert!(!cookie.contains("HttpOnly"));
    }

    #[test]
    fn test_real_world_auth_cookie() {
        let (cookie, count) = build_set_cookie(
            "auth_token",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9",
            None,
            Some(86400), // 24 hours
            Some(".app.example.com"),
            Some("/api"),
            Some(true),
            Some(true),
            Some("Strict"),
        );
        assert_eq!(count, 7);
        assert!(cookie.starts_with("auth_token=eyJhbGci"));
        assert!(cookie.contains("Max-Age=86400"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("HttpOnly"));
    }
}
