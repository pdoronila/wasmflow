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
            name: "Minimum".to_string(),
            version: "1.0.0".to_string(),
            description: "Returns the smallest of all input values".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input1".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "First number".to_string(),
            },
            PortSpec {
                name: "input2".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Second number".to_string(),
            },
            PortSpec {
                name: "input3".to_string(),
                data_type: DataType::F32Type,
                optional: true,
                description: "Optional third number".to_string(),
            },
            PortSpec {
                name: "input4".to_string(),
                data_type: DataType::F32Type,
                optional: true,
                description: "Optional fourth number".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Minimum of all input values".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Minimum component executing");

        let mut values = Vec::new();

        for (name, value) in &inputs {
            match value {
                Value::F32Val(f) => values.push(*f),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected number for input '{}', got {:?}", name, value),
                        input_name: Some(name.clone()),
                        recovery_hint: Some("Provide numeric values".to_string()),
                    });
                }
            }
        }

        if values.is_empty() {
            return Err(ExecutionError {
                message: "At least one input value is required".to_string(),
                input_name: None,
                recovery_hint: Some("Connect at least one numeric value".to_string()),
            });
        }

        let result = values
            .iter()
            .fold(f32::INFINITY, |min, &val| if val < min { val } else { min });

        Ok(vec![("result".to_string(), Value::F32Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_multiple_values() {
        let inputs = vec![
            ("input1".to_string(), Value::F32Val(5.0)),
            ("input2".to_string(), Value::F32Val(2.0)),
            ("input3".to_string(), Value::F32Val(8.0)),
            ("input4".to_string(), Value::F32Val(1.0)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 1.0).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_min_negative_values() {
        let inputs = vec![
            ("input1".to_string(), Value::F32Val(-3.0)),
            ("input2".to_string(), Value::F32Val(0.0)),
            ("input3".to_string(), Value::F32Val(3.0)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - (-3.0)).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }
}
