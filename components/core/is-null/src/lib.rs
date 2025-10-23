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
            name: "Is Null".to_string(),
            version: "1.0.0".to_string(),
            description: "Checks if a value is null (no input provided)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "value".to_string(),
            data_type: DataType::AnyType,
            optional: true,
            description: "Optional value to check".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "True if value is null (not provided), false otherwise".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Is Null component executing");

        let is_null = inputs.iter().find(|(n, _)| n == "value").is_none();

        Ok(vec![("result".to_string(), Value::BoolVal(is_null))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_input_is_null() {
        let inputs = vec![];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_empty_string_not_null() {
        let inputs = vec![("value".to_string(), Value::StringVal("".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_zero_not_null() {
        let inputs = vec![("value".to_string(), Value::U32Val(0))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }
}
