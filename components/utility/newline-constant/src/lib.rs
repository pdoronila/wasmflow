//! Newline Constant - Outputs a proper newline character
//!
//! This component exists to solve the problem of Constant nodes treating
//! "\n" as literal text. It outputs an actual newline character.

wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Newline".to_string(),
            version: "1.0.0".to_string(),
            description: "Outputs a newline character (\\n) - useful for splitting text by lines".to_string(),
            author: "WasmFlow Utility".to_string(),
            category: Some("Utility".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "newline".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "A single newline character (\\n)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(_inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        Ok(vec![
            ("newline".to_string(), Value::StringVal("\n".to_string())),
        ])
    }
}

export!(Component);
