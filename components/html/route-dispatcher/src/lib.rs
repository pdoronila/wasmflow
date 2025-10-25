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
            name: "Route Dispatcher".to_string(),
            version: "1.0.0".to_string(),
            description: "Matches HTTP method and path against a list of routes and returns which route matched".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
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
                description: "URL path to match (e.g., '/api/users/123')".to_string(),
            },
            PortSpec {
                name: "routes".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "List of routes in 'METHOD /path/pattern' format (e.g., 'GET /api/users/:id')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "route_index".to_string(),
                data_type: DataType::I32Type,
                optional: false,
                description: "Index of the matched route (0-based), or -1 if no match".to_string(),
            },
            PortSpec {
                name: "matched_route".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "The matched route string (empty if no match)".to_string(),
            },
            PortSpec {
                name: "pattern".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "The path pattern from the matched route (empty if no match)".to_string(),
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
        // Extract method
        let method = inputs
            .iter()
            .find(|(name, _)| name == "method")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: method".to_string(),
                input_name: Some("method".to_string()),
                recovery_hint: Some("Connect an HTTP method (GET, POST, etc.) to this input".to_string()),
            })?;

        let method_str = match &method.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'method', got {:?}", method.1),
                    input_name: Some("method".to_string()),
                    recovery_hint: Some("Provide a string method (GET, POST, etc.)".to_string()),
                });
            }
        };

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

        // Extract routes list
        let routes = inputs
            .iter()
            .find(|(name, _)| name == "routes")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: routes".to_string(),
                input_name: Some("routes".to_string()),
                recovery_hint: Some("Connect a list of routes to this input".to_string()),
            })?;

        let routes_list = match &routes.1 {
            Value::StringListVal(list) => list,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected StringListVal for 'routes', got {:?}", routes.1),
                    input_name: Some("routes".to_string()),
                    recovery_hint: Some("Provide a list of route strings".to_string()),
                });
            }
        };

        // Find matching route
        let (route_index, matched_route, pattern) = find_matching_route(method_str, path_str, routes_list)
            .map_err(|e| ExecutionError {
                message: format!("Error matching routes: {}", e),
                input_name: Some("routes".to_string()),
                recovery_hint: Some("Ensure routes are in 'METHOD /path/pattern' format".to_string()),
            })?;

        Ok(vec![
            ("route_index".to_string(), Value::I32Val(route_index)),
            ("matched_route".to_string(), Value::StringVal(matched_route)),
            ("pattern".to_string(), Value::StringVal(pattern)),
        ])
    }
}

// ============================================================================
// Route Matching Logic
// ============================================================================

/// Find the first route that matches the method and path
/// Returns: (index, matched_route, pattern) or (-1, "", "") if no match
fn find_matching_route(
    method: &str,
    path: &str,
    routes: &[String],
) -> Result<(i32, String, String), String> {
    let method_upper = method.to_uppercase();

    for (index, route) in routes.iter().enumerate() {
        // Parse route string: "METHOD /path/pattern"
        let parts: Vec<&str> = route.splitn(2, ' ').collect();

        if parts.len() != 2 {
            return Err(format!(
                "Invalid route format: '{}'. Expected 'METHOD /path/pattern'",
                route
            ));
        }

        let route_method = parts[0].to_uppercase();
        let route_pattern = parts[1];

        // Check if method matches (or route method is "*" for any method)
        if route_method != method_upper && route_method != "*" {
            continue;
        }

        // Check if path matches pattern
        if match_path(path, route_pattern) {
            return Ok((index as i32, route.clone(), route_pattern.to_string()));
        }
    }

    // No match found
    Ok((-1, String::new(), String::new()))
}

/// Match a path against a pattern (similar logic to path-matcher component)
fn match_path(path: &str, pattern: &str) -> bool {
    let path_segments: Vec<&str> = path.split('/').collect();
    let pattern_segments: Vec<&str> = pattern.split('/').collect();

    for (i, pattern_seg) in pattern_segments.iter().enumerate() {
        if i >= path_segments.len() {
            // Pattern has more segments than path
            if *pattern_seg == "*" {
                // Wildcard at end with no more path - still matches
                return true;
            }
            return false;
        }

        if pattern_seg.starts_with(':') {
            // Named parameter - always matches this segment
            continue;
        } else if *pattern_seg == "*" {
            // Wildcard - matches everything remaining
            return true;
        } else if *pattern_seg != path_segments[i] {
            // Exact match required but segments don't match
            return false;
        }
    }

    // Check if path has more segments than pattern
    if path_segments.len() > pattern_segments.len() {
        return false;
    }

    true
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_route_match() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("GET".to_string())),
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /api/users".to_string(),
                "POST /api/users".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(0)); // route_index
        assert_eq!(result[1].1, Value::StringVal("GET /api/users".to_string())); // matched_route
        assert_eq!(result[2].1, Value::StringVal("/api/users".to_string())); // pattern
    }

    #[test]
    fn test_parameterized_route_match() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("GET".to_string())),
            ("path".to_string(), Value::StringVal("/api/users/123".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /api/users".to_string(),
                "GET /api/users/:id".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(1)); // Second route matches
        assert_eq!(result[2].1, Value::StringVal("/api/users/:id".to_string()));
    }

    #[test]
    fn test_wildcard_route_match() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("GET".to_string())),
            ("path".to_string(), Value::StringVal("/static/css/main.css".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /api/*".to_string(),
                "GET /static/*".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(1));
        assert_eq!(result[2].1, Value::StringVal("/static/*".to_string()));
    }

    #[test]
    fn test_method_mismatch() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("POST".to_string())),
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /api/users".to_string(),
                "DELETE /api/users".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(-1)); // No match
        assert_eq!(result[1].1, Value::StringVal("".to_string()));
        assert_eq!(result[2].1, Value::StringVal("".to_string()));
    }

    #[test]
    fn test_any_method_wildcard() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("PATCH".to_string())),
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /api/users".to_string(),
                "* /api/users".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(1)); // Second route matches with *
    }

    #[test]
    fn test_first_match_wins() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("GET".to_string())),
            ("path".to_string(), Value::StringVal("/api/users/123".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /api/*".to_string(),
                "GET /api/users/:id".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(0)); // First match wins
    }

    #[test]
    fn test_case_insensitive_method() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("get".to_string())),
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /api/users".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(0)); // Should match (case insensitive)
    }

    #[test]
    fn test_multiple_parameters() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("GET".to_string())),
            ("path".to_string(), Value::StringVal("/api/users/123/posts/456".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /api/users/:userId/posts/:postId".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(0));
    }

    #[test]
    fn test_root_path() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("GET".to_string())),
            ("path".to_string(), Value::StringVal("/".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /".to_string(),
                "GET /api".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(0));
    }

    #[test]
    fn test_empty_routes_list() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("GET".to_string())),
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(-1)); // No routes, no match
    }

    #[test]
    fn test_invalid_route_format() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("GET".to_string())),
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "InvalidRouteNoSpace".to_string(),
            ])),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid route format"));
    }

    #[test]
    fn test_missing_method_input() {
        let inputs = vec![
            ("path".to_string(), Value::StringVal("/api/users".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![])),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("method".to_string()));
    }

    #[test]
    fn test_http_methods() {
        let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"];

        for method in methods {
            let inputs = vec![
                ("method".to_string(), Value::StringVal(method.to_string())),
                ("path".to_string(), Value::StringVal("/api".to_string())),
                ("routes".to_string(), Value::StringListVal(vec![
                    format!("{} /api", method),
                ])),
            ];

            let result = Component::execute(inputs).unwrap();
            assert_eq!(result[0].1, Value::I32Val(0));
        }
    }

    #[test]
    fn test_complex_routing_table() {
        let inputs = vec![
            ("method".to_string(), Value::StringVal("POST".to_string())),
            ("path".to_string(), Value::StringVal("/api/users/123/posts".to_string())),
            ("routes".to_string(), Value::StringListVal(vec![
                "GET /".to_string(),
                "GET /api/users".to_string(),
                "GET /api/users/:id".to_string(),
                "POST /api/users/:id/posts".to_string(),
                "DELETE /api/users/:id".to_string(),
                "* /static/*".to_string(),
            ])),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::I32Val(3)); // Fourth route
        assert_eq!(result[2].1, Value::StringVal("/api/users/:id/posts".to_string()));
    }
}
