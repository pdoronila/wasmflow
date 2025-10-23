//! Echo Component - Pass-Through Component
//!
//! This component takes any input value and outputs it unchanged.
//! It demonstrates a simple pass-through component that works with any data type.

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
            name: "Echo".to_string(),
            version: "1.0.0".to_string(),
            description: "Passes through any input value unchanged".to_string(),
            author: "WasmFlow".to_string(),
            category: Some("Utility".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "input".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "Value to echo".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "output".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "Echoed value".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None // No special capabilities required - pure pass-through
    }
}

// Implement the execution interface
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Echo component executing");

        // Extract input value
        let value = inputs
            .iter()
            .find(|(name, _)| name == "input")
            .map(|(_, val)| val.clone())
            .ok_or_else(|| ExecutionError {
                message: "Missing 'input' value".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect a value to the 'input' port".to_string()),
            })?;

        // Log the value type
        let value_type = match &value {
            Value::U32Val(_) => "u32",
            Value::I32Val(_) => "i32",
            Value::F32Val(_) => "f32",
            Value::StringVal(_) => "string",
            Value::BinaryVal(_) => "binary",
        };
        host::log("debug", &format!("Echoing {} value", value_type));

        // Return the same value as output
        Ok(vec![("output".to_string(), value)])
    }
}

export!(Component);
