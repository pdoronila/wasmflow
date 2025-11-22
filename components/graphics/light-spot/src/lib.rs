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

use glam::Vec3;
use serde::{Deserialize, Serialize};

struct Component;

/// Spot light data structure (for serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpotLightData {
    light_type: String,
    position: [f32; 3],
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32,
    inner_angle: f32, // In degrees
    outer_angle: f32, // In degrees
    radius: f32,
}

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Spot Light".to_string(),
            version: "1.0.0".to_string(),
            description: "Spot light source with cone-shaped emission and smooth falloff".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "position".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Light position in world space".to_string(),
            },
            PortSpec {
                name: "direction".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Light direction (will be normalized)".to_string(),
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
            PortSpec {
                name: "inner_angle".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Inner cone angle in degrees (full intensity)".to_string(),
            },
            PortSpec {
                name: "outer_angle".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Outer cone angle in degrees (zero intensity)".to_string(),
            },
            PortSpec {
                name: "radius".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Light attenuation radius".to_string(),
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
        // Extract position
        let position_input = inputs
            .iter()
            .find(|(name, _)| name == "position")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: position".to_string(),
                input_name: Some("position".to_string()),
                recovery_hint: Some("Connect a vec3 value to the position input".to_string()),
            })?;
        let position_vec = match &position_input.1 {
            Value::Vec3Val(v) => v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected vec3 for 'position', got {:?}", position_input.1),
                    input_name: Some("position".to_string()),
                    recovery_hint: Some("Provide a vec3 value".to_string()),
                });
            }
        };

        let position = [position_vec.x, position_vec.y, position_vec.z];

        // Extract and normalize direction
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

        let dir_glam = Vec3::new(direction_vec.x, direction_vec.y, direction_vec.z);
        let normalized_dir = dir_glam.normalize_or_zero();

        if normalized_dir == Vec3::ZERO {
            return Err(ExecutionError {
                message: "Direction cannot be zero vector".to_string(),
                input_name: Some("direction".to_string()),
                recovery_hint: Some("Provide a non-zero direction vector".to_string()),
            });
        }

        let direction = [normalized_dir.x, normalized_dir.y, normalized_dir.z];

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

        // Extract inner angle
        let inner_angle_input = inputs
            .iter()
            .find(|(name, _)| name == "inner_angle")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: inner_angle".to_string(),
                input_name: Some("inner_angle".to_string()),
                recovery_hint: Some("Connect an f32 value to the inner_angle input".to_string()),
            })?;
        let inner_angle = match &inner_angle_input.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'inner_angle', got {:?}", inner_angle_input.1),
                    input_name: Some("inner_angle".to_string()),
                    recovery_hint: Some("Provide an f32 value in degrees".to_string()),
                });
            }
        };

        if inner_angle < 0.0 || inner_angle > 90.0 {
            return Err(ExecutionError {
                message: format!("Inner angle must be in range [0, 90], got {}", inner_angle),
                input_name: Some("inner_angle".to_string()),
                recovery_hint: Some("Provide an angle between 0 and 90 degrees".to_string()),
            });
        }

        // Extract outer angle
        let outer_angle_input = inputs
            .iter()
            .find(|(name, _)| name == "outer_angle")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: outer_angle".to_string(),
                input_name: Some("outer_angle".to_string()),
                recovery_hint: Some("Connect an f32 value to the outer_angle input".to_string()),
            })?;
        let outer_angle = match &outer_angle_input.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'outer_angle', got {:?}", outer_angle_input.1),
                    input_name: Some("outer_angle".to_string()),
                    recovery_hint: Some("Provide an f32 value in degrees".to_string()),
                });
            }
        };

        if outer_angle < 0.0 || outer_angle > 90.0 {
            return Err(ExecutionError {
                message: format!("Outer angle must be in range [0, 90], got {}", outer_angle),
                input_name: Some("outer_angle".to_string()),
                recovery_hint: Some("Provide an angle between 0 and 90 degrees".to_string()),
            });
        }

        if inner_angle >= outer_angle {
            return Err(ExecutionError {
                message: format!(
                    "Inner angle ({}) must be less than outer angle ({})",
                    inner_angle, outer_angle
                ),
                input_name: Some("inner_angle".to_string()),
                recovery_hint: Some("Inner angle defines full intensity, outer angle defines zero intensity".to_string()),
            });
        }

        // Extract radius
        let radius_input = inputs
            .iter()
            .find(|(name, _)| name == "radius")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: radius".to_string(),
                input_name: Some("radius".to_string()),
                recovery_hint: Some("Connect an f32 value to the radius input".to_string()),
            })?;
        let radius = match &radius_input.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'radius', got {:?}", radius_input.1),
                    input_name: Some("radius".to_string()),
                    recovery_hint: Some("Provide an f32 value".to_string()),
                });
            }
        };

        if radius <= 0.0 {
            return Err(ExecutionError {
                message: "Light radius must be positive".to_string(),
                input_name: Some("radius".to_string()),
                recovery_hint: Some("Provide a positive radius value".to_string()),
            });
        }

        // Create light data structure
        let light_data = SpotLightData {
            light_type: "spot".to_string(),
            position,
            direction,
            color: clamped_color,
            intensity,
            inner_angle,
            outer_angle,
            radius,
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
    fn test_spot_light_basic() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 5.0,
                    z: 0.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
            ("inner_angle".to_string(), Value::F32Val(15.0)),
            ("outer_angle".to_string(), Value::F32Val(30.0)),
            ("radius".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "light_data");

        if let Value::StringVal(json) = &result[0].1 {
            let light: SpotLightData = serde_json::from_str(json).unwrap();
            assert_eq!(light.light_type, "spot");
            assert_eq!(light.position, [0.0, 5.0, 0.0]);
            assert_eq!(light.direction, [0.0, -1.0, 0.0]);
            assert_eq!(light.color, [1.0, 1.0, 1.0]);
            assert_eq!(light.intensity, 1.0);
            assert_eq!(light.inner_angle, 15.0);
            assert_eq!(light.outer_angle, 30.0);
            assert_eq!(light.radius, 10.0);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_spot_light_direction_normalization() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 3.0,  // Not normalized
                    y: 4.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
            ("inner_angle".to_string(), Value::F32Val(10.0)),
            ("outer_angle".to_string(), Value::F32Val(20.0)),
            ("radius".to_string(), Value::F32Val(5.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::StringVal(json) = &result[0].1 {
            let light: SpotLightData = serde_json::from_str(json).unwrap();
            // Direction should be normalized: [3, 4, 0] -> [0.6, 0.8, 0.0]
            assert!((light.direction[0] - 0.6).abs() < 0.001);
            assert!((light.direction[1] - 0.8).abs() < 0.001);
            assert!((light.direction[2] - 0.0).abs() < 0.001);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_spot_light_clamp_color() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.5,  // Over 1.0
                    y: 0.5,
                    z: -0.2, // Below 0.0
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
            ("inner_angle".to_string(), Value::F32Val(20.0)),
            ("outer_angle".to_string(), Value::F32Val(45.0)),
            ("radius".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::StringVal(json) = &result[0].1 {
            let light: SpotLightData = serde_json::from_str(json).unwrap();
            // Color should be clamped to [0.0, 1.0]
            assert_eq!(light.color, [1.0, 0.5, 0.0]);
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_spot_light_invalid_inner_angle() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
            ("inner_angle".to_string(), Value::F32Val(100.0)), // > 90
            ("outer_angle".to_string(), Value::F32Val(30.0)),
            ("radius".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("range [0, 90]"));
    }

    #[test]
    fn test_spot_light_inner_greater_than_outer() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
            ("inner_angle".to_string(), Value::F32Val(30.0)), // >= outer
            ("outer_angle".to_string(), Value::F32Val(15.0)),
            ("radius".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must be less than"));
    }

    #[test]
    fn test_spot_light_zero_direction() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
            ("inner_angle".to_string(), Value::F32Val(15.0)),
            ("outer_angle".to_string(), Value::F32Val(30.0)),
            ("radius".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("zero vector"));
    }

    #[test]
    fn test_spot_light_negative_intensity() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(-1.0)),
            ("inner_angle".to_string(), Value::F32Val(15.0)),
            ("outer_angle".to_string(), Value::F32Val(30.0)),
            ("radius".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("non-negative"));
    }

    #[test]
    fn test_spot_light_zero_radius() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(1.0)),
            ("inner_angle".to_string(), Value::F32Val(15.0)),
            ("outer_angle".to_string(), Value::F32Val(30.0)),
            ("radius".to_string(), Value::F32Val(0.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must be positive"));
    }

    #[test]
    fn test_spot_light_narrow_cone() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 2.0,
                    y: 3.0,
                    z: 4.0,
                }),
            ),
            (
                "direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.9,
                    y: 0.7,
                    z: 0.5,
                }),
            ),
            ("intensity".to_string(), Value::F32Val(3.0)),
            ("inner_angle".to_string(), Value::F32Val(5.0)),  // Narrow
            ("outer_angle".to_string(), Value::F32Val(10.0)),
            ("radius".to_string(), Value::F32Val(20.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::StringVal(json) = &result[0].1 {
            let light: SpotLightData = serde_json::from_str(json).unwrap();
            assert_eq!(light.inner_angle, 5.0);
            assert_eq!(light.outer_angle, 10.0);
            assert_eq!(light.intensity, 3.0);
        } else {
            panic!("Expected StringVal");
        }
    }
}
