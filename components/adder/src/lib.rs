//! Example Adder Component - Basic WasmFlow Component
//!
//! This component adds two numbers together.
//! It demonstrates a pure computation component with no special capabilities.

// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

// Implement the metadata interface
impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Add Numbers".to_string(),
            version: "1.0.0".to_string(),
            description: "Adds two numbers together".to_string(),
            author: "WasmFlow Examples".to_string(),
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
            description: "Sum of a and b".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None // No special capabilities required - pure computation
    }
}

// Implement the execution interface
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Add Numbers component executing");

        // Extract input value 'a'
        let a = inputs
            .iter()
            .find(|(name, _)| name == "a")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'a' value".to_string(),
                input_name: Some("a".to_string()),
                recovery_hint: Some("Connect an F32 value to the 'a' input port".to_string()),
            })?;

        // Extract input value 'b'
        let b = inputs
            .iter()
            .find(|(name, _)| name == "b")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'b' value".to_string(),
                input_name: Some("b".to_string()),
                recovery_hint: Some("Connect an F32 value to the 'b' input port".to_string()),
            })?;

        // Perform addition
        let sum = a + b;

        // Log the result
        let log_msg = format!("{} + {} = {}", a, b, sum);
        host::log("info", &log_msg);

        // Return the output
        Ok(vec![("sum".to_string(), Value::F32Val(sum))])
    }
}

export!(Component);
