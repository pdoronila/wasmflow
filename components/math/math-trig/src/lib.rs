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
            name: "Trigonometry".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculates trigonometric functions (sin, cos, tan)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "angle".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Angle in radians".to_string(),
            },
            PortSpec {
                name: "operation".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Trigonometric function: sin, cos, or tan".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Result of trigonometric function".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Trigonometry component executing");

        let angle = inputs
            .iter()
            .find(|(n, _)| n == "angle")
            .and_then(|(_, v)| if let Value::F32Val(f) = v { Some(*f) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'angle' input".to_string(),
                input_name: Some("angle".to_string()),
                recovery_hint: Some("Provide an angle in radians".to_string()),
            })?;

        let operation = inputs
            .iter()
            .find(|(n, _)| n == "operation")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'operation' input".to_string(),
                input_name: Some("operation".to_string()),
                recovery_hint: Some("Provide a valid operation string".to_string()),
            })?;

        let result = match operation.as_str() {
            "sin" => angle.sin(),
            "cos" => angle.cos(),
            "tan" => angle.tan(),
            _ => {
                return Err(ExecutionError {
                    message: format!("Invalid operation: {}", operation),
                    input_name: Some("operation".to_string()),
                    recovery_hint: Some("Use 'sin', 'cos', or 'tan'".to_string()),
                });
            }
        };

        Ok(vec![("result".to_string(), Value::F32Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_sin_zero() {
        let inputs = vec![
            ("angle".to_string(), Value::F32Val(0.0)),
            ("operation".to_string(), Value::StringVal("sin".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!(f.abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_sin_pi_over_2() {
        let inputs = vec![
            ("angle".to_string(), Value::F32Val(PI / 2.0)),
            ("operation".to_string(), Value::StringVal("sin".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 1.0).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_cos_pi() {
        let inputs = vec![
            ("angle".to_string(), Value::F32Val(PI)),
            ("operation".to_string(), Value::StringVal("cos".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - (-1.0)).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_invalid_operation() {
        let inputs = vec![
            ("angle".to_string(), Value::F32Val(0.0)),
            ("operation".to_string(), Value::StringVal("invalid".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
