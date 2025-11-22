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
            name: "PBR Material".to_string(),
            version: "1.0.0".to_string(),
            description: "PBR material properties with automatic F0 calculation from metallic/roughness workflow".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics/PBR".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "base_color".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Base color (albedo) in linear RGB space".to_string(),
            },
            PortSpec {
                name: "metallic".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Metallic value (0=dielectric, 1=metal)".to_string(),
            },
            PortSpec {
                name: "roughness".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Surface roughness (0=smooth, 1=rough)".to_string(),
            },
            PortSpec {
                name: "ao".to_string(),
                data_type: DataType::F32Type,
                optional: true,
                description: "Ambient occlusion (0=fully occluded, 1=no occlusion)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "f0".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Base reflectivity at normal incidence (calculated from metallic workflow)".to_string(),
            },
            PortSpec {
                name: "roughness".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Surface roughness (passed through)".to_string(),
            },
            PortSpec {
                name: "ao".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Ambient occlusion (passed through)".to_string(),
            },
            PortSpec {
                name: "base_color".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Base color (passed through for diffuse calculations)".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract base color
        let base_color = extract_vec3(&inputs, "base_color")?;
        let base_color_vec = Vec3::new(base_color.x, base_color.y, base_color.z);

        // Validate base color is in valid range [0, 1]
        if base_color_vec.x < 0.0 || base_color_vec.x > 1.0
            || base_color_vec.y < 0.0 || base_color_vec.y > 1.0
            || base_color_vec.z < 0.0 || base_color_vec.z > 1.0
        {
            return Err(ExecutionError {
                message: format!(
                    "Base color values must be in range [0, 1], got ({}, {}, {})",
                    base_color_vec.x, base_color_vec.y, base_color_vec.z
                ),
                input_name: Some("base_color".to_string()),
                recovery_hint: Some("Use linear RGB values in range [0, 1]".to_string()),
            });
        }

        // Extract metallic
        let metallic = extract_f32(&inputs, "metallic")?;
        if metallic < 0.0 || metallic > 1.0 {
            return Err(ExecutionError {
                message: format!("Metallic must be in range [0, 1], got {}", metallic),
                input_name: Some("metallic".to_string()),
                recovery_hint: Some("Use 0 for dielectrics, 1 for metals".to_string()),
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

        // Extract AO (optional, default to 1.0 = no occlusion)
        let ao = if let Some(input) = inputs.iter().find(|(name, _)| name == "ao") {
            match &input.1 {
                Value::F32Val(v) => {
                    if *v < 0.0 || *v > 1.0 {
                        return Err(ExecutionError {
                            message: format!("AO must be in range [0, 1], got {}", v),
                            input_name: Some("ao".to_string()),
                            recovery_hint: Some(
                                "Use 0 for fully occluded, 1 for no occlusion".to_string(),
                            ),
                        });
                    }
                    *v
                }
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected f32 for 'ao', got {:?}", input.1),
                        input_name: Some("ao".to_string()),
                        recovery_hint: Some("Provide an f32 value".to_string()),
                    });
                }
            }
        } else {
            1.0 // Default: no occlusion
        };

        // Calculate F0 from metallic workflow
        // Dielectrics (metallic=0): F0 = 0.04
        // Metals (metallic=1): F0 = base_color
        // Mixed: F0 = lerp(0.04, base_color, metallic)
        let f0 = calculate_f0(base_color_vec, metallic);

        Ok(vec![
            (
                "f0".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: f0.x,
                    y: f0.y,
                    z: f0.z,
                }),
            ),
            ("roughness".to_string(), Value::F32Val(roughness)),
            ("ao".to_string(), Value::F32Val(ao)),
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: base_color_vec.x,
                    y: base_color_vec.y,
                    z: base_color_vec.z,
                }),
            ),
        ])
    }
}

/// Calculate F0 (base reflectivity) from metallic/roughness workflow
///
/// Formula: F0 = lerp(0.04, base_color, metallic)
///
/// Where:
/// - 0.04 is the typical F0 for dielectrics (4% reflection)
/// - base_color is used as F0 for metals
/// - metallic blends between the two
fn calculate_f0(base_color: Vec3, metallic: f32) -> Vec3 {
    const DIELECTRIC_F0: f32 = 0.04;
    let dielectric = Vec3::splat(DIELECTRIC_F0);

    // Linear interpolation: lerp(a, b, t) = a + (b - a) * t
    dielectric + (base_color - dielectric) * metallic
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
    fn test_dielectric_material() {
        // Pure dielectric (metallic=0) should have F0 = 0.04
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.8,
                    y: 0.2,
                    z: 0.1,
                }),
            ),
            ("metallic".to_string(), Value::F32Val(0.0)),
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs).unwrap();

        // Find F0 output
        let f0 = result
            .iter()
            .find(|(name, _)| name == "f0")
            .and_then(|(_, v)| match v {
                Value::Vec3Val(vec) => Some(vec),
                _ => None,
            })
            .unwrap();

        // Dielectric F0 should be 0.04 for all channels
        assert!((f0.x - 0.04).abs() < 0.001);
        assert!((f0.y - 0.04).abs() < 0.001);
        assert!((f0.z - 0.04).abs() < 0.001);

        // AO should default to 1.0
        let ao = result
            .iter()
            .find(|(name, _)| name == "ao")
            .and_then(|(_, v)| match v {
                Value::F32Val(a) => Some(*a),
                _ => None,
            })
            .unwrap();
        assert_eq!(ao, 1.0);
    }

    #[test]
    fn test_metallic_material() {
        // Pure metal (metallic=1) should have F0 = base_color
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.71,
                    z: 0.29,
                }), // Gold-like
            ),
            ("metallic".to_string(), Value::F32Val(1.0)),
            ("roughness".to_string(), Value::F32Val(0.3)),
        ];

        let result = Component::execute(inputs).unwrap();

        let f0 = result
            .iter()
            .find(|(name, _)| name == "f0")
            .and_then(|(_, v)| match v {
                Value::Vec3Val(vec) => Some(vec),
                _ => None,
            })
            .unwrap();

        // Metal F0 should equal base_color
        assert!((f0.x - 1.0).abs() < 0.001);
        assert!((f0.y - 0.71).abs() < 0.001);
        assert!((f0.z - 0.29).abs() < 0.001);
    }

    #[test]
    fn test_mixed_material() {
        // Mixed material (metallic=0.5) should interpolate
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.8,
                    y: 0.8,
                    z: 0.8,
                }),
            ),
            ("metallic".to_string(), Value::F32Val(0.5)),
            ("roughness".to_string(), Value::F32Val(0.5)),
            ("ao".to_string(), Value::F32Val(0.9)),
        ];

        let result = Component::execute(inputs).unwrap();

        let f0 = result
            .iter()
            .find(|(name, _)| name == "f0")
            .and_then(|(_, v)| match v {
                Value::Vec3Val(vec) => Some(vec),
                _ => None,
            })
            .unwrap();

        // F0 should be lerp(0.04, 0.8, 0.5) = 0.04 + (0.8 - 0.04) * 0.5 = 0.42
        let expected = 0.04 + (0.8 - 0.04) * 0.5;
        assert!((f0.x - expected).abs() < 0.001);
        assert!((f0.y - expected).abs() < 0.001);
        assert!((f0.z - expected).abs() < 0.001);

        // Check AO was passed through
        let ao = result
            .iter()
            .find(|(name, _)| name == "ao")
            .and_then(|(_, v)| match v {
                Value::F32Val(a) => Some(*a),
                _ => None,
            })
            .unwrap();
        assert_eq!(ao, 0.9);
    }

    #[test]
    fn test_roughness_passthrough() {
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                }),
            ),
            ("metallic".to_string(), Value::F32Val(0.0)),
            ("roughness".to_string(), Value::F32Val(0.7)),
        ];

        let result = Component::execute(inputs).unwrap();

        let roughness = result
            .iter()
            .find(|(name, _)| name == "roughness")
            .and_then(|(_, v)| match v {
                Value::F32Val(r) => Some(*r),
                _ => None,
            })
            .unwrap();

        assert_eq!(roughness, 0.7);
    }

    #[test]
    fn test_invalid_metallic() {
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                }),
            ),
            ("metallic".to_string(), Value::F32Val(1.5)), // Invalid
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_roughness() {
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                }),
            ),
            ("metallic".to_string(), Value::F32Val(0.5)),
            ("roughness".to_string(), Value::F32Val(-0.1)), // Invalid
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_base_color() {
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.5, // Invalid
                    y: 0.5,
                    z: 0.5,
                }),
            ),
            ("metallic".to_string(), Value::F32Val(0.5)),
            ("roughness".to_string(), Value::F32Val(0.5)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ao() {
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                }),
            ),
            ("metallic".to_string(), Value::F32Val(0.5)),
            ("roughness".to_string(), Value::F32Val(0.5)),
            ("ao".to_string(), Value::F32Val(1.5)), // Invalid
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_colored_metal() {
        // Colored metal like copper
        let inputs = vec![
            (
                "base_color".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.95,
                    y: 0.64,
                    z: 0.54,
                }), // Copper
            ),
            ("metallic".to_string(), Value::F32Val(1.0)),
            ("roughness".to_string(), Value::F32Val(0.2)),
        ];

        let result = Component::execute(inputs).unwrap();

        let f0 = result
            .iter()
            .find(|(name, _)| name == "f0")
            .and_then(|(_, v)| match v {
                Value::Vec3Val(vec) => Some(vec),
                _ => None,
            })
            .unwrap();

        // For pure metal, F0 should preserve the color
        assert!((f0.x - 0.95).abs() < 0.001);
        assert!((f0.y - 0.64).abs() < 0.001);
        assert!((f0.z - 0.54).abs() < 0.001);
    }
}
