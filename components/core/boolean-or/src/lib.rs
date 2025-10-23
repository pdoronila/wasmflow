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
            name: "Boolean OR".to_string(),
            version: "1.0.0".to_string(),
            description: "Returns true if any input is true (logical OR operation)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input1".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "First boolean value".to_string(),
            },
            PortSpec {
                name: "input2".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "Second boolean value".to_string(),
            },
            PortSpec {
                name: "input3".to_string(),
                data_type: DataType::BoolType,
                optional: true,
                description: "Optional third boolean value".to_string(),
            },
            PortSpec {
                name: "input4".to_string(),
                data_type: DataType::BoolType,
                optional: true,
                description: "Optional fourth boolean value".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "True if any input is true, false otherwise".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Boolean OR component executing");

        let mut result = false;

        for (name, value) in &inputs {
            match value {
                Value::BoolVal(b) => {
                    result = result || *b;
                }
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected boolean for input '{}', got {:?}", name, value),
                        input_name: Some(name.clone()),
                        recovery_hint: Some("Provide a boolean value (true/false)".to_string()),
                    });
                }
            }
        }

        Ok(vec![("result".to_string(), Value::BoolVal(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_true_others_false() {
        let inputs = vec![
            ("input1".to_string(), Value::BoolVal(true)),
            ("input2".to_string(), Value::BoolVal(false)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_all_false() {
        let inputs = vec![
            ("input1".to_string(), Value::BoolVal(false)),
            ("input2".to_string(), Value::BoolVal(false)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_all_true() {
        let inputs = vec![
            ("input1".to_string(), Value::BoolVal(true)),
            ("input2".to_string(), Value::BoolVal(true)),
            ("input3".to_string(), Value::BoolVal(true)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }
}
