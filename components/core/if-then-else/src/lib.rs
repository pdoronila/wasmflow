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
            name: "If-Then-Else".to_string(),
            version: "1.0.0".to_string(),
            description: "Routes different values to then-output or else-output based on a condition. If condition is true, outputs then-value to then-output. If false, outputs else-value to else-output.".to_string(),
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
                name: "then-value".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Value to send to then-output when condition is true".to_string(),
            },
            PortSpec {
                name: "else-value".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Value to send to else-output when condition is false".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "then-output".to_string(),
                data_type: DataType::AnyType,
                optional: true,
                description: "Receives then-value when condition is true".to_string(),
            },
            PortSpec {
                name: "else-output".to_string(),
                data_type: DataType::AnyType,
                optional: true,
                description: "Receives else-value when condition is false".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "If-Then-Else component executing");

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

        // Extract then-value
        let then_value = inputs
            .iter()
            .find(|(n, _)| n == "then-value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: then-value".to_string(),
                input_name: Some("then-value".to_string()),
                recovery_hint: Some("Connect a value to send when condition is true".to_string()),
            })?;

        // Extract else-value
        let else_value = inputs
            .iter()
            .find(|(n, _)| n == "else-value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: else-value".to_string(),
                input_name: Some("else-value".to_string()),
                recovery_hint: Some("Connect a value to send when condition is false".to_string()),
            })?;

        // Route to appropriate output based on condition
        if condition {
            host::log("debug", "Condition is true, routing to then-output");
            Ok(vec![("then-output".to_string(), then_value.1.clone())])
        } else {
            host::log("debug", "Condition is false, routing to else-output");
            Ok(vec![("else-output".to_string(), else_value.1.clone())])
        }
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_to_then() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(true)),
            ("then-value".to_string(), Value::StringVal("hello".to_string())),
            ("else-value".to_string(), Value::StringVal("world".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "then-output");
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_route_to_else() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(false)),
            ("then-value".to_string(), Value::StringVal("hello".to_string())),
            ("else-value".to_string(), Value::StringVal("world".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "else-output");
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "world"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_route_with_different_types() {
        // Then value is string, else value is number - both are valid
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(true)),
            ("then-value".to_string(), Value::StringVal("success".to_string())),
            ("else-value".to_string(), Value::U32Val(500)),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].0, "then-output");
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "success"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_route_numbers() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(false)),
            ("then-value".to_string(), Value::U32Val(200)),
            ("else-value".to_string(), Value::U32Val(404)),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].0, "else-output");
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 404),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_missing_condition() {
        let inputs = vec![
            ("then-value".to_string(), Value::StringVal("test".to_string())),
            ("else-value".to_string(), Value::StringVal("test2".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_then_value() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(true)),
            ("else-value".to_string(), Value::StringVal("test".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_else_value() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(false)),
            ("then-value".to_string(), Value::StringVal("test".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_condition_type() {
        let inputs = vec![
            ("condition".to_string(), Value::U32Val(1)),
            ("then-value".to_string(), Value::StringVal("test".to_string())),
            ("else-value".to_string(), Value::StringVal("test2".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
