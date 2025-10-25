//! List Filter Regex Component - Keep list items matching a regex pattern
//!
//! This component filters a list of strings, keeping only items that match
//! the provided regular expression pattern.

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
            name: "List Filter Regex".to_string(),
            version: "1.0.0".to_string(),
            description: "Keep only list items matching a regular expression pattern".to_string(),
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
                description: "Regular expression pattern to match".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "matched".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Items that matched the pattern".to_string(),
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
                "Provide a valid regular expression. Examples: '.*\\.rs$', '^[0-9]+$', 'test.*'".to_string()
            ),
        })?;

        let original_count = list_values.len();

        // Filter list - keep items that match
        let matched: Vec<String> = list_values
            .into_iter()
            .filter(|item| regex.is_match(item))
            .collect();

        let matched_count = matched.len();
        let removed_count = original_count - matched_count;

        Ok(vec![
            ("matched".to_string(), Value::StringListVal(matched)),
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
    fn test_some_matches() {
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
            Value::StringListVal(matched) => {
                assert_eq!(matched.len(), 2);
                assert_eq!(matched, &vec!["a.rs", "c.rs"]);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 1),
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
            Value::StringListVal(matched) => assert_eq!(matched.len(), 0),
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 2),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_all_match() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a.rs".to_string(),
                "b.rs".to_string(),
            ])),
            ("pattern".to_string(), Value::StringVal(r".*\.rs$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(matched) => {
                assert_eq!(matched.len(), 2);
                assert_eq!(matched, &vec!["a.rs", "b.rs"]);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_empty_list() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![])),
            ("pattern".to_string(), Value::StringVal(r".*\.rs$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(matched) => assert_eq!(matched.len(), 0),
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
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
