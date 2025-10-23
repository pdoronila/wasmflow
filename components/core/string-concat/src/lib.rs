wit_bindgen::generate!({
    world: "component",
    path: "../wit",
});

use exports::metadata::Guest as MetadataGuest;
use exports::execution::Guest as ExecutionGuest;
use exports::{
    ComponentInfo, PortSpec, DataType, InputValue, OutputValue,
    ExecutionError, NodeValue,
};

export!(Component);

struct Component;

// ============================================================================
// Metadata Interface
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "String Concat".to_string(),
            description: "Joins multiple strings into a single string".to_string(),
            category: "Text".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input1".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "First string".to_string(),
            },
            PortSpec {
                name: "input2".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Second string".to_string(),
            },
            PortSpec {
                name: "input3".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Third string".to_string(),
            },
            PortSpec {
                name: "input4".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Fourth string".to_string(),
            },
            PortSpec {
                name: "input5".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Fifth string".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Concatenated string".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None  // Pure computation, no capabilities needed
    }
}

// ============================================================================
// Execution Interface
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
        // Extract all string inputs
        let mut strings = Vec::new();

        for input in &inputs {
            match &input.value {
                NodeValue::String(s) => strings.push(s.clone()),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string, got {:?}", input.value),
                        input_name: Some(input.name.clone()),
                        recovery_hint: Some("Provide string values".to_string()),
                    });
                }
            }
        }

        // Concatenate all strings
        let result = strings.join("");

        // Return output
        Ok(vec![OutputValue {
            name: "result".to_string(),
            value: NodeValue::String(result),
        }])
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_two_strings() {
        let inputs = vec![
            InputValue {
                name: "input1".to_string(),
                value: NodeValue::String("Hello".to_string()),
            },
            InputValue {
                name: "input2".to_string(),
                value: NodeValue::String(" World".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "Hello World"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_concat_multiple_strings() {
        let inputs = vec![
            InputValue {
                name: "input1".to_string(),
                value: NodeValue::String("a".to_string()),
            },
            InputValue {
                name: "input2".to_string(),
                value: NodeValue::String("b".to_string()),
            },
            InputValue {
                name: "input3".to_string(),
                value: NodeValue::String("c".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "abc"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_concat_with_empty_string() {
        let inputs = vec![
            InputValue {
                name: "input1".to_string(),
                value: NodeValue::String("".to_string()),
            },
            InputValue {
                name: "input2".to_string(),
                value: NodeValue::String("test".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string output"),
        }
    }
}
