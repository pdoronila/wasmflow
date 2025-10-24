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
            name: "Is Empty".to_string(),
            version: "1.0.0".to_string(),
            description: "Checks if a string or list is empty".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "value".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "String or list value to check".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "True if value is empty, false otherwise".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Is Empty component executing");

        let value = inputs
            .iter()
            .find(|(n, _)| n == "value")
            .map(|(_, v)| v)
            .ok_or_else(|| ExecutionError {
                message: "Missing 'value' input".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Provide a string or list value to check".to_string()),
            })?;

        let is_empty = match value {
            Value::StringVal(s) => s.is_empty(),
            Value::StringListVal(list) => list.is_empty(),
            Value::U32ListVal(list) => list.is_empty(),
            Value::F32ListVal(list) => list.is_empty(),
            Value::BinaryVal(b) => b.is_empty(),
            _ => {
                return Err(ExecutionError {
                    message: format!("Cannot check if {:?} is empty (only strings and lists supported)", value),
                    input_name: Some("value".to_string()),
                    recovery_hint: Some("Provide a string or list value".to_string()),
                });
            }
        };

        Ok(vec![("result".to_string(), Value::BoolVal(is_empty))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let inputs = vec![("value".to_string(), Value::StringVal("".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_non_empty_string() {
        let inputs = vec![("value".to_string(), Value::StringVal("hello".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_empty_list() {
        let inputs = vec![("value".to_string(), Value::StringListVal(vec![]))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_non_empty_list() {
        let inputs = vec![(
            "value".to_string(),
            Value::StringListVal(vec!["item".to_string()]),
        )];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_number_error() {
        let inputs = vec![("value".to_string(), Value::U32Val(42))];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
