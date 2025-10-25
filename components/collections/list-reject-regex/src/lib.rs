//! List Reject Regex Component - Remove items matching a regex pattern (blocklist)
//!
//! This component filters a list of strings, removing items that match
//! the provided regular expression pattern (inverse of filter).

wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

// ============================================================================
// METADATA INTERFACE
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "List Reject Regex".to_string(),
            version: "1.0.0".to_string(),
            description: "Remove list items matching a regular expression pattern (blocklist)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Collections".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "list".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "List of strings to filter".to_string(),
            },
            PortSpec {
                name: "pattern".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Regular expression pattern to reject".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "kept".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Items that did NOT match the pattern".to_string(),
            },
            PortSpec {
                name: "removed_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of items removed".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

// ============================================================================
// EXECUTION INTERFACE
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract list input
        let list_values = inputs
            .iter()
            .find(|(name, _)| name == "list")
            .and_then(|(_, val)| match val {
                Value::StringListVal(items) => Some(items.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'list' input".to_string(),
                input_name: Some("list".to_string()),
                recovery_hint: Some("Connect a StringListVal to the 'list' port".to_string()),
            })?;

        // Extract pattern input
        let pattern = inputs
            .iter()
            .find(|(name, _)| name == "pattern")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'pattern' input".to_string(),
                input_name: Some("pattern".to_string()),
                recovery_hint: Some("Connect a String value to the 'pattern' port".to_string()),
            })?;

        // Compile regex pattern
        let regex = regex::Regex::new(&pattern).map_err(|e| ExecutionError {
            message: format!("Invalid regular expression pattern: {}", e),
            input_name: Some("pattern".to_string()),
            recovery_hint: Some(
                "Provide a valid regular expression. Examples: '(node_modules|\\.git|target)/', '.*\\.tmp$'".to_string()
            ),
        })?;

        let original_count = list_values.len();

        // Filter list - keep items that DON'T match (inverse of filter)
        let kept: Vec<String> = list_values
            .into_iter()
            .filter(|item| !regex.is_match(item))  // Note the negation!
            .collect();

        let kept_count = kept.len();
        let removed_count = original_count - kept_count;

        Ok(vec![
            ("kept".to_string(), Value::StringListVal(kept)),
            ("removed_count".to_string(), Value::U32Val(removed_count as u32)),
        ])
    }
}

export!(Component);

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_matches_removed() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a.rs".to_string(),
                "b.txt".to_string(),
                "c.rs".to_string(),
            ])),
            ("pattern".to_string(), Value::StringVal(r".*\.rs$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(kept) => {
                assert_eq!(kept.len(), 1);
                assert_eq!(kept[0], "b.txt");
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 2),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_no_matches() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a.txt".to_string(),
                "b.md".to_string(),
            ])),
            ("pattern".to_string(), Value::StringVal(r".*\.rs$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(kept) => {
                assert_eq!(kept.len(), 2);
                assert_eq!(kept, &vec!["a.txt", "b.md"]);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_all_match_all_removed() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a.rs".to_string(),
                "b.rs".to_string(),
            ])),
            ("pattern".to_string(), Value::StringVal(r".*\.rs$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(kept) => {
                assert_eq!(kept.len(), 0);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 2),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_common_blocklist_pattern() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "src/main.rs".to_string(),
                ".git/config".to_string(),
                "target/debug/app".to_string(),
                "README.md".to_string(),
            ])),
            ("pattern".to_string(), Value::StringVal(r"(\.git|target)/".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(kept) => {
                assert_eq!(kept.len(), 2);
                assert!(kept.contains(&"src/main.rs".to_string()));
                assert!(kept.contains(&"README.md".to_string()));
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 2),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_invalid_pattern() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec!["test".to_string()])),
            ("pattern".to_string(), Value::StringVal("[invalid(".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid regular expression"));
    }
}
