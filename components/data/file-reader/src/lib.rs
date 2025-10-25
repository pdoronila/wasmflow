//! Example File Reader Component - Capability-Requiring WasmFlow Component
//!
//! This component reads a file from the filesystem.
//! It demonstrates how to declare and use file-read capabilities.

// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use std::fs;
use wasmflow::node::host;
use wasmflow::node::types::*;

struct Component;

// Implement the metadata interface
impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Read File".to_string(),
            version: "1.0.0".to_string(),
            description: "Reads text content from a file".to_string(),
            author: "WasmFlow Examples".to_string(),
            category: Some("Files".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "path".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "File path to read".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "content".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "File contents as text".to_string(),
            },
            PortSpec {
                name: "size".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "File size in bytes".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        // Request file-read access to common directories
        // User will be prompted to approve this capability
        // Users can also grant "Full Access" via the UI for unrestricted file access
        Some(vec![
            "file-read:/tmp".to_string(),
            "file-read:/Users".to_string(),
        ])
    }
}

// Implement the execution interface
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Read File component executing");

        // Extract file path
        let path = inputs
            .iter()
            .find(|(name, _)| name == "path")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'path' value".to_string(),
                input_name: Some("path".to_string()),
                recovery_hint: Some("Connect a String value with the file path".to_string()),
            })?;

        host::log("info", &format!("Reading file: {}", path));

        // Note: Path validation is handled by the host runtime based on granted capabilities.
        // If Full Access is granted, any path is allowed.
        // If restricted capabilities are granted, the host will enforce the restrictions.

        // Read file content
        let content = fs::read_to_string(&path).map_err(|e| ExecutionError {
            message: format!("Failed to read file: {}", e),
            input_name: Some("path".to_string()),
            recovery_hint: Some("Check that the file exists and is readable".to_string()),
        })?;

        // Get file size
        let metadata = fs::metadata(&path).map_err(|e| ExecutionError {
            message: format!("Failed to get file metadata: {}", e),
            input_name: Some("path".to_string()),
            recovery_hint: None,
        })?;
        let size = metadata.len() as u32;

        host::log(
            "info",
            &format!("Successfully read {} bytes from {}", size, path),
        );

        // Return outputs
        Ok(vec![
            ("content".to_string(), Value::StringVal(content)),
            ("size".to_string(), Value::U32Val(size)),
        ])
    }
}

export!(Component);
