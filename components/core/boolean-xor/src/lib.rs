// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "./wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Boolean XOR".to_string(),
            version: "1.0.0".to_string(),
            description: "Returns true if exactly one input is true (logical XOR operation)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "left".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "First boolean value".to_string(),
            },
            PortSpec {
                name: "right".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "Second boolean value".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "True if exactly one input is true, false otherwise".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Boolean XOR component executing");

        let left = inputs
            .iter()
            .find(|(n, _)| n == "left")
            .and_then(|(_, v)| if let Value::BoolVal(b) = v { Some(*b) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'left' input".to_string(),
                input_name: Some("left".to_string()),
                recovery_hint: Some("Provide a boolean value (true/false)".to_string()),
            })?;

        let right = inputs
            .iter()
            .find(|(n, _)| n == "right")
            .and_then(|(_, v)| if let Value::BoolVal(b) = v { Some(*b) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'right' input".to_string(),
                input_name: Some("right".to_string()),
                recovery_hint: Some("Provide a boolean value (true/false)".to_string()),
            })?;

        let result = left ^ right;

        Ok(vec![("result".to_string(), Value::BoolVal(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_xor_false() {
        let inputs = vec![
            ("left".to_string(), Value::BoolVal(true)),
            ("right".to_string(), Value::BoolVal(false)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_true_xor_true() {
        let inputs = vec![
            ("left".to_string(), Value::BoolVal(true)),
            ("right".to_string(), Value::BoolVal(true)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_false_xor_false() {
        let inputs = vec![
            ("left".to_string(), Value::BoolVal(false)),
            ("right".to_string(), Value::BoolVal(false)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }
}
