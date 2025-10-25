//! List Count Regex Any Component - Count items matching ANY of multiple regex patterns
//!
//! This component counts how many items in a list match at least one of the
//! provided regular expression patterns (OR logic). Critical for Kernel message engine!

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
            name: "List Count Regex Any".to_string(),
            version: "1.0.0".to_string(),
            description: "Count items matching ANY of multiple regex patterns (OR logic)".to_string(),
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
                description: "List of strings to analyze".to_string(),
            },
            PortSpec {
                name: "patterns".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Regular expression patterns (OR logic)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of items matching at least one pattern".to_string(),
            },
            PortSpec {
                name: "percentage".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Percentage of items matching".to_string(),
            },
            PortSpec {
                name: "total".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Total items in input list".to_string(),
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

        // Extract patterns input
        let pattern_strings = inputs
            .iter()
            .find(|(name, _)| name == "patterns")
            .and_then(|(_, val)| match val {
                Value::StringListVal(items) => Some(items.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'patterns' input".to_string(),
                input_name: Some("patterns".to_string()),
                recovery_hint: Some("Connect a StringListVal to the 'patterns' port".to_string()),
            })?;

        // Validate at least one pattern
        if pattern_strings.is_empty() {
            return Err(ExecutionError {
                message: "At least one pattern is required".to_string(),
                input_name: Some("patterns".to_string()),
                recovery_hint: Some("Provide at least one regex pattern".to_string()),
            });
        }

        // Compile all regex patterns
        let mut regexes = Vec::new();
        for pattern in &pattern_strings {
            let regex = regex::Regex::new(pattern).map_err(|e| ExecutionError {
                message: format!("Invalid regular expression pattern '{}': {}", pattern, e),
                input_name: Some("patterns".to_string()),
                recovery_hint: Some(
                    "Ensure all patterns are valid regular expressions".to_string()
                ),
            })?;
            regexes.push(regex);
        }

        let total = list_values.len() as u32;

        // Count items that match ANY pattern
        let count = list_values
            .iter()
            .filter(|item| {
                // Check if item matches at least one pattern
                regexes.iter().any(|regex| regex.is_match(item))
            })
            .count() as u32;

        // Calculate percentage
        let percentage = if total > 0 {
            (count as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Ok(vec![
            ("count".to_string(), Value::U32Val(count)),
            ("percentage".to_string(), Value::F32Val(percentage)),
            ("total".to_string(), Value::U32Val(total)),
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
    fn test_multiple_patterns() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a.rs".to_string(),
                "b.toml".to_string(),
                "c.txt".to_string(),
            ])),
            ("patterns".to_string(), Value::StringListVal(vec![
                r".*\.rs$".to_string(),
                r".*\.toml$".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::U32Val(count) => assert_eq!(*count, 2),
            _ => panic!("Expected U32Val output"),
        }

        match &result[1].1 {
            Value::F32Val(percentage) => {
                assert!((percentage - 66.66666).abs() < 0.01);
            }
            _ => panic!("Expected F32Val output"),
        }

        match &result[2].1 {
            Value::U32Val(total) => assert_eq!(*total, 3),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_overlapping_patterns() {
        // Item matches multiple patterns but should only be counted once
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a.rs".to_string(),
            ])),
            ("patterns".to_string(), Value::StringListVal(vec![
                r".*\.rs$".to_string(),
                r"^a.*".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::U32Val(count) => assert_eq!(*count, 1),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_no_patterns() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec!["test".to_string()])),
            ("patterns".to_string(), Value::StringListVal(vec![])),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("At least one pattern"));
    }

    #[test]
    fn test_no_matches() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![
                "a.txt".to_string(),
                "b.md".to_string(),
            ])),
            ("patterns".to_string(), Value::StringListVal(vec![
                r".*\.rs$".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }

        match &result[1].1 {
            Value::F32Val(percentage) => assert_eq!(*percentage, 0.0),
            _ => panic!("Expected F32Val output"),
        }
    }

    #[test]
    fn test_empty_list() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![])),
            ("patterns".to_string(), Value::StringListVal(vec![r".*\.rs$".to_string()])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }

        match &result[1].1 {
            Value::F32Val(percentage) => assert_eq!(*percentage, 0.0),
            _ => panic!("Expected F32Val output"),
        }

        match &result[2].1 {
            Value::U32Val(total) => assert_eq!(*total, 0),
            _ => panic!("Expected U32Val output"),
        }
    }
}
