wit_bindgen::generate!({
    world: "component",
    path: "./wit",
});

use exports::metadata::Guest as MetadataGuest;
use exports::execution::Guest as ExecutionGuest;
use exports::{ComponentInfo, PortSpec, DataType, InputValue, OutputValue, ExecutionError, NodeValue};

export!(Component);

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "String Case".to_string(),
            description: "Converts string case (uppercase, lowercase, titlecase)".to_string(),
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
                name: "operation".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Operation: uppercase, lowercase, or titlecase".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Transformed string".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

fn titlecase(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c.is_whitespace() {
            result.push(c);
            capitalize_next = true;
        } else if capitalize_next {
            for upper in c.to_uppercase() {
                result.push(upper);
            }
            capitalize_next = false;
        } else {
            for lower in c.to_lowercase() {
                result.push(lower);
            }
        }
    }

    result
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

        let operation = inputs.iter()
            .find(|i| i.name == "operation")
            .and_then(|i| match &i.value {
                NodeValue::String(s) => Some(s),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'operation' input".to_string(),
                input_name: Some("operation".to_string()),
                recovery_hint: Some("Provide operation: uppercase, lowercase, or titlecase".to_string()),
            })?;

        let result = match operation.as_str() {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "titlecase" => titlecase(text),
            _ => {
                return Err(ExecutionError {
                    message: format!("Invalid operation '{}'. Must be 'uppercase', 'lowercase', or 'titlecase'", operation),
                    input_name: Some("operation".to_string()),
                    recovery_hint: Some("Use one of: uppercase, lowercase, titlecase".to_string()),
                });
            }
        };

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
    fn test_uppercase() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("hello world".to_string()),
            },
            InputValue {
                name: "operation".to_string(),
                value: NodeValue::String("uppercase".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "HELLO WORLD"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_lowercase() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("HELLO WORLD".to_string()),
            },
            InputValue {
                name: "operation".to_string(),
                value: NodeValue::String("lowercase".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_titlecase() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("hello world".to_string()),
            },
            InputValue {
                name: "operation".to_string(),
                value: NodeValue::String("titlecase".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "Hello World"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_invalid_operation() {
        let inputs = vec![
            InputValue {
                name: "text".to_string(),
                value: NodeValue::String("hello".to_string()),
            },
            InputValue {
                name: "operation".to_string(),
                value: NodeValue::String("invalid".to_string()),
            },
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
