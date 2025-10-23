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
            name: "String Length".to_string(),
            description: "Returns the number of characters in a string".to_string(),
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
            name: "length".to_string(),
            data_type: DataType::U32Type,
            optional: false,
            description: "Character count (Unicode-aware)".to_string(),
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

        // Use chars().count() for Unicode-correct length
        let length = text.chars().count() as u32;

        Ok(vec![OutputValue {
            name: "length".to_string(),
            value: NodeValue::U32(length),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_string() {
        let inputs = vec![InputValue {
            name: "text".to_string(),
            value: NodeValue::String("Hello".to_string()),
        }];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::U32(len) => assert_eq!(*len, 5),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_unicode_emojis() {
        let inputs = vec![InputValue {
            name: "text".to_string(),
            value: NodeValue::String("🚀🌟".to_string()),
        }];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::U32(len) => assert_eq!(*len, 2), // 2 characters, not bytes
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_empty_string() {
        let inputs = vec![InputValue {
            name: "text".to_string(),
            value: NodeValue::String("".to_string()),
        }];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::U32(len) => assert_eq!(*len, 0),
            _ => panic!("Expected u32 output"),
        }
    }
}
