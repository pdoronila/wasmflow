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

/// Directional light data structure (for serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DirectionalLightData {
    light_type: String,
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32,
}

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Directional Light".to_string(),
            version: "1.0.0".to_string(),
            description: "Directional light source (like the sun) with parallel rays".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "direction".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Light direction vector (will be normalized)".to_string(),
            },
            PortSpec {
                name: "color".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Light color (RGB, 0.0-1.0 range)".to_string(),
            },
            PortSpec {
                name: "intensity".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Light intensity multiplier".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "light_data".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "JSON-encoded light data".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract direction
        let direction_input = inputs
            .iter()
            .find(|(name, _)| name == "direction")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: direction".to_string(),
                input_name: Some("direction".to_string()),
                recovery_hint: Some("Connect a vec3 value to the direction input".to_string()),
            })?;
        let direction_vec = match &direction_input.1 {
            Value::Vec3Val(v) => v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected vec3 for 'direction', got {:?}", direction_input.1),
                    input_name: Some("direction".to_string()),
                    recovery_hint: Some("Provide a vec3 value".to_string()),
                });
            }
        };

        // Normalize direction vector
        let len = (direction_vec.x * direction_vec.x
            + direction_vec.y * direction_vec.y
            + direction_vec.z * direction_vec.z)
            .sqrt();

        if len < 1e-6 {
            return Err(ExecutionError {
                message: "Direction vector has zero length".to_string(),
                input_name: Some("direction".to_string()),
                recovery_hint: Some("Provide a non-zero direction vector".to_string()),
            });
        }

        let normalized_direction = [
            direction_vec.x / len,
            direction_vec.y / len,
            direction_vec.z / len,
        ];

        // Extract color
        let color_input = inputs
            .iter()
            .find(|(name, _)| name == "color")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: color".to_string(),
                input_name: Some("color".to_string()),
                recovery_hint: Some("Connect a vec3 value to the color input".to_string()),
            })?;
        let color_vec = match &color_input.1 {
            Value::Vec3Val(v) => v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected vec3 for 'color', got {:?}", color_input.1),
                    input_name: Some("color".to_string()),
                    recovery_hint: Some("Provide a vec3 value (RGB)".to_string()),
                });
            }
        };

        // Clamp color to [0.0, 1.0]
        let clamped_color = [
            color_vec.x.max(0.0).min(1.0),
            color_vec.y.max(0.0).min(1.0),
            color_vec.z.max(0.0).min(1.0),
        ];

        // Extract intensity
        let intensity_input = inputs
            .iter()
            .find(|(name, _)| name == "intensity")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: intensity".to_string(),
                input_name: Some("intensity".to_string()),
                recovery_hint: Some("Connect an f32 value to the intensity input".to_string()),
            })?;
        let intensity = match &intensity_input.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'intensity', got {:?}", intensity_input.1),
                    input_name: Some("intensity".to_string()),
                    recovery_hint: Some("Provide an f32 value".to_string()),
                });
            }
        };

        if intensity < 0.0 {
            return Err(ExecutionError {
                message: "Light intensity must be non-negative".to_string(),
                input_name: Some("intensity".to_string()),
                recovery_hint: Some("Provide a positive intensity value".to_string()),
            });
        }

        // Create light data structure
        let light_data = DirectionalLightData {
            light_type: "directional".to_string(),
            direction: normalized_direction,
            color: clamped_color,
            intensity,
        };

        // Serialize to JSON
        let json_str = serde_json::to_string(&light_data).map_err(|e| ExecutionError {
            message: format!("Failed to serialize light data: {}", e),
            input_name: None,
            recovery_hint: Some("Internal error - check light data validity".to_string()),
        })?;

        Ok(vec![("light_data".to_string(), Value::StringVal(json_str))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directional_light_basic() {
        let inputs = vec![
            (
                "direction".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "light_data");

        if let Value::StringVal(json) = &result[0].1 {
            let light: DirectionalLightData = serde_json::from_str(json).unwrap();
            assert_eq!(light.light_type, "directional");
            assert_eq!(light.direction, [0.0, -1.0, 0.0]); // Already normalized
            assert_eq!(light.color, [1.0, 1.0, 1.0]);
            assert_eq!(light.intensity, 1.0);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_directional_light_normalize_direction() {
        let inputs = vec![
            (
                "direction".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(2.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::StringVal(json) = &result[0].1 {
            let light: DirectionalLightData = serde_json::from_str(json).unwrap();
            // Direction should be normalized to (1, 0, 0)
            assert!((light.direction[0] - 1.0).abs() < 1e-6);
            assert!((light.direction[1] - 0.0).abs() < 1e-6);
            assert!((light.direction[2] - 0.0).abs() < 1e-6);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_directional_light_clamp_color() {
        let inputs = vec![
            (
                "direction".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 1.5,  // Over 1.0
                    y: 0.5,
                    z: -0.2, // Below 0.0
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::StringVal(json) = &result[0].1 {
            let light: DirectionalLightData = serde_json::from_str(json).unwrap();
            // Color should be clamped to [0.0, 1.0]
            assert_eq!(light.color, [1.0, 0.5, 0.0]);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_directional_light_zero_direction() {
        let inputs = vec![
            (
                "direction".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("zero length"));
    }

    #[test]
    fn test_directional_light_negative_intensity() {
        let inputs = vec![
            (
                "direction".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(-1.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("non-negative"));
    }

    #[test]
    fn test_directional_light_missing_input() {
        let inputs = vec![
            (
                "direction".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            // Missing intensity
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("intensity".to_string()));
    }
}
