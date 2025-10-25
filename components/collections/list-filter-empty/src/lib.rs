//! List Filter Empty Component - Remove empty/whitespace strings from a list
//!
//! This component filters out empty strings and whitespace-only strings from a
//! StringListVal, returning only non-empty items.

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
            name: "List Filter Empty".to_string(),
            version: "1.0.0".to_string(),
            description: "Remove empty strings and whitespace-only strings from a list".to_string(),
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
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "filtered".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "List with empty/whitespace items removed".to_string(),
            },
            PortSpec {
                name: "removed_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of items that were removed".to_string(),
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

        let original_count = list_values.len();

        // Filter out empty and whitespace-only strings
        let filtered: Vec<String> = list_values
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect();

        let filtered_count = filtered.len();
        let removed_count = original_count - filtered_count;

        Ok(vec![
            ("filtered".to_string(), Value::StringListVal(filtered)),
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
    fn test_all_valid_items() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        assert_eq!(result.len(), 2);

        // Check filtered list
        match &result[0].1 {
            Value::StringListVal(filtered) => {
                assert_eq!(filtered.len(), 3);
                assert_eq!(filtered, &vec!["a", "b", "c"]);
            }
            _ => panic!("Expected StringListVal output"),
        }

        // Check removed count
        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_some_empty_items() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a".to_string(),
                "".to_string(),
                "b".to_string(),
                " ".to_string(),
                "c".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(filtered) => {
                assert_eq!(filtered.len(), 3);
                assert_eq!(filtered, &vec!["a", "b", "c"]);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 2),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_all_empty_items() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "".to_string(),
                " ".to_string(),
                "\t".to_string(),
                "  \n  ".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(filtered) => {
                assert_eq!(filtered.len(), 0);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 4),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_empty_input_list() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(filtered) => {
                assert_eq!(filtered.len(), 0);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_whitespace_variations() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a".to_string(),
                "  ".to_string(),
                "\n\t".to_string(),
                "b".to_string(),
                "\r\n".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(filtered) => {
                assert_eq!(filtered.len(), 2);
                assert_eq!(filtered, &vec!["a", "b"]);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 3),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_strings_with_leading_trailing_whitespace() {
        // Strings with actual content but also whitespace should be kept
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                " a ".to_string(),  // has content, should be kept
                "  ".to_string(),   // only whitespace, should be removed
                " b".to_string(),   // has content, should be kept
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(filtered) => {
                assert_eq!(filtered.len(), 2);
                // Note: items are kept as-is, including their whitespace
                assert_eq!(filtered, &vec![" a ", " b"]);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 1),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_missing_list_input() {
        let inputs = vec![];
        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("list"));
    }

    #[test]
    fn test_wrong_input_type() {
        let inputs = vec![
            ("list".to_string(), Value::StringVal("not a list".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
