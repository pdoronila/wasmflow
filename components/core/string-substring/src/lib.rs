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
            name: "String Substring".to_string(),
            description: "Extracts a portion of a string".to_string(),
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
                description: "Input string".to_string(),
            },
            PortSpec {
                name: "start".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Start index (0-based, character index)".to_string(),
            },
            PortSpec {
                name: "length".to_string(),
                data_type: DataType::U32Type,
                optional: true,
                description: "Number of characters (to end if omitted)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Extracted substring".to_string(),
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

        let start = inputs.iter()
            .find(|i| i.name == "start")
            .and_then(|i| match &i.value {
                NodeValue::U32(n) => Some(*n as usize),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'start' input".to_string(),
                input_name: Some("start".to_string()),
                recovery_hint: Some("Provide a start index (0-based)".to_string()),
            })?;

        let length = inputs.iter()
            .find(|i| i.name == "length")
            .and_then(|i| match &i.value {
                NodeValue::U32(n) => Some(*n as usize),
                _ => None,
            });

        // Character-based indexing (Unicode-aware)
        let chars: Vec<char> = text.chars().collect();

        if start >= chars.len() {
            // Start beyond end, return empty string
            return Ok(vec![OutputValue {
                name: "result".to_string(),
                value: NodeValue::String(String::new()),
            }]);
        }

        let end = if let Some(len) = length {
            (start + len).min(chars.len())
        } else {
            chars.len()
        };

        let result: String = chars[start..end].iter().collect();

        Ok(vec![OutputValue {
            name: "result".to_string(),
            value: NodeValue::String(result),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_with_length() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("Hello World".to_string()),
            },
            InputValue {
                name: "start".to_string(),
                value: NodeValue::U32(0),
            },
            InputValue {
                name: "length".to_string(),
                value: NodeValue::U32(5),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "Hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_extract_to_end() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("Hello World".to_string()),
            },
            InputValue {
                name: "start".to_string(),
                value: NodeValue::U32(6),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "World"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_start_beyond_end() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("Hello".to_string()),
            },
            InputValue {
                name: "start".to_string(),
                value: NodeValue::U32(10),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, ""),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_unicode_characters() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("🚀🌟✨".to_string()),
            },
            InputValue {
                name: "start".to_string(),
                value: NodeValue::U32(1),
            },
            InputValue {
                name: "length".to_string(),
                value: NodeValue::U32(2),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "🌟✨"),
            _ => panic!("Expected string output"),
        }
    }
}
