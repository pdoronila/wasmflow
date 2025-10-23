//! Double Number Component - Example WasmFlow Component
//!
//! This component multiplies an input number by 2.
//! It demonstrates the basic structure of a WasmFlow component using proper WIT bindings.

#[allow(warnings)]
mod bindings;

use bindings::exports::wasmflow::node::execution::Guest as ExecutionGuest;
use bindings::exports::wasmflow::node::metadata::Guest as MetadataGuest;
use bindings::wasmflow::node::host;
use bindings::wasmflow::node::types::*;

struct Component;

// Implement the metadata interface
impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Double Number".to_string(),
            version: "1.0.0".to_string(),
            description: "Multiplies input by 2".to_string(),
            author: "WasmFlow Example".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "input".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Number to double".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "output".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Doubled number".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None // No special capabilities required
    }
}

// Implement the execution interface
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Log execution start
        host::log("debug", "Double Number component executing");

        // Extract input value
        let input_value = inputs
            .iter()
            .find(|(name, _)| name == "input")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'input' value".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect an F32 value to the input port".to_string()),
            })?;

        // Perform the doubling operation
        let result = input_value * 2.0;

        // Log the result
        let log_msg = format!("Doubled {} to {}", input_value, result);
        host::log("info", &log_msg);

        // Return the output
        Ok(vec![("output".to_string(), Value::F32Val(result))])
    }
}

bindings::export!(Component with_types_in bindings);
