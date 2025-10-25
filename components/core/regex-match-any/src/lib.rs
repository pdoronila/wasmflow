//! Regex Match Any Component - Test string against multiple regex patterns
//!
//! This component tests whether a text string matches at least one of multiple
//! regex patterns (OR logic), returning match status and details.

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
            name: "Regex Match Any".to_string(),
            version: "1.0.0".to_string(),
            description: "Test if text matches at least one of multiple regex patterns (OR logic)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Text".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "text".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Text to test against patterns".to_string(),
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
                name: "matches".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if text matches at least one pattern".to_string(),
            },
            PortSpec {
                name: "matched_pattern".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "First pattern that matched (empty if none)".to_string(),
            },
            PortSpec {
                name: "match_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "How many patterns matched".to_string(),
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
        // Extract text input
        let text = inputs
            .iter()
            .find(|(name, _)| name == "text")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Connect a String value to the 'text' port".to_string()),
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

        // Compile all regex patterns and test
        let mut matched_pattern = String::new();
        let mut match_count = 0u32;

        for pattern in &pattern_strings {
            let regex = regex::Regex::new(pattern).map_err(|e| ExecutionError {
                message: format!("Invalid regular expression pattern '{}': {}", pattern, e),
                input_name: Some("patterns".to_string()),
                recovery_hint: Some(
                    "Ensure all patterns are valid regular expressions".to_string()
                ),
            })?;

            if regex.is_match(&text) {
                match_count += 1;
                if matched_pattern.is_empty() {
                    matched_pattern = pattern.clone();
                }
            }
        }

        let matches = match_count > 0;

        Ok(vec![
            ("matches".to_string(), Value::BoolVal(matches)),
            ("matched_pattern".to_string(), Value::StringVal(matched_pattern)),
            ("match_count".to_string(), Value::U32Val(match_count)),
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
    fn test_matches_one_pattern() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("file.rs".to_string())),
            ("patterns".to_string(), Value::StringListVal(vec![
                r".*\.rs$".to_string(),
                r".*\.txt$".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected BoolVal output"),
        }

        match &result[1].1 {
            Value::StringVal(s) => assert_eq!(s, r".*\.rs$"),
            _ => panic!("Expected StringVal output"),
        }

        match &result[2].1 {
            Value::U32Val(count) => assert_eq!(*count, 1),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_matches_multiple_patterns() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("a.rs".to_string())),
            ("patterns".to_string(), Value::StringListVal(vec![
                r".*\.rs$".to_string(),
                r"^a.*".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected BoolVal output"),
        }

        match &result[2].1 {
            Value::U32Val(count) => assert_eq!(*count, 2),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_no_match() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("file.txt".to_string())),
            ("patterns".to_string(), Value::StringListVal(vec![
                r".*\.rs$".to_string(),
            ])),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected BoolVal output"),
        }

        match &result[1].1 {
            Value::StringVal(s) => assert_eq!(s, ""),
            _ => panic!("Expected StringVal output"),
        }

        match &result[2].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_empty_patterns_list() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("test".to_string())),
            ("patterns".to_string(), Value::StringListVal(vec![])),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("At least one pattern"));
    }

    #[test]
    fn test_one_invalid_pattern() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("test".to_string())),
            ("patterns".to_string(), Value::StringListVal(vec![
                r".*\.rs$".to_string(),
                "[invalid(".to_string(),
            ])),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid regular expression"));
    }
}
