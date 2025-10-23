wit_bindgen::generate!({
    world: "component",
    path: "../wit",
});

use exports::metadata::Guest as MetadataGuest;
use exports::execution::Guest as ExecutionGuest;
use exports::{ComponentInfo, PortSpec, DataType, InputValue, OutputValue, ExecutionError, NodeValue};

export!(Component);

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "String Contains".to_string(),
            description: "Checks if a string contains a substring".to_string(),
            category: "Text".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "text".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "String to search in".to_string(),
            },
            PortSpec {
                name: "substring".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Substring to search for".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "True if substring found".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
        let text = inputs.iter()
            .find(|i| i.name == "text")
            .and_then(|i| match &i.value {
                NodeValue::String(s) => Some(s),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide a string value".to_string()),
            })?;

        let substring = inputs.iter()
            .find(|i| i.name == "substring")
            .and_then(|i| match &i.value {
                NodeValue::String(s) => Some(s),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'substring' input".to_string(),
                input_name: Some("substring".to_string()),
                recovery_hint: Some("Provide a substring to search for".to_string()),
            })?;

        Ok(vec![OutputValue {
            name: "result".to_string(),
            value: NodeValue::Bool(text.contains(substring.as_str())),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substring_found() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("Hello World".to_string()),
            },
            InputValue {
                name: "substring".to_string(),
                value: NodeValue::String("World".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::Bool(b) => assert!(*b),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_case_sensitive() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("Hello World".to_string()),
            },
            InputValue {
                name: "substring".to_string(),
                value: NodeValue::String("world".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::Bool(b) => assert!(!*b), // Case-sensitive: should be false
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_empty_substring() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("Hello World".to_string()),
            },
            InputValue {
                name: "substring".to_string(),
                value: NodeValue::String("".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::Bool(b) => assert!(*b), // Empty substring always matches
            _ => panic!("Expected bool output"),
        }
    }
}
