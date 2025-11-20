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

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "RGB Color Constructor".to_string(),
            version: "1.0.0".to_string(),
            description: "Create a color from RGB components (values 0-1)".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "r".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Red component (0-1)".to_string(),
            },
            PortSpec {
                name: "g".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Green component (0-1)".to_string(),
            },
            PortSpec {
                name: "b".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Blue component (0-1)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "color".to_string(),
            data_type: DataType::Vec3Type,
            optional: false,
            description: "Resulting color as vec3".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract r component
        let r_input = inputs
            .iter()
            .find(|(name, _)| name == "r")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: r".to_string(),
                input_name: Some("r".to_string()),
                recovery_hint: Some("Connect an f32 value to the r input".to_string()),
            })?;
        let r_val = match &r_input.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'r', got {:?}", r_input.1),
                    input_name: Some("r".to_string()),
                    recovery_hint: Some("Provide an f32 value between 0.0 and 1.0".to_string()),
                })
            }
        };

        // Extract g component
        let g_input = inputs
            .iter()
            .find(|(name, _)| name == "g")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: g".to_string(),
                input_name: Some("g".to_string()),
                recovery_hint: Some("Connect an f32 value to the g input".to_string()),
            })?;
        let g_val = match &g_input.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'g', got {:?}", g_input.1),
                    input_name: Some("g".to_string()),
                    recovery_hint: Some("Provide an f32 value between 0.0 and 1.0".to_string()),
                })
            }
        };

        // Extract b component
        let b_input = inputs
            .iter()
            .find(|(name, _)| name == "b")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: b".to_string(),
                input_name: Some("b".to_string()),
                recovery_hint: Some("Connect an f32 value to the b input".to_string()),
            })?;
        let b_val = match &b_input.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'b', got {:?}", b_input.1),
                    input_name: Some("b".to_string()),
                    recovery_hint: Some("Provide an f32 value between 0.0 and 1.0".to_string()),
                })
            }
        };

        // Clamp values to [0.0, 1.0] range
        let r_clamped = r_val.clamp(0.0, 1.0);
        let g_clamped = g_val.clamp(0.0, 1.0);
        let b_clamped = b_val.clamp(0.0, 1.0);

        // Construct vec3 color
        let color = Vec3 {
            x: r_clamped,
            y: g_clamped,
            z: b_clamped,
        };

        Ok(vec![("color".to_string(), Value::Vec3Val(color))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_rgb_basic() {
        let inputs = vec![
            ("r".to_string(), Value::F32Val(1.0)),
            ("g".to_string(), Value::F32Val(0.0)),
            ("b".to_string(), Value::F32Val(0.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "color");

        if let Value::Vec3Val(color) = &result[0].1 {
            assert_eq!(color.x, 1.0);
            assert_eq!(color.y, 0.0);
            assert_eq!(color.z, 0.0);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_color_rgb_gray() {
        let inputs = vec![
            ("r".to_string(), Value::F32Val(0.5)),
            ("g".to_string(), Value::F32Val(0.5)),
            ("b".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::Vec3Val(color) = &result[0].1 {
            assert_eq!(color.x, 0.5);
            assert_eq!(color.y, 0.5);
            assert_eq!(color.z, 0.5);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_color_rgb_clamping_high() {
        // Test that values > 1.0 are clamped
        let inputs = vec![
            ("r".to_string(), Value::F32Val(1.5)),
            ("g".to_string(), Value::F32Val(2.0)),
            ("b".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::Vec3Val(color) = &result[0].1 {
            assert_eq!(color.x, 1.0);
            assert_eq!(color.y, 1.0);
            assert_eq!(color.z, 1.0);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_color_rgb_clamping_low() {
        // Test that values < 0.0 are clamped
        let inputs = vec![
            ("r".to_string(), Value::F32Val(-0.5)),
            ("g".to_string(), Value::F32Val(-1.0)),
            ("b".to_string(), Value::F32Val(-10.0)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::Vec3Val(color) = &result[0].1 {
            assert_eq!(color.x, 0.0);
            assert_eq!(color.y, 0.0);
            assert_eq!(color.z, 0.0);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_color_rgb_missing_input() {
        let inputs = vec![
            ("r".to_string(), Value::F32Val(1.0)),
            ("g".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("b".to_string()));
    }

    #[test]
    fn test_color_rgb_common_colors() {
        // Test white
        let inputs = vec![
            ("r".to_string(), Value::F32Val(1.0)),
            ("g".to_string(), Value::F32Val(1.0)),
            ("b".to_string(), Value::F32Val(1.0)),
        ];
        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(color) = &result[0].1 {
            assert_eq!(color.x, 1.0);
            assert_eq!(color.y, 1.0);
            assert_eq!(color.z, 1.0);
        }

        // Test black
        let inputs = vec![
            ("r".to_string(), Value::F32Val(0.0)),
            ("g".to_string(), Value::F32Val(0.0)),
            ("b".to_string(), Value::F32Val(0.0)),
        ];
        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(color) = &result[0].1 {
            assert_eq!(color.x, 0.0);
            assert_eq!(color.y, 0.0);
            assert_eq!(color.z, 0.0);
        }

        // Test blue
        let inputs = vec![
            ("r".to_string(), Value::F32Val(0.0)),
            ("g".to_string(), Value::F32Val(0.0)),
            ("b".to_string(), Value::F32Val(1.0)),
        ];
        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(color) = &result[0].1 {
            assert_eq!(color.x, 0.0);
            assert_eq!(color.y, 0.0);
            assert_eq!(color.z, 1.0);
        }
    }
}
