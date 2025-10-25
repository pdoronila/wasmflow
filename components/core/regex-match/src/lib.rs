//! Regex Match Component - Test a string against a regular expression pattern
//!
//! This component tests whether a text string matches a regex pattern,
//! returning a boolean result.

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
            name: "Regex Match".to_string(),
            version: "1.0.0".to_string(),
            description: "Test if text matches a regular expression pattern".to_string(),
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
                description: "Text to test against pattern".to_string(),
            },
            PortSpec {
                name: "pattern".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Regular expression pattern".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "matches".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if text matches pattern".to_string(),
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

        // Test match
        let matches = regex.is_match(&text);

        Ok(vec![
            ("matches".to_string(), Value::BoolVal(matches)),
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
    fn test_valid_pattern_match() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("hello.rs".to_string())),
            ("pattern".to_string(), Value::StringVal(r".*\.rs$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "matches");
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_valid_pattern_no_match() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("hello.txt".to_string())),
            ("pattern".to_string(), Value::StringVal(r".*\.rs$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("test".to_string())),
            ("pattern".to_string(), Value::StringVal("[invalid(".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid regular expression"));
    }

    #[test]
    fn test_empty_string_match() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("".to_string())),
            ("pattern".to_string(), Value::StringVal("^$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_empty_string_no_match() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("".to_string())),
            ("pattern".to_string(), Value::StringVal(".+".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_unicode_text() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("test_файл.rs".to_string())),
            ("pattern".to_string(), Value::StringVal(r".*\.rs$".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_complex_pattern() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("user@example.com".to_string())),
            ("pattern".to_string(), Value::StringVal(
                r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string()
            )),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_missing_text_input() {
        let inputs = vec![
            ("pattern".to_string(), Value::StringVal("test".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("text"));
    }

    #[test]
    fn test_missing_pattern_input() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("test".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("pattern"));
    }
}
