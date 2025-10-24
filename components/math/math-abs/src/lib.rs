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
            name: "Absolute Value".to_string(),
            version: "1.0.0".to_string(),
            description: "Returns the absolute value of a number".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "value".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Input number".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Absolute value (always non-negative)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Absolute Value component executing");

        let value = inputs
            .iter()
            .find(|(n, _)| n == "value")
            .and_then(|(_, v)| if let Value::F32Val(f) = v { Some(*f) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'value' input".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Provide a numeric value".to_string()),
            })?;

        let result = value.abs();

        Ok(vec![("result".to_string(), Value::F32Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs_negative() {
        let inputs = vec![("value".to_string(), Value::F32Val(-5.0))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 5.0).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_abs_positive() {
        let inputs = vec![("value".to_string(), Value::F32Val(5.0))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 5.0).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_abs_zero() {
        let inputs = vec![("value".to_string(), Value::F32Val(0.0))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 0.0).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }
}
