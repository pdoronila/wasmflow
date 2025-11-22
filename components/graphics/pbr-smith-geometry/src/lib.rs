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
            name: "PBR Smith Geometry".to_string(),
            version: "1.0.0".to_string(),
            description: "Smith geometry/visibility term with GGX distribution for PBR lighting".to_string(),
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
                name: "view_dir".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "View direction (normalized)".to_string(),
            },
            PortSpec {
                name: "light_dir".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Light direction (normalized)".to_string(),
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
            name: "geometry".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Geometry/visibility term (shadowing-masking factor)".to_string(),
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

        // Extract view direction
        let view_dir = extract_vec3(&inputs, "view_dir")?;
        let view_vec = Vec3::new(view_dir.x, view_dir.y, view_dir.z);
        let view_normalized = view_vec.normalize_or_zero();
        if view_normalized == Vec3::ZERO {
            return Err(ExecutionError {
                message: "view_dir cannot be zero vector".to_string(),
                input_name: Some("view_dir".to_string()),
                recovery_hint: Some("Provide a non-zero view direction".to_string()),
            });
        }

        // Extract light direction
        let light_dir = extract_vec3(&inputs, "light_dir")?;
        let light_vec = Vec3::new(light_dir.x, light_dir.y, light_dir.z);
        let light_normalized = light_vec.normalize_or_zero();
        if light_normalized == Vec3::ZERO {
            return Err(ExecutionError {
                message: "light_dir cannot be zero vector".to_string(),
                input_name: Some("light_dir".to_string()),
                recovery_hint: Some("Provide a non-zero light direction".to_string()),
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

        // Calculate Smith geometry term
        let geometry = smith_geometry_ggx(
            normal_normalized,
            view_normalized,
            light_normalized,
            roughness,
        );

        Ok(vec![("geometry".to_string(), Value::F32Val(geometry))])
    }
}

/// Smith geometry/visibility term using GGX distribution
///
/// Formula: G(v, l, α) = G1(v) * G1(l)
///
/// Where G1 is:
/// G1(v) = (2 * (n · v)) / ((n · v) + sqrt(α² + (1 - α²) * (n · v)²))
///
/// Parameters:
/// - n = surface normal (normalized)
/// - v = view direction (normalized)
/// - l = light direction (normalized)
/// - roughness = surface roughness [0, 1]
fn smith_geometry_ggx(normal: Vec3, view: Vec3, light: Vec3, roughness: f32) -> f32 {
    let a = roughness * roughness; // α = roughness²
    let a2 = a * a; // α²

    // G1 for view direction
    let g1_view = smith_g1_ggx(normal, view, a2);

    // G1 for light direction
    let g1_light = smith_g1_ggx(normal, light, a2);

    // G = G1(v) * G1(l)
    g1_view * g1_light
}

/// Single direction term for Smith-GGX geometry
///
/// Formula: G1(v) = (2 * (n · v)) / ((n · v) + sqrt(α² + (1 - α²) * (n · v)²))
fn smith_g1_ggx(normal: Vec3, direction: Vec3, a2: f32) -> f32 {
    let n_dot_v = normal.dot(direction).max(0.0);

    // Handle edge case where direction is perpendicular to normal
    if n_dot_v <= 0.0 {
        return 0.0;
    }

    let n_dot_v2 = n_dot_v * n_dot_v;

    // Calculate denominator: (n · v) + sqrt(α² + (1 - α²) * (n · v)²)
    let denom = n_dot_v + (a2 + (1.0 - a2) * n_dot_v2).sqrt();

    // G1 = (2 * (n · v)) / denom
    (2.0 * n_dot_v) / denom
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
    fn test_smith_smooth_aligned() {
        // Smooth surface with aligned view and light should have high geometry term
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
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "light_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(0.1)), // Smooth
        ];

        let result = Component::execute(inputs).unwrap();
        let geometry = if let Value::F32Val(g) = result[0].1 {
            g
        } else {
            panic!("Expected F32Val");
        };

        // Smooth surface with perfect alignment should have geometry close to 1.0
        assert!(geometry > 0.9);
        assert!(geometry <= 1.0);
    }

    #[test]
    fn test_smith_rough_surface() {
        // Rough surface should have lower geometry term than smooth surface
        // But with perfect alignment, it can still be high
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
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "light_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(0.9)), // Rough
        ];

        let result = Component::execute(inputs).unwrap();
        let geometry = if let Value::F32Val(g) = result[0].1 {
            g
        } else {
            panic!("Expected F32Val");
        };

        // Rough surface with perfect alignment can still have high geometry
        assert!(geometry > 0.0);
        assert!(geometry <= 1.0);
    }

    #[test]
    fn test_smith_grazing_view() {
        // Grazing view angle should reduce geometry term compared to aligned case
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
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.866, // 30° from horizontal (60° from normal)
                    y: 0.5,
                    z: 0.0,
                }),
            ),
            (
                "light_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();
        let geometry = if let Value::F32Val(g) = result[0].1 {
            g
        } else {
            panic!("Expected F32Val");
        };

        // Grazing angle should reduce geometry term, but not drastically at 60°
        assert!(geometry > 0.0);
        assert!(geometry < 1.0);
    }

    #[test]
    fn test_smith_grazing_light() {
        // Grazing light angle should reduce geometry term compared to aligned case
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
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "light_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.866, // 30° from horizontal (60° from normal)
                    y: 0.5,
                    z: 0.0,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();
        let geometry = if let Value::F32Val(g) = result[0].1 {
            g
        } else {
            panic!("Expected F32Val");
        };

        // Grazing angle should reduce geometry term, but not drastically at 60°
        assert!(geometry > 0.0);
        assert!(geometry < 1.0);
    }

    #[test]
    fn test_smith_invalid_roughness() {
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
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "light_dir".to_string(),
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
    fn test_smith_perpendicular_view() {
        // View perpendicular to normal should give zero geometry
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
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "light_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();
        let geometry = if let Value::F32Val(g) = result[0].1 {
            g
        } else {
            panic!("Expected F32Val");
        };

        // Perpendicular view should give zero or near-zero geometry
        assert!(geometry < 0.01);
    }

    #[test]
    fn test_smith_medium_roughness() {
        // Test with medium roughness and moderate angles
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
                "view_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.866, // ~30° from normal
                    z: 0.5,
                }),
            ),
            (
                "light_dir".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.866, // ~30° from normal
                    z: 0.0,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();
        let geometry = if let Value::F32Val(g) = result[0].1 {
            g
        } else {
            panic!("Expected F32Val");
        };

        // Medium roughness with moderate angles should give reasonable geometry
        assert!(geometry > 0.1);
        assert!(geometry < 1.0);
    }
}
