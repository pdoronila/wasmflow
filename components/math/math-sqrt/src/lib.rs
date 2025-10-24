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
            name: "Square Root".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculates the square root of a number".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "value".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Number to calculate square root of".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Square root of the input value".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Square Root component executing");

        let value = inputs
            .iter()
            .find(|(n, _)| n == "value")
            .and_then(|(_, v)| if let Value::F32Val(f) = v { Some(*f) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'value' input".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Provide a numeric value".to_string()),
            })?;

        if value < 0.0 {
            return Err(ExecutionError {
                message: format!("Cannot calculate square root of negative number: {}", value),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Provide a non-negative number".to_string()),
            });
        }

        let result = value.sqrt();

        Ok(vec![("result".to_string(), Value::F32Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt_perfect_square() {
        let inputs = vec![("value".to_string(), Value::F32Val(16.0))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 4.0).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_sqrt_irrational() {
        let inputs = vec![("value".to_string(), Value::F32Val(2.0))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 1.414).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_sqrt_negative_error() {
        let inputs = vec![("value".to_string(), Value::F32Val(-1.0))];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
