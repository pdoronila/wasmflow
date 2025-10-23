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

// Implement the metadata interface
impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "ComponentName".to_string(),
            version: "1.0.0".to_string(),
            description: "Component description".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Core".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input1".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Input description".to_string(),
            }
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "output".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Output description".to_string(),
            }
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None // Pure computation - no special capabilities needed
    }
}

// Implement the execution interface
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Component executing");

        // Extract input
        let input_val = extract_string_input(&inputs, "input1")?;

        // TODO: Implement component logic here

        // Return output
        Ok(vec![
            ("output".to_string(), Value::StringVal(input_val))
        ])
    }
}

export!(Component);

// Helper function to extract string input
fn extract_string_input(inputs: &[(String, Value)], name: &str) -> Result<String, ExecutionError> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| {
            if let Value::StringVal(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| ExecutionError {
            message: format!("Missing or invalid '{}' input", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a string value".to_string()),
        })
}

// Helper function to extract u32 input
fn extract_u32_input(inputs: &[(String, Value)], name: &str) -> Result<u32, ExecutionError> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| {
            if let Value::U32Val(n) = v {
                Some(*n)
            } else {
                None
            }
        })
        .ok_or_else(|| ExecutionError {
            message: format!("Missing or invalid '{}' input", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a u32 value".to_string()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component() {
        // TODO: Add tests
    }
}
