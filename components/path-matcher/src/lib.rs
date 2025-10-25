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
            name: "Path Matcher".to_string(),
            version: "1.0.0".to_string(),
            description: "Matches URL paths against patterns and extracts named parameters (e.g., /api/users/:id)".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "path".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "URL path to match (e.g., '/api/users/123')".to_string(),
            },
            PortSpec {
                name: "pattern".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Pattern to match against. Use :name for parameters, * for wildcards (e.g., '/api/users/:id')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "matched".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if path matches the pattern".to_string(),
            },
            PortSpec {
                name: "params".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Extracted parameters as JSON object (e.g., {\"id\":\"123\"})".to_string(),
            },
            PortSpec {
                name: "wildcard".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Wildcard captured value (if pattern contains *)".to_string(),
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
        // Extract path
        let path = inputs
            .iter()
            .find(|(name, _)| name == "path")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: path".to_string(),
                input_name: Some("path".to_string()),
                recovery_hint: Some("Connect a URL path to this input".to_string()),
            })?;

        let path_str = match &path.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'path', got {:?}", path.1),
                    input_name: Some("path".to_string()),
                    recovery_hint: Some("Provide a string path".to_string()),
                });
            }
        };

        // Extract pattern
        let pattern = inputs
            .iter()
            .find(|(name, _)| name == "pattern")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: pattern".to_string(),
                input_name: Some("pattern".to_string()),
                recovery_hint: Some("Connect a pattern to this input (e.g., '/api/users/:id')".to_string()),
            })?;

        let pattern_str = match &pattern.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'pattern', got {:?}", pattern.1),
                    input_name: Some("pattern".to_string()),
                    recovery_hint: Some("Provide a string pattern".to_string()),
                });
            }
        };

        // Match the path against the pattern
        let (matched, params, wildcard) = match_path(path_str, pattern_str);

        // Build params JSON
        let params_json = build_params_json(&params);

        Ok(vec![
            ("matched".to_string(), Value::BoolVal(matched)),
            ("params".to_string(), Value::StringVal(params_json)),
            ("wildcard".to_string(), Value::StringVal(wildcard)),
        ])
    }
}

// ============================================================================
// Path Matching Logic
// ============================================================================

/// Match a path against a pattern and extract parameters
/// Returns: (matched, params, wildcard)
fn match_path(path: &str, pattern: &str) -> (bool, Vec<(String, String)>, String) {
    // Split both path and pattern by '/'
    let path_segments: Vec<&str> = path.split('/').collect();
    let pattern_segments: Vec<&str> = pattern.split('/').collect();

    let mut params = Vec::new();
    let mut wildcard = String::new();

    // Check each segment
    for (i, pattern_seg) in pattern_segments.iter().enumerate() {
        // Check if we've run out of path segments
        if i >= path_segments.len() {
            // Pattern has more segments than path
            // Only matches if remaining pattern segments are wildcard
            if *pattern_seg == "*" {
                // Wildcard at end with no more path - empty wildcard
                return (true, params, wildcard);
            }
            return (false, params, wildcard);
        }

        let path_seg = path_segments[i];

        if pattern_seg.starts_with(':') {
            // Named parameter - extract name and value
            let param_name = &pattern_seg[1..]; // Remove leading ':'
            params.push((param_name.to_string(), path_seg.to_string()));
        } else if *pattern_seg == "*" {
            // Wildcard - capture remaining path
            wildcard = path_segments[i..].join("/");
            return (true, params, wildcard);
        } else if *pattern_seg != path_seg {
            // Exact match required but segments don't match
            return (false, params, wildcard);
        }
        // Else: exact match, continue
    }

    // Check if path has more segments than pattern
    if path_segments.len() > pattern_segments.len() {
        // Path has more segments than pattern
        return (false, params, wildcard);
    }

    // All segments matched
    (true, params, wildcard)
}

/// Build JSON object from parameter key-value pairs
fn build_params_json(params: &[(String, String)]) -> String {
    if params.is_empty() {
        return "{}".to_string();
    }

    let pairs: Vec<String> = params
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
    fn test_exact_match() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
            ("pattern".to_string(), Value::StringVal("/api/users".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true)); // matched
        assert_eq!(result[1].1, Value::StringVal("{}".to_string())); // params
        assert_eq!(result[2].1, Value::StringVal("".to_string())); // wildcard
    }

    #[test]
    fn test_single_parameter() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api/users/123".to_string())),
            ("pattern".to_string(), Value::StringVal("/api/users/:id".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));

        if let Value::StringVal(params) = &result[1].1 {
            assert!(params.contains("\"id\":\"123\""));
        } else {
            panic!("Expected StringVal for params");
        }
    }

    #[test]
    fn test_multiple_parameters() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api/users/123/posts/456".to_string())),
            ("pattern".to_string(), Value::StringVal("/api/users/:userId/posts/:postId".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));

        if let Value::StringVal(params) = &result[1].1 {
            assert!(params.contains("\"userId\":\"123\""));
            assert!(params.contains("\"postId\":\"456\""));
        }
    }

    #[test]
    fn test_wildcard_match() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/static/css/main.css".to_string())),
            ("pattern".to_string(), Value::StringVal("/static/*".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));
        assert_eq!(result[2].1, Value::StringVal("css/main.css".to_string())); // wildcard
    }

    #[test]
    fn test_wildcard_captures_all_remaining() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/files/docs/2024/report.pdf".to_string())),
            ("pattern".to_string(), Value::StringVal("/files/*".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));
        assert_eq!(result[2].1, Value::StringVal("docs/2024/report.pdf".to_string()));
    }

    #[test]
    fn test_no_match_different_path() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api/posts".to_string())),
            ("pattern".to_string(), Value::StringVal("/api/users".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(false));
        assert_eq!(result[1].1, Value::StringVal("{}".to_string()));
    }

    #[test]
    fn test_no_match_path_too_short() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api".to_string())),
            ("pattern".to_string(), Value::StringVal("/api/users/:id".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(false));
    }

    #[test]
    fn test_no_match_path_too_long() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api/users/123/extra".to_string())),
            ("pattern".to_string(), Value::StringVal("/api/users/:id".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(false));
    }

    #[test]
    fn test_root_path() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/".to_string())),
            ("pattern".to_string(), Value::StringVal("/".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));
    }

    #[test]
    fn test_empty_path_segments() {
        // Paths like "/api//users" should work
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api//users".to_string())),
            ("pattern".to_string(), Value::StringVal("/api/:empty/users".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));

        if let Value::StringVal(params) = &result[1].1 {
            assert!(params.contains("\"empty\":\"\""));
        }
    }

    #[test]
    fn test_parameter_with_special_chars() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/users/john@example.com".to_string())),
            ("pattern".to_string(), Value::StringVal("/users/:email".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));

        if let Value::StringVal(params) = &result[1].1 {
            assert!(params.contains("\"email\":\"john@example.com\""));
        }
    }

    #[test]
    fn test_missing_path_input() {
        let inputs = vec![
            ("pattern".to_string(), Value::StringVal("/api/users".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("path".to_string()));
    }

    #[test]
    fn test_missing_pattern_input() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("pattern".to_string()));
    }

    #[test]
    fn test_wildcard_with_no_remaining_path() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/static".to_string())),
            ("pattern".to_string(), Value::StringVal("/static/*".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));
        assert_eq!(result[2].1, Value::StringVal("".to_string())); // Empty wildcard
    }

    #[test]
    fn test_mixed_parameters_and_exact() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api/users/123/profile".to_string())),
            ("pattern".to_string(), Value::StringVal("/api/users/:id/profile".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::BoolVal(true));

        if let Value::StringVal(params) = &result[1].1 {
            assert!(params.contains("\"id\":\"123\""));
        }
    }
}
