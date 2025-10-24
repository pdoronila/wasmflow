// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "./wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Add".to_string(),
            version: "1.0.0".to_string(),
            description: "Adds two numbers and returns the sum".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "a".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "First number".to_string(),
            },
            PortSpec {
                name: "b".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Second number".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "sum".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Sum of inputs (a + b)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract input 'a'
        let a = inputs
            .iter()
            .find(|(name, _)| name == "a")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: a".to_string(),
                input_name: Some("a".to_string()),
                recovery_hint: Some("Connect a value to the 'a' input".to_string()),
            })?;

        // Extract input 'b'
        let b = inputs
            .iter()
            .find(|(name, _)| name == "b")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: b".to_string(),
                input_name: Some("b".to_string()),
                recovery_hint: Some("Connect a value to the 'b' input".to_string()),
            })?;

        // Perform addition based on type
        let result = match (&a.1, &b.1) {
            (Value::F32Val(a_val), Value::F32Val(b_val)) => Value::F32Val(a_val + b_val),
            (Value::I32Val(a_val), Value::I32Val(b_val)) => Value::I32Val(a_val + b_val),
            (Value::U32Val(a_val), Value::U32Val(b_val)) => Value::U32Val(a_val + b_val),
            _ => {
                return Err(ExecutionError {
                    message: format!(
                        "Type mismatch: inputs must be the same numeric type. Got {:?} and {:?}",
                        a.1, b.1
                    ),
                    input_name: None,
                    recovery_hint: Some(
                        "Ensure both inputs are the same type (f32, i32, or u32)".to_string(),
                    ),
                });
            }
        };

        Ok(vec![("sum".to_string(), result)])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_f32() {
        let inputs = vec![
            ("a".to_string(), Value::F32Val(5.0)),
            ("b".to_string(), Value::F32Val(3.0)),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].0, "sum");
        match result[0].1 {
            Value::F32Val(v) => assert!((v - 8.0).abs() < 0.001),
            _ => panic!("Expected F32Val"),
        }
    }

    #[test]
    fn test_add_i32() {
        let inputs = vec![
            ("a".to_string(), Value::I32Val(10)),
            ("b".to_string(), Value::I32Val(-3)),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].0, "sum");
        match result[0].1 {
            Value::I32Val(v) => assert_eq!(v, 7),
            _ => panic!("Expected I32Val"),
        }
    }

    #[test]
    fn test_add_u32() {
        let inputs = vec![
            ("a".to_string(), Value::U32Val(100)),
            ("b".to_string(), Value::U32Val(50)),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].0, "sum");
        match result[0].1 {
            Value::U32Val(v) => assert_eq!(v, 150),
            _ => panic!("Expected U32Val"),
        }
    }

    #[test]
    fn test_add_type_mismatch() {
        let inputs = vec![
            ("a".to_string(), Value::F32Val(5.0)),
            ("b".to_string(), Value::I32Val(3)),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_missing_input() {
        let inputs = vec![("a".to_string(), Value::F32Val(5.0))];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
