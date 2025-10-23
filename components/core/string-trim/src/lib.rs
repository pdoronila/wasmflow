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
            name: "String Trim".to_string(),
            description: "Removes leading and trailing whitespace from a string".to_string(),
            category: "Text".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "text".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Input string".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Trimmed string".to_string(),
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

        Ok(vec![OutputValue {
            name: "result".to_string(),
            value: NodeValue::String(text.trim().to_string()),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leading_trailing_spaces() {
        let inputs = vec![InputValue {
            name: "text".to_string(),
            value: NodeValue::String("  hello  ".to_string()),
        }];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_tab_newline() {
        let inputs = vec![InputValue {
            name: "text".to_string(),
            value: NodeValue::String("\thello\n".to_string()),
        }];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_middle_whitespace_preserved() {
        let inputs = vec![InputValue {
            name: "text".to_string(),
            value: NodeValue::String("  hello world  ".to_string()),
        }];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string output"),
        }
    }
}
