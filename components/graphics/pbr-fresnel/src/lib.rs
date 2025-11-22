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

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "PBR Fresnel (Schlick)".to_string(),
            version: "1.0.0".to_string(),
            description: "Fresnel-Schlick approximation for PBR lighting (calculates reflection at different angles)".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics/PBR".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "f0".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Base reflectivity at normal incidence (0° angle)".to_string(),
            },
            PortSpec {
                name: "view_dir".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "View direction (normalized)".to_string(),
            },
            PortSpec {
                name: "half_vector".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Half vector between view and light (normalized)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "fresnel".to_string(),
            data_type: DataType::Vec3Type,
            optional: false,
            description: "Fresnel term (reflectance at current angle)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract F0 (base reflectivity)
        let f0 = extract_vec3(&inputs, "f0")?;
        let f0_vec = Vec3::new(f0.x, f0.y, f0.z);

        // Validate F0 is in valid range [0, 1]
        if f0_vec.x < 0.0 || f0_vec.x > 1.0 ||
           f0_vec.y < 0.0 || f0_vec.y > 1.0 ||
           f0_vec.z < 0.0 || f0_vec.z > 1.0 {
            return Err(ExecutionError {
                message: format!(
                    "F0 values must be in range [0, 1], got ({}, {}, {})",
                    f0_vec.x, f0_vec.y, f0_vec.z
                ),
                input_name: Some("f0".to_string()),
                recovery_hint: Some("F0 represents base reflectivity - use values between 0 and 1".to_string()),
            });
        }

        // Extract view direction
        let view_dir = extract_vec3(&inputs, "view_dir")?;
        let view_vec = Vec3::new(view_dir.x, view_dir.y, view_dir.z);

        // Normalize if needed
        let view_normalized = view_vec.normalize_or_zero();
        if view_normalized == Vec3::ZERO {
            return Err(ExecutionError {
                message: "view_dir cannot be zero vector".to_string(),
                input_name: Some("view_dir".to_string()),
                recovery_hint: Some("Provide a non-zero view direction".to_string()),
            });
        }

        // Extract half vector
        let half_vector = extract_vec3(&inputs, "half_vector")?;
        let half_vec = Vec3::new(half_vector.x, half_vector.y, half_vector.z);

        // Normalize if needed
        let half_normalized = half_vec.normalize_or_zero();
        if half_normalized == Vec3::ZERO {
            return Err(ExecutionError {
                message: "half_vector cannot be zero vector".to_string(),
                input_name: Some("half_vector".to_string()),
                recovery_hint: Some("Provide a non-zero half vector".to_string()),
            });
        }

        // Calculate Fresnel-Schlick: F = F0 + (1 - F0) * (1 - (v · h))^5
        let fresnel = fresnel_schlick(f0_vec, view_normalized, half_normalized);

        Ok(vec![(
            "fresnel".to_string(),
            Value::Vec3Val(wasmflow::node::types::Vec3 {
                x: fresnel.x,
                y: fresnel.y,
                z: fresnel.z,
            }),
        )])
    }
}

/// Fresnel-Schlick approximation
///
/// Formula: F = F0 + (1 - F0) * (1 - (v · h))^5
///
/// Where:
/// - F0 is the base reflectivity at normal incidence
/// - v is the view direction (normalized)
/// - h is the half vector (normalized)
fn fresnel_schlick(f0: Vec3, view: Vec3, half: Vec3) -> Vec3 {
    let cos_theta = view.dot(half).max(0.0);
    let one_minus_cos = 1.0 - cos_theta;
    let power5 = one_minus_cos.powi(5);

    // F = F0 + (1 - F0) * (1 - cos(θ))^5
    f0 + (Vec3::ONE - f0) * power5
}

// Helper function
fn extract_vec3(
    inputs: &[(String, Value)],
    name: &str,
) -> Result<wasmflow::node::types::Vec3, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a vec3 value to this input".to_string()),
        })?;

    match &input.1 {
        Value::Vec3Val(v) => Ok(v.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected vec3 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a vec3 value".to_string()),
        }),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresnel_at_normal_incidence() {
        // At normal incidence (view aligned with normal), Fresnel should be close to F0
        let inputs = vec![
            (
                "f0".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.04,
                    y: 0.04,
                    z: 0.04,
                }),
            ),
            (
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "half_vector".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        let fresnel = if let Value::Vec3Val(f) = &result[0].1 {
            f
        } else {
            panic!("Expected Vec3Val");
        };

        // At 0° angle, fresnel ≈ F0 = 0.04
        assert!((fresnel.x - 0.04).abs() < 0.001);
        assert!((fresnel.y - 0.04).abs() < 0.001);
        assert!((fresnel.z - 0.04).abs() < 0.001);
    }

    #[test]
    fn test_fresnel_at_grazing_angle() {
        // At grazing angle (90°), Fresnel should approach 1.0
        let inputs = vec![
            (
                "f0".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.04,
                    y: 0.04,
                    z: 0.04,
                }),
            ),
            (
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "half_vector".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        let fresnel = if let Value::Vec3Val(f) = &result[0].1 {
            f
        } else {
            panic!("Expected Vec3Val");
        };

        // At 90° angle, fresnel should be close to 1.0
        assert!(fresnel.x > 0.9);
        assert!(fresnel.y > 0.9);
        assert!(fresnel.z > 0.9);
    }

    #[test]
    fn test_fresnel_metallic_surface() {
        // Metals have higher F0 values
        let inputs = vec![
            (
                "f0".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.71,
                    z: 0.29,
                }), // Gold-like F0
            ),
            (
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.707,
                    z: 0.707,
                }),
            ),
            (
                "half_vector".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        let fresnel = if let Value::Vec3Val(f) = &result[0].1 {
            f
        } else {
            panic!("Expected Vec3Val");
        };

        // Metallic surfaces should maintain their color
        assert!(fresnel.x > 0.5); // Red channel (high for gold)
        assert!(fresnel.y > 0.3); // Green channel
        assert!(fresnel.z > 0.1); // Blue channel (low for gold)
    }

    #[test]
    fn test_fresnel_invalid_f0() {
        // F0 values outside [0, 1] should error
        let inputs = vec![
            (
                "f0".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.5,
                    y: 0.04,
                    z: 0.04,
                }),
            ),
            (
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "half_vector".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_fresnel_intermediate_angle() {
        // Test at 45° angle
        let inputs = vec![
            (
                "f0".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.04,
                    y: 0.04,
                    z: 0.04,
                }),
            ),
            (
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.707,
                    z: 0.707,
                }),
            ),
            (
                "half_vector".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        let fresnel = if let Value::Vec3Val(f) = &result[0].1 {
            f
        } else {
            panic!("Expected Vec3Val");
        };

        // At 45°, fresnel should be between F0 and 1.0
        assert!(fresnel.x > 0.04);
        assert!(fresnel.x < 1.0);
    }
}
