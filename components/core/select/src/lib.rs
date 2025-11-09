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
            name: "Select".to_string(),
            version: "1.0.0".to_string(),
            description: "Selects between two values based on a boolean condition (ternary operator: condition ? true-value : false-value)".to_string(),
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
                name: "true-value".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Value to return if condition is true".to_string(),
            },
            PortSpec {
                name: "false-value".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Value to return if condition is false".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "Selected value based on condition".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Select component executing");

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

        // Extract true-value
        let true_value = inputs
            .iter()
            .find(|(n, _)| n == "true-value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: true-value".to_string(),
                input_name: Some("true-value".to_string()),
                recovery_hint: Some("Connect a value to use when condition is true".to_string()),
            })?;

        // Extract false-value
        let false_value = inputs
            .iter()
            .find(|(n, _)| n == "false-value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: false-value".to_string(),
                input_name: Some("false-value".to_string()),
                recovery_hint: Some("Connect a value to use when condition is false".to_string()),
            })?;

        // Select the appropriate value based on condition
        let result = if condition {
            true_value.1.clone()
        } else {
            false_value.1.clone()
        };

        Ok(vec![("result".to_string(), result)])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_true_condition() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(true)),
            ("true-value".to_string(), Value::StringVal("yes".to_string())),
            ("false-value".to_string(), Value::StringVal("no".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "yes"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_select_false_condition() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(false)),
            ("true-value".to_string(), Value::StringVal("yes".to_string())),
            ("false-value".to_string(), Value::StringVal("no".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "no"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_select_with_numbers() {
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(true)),
            ("true-value".to_string(), Value::U32Val(100)),
            ("false-value".to_string(), Value::U32Val(200)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 100),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_select_mixed_types() {
        // Users can select between different types - component doesn't enforce type matching
        let inputs = vec![
            ("condition".to_string(), Value::BoolVal(false)),
            ("true-value".to_string(), Value::StringVal("text".to_string())),
            ("false-value".to_string(), Value::U32Val(42)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 42),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_missing_condition() {
        let inputs = vec![
            ("true-value".to_string(), Value::StringVal("yes".to_string())),
            ("false-value".to_string(), Value::StringVal("no".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_condition_type() {
        let inputs = vec![
            ("condition".to_string(), Value::StringVal("true".to_string())),
            ("true-value".to_string(), Value::StringVal("yes".to_string())),
            ("false-value".to_string(), Value::StringVal("no".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
