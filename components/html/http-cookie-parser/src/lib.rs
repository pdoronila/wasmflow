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
            name: "HTTP Cookie Parser".to_string(),
            version: "1.0.0".to_string(),
            description: "Parse HTTP Cookie header into key-value pairs".to_string(),
            author: "WasmFlow HTTP Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "cookie_header".to_string(),
            data_type: DataType::String,
            description: "Value of the Cookie header (e.g., 'session=abc123; user=alice')".to_string(),
            required: true,
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "cookies_json".to_string(),
                data_type: DataType::String,
                description: "Cookies as JSON object with cookie names as keys".to_string(),
                required: true,
            },
            PortSpec {
                name: "cookie_count".to_string(),
                data_type: DataType::U32,
                description: "Number of cookies parsed".to_string(),
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
        let cookie_header = inputs
            .iter()
            .find(|(name, _)| name == "cookie_header")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: cookie_header".to_string(),
                input_name: Some("cookie_header".to_string()),
                recovery_hint: Some("Provide the Cookie header value to parse".to_string()),
            })?;

        let header_value = match &cookie_header.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!(
                        "Expected string for 'cookie_header', got {:?}",
                        cookie_header.1
                    ),
                    input_name: Some("cookie_header".to_string()),
                    recovery_hint: Some(
                        "Provide a string value like 'session=abc; user=alice'".to_string(),
                    ),
                });
            }
        };

        // Parse cookies
        let (cookies_json, count) = parse_cookies(header_value);

        Ok(vec![
            ("cookies_json".to_string(), Value::StringVal(cookies_json)),
            ("cookie_count".to_string(), Value::U32Val(count)),
        ])
    }
}

fn parse_cookies(header_value: &str) -> (String, u32) {
    if header_value.trim().is_empty() {
        return ("{}".to_string(), 0);
    }

    let mut cookies = Vec::new();
    let mut count = 0u32;

    // Split by semicolon
    for pair in header_value.split(';') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        // Split by first '=' to get name and value
        if let Some(eq_pos) = pair.find('=') {
            let name = pair[..eq_pos].trim();
            let value = pair[eq_pos + 1..].trim();

            if !name.is_empty() {
                cookies.push(format!("\"{}\":\"{}\"", escape_json(name), escape_json(value)));
                count += 1;
            }
        } else {
            // Cookie without value (name only) - treat as empty value
            if !pair.is_empty() {
                cookies.push(format!("\"{}\":\"\"", escape_json(pair)));
                count += 1;
            }
        }
    }

    if cookies.is_empty() {
        return ("{}".to_string(), 0);
    }

    (format!("{{{}}}", cookies.join(",")), count)
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_cookie() {
        let (json, count) = parse_cookies("session=abc123");
        assert_eq!(count, 1);
        assert_eq!(json, "{\"session\":\"abc123\"}");
    }

    #[test]
    fn test_multiple_cookies() {
        let (json, count) = parse_cookies("session=abc123; user=alice; theme=dark");
        assert_eq!(count, 3);
        assert!(json.contains("\"session\":\"abc123\""));
        assert!(json.contains("\"user\":\"alice\""));
        assert!(json.contains("\"theme\":\"dark\""));
    }

    #[test]
    fn test_cookies_with_spaces() {
        let (json, count) = parse_cookies("name=value; another = test ");
        assert_eq!(count, 2);
        assert!(json.contains("\"name\":\"value\""));
        assert!(json.contains("\"another\":\"test\""));
    }

    #[test]
    fn test_empty_cookie_header() {
        let (json, count) = parse_cookies("");
        assert_eq!(count, 0);
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_whitespace_only() {
        let (json, count) = parse_cookies("   ");
        assert_eq!(count, 0);
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_cookie_with_equals_in_value() {
        let (json, count) = parse_cookies("data=key=value");
        assert_eq!(count, 1);
        assert_eq!(json, "{\"data\":\"key=value\"}");
    }

    #[test]
    fn test_cookie_without_value() {
        let (json, count) = parse_cookies("flag");
        assert_eq!(count, 1);
        assert_eq!(json, "{\"flag\":\"\"}");
    }

    #[test]
    fn test_multiple_cookies_some_without_values() {
        let (json, count) = parse_cookies("session=abc; flag; user=alice");
        assert_eq!(count, 3);
        assert!(json.contains("\"session\":\"abc\""));
        assert!(json.contains("\"flag\":\"\""));
        assert!(json.contains("\"user\":\"alice\""));
    }

    #[test]
    fn test_cookie_with_special_characters() {
        let (json, count) = parse_cookies("token=abc-123_xyz.789");
        assert_eq!(count, 1);
        assert!(json.contains("\"token\":\"abc-123_xyz.789\""));
    }

    #[test]
    fn test_json_escaping() {
        let (json, _) = parse_cookies("data=\"quoted\"");
        assert!(json.contains("\\\"quoted\\\""));
    }

    #[test]
    fn test_cookie_with_url_encoded_value() {
        // Component doesn't decode URL encoding - that's separate
        let (json, count) = parse_cookies("name=John%20Doe");
        assert_eq!(count, 1);
        assert!(json.contains("\"name\":\"John%20Doe\""));
    }

    #[test]
    fn test_empty_cookie_names_ignored() {
        let (json, count) = parse_cookies("=value; name=value2");
        assert_eq!(count, 1);
        assert!(json.contains("\"name\":\"value2\""));
        assert!(!json.contains("\"\":\"value\""));
    }

    #[test]
    fn test_trailing_semicolon() {
        let (json, count) = parse_cookies("session=abc; user=alice;");
        assert_eq!(count, 2);
        assert!(json.contains("\"session\":\"abc\""));
        assert!(json.contains("\"user\":\"alice\""));
    }

    #[test]
    fn test_multiple_semicolons() {
        let (json, count) = parse_cookies("session=abc;; user=alice");
        assert_eq!(count, 2);
        assert!(json.contains("\"session\":\"abc\""));
        assert!(json.contains("\"user\":\"alice\""));
    }

    #[test]
    fn test_real_world_cookie() {
        let (json, count) = parse_cookies(
            "SESSIONID=38afes7a8; _ga=GA1.2.1234567890.1234567890; logged_in=yes",
        );
        assert_eq!(count, 3);
        assert!(json.contains("\"SESSIONID\":\"38afes7a8\""));
        assert!(json.contains("\"_ga\":\"GA1.2.1234567890.1234567890\""));
        assert!(json.contains("\"logged_in\":\"yes\""));
    }

    #[test]
    fn test_cookie_with_base64_value() {
        let (json, count) = parse_cookies("token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
        assert_eq!(count, 1);
        assert!(json.contains("\"token\":\"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9\""));
    }
}
