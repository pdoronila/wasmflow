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

use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use wasmflow::node::types::*;

use serde::{Deserialize, Serialize};

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Render Target".to_string(),
            version: "1.0.0".to_string(),
            description: "Configure render target parameters (resolution, format, depth, MSAA)".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "width".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Render target width in pixels".to_string(),
            },
            PortSpec {
                name: "height".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Render target height in pixels".to_string(),
            },
            PortSpec {
                name: "format".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Color format: rgba8, rgba16-float, rgba32-float".to_string(),
            },
            PortSpec {
                name: "depth".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "Enable depth buffer".to_string(),
            },
            PortSpec {
                name: "multisample".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "MSAA sample count (1, 2, 4, 8)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "config".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "JSON configuration for render system".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract width
        let width = extract_u32(&inputs, "width")?;
        if width == 0 {
            return Err(ExecutionError {
                message: "Width must be greater than 0".to_string(),
                input_name: Some("width".to_string()),
                recovery_hint: Some("Provide a positive width value (e.g., 800, 1920)".to_string()),
            });
        }

        // Extract height
        let height = extract_u32(&inputs, "height")?;
        if height == 0 {
            return Err(ExecutionError {
                message: "Height must be greater than 0".to_string(),
                input_name: Some("height".to_string()),
                recovery_hint: Some("Provide a positive height value (e.g., 600, 1080)".to_string()),
            });
        }

        // Extract format
        let format = extract_string(&inputs, "format")?;
        let format_lower = format.to_lowercase();
        let valid_formats = ["rgba8", "rgba16-float", "rgba32-float", "rgb8", "r8"];
        if !valid_formats.contains(&format_lower.as_str()) {
            return Err(ExecutionError {
                message: format!("Unsupported format: {}", format),
                input_name: Some("format".to_string()),
                recovery_hint: Some(format!(
                    "Use one of: {}",
                    valid_formats.join(", ")
                )),
            });
        }

        // Extract depth
        let depth = extract_bool(&inputs, "depth")?;

        // Extract multisample
        let multisample = extract_u32(&inputs, "multisample")?;
        let valid_samples = [1, 2, 4, 8];
        if !valid_samples.contains(&multisample) {
            return Err(ExecutionError {
                message: format!("Invalid MSAA sample count: {}", multisample),
                input_name: Some("multisample".to_string()),
                recovery_hint: Some("Use 1 (no MSAA), 2, 4, or 8 samples".to_string()),
            });
        }

        // Create configuration
        let config = RenderTargetConfig {
            width,
            height,
            format: format_lower,
            depth,
            multisample,
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&config).map_err(|e| ExecutionError {
            message: format!("Failed to serialize config: {}", e),
            input_name: None,
            recovery_hint: Some("This is an internal error. Please report it.".to_string()),
        })?;

        Ok(vec![("config".to_string(), Value::StringVal(json))])
    }
}

// Configuration data structure
#[derive(Serialize, Deserialize, Debug)]
struct RenderTargetConfig {
    width: u32,
    height: u32,
    format: String,
    depth: bool,
    multisample: u32,
}

// Helper functions
fn extract_u32(inputs: &[(String, Value)], name: &str) -> Result<u32, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.1 {
        Value::U32Val(v) => Ok(*v),
        _ => Err(ExecutionError {
            message: format!("Expected u32 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a u32 value".to_string()),
        }),
    }
}

fn extract_string(inputs: &[(String, Value)], name: &str) -> Result<String, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.1 {
        Value::StringVal(v) => Ok(v.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected string for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a string value".to_string()),
        }),
    }
}

fn extract_bool(inputs: &[(String, Value)], name: &str) -> Result<bool, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.1 {
        Value::BoolVal(v) => Ok(*v),
        _ => Err(ExecutionError {
            message: format!("Expected bool for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a boolean value".to_string()),
        }),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_target_basic() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(1920)),
            ("height".to_string(), Value::U32Val(1080)),
            ("format".to_string(), Value::StringVal("rgba8".to_string())),
            ("depth".to_string(), Value::BoolVal(true)),
            ("multisample".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "config");

        if let Value::StringVal(json) = &result[0].1 {
            // Verify it's valid JSON
            let config: RenderTargetConfig = serde_json::from_str(json).unwrap();
            assert_eq!(config.width, 1920);
            assert_eq!(config.height, 1080);
            assert_eq!(config.format, "rgba8");
            assert_eq!(config.depth, true);
            assert_eq!(config.multisample, 1);
        } else {
            panic!("Expected StringVal for config");
        }
    }

    #[test]
    fn test_render_target_hdr() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(3840)),
            ("height".to_string(), Value::U32Val(2160)),
            ("format".to_string(), Value::StringVal("rgba32-float".to_string())),
            ("depth".to_string(), Value::BoolVal(true)),
            ("multisample".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            let config: RenderTargetConfig = serde_json::from_str(json).unwrap();
            assert_eq!(config.width, 3840);
            assert_eq!(config.height, 2160);
            assert_eq!(config.format, "rgba32-float");
            assert_eq!(config.multisample, 4);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_render_target_no_depth() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(800)),
            ("height".to_string(), Value::U32Val(600)),
            ("format".to_string(), Value::StringVal("rgba8".to_string())),
            ("depth".to_string(), Value::BoolVal(false)),
            ("multisample".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            let config: RenderTargetConfig = serde_json::from_str(json).unwrap();
            assert_eq!(config.depth, false);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_render_target_msaa_8x() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(1280)),
            ("height".to_string(), Value::U32Val(720)),
            ("format".to_string(), Value::StringVal("rgba8".to_string())),
            ("depth".to_string(), Value::BoolVal(true)),
            ("multisample".to_string(), Value::U32Val(8)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            let config: RenderTargetConfig = serde_json::from_str(json).unwrap();
            assert_eq!(config.multisample, 8);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_render_target_invalid_width() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(0)),
            ("height".to_string(), Value::U32Val(1080)),
            ("format".to_string(), Value::StringVal("rgba8".to_string())),
            ("depth".to_string(), Value::BoolVal(true)),
            ("multisample".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("width".to_string()));
    }

    #[test]
    fn test_render_target_invalid_format() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(1920)),
            ("height".to_string(), Value::U32Val(1080)),
            ("format".to_string(), Value::StringVal("invalid".to_string())),
            ("depth".to_string(), Value::BoolVal(true)),
            ("multisample".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("format".to_string()));
    }

    #[test]
    fn test_render_target_invalid_multisample() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(1920)),
            ("height".to_string(), Value::U32Val(1080)),
            ("format".to_string(), Value::StringVal("rgba8".to_string())),
            ("depth".to_string(), Value::BoolVal(true)),
            ("multisample".to_string(), Value::U32Val(3)), // Invalid
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("multisample".to_string()));
    }

    #[test]
    fn test_render_target_format_case_insensitive() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(1920)),
            ("height".to_string(), Value::U32Val(1080)),
            ("format".to_string(), Value::StringVal("RGBA8".to_string())),
            ("depth".to_string(), Value::BoolVal(true)),
            ("multisample".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            let config: RenderTargetConfig = serde_json::from_str(json).unwrap();
            // Should be normalized to lowercase
            assert_eq!(config.format, "rgba8");
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_render_target_json_structure() {
        let inputs = vec![
            ("width".to_string(), Value::U32Val(1920)),
            ("height".to_string(), Value::U32Val(1080)),
            ("format".to_string(), Value::StringVal("rgba8".to_string())),
            ("depth".to_string(), Value::BoolVal(true)),
            ("multisample".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(json) = &result[0].1 {
            // Verify JSON structure
            assert!(json.contains("\"width\""));
            assert!(json.contains("\"height\""));
            assert!(json.contains("\"format\""));
            assert!(json.contains("\"depth\""));
            assert!(json.contains("\"multisample\""));
            assert!(json.contains("1920"));
            assert!(json.contains("1080"));
        } else {
            panic!("Expected StringVal");
        }
    }
}
