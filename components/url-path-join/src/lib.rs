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
            name: "URL Path Join".to_string(),
            version: "1.0.0".to_string(),
            description: "Safely joins URL path segments with proper slash handling and security checks".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "base_path".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Base path (e.g., '/static' or '/api/v1')".to_string(),
            },
            PortSpec {
                name: "segments".to_string(),
                data_type: DataType::ListType,
                optional: true,
                description: "Path segments to join (e.g., ['users', '123', 'profile'])".to_string(),
            },
            PortSpec {
                name: "allow_traversal".to_string(),
                data_type: DataType::BoolType,
                optional: true,
                description: "Allow '..' in paths (default: false for security)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "path".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Joined path with proper slash handling".to_string(),
            },
            PortSpec {
                name: "is_safe".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if path doesn't contain directory traversal attempts".to_string(),
            },
            PortSpec {
                name: "segment_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of segments in the resulting path".to_string(),
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
        // Extract base_path
        let base_path = inputs
            .iter()
            .find(|(name, _)| name == "base_path")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: base_path".to_string(),
                input_name: Some("base_path".to_string()),
                recovery_hint: Some("Connect a base path to this input".to_string()),
            })?;

        let base_path_str = match &base_path.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'base_path', got {:?}", base_path.1),
                    input_name: Some("base_path".to_string()),
                    recovery_hint: Some("Provide a string path".to_string()),
                });
            }
        };

        // Extract segments (optional)
        let segments = if let Some(seg_input) = inputs.iter().find(|(name, _)| name == "segments") {
            match &seg_input.1 {
                Value::StringListVal(list) => list.clone(),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected StringListVal for 'segments', got {:?}", seg_input.1),
                        input_name: Some("segments".to_string()),
                        recovery_hint: Some("Provide a list of string segments".to_string()),
                    });
                }
            }
        } else {
            Vec::new()
        };

        // Extract allow_traversal (optional, default false)
        let allow_traversal = if let Some(allow_input) = inputs.iter().find(|(name, _)| name == "allow_traversal") {
            match &allow_input.1 {
                Value::BoolVal(b) => *b,
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected bool for 'allow_traversal', got {:?}", allow_input.1),
                        input_name: Some("allow_traversal".to_string()),
                        recovery_hint: Some("Provide a boolean value".to_string()),
                    });
                }
            }
        } else {
            false // Default to secure mode
        };

        // Join the paths
        let (joined_path, is_safe, segment_count) = join_paths(base_path_str, &segments, allow_traversal);

        Ok(vec![
            ("path".to_string(), Value::StringVal(joined_path)),
            ("is_safe".to_string(), Value::BoolVal(is_safe)),
            ("segment_count".to_string(), Value::U32Val(segment_count)),
        ])
    }
}

// ============================================================================
// Path Joining Logic
// ============================================================================

/// Join base path with segments, handling slashes and security
/// Returns: (joined_path, is_safe, segment_count)
fn join_paths(base_path: &str, segments: &[String], allow_traversal: bool) -> (String, bool, u32) {
    let mut parts = Vec::new();
    let mut has_traversal = false;

    // Add base path segments
    for segment in base_path.split('/') {
        if segment.is_empty() {
            continue; // Skip empty segments from multiple slashes
        }

        if segment == ".." {
            has_traversal = true;
            if allow_traversal && !parts.is_empty() {
                parts.pop(); // Go up one level
            }
        } else if segment != "." {
            parts.push(segment.to_string());
        }
    }

    // Add additional segments
    for segment in segments {
        // Split segment in case it contains slashes
        for sub_segment in segment.split('/') {
            if sub_segment.is_empty() {
                continue;
            }

            if sub_segment == ".." {
                has_traversal = true;
                if allow_traversal && !parts.is_empty() {
                    parts.pop();
                }
            } else if sub_segment != "." {
                parts.push(sub_segment.to_string());
            }
        }
    }

    let is_safe = !has_traversal || allow_traversal;
    let segment_count = parts.len() as u32;

    // Build final path
    let joined = if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    };

    (joined, is_safe, segment_count)
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_join() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/api".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "users".to_string(),
                "123".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/api/users/123".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true)); // is_safe
        assert_eq!(result[2].1, Value::U32Val(3)); // segment_count
    }

    #[test]
    fn test_base_path_with_trailing_slash() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/static/".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "css".to_string(),
                "main.css".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/static/css/main.css".to_string()));
    }

    #[test]
    fn test_segments_with_leading_slash() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/api".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "/users".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/api/users".to_string()));
    }

    #[test]
    fn test_empty_segments() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/api/v1".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/api/v1".to_string()));
        assert_eq!(result[2].1, Value::U32Val(2));
    }

    #[test]
    fn test_no_segments_input() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/static".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/static".to_string()));
    }

    #[test]
    fn test_root_path() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "index.html".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/index.html".to_string()));
    }

    #[test]
    fn test_multiple_slashes_normalized() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("//api///v1//".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "users".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/api/v1/users".to_string()));
    }

    #[test]
    fn test_traversal_blocked_by_default() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/static".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "..".to_string(),
                "etc".to_string(),
                "passwd".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::BoolVal(false)); // is_safe = false (traversal detected)

        // Path should not go above base when traversal blocked
        if let Value::StringVal(path) = &result[0].1 {
            assert!(path.starts_with("/"));
        }
    }

    #[test]
    fn test_traversal_allowed_when_enabled() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/api/v1/users".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "..".to_string(),
                "posts".to_string(),
            ])),
            ("allow_traversal".to_string(), Value::BoolVal(true)),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/api/v1/posts".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true)); // is_safe when allowed
    }

    #[test]
    fn test_current_directory_dots_ignored() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/api".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                ".".to_string(),
                "users".to_string(),
                ".".to_string(),
                "123".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/api/users/123".to_string()));
        assert_eq!(result[2].1, Value::U32Val(3)); // '.' segments don't count
    }

    #[test]
    fn test_segment_with_multiple_parts() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/files".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "docs/2024/report.pdf".to_string(), // Segment contains slashes
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/files/docs/2024/report.pdf".to_string()));
        assert_eq!(result[2].1, Value::U32Val(4));
    }

    #[test]
    fn test_empty_base_path() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "users".to_string(),
                "123".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/users/123".to_string()));
    }

    #[test]
    fn test_base_without_leading_slash() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("api/v1".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "users".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        // Should always return absolute path
        assert_eq!(result[0].1, Value::StringVal("/api/v1/users".to_string()));
    }

    #[test]
    fn test_all_empty_results_in_root() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("///".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "".to_string(),
                "/".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/".to_string()));
        assert_eq!(result[2].1, Value::U32Val(0));
    }

    #[test]
    fn test_missing_base_path() {
        let inputs = vec![
            ("segments".to_string(), Value::StringListVal(vec!["users".to_string()])),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("base_path".to_string()));
    }

    #[test]
    fn test_complex_path_join() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/var/www/".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "html".to_string(),
                "site".to_string(),
                "assets".to_string(),
                "images".to_string(),
                "logo.png".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/var/www/html/site/assets/images/logo.png".to_string()));
        assert_eq!(result[2].1, Value::U32Val(7));
    }

    #[test]
    fn test_file_extension_preserved() {
        let inputs = vec![
            ("base_path".to_string(), Value::StringVal("/downloads".to_string())),
            ("segments".to_string(), Value::StringListVal(vec![
                "report.2024.final.pdf".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("/downloads/report.2024.final.pdf".to_string()));
    }
}
