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
            name: "Normal Map".to_string(),
            version: "1.0.0".to_string(),
            description: "Converts tangent-space normals to world-space using TBN matrix".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics/Textures".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "tangent_normal".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Normal from normal map in tangent space (typically from texture)".to_string(),
            },
            PortSpec {
                name: "normal".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Vertex normal in world space".to_string(),
            },
            PortSpec {
                name: "tangent".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Vertex tangent in world space".to_string(),
            },
            PortSpec {
                name: "bitangent".to_string(),
                data_type: DataType::Vec3Type,
                optional: true,
                description: "Vertex bitangent in world space (calculated from normal × tangent if not provided)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "world_normal".to_string(),
            data_type: DataType::Vec3Type,
            optional: false,
            description: "Normal in world space (ready for lighting calculations)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract tangent-space normal (from normal map texture)
        let tangent_normal = extract_vec3(&inputs, "tangent_normal")?;
        let mut tangent_normal_vec = Vec3::new(tangent_normal.x, tangent_normal.y, tangent_normal.z);

        // Normal maps typically store normals in [0,1] range, convert to [-1,1]
        // If already in [-1,1], this won't hurt much
        tangent_normal_vec = tangent_normal_vec * 2.0 - Vec3::ONE;

        // Normalize the tangent-space normal
        tangent_normal_vec = tangent_normal_vec.normalize_or_zero();
        if tangent_normal_vec == Vec3::ZERO {
            return Err(ExecutionError {
                message: "tangent_normal cannot be zero vector after normalization".to_string(),
                input_name: Some("tangent_normal".to_string()),
                recovery_hint: Some("Normal map texture may have invalid data".to_string()),
            });
        }

        // Extract world-space normal
        let normal = extract_vec3(&inputs, "normal")?;
        let normal_vec = Vec3::new(normal.x, normal.y, normal.z);
        let N = normal_vec.normalize_or_zero();
        if N == Vec3::ZERO {
            return Err(ExecutionError {
                message: "normal cannot be zero vector".to_string(),
                input_name: Some("normal".to_string()),
                recovery_hint: Some("Provide a non-zero normal vector".to_string()),
            });
        }

        // Extract world-space tangent
        let tangent = extract_vec3(&inputs, "tangent")?;
        let tangent_vec = Vec3::new(tangent.x, tangent.y, tangent.z);
        let T = tangent_vec.normalize_or_zero();
        if T == Vec3::ZERO {
            return Err(ExecutionError {
                message: "tangent cannot be zero vector".to_string(),
                input_name: Some("tangent".to_string()),
                recovery_hint: Some("Provide a non-zero tangent vector".to_string()),
            });
        }

        // Calculate or extract bitangent
        let B = if let Some(bitangent_input) = inputs.iter().find(|(name, _)| name == "bitangent") {
            // Use provided bitangent
            match &bitangent_input.1 {
                Value::Vec3Val(v) => {
                    let bitangent_vec = Vec3::new(v.x, v.y, v.z);
                    let b = bitangent_vec.normalize_or_zero();
                    if b == Vec3::ZERO {
                        return Err(ExecutionError {
                            message: "bitangent cannot be zero vector".to_string(),
                            input_name: Some("bitangent".to_string()),
                            recovery_hint: Some("Provide a non-zero bitangent vector or omit to calculate automatically".to_string()),
                        });
                    }
                    b
                }
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected vec3 for 'bitangent', got {:?}", bitangent_input.1),
                        input_name: Some("bitangent".to_string()),
                        recovery_hint: Some("Provide a vec3 value".to_string()),
                    });
                }
            }
        } else {
            // Calculate bitangent from N × T
            N.cross(T).normalize_or_zero()
        };

        if B == Vec3::ZERO {
            return Err(ExecutionError {
                message: "Calculated bitangent is zero (normal and tangent may be parallel)".to_string(),
                input_name: Some("tangent".to_string()),
                recovery_hint: Some("Ensure normal and tangent are not parallel".to_string()),
            });
        }

        // Construct TBN matrix and transform normal
        // TBN matrix: [T B N] columns
        // world_normal = T * tangent_normal.x + B * tangent_normal.y + N * tangent_normal.z
        let world_normal = T * tangent_normal_vec.x
            + B * tangent_normal_vec.y
            + N * tangent_normal_vec.z;

        let world_normal = world_normal.normalize_or_zero();

        Ok(vec![(
            "world_normal".to_string(),
            Value::Vec3Val(wasmflow::node::types::Vec3 {
                x: world_normal.x,
                y: world_normal.y,
                z: world_normal.z,
            }),
        )])
    }
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
    fn test_normal_map_basic() {
        // Test with identity normal (pointing up in tangent space)
        let inputs = vec![
            (
                "tangent_normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0, // Encoded as [0.5, 0.5, 1.0] in texture
                }),
            ),
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "tangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "world_normal");

        // Result should be close to (0, 1, 0) since tangent normal is mostly pointing up
        if let Value::Vec3Val(n) = &result[0].1 {
            assert!((n.y - 1.0).abs() < 0.2); // Should be pointing mostly up
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_normal_map_with_bitangent() {
        // Test with explicit bitangent
        let inputs = vec![
            (
                "tangent_normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0,
                }),
            ),
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "tangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "bitangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].0, "world_normal");
    }

    #[test]
    fn test_normal_map_angled_surface() {
        // Test with angled surface normal
        let inputs = vec![
            (
                "tangent_normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0,
                }),
            ),
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.707,
                    y: 0.707,
                    z: 0.0,
                }), // 45° angle
            ),
            (
                "tangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.707,
                    y: -0.707,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(n) = &result[0].1 {
            // Normal should be transformed to world space
            let length = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
            assert!((length - 1.0).abs() < 0.01); // Should be normalized
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_normal_map_flat_texture() {
        // Test with flat normal map (0.5, 0.5, 1.0 -> 0, 0, 1 in tangent space)
        let inputs = vec![
            (
                "tangent_normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0, // Flat normal map
                }),
            ),
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "tangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(n) = &result[0].1 {
            // Should be very close to original normal (0, 1, 0)
            assert!((n.x - 0.0).abs() < 0.1);
            assert!((n.y - 1.0).abs() < 0.1);
            assert!((n.z - 0.0).abs() < 0.1);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_normal_map_zero_tangent_normal() {
        // Test error handling for zero tangent normal after conversion
        // [0.5, 0.5, 0.5] -> [0, 0, 0] after * 2.0 - 1.0
        let inputs = vec![
            (
                "tangent_normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                }),
            ),
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "tangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_normal_map_zero_normal() {
        // Test error handling for zero normal
        let inputs = vec![
            (
                "tangent_normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0,
                }),
            ),
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "tangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_normal_map_zero_tangent() {
        // Test error handling for zero tangent
        let inputs = vec![
            (
                "tangent_normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 1.0,
                }),
            ),
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "tangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_normal_map_perpendicular_bump() {
        // Test with normal pointing to the side (simulating a bump)
        let inputs = vec![
            (
                "tangent_normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0, // Pointing right in tangent space
                    y: 0.5,
                    z: 0.5,
                }),
            ),
            (
                "normal".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            ),
            (
                "tangent".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(n) = &result[0].1 {
            // Should have significant X component
            assert!(n.x.abs() > 0.3);

            // Should still be normalized
            let length = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
            assert!((length - 1.0).abs() < 0.01);
        } else {
            panic!("Expected Vec3Val");
        }
    }
}
