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

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "String Split".to_string(),
            description: "Splits a string on delimiter into a list of substrings".to_string(),
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
                description: "Input string to split".to_string(),
            },
            PortSpec {
                name: "delimiter".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Delimiter to split on".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "parts".to_string(),
            data_type: DataType::ListType,
            optional: false,
            description: "List of substrings".to_string(),
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
                NodeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide a string value".to_string()),
            })?;

        let delimiter = inputs.iter()
            .find(|i| i.name == "delimiter")
            .and_then(|i| match &i.value {
                NodeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'delimiter' input".to_string(),
                input_name: Some("delimiter".to_string()),
                recovery_hint: Some("Provide a string delimiter".to_string()),
            })?;

        // Split string - handle empty delimiter (split into chars)
        let parts: Vec<NodeValue> = if delimiter.is_empty() {
            text.chars()
                .map(|c| NodeValue::String(c.to_string()))
                .collect()
        } else {
            text.split(&delimiter)
                .map(|s| NodeValue::String(s.to_string()))
                .collect()
        };

        Ok(vec![OutputValue {
            name: "parts".to_string(),
            value: NodeValue::List(parts),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_on_comma() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("a,b,c".to_string()),
            },
            InputValue {
                name: "delimiter".to_string(),
                value: NodeValue::String(",".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::List(parts) => {
                assert_eq!(parts.len(), 3);
                match &parts[0] {
                    NodeValue::String(s) => assert_eq!(s, "a"),
                    _ => panic!("Expected string"),
                }
            },
            _ => panic!("Expected list output"),
        }
    }

    #[test]
    fn test_split_on_empty_delimiter() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("hello".to_string()),
            },
            InputValue {
                name: "delimiter".to_string(),
                value: NodeValue::String("".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::List(parts) => {
                assert_eq!(parts.len(), 5); // h, e, l, l, o
            },
            _ => panic!("Expected list output"),
        }
    }

    #[test]
    fn test_split_consecutive_delimiters() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("a,,b".to_string()),
            },
            InputValue {
                name: "delimiter".to_string(),
                value: NodeValue::String(",".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::List(parts) => {
                assert_eq!(parts.len(), 3); // "a", "", "b"
            },
            _ => panic!("Expected list output"),
        }
    }
}
