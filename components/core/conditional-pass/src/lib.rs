// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "./wit",
    world: "component",
    with: {
        "wasmflow:node/types@1.1.0": generate,
        "wasmflow:node/host@1.1.0": generate,
        "wasmflow:node/metadata@1.1.0": generate,
        "wasmflow:node/execution@1.1.0": generate,
    },
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Conditional Pass".to_string(),
            version: "1.0.0".to_string(),
            description: "Passes a value through if condition is true, otherwise blocks (outputs nothing). Acts as a filter or gate.".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "condition".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "Boolean condition to evaluate".to_string(),
            },
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Value to pass through if condition is true".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::AnyType,
            optional: true,
            description: "Value passed through when condition is true (no output when false)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Conditional Pass component executing");

        // Extract condition
        let condition = inputs
            .iter()
            .find(|(n, _)| n == "condition")
            .and_then(|(_, v)| if let Value::BoolVal(b) = v { Some(*b) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'condition' input".to_string(),
                input_name: Some("condition".to_string()),
                recovery_hint: Some("Provide a boolean value (true/false)".to_string()),
            })?;

        // If condition is false, return empty (no output)
        if !condition {
            host::log("debug", "Condition is false, blocking value");
            return Ok(vec![]);
        }

        // Extract value
        let value = inputs
            .iter()
            .find(|(n, _)| n == "value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: value".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Connect a value to pass through this component".to_string()),
            })?;

        // Pass value through
        host::log("debug", "Condition is true, passing value through");
        Ok(vec![("result".to_string(), value.1.clone())])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_when_true() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(true)),
            ("value".to_string(), Value::StringVal("hello".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_block_when_false() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(false)),
            ("value".to_string(), Value::StringVal("blocked".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 0, "Expected no output when condition is false");
    }

    #[test]
    fn test_pass_number() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(true)),
            ("value".to_string(), Value::U32Val(42)),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 42),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_pass_list() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(true)),
            ("value".to_string(), Value::StringListVal(vec!["a".to_string(), "b".to_string()])),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].1 {
            Value::StringListVal(list) => assert_eq!(list.len(), 2),
            _ => panic!("Expected string list output"),
        }
    }

    #[test]
    fn test_missing_condition() {
        let inputs = vec![
            ("value".to_string(), Value::StringVal("test".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_condition_type() {
        let inputs = vec![
            ("condition".to_string(), Value::StringVal("true".to_string())),
            ("value".to_string(), Value::StringVal("test".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
