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
            name: "Boolean NOT".to_string(),
            version: "1.0.0".to_string(),
            description: "Inverts a boolean value (logical NOT operation)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "input".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "Boolean value to invert".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "Inverted boolean value".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Boolean NOT component executing");

        let input = inputs
            .iter()
            .find(|(n, _)| n == "input")
            .and_then(|(_, v)| if let Value::BoolVal(b) = v { Some(*b) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'input' value".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Provide a boolean value (true/false)".to_string()),
            })?;

        let result = !input;

        Ok(vec![("result".to_string(), Value::BoolVal(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_true() {
        let inputs = vec![("input".to_string(), Value::BoolVal(true))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_not_false() {
        let inputs = vec![("input".to_string(), Value::BoolVal(false))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }
}
