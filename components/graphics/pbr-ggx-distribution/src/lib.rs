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
use std::f32::consts::PI;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "PBR GGX Distribution".to_string(),
            version: "1.0.0".to_string(),
            description: "GGX/Trowbridge-Reitz normal distribution function for PBR lighting".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics/PBR".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "normal".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Surface normal (normalized)".to_string(),
            },
            PortSpec {
                name: "half_vector".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Half vector between view and light (normalized)".to_string(),
            },
            PortSpec {
                name: "roughness".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Surface roughness (0=smooth, 1=rough)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "distribution".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "GGX distribution value (microfacet density)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract normal
        let normal = extract_vec3(&inputs, "normal")?;
        let normal_vec = Vec3::new(normal.x, normal.y, normal.z);
        let normal_normalized = normal_vec.normalize_or_zero();
        if normal_normalized == Vec3::ZERO {
            return Err(ExecutionError {
                message: "normal cannot be zero vector".to_string(),
                input_name: Some("normal".to_string()),
                recovery_hint: Some("Provide a non-zero normal vector".to_string()),
            });
        }

        // Extract half vector
        let half_vector = extract_vec3(&inputs, "half_vector")?;
        let half_vec = Vec3::new(half_vector.x, half_vector.y, half_vector.z);
        let half_normalized = half_vec.normalize_or_zero();
        if half_normalized == Vec3::ZERO {
            return Err(ExecutionError {
                message: "half_vector cannot be zero vector".to_string(),
                input_name: Some("half_vector".to_string()),
                recovery_hint: Some("Provide a non-zero half vector".to_string()),
            });
        }

        // Extract roughness
        let roughness = extract_f32(&inputs, "roughness")?;
        if roughness < 0.0 || roughness > 1.0 {
            return Err(ExecutionError {
                message: format!("Roughness must be in range [0, 1], got {}", roughness),
                input_name: Some("roughness".to_string()),
                recovery_hint: Some("Use 0 for smooth surfaces, 1 for very rough surfaces".to_string()),
            });
        }

        // Calculate GGX distribution
        let distribution = ggx_distribution(normal_normalized, half_normalized, roughness);

        Ok(vec![(
            "distribution".to_string(),
            Value::F32Val(distribution),
        )])
    }
}

/// GGX/Trowbridge-Reitz normal distribution function
///
/// Formula: D(h) = α² / (π * ((n · h)² * (α² - 1) + 1)²)
///
/// Where:
/// - α = roughness²
/// - n = surface normal (normalized)
/// - h = half vector (normalized)
fn ggx_distribution(normal: Vec3, half: Vec3, roughness: f32) -> f32 {
    let a = roughness * roughness; // α = roughness²
    let a2 = a * a; // α²

    let n_dot_h = normal.dot(half).max(0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;

    let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    let denom2 = denom * denom;

    // D = α² / (π * denom²)
    a2 / (PI * denom2)
}

// Helper functions
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

fn extract_f32(inputs: &[(String, Value)], name: &str) -> Result<f32, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.1 {
        Value::F32Val(v) => Ok(*v),
        _ => Err(ExecutionError {
            message: format!("Expected f32 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide an f32 value".to_string()),
        }),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ggx_smooth_surface() {
        // Smooth surface (roughness=0) should have very high peak at perfect alignment
        let inputs = vec![
            (
                "normal".to_string(),
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
            ("roughness".to_string(), Value::F32Val(0.01)), // Near-zero roughness
        ];

        let result = Component::execute(inputs).unwrap();
        let distribution = if let Value::F32Val(d) = result[0].1 {
            d
        } else {
            panic!("Expected F32Val");
        };

        // Very smooth surface should have high peak
        assert!(distribution > 1000.0);
    }

    #[test]
    fn test_ggx_rough_surface() {
        // Rough surface should have broader distribution
        let inputs = vec![
            (
                "normal".to_string(),
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
            ("roughness".to_string(), Value::F32Val(0.9)),
        ];

        let result = Component::execute(inputs).unwrap();
        let distribution = if let Value::F32Val(d) = result[0].1 {
            d
        } else {
            panic!("Expected F32Val");
        };

        // Rough surface should have lower, broader peak
        assert!(distribution < 1.0);
        assert!(distribution > 0.0);
    }

    #[test]
    fn test_ggx_grazing_angle() {
        // At grazing angle, distribution should be lower
        let inputs = vec![
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "half_vector".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.707,
                    y: 0.707,
                    z: 0.0,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();
        let distribution = if let Value::F32Val(d) = result[0].1 {
            d
        } else {
            panic!("Expected F32Val");
        };

        // At angle, distribution should be lower than at alignment
        assert!(distribution > 0.0);
        assert!(distribution < 10.0);
    }

    #[test]
    fn test_ggx_invalid_roughness() {
        let inputs = vec![
            (
                "normal".to_string(),
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
            ("roughness".to_string(), Value::F32Val(1.5)), // Invalid
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_ggx_medium_roughness() {
        // Test with medium roughness
        let inputs = vec![
            (
                "normal".to_string(),
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
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();
        let distribution = if let Value::F32Val(d) = result[0].1 {
            d
        } else {
            panic!("Expected F32Val");
        };

        // Medium roughness should give moderate peak
        assert!(distribution > 0.1);
        assert!(distribution < 100.0);
    }

    #[test]
    fn test_ggx_perpendicular() {
        // When half vector is perpendicular to normal, distribution should be ~0
        let inputs = vec![
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "half_vector".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();
        let distribution = if let Value::F32Val(d) = result[0].1 {
            d
        } else {
            panic!("Expected F32Val");
        };

        // Perpendicular should give very low distribution
        assert!(distribution < 0.1);
    }
}
