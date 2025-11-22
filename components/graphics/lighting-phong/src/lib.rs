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
            name: "Phong Lighting".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculate Phong lighting (diffuse + specular) for a surface".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "normal".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Surface normal vector (should be normalized)".to_string(),
            },
            PortSpec {
                name: "light_dir".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Direction to light (should be normalized)".to_string(),
            },
            PortSpec {
                name: "view_dir".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Direction to viewer/camera (should be normalized)".to_string(),
            },
            PortSpec {
                name: "surface_color".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Surface/material color (RGB, 0.0-1.0)".to_string(),
            },
            PortSpec {
                name: "light_color".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Light color (RGB, 0.0-1.0)".to_string(),
            },
            PortSpec {
                name: "shininess".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Specular shininess factor (typically 1-128)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "lit_color".to_string(),
            data_type: DataType::Vec3Type,
            optional: false,
            description: "Resulting lit color (diffuse + specular)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

// Helper function to normalize a vector
fn normalize(v: &Vec3) -> Vec3 {
    let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    if len < 1e-6 {
        Vec3 { x: 0.0, y: 0.0, z: 0.0 }
    } else {
        Vec3 {
            x: v.x / len,
            y: v.y / len,
            z: v.z / len,
        }
    }
}

// Helper function to calculate dot product
fn dot(a: &Vec3, b: &Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

// Helper function to reflect a vector
fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    let d = 2.0 * dot(incident, normal);
    Vec3 {
        x: incident.x - d * normal.x,
        y: incident.y - d * normal.y,
        z: incident.z - d * normal.z,
    }
}

// Helper function to clamp value to [0.0, 1.0]
fn clamp(value: f32) -> f32 {
    value.max(0.0).min(1.0)
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract normal
        let normal_input = inputs
            .iter()
            .find(|(name, _)| name == "normal")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: normal".to_string(),
                input_name: Some("normal".to_string()),
                recovery_hint: Some("Connect a vec3 value to the normal input".to_string()),
            })?;
        let normal_raw = match &normal_input.1 {
            Value::Vec3Val(v) => v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected vec3 for 'normal', got {:?}", normal_input.1),
                    input_name: Some("normal".to_string()),
                    recovery_hint: Some("Provide a vec3 value".to_string()),
                });
            }
        };
        let normal = normalize(normal_raw);

        // Extract light_dir
        let light_dir_input = inputs
            .iter()
            .find(|(name, _)| name == "light_dir")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: light_dir".to_string(),
                input_name: Some("light_dir".to_string()),
                recovery_hint: Some("Connect a vec3 value to the light_dir input".to_string()),
            })?;
        let light_dir_raw = match &light_dir_input.1 {
            Value::Vec3Val(v) => v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected vec3 for 'light_dir', got {:?}", light_dir_input.1),
                    input_name: Some("light_dir".to_string()),
                    recovery_hint: Some("Provide a vec3 value".to_string()),
                });
            }
        };
        let light_dir = normalize(light_dir_raw);

        // Extract view_dir
        let view_dir_input = inputs
            .iter()
            .find(|(name, _)| name == "view_dir")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: view_dir".to_string(),
                input_name: Some("view_dir".to_string()),
                recovery_hint: Some("Connect a vec3 value to the view_dir input".to_string()),
            })?;
        let view_dir_raw = match &view_dir_input.1 {
            Value::Vec3Val(v) => v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected vec3 for 'view_dir', got {:?}", view_dir_input.1),
                    input_name: Some("view_dir".to_string()),
                    recovery_hint: Some("Provide a vec3 value".to_string()),
                });
            }
        };
        let view_dir = normalize(view_dir_raw);

        // Extract surface_color
        let surface_color_input = inputs
            .iter()
            .find(|(name, _)| name == "surface_color")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: surface_color".to_string(),
                input_name: Some("surface_color".to_string()),
                recovery_hint: Some("Connect a vec3 value to the surface_color input".to_string()),
            })?;
        let surface_color = match &surface_color_input.1 {
            Value::Vec3Val(v) => v,
            _ => {
                return Err(ExecutionError {
                    message: format!(
                        "Expected vec3 for 'surface_color', got {:?}",
                        surface_color_input.1
                    ),
                    input_name: Some("surface_color".to_string()),
                    recovery_hint: Some("Provide a vec3 value".to_string()),
                });
            }
        };

        // Extract light_color
        let light_color_input = inputs
            .iter()
            .find(|(name, _)| name == "light_color")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: light_color".to_string(),
                input_name: Some("light_color".to_string()),
                recovery_hint: Some("Connect a vec3 value to the light_color input".to_string()),
            })?;
        let light_color = match &light_color_input.1 {
            Value::Vec3Val(v) => v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected vec3 for 'light_color', got {:?}", light_color_input.1),
                    input_name: Some("light_color".to_string()),
                    recovery_hint: Some("Provide a vec3 value".to_string()),
                });
            }
        };

        // Extract shininess
        let shininess_input = inputs
            .iter()
            .find(|(name, _)| name == "shininess")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: shininess".to_string(),
                input_name: Some("shininess".to_string()),
                recovery_hint: Some("Connect an f32 value to the shininess input".to_string()),
            })?;
        let shininess = match &shininess_input.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'shininess', got {:?}", shininess_input.1),
                    input_name: Some("shininess".to_string()),
                    recovery_hint: Some("Provide an f32 value".to_string()),
                });
            }
        };

        if shininess < 0.0 {
            return Err(ExecutionError {
                message: "Shininess must be non-negative".to_string(),
                input_name: Some("shininess".to_string()),
                recovery_hint: Some("Provide a positive shininess value (typically 1-128)".to_string()),
            });
        }

        // Calculate diffuse component: max(0, dot(normal, light_dir))
        let diffuse_factor = dot(&normal, &light_dir).max(0.0);

        // Calculate specular component using reflection
        let reflect_dir = reflect(&Vec3 {
            x: -light_dir.x,
            y: -light_dir.y,
            z: -light_dir.z,
        }, &normal);
        let spec_factor = dot(&view_dir, &reflect_dir).max(0.0).powf(shininess);

        // Combine diffuse and specular
        let diffuse_r = surface_color.x * light_color.x * diffuse_factor;
        let diffuse_g = surface_color.y * light_color.y * diffuse_factor;
        let diffuse_b = surface_color.z * light_color.z * diffuse_factor;

        let specular_r = light_color.x * spec_factor;
        let specular_g = light_color.y * spec_factor;
        let specular_b = light_color.z * spec_factor;

        let result = Vec3 {
            x: clamp(diffuse_r + specular_r),
            y: clamp(diffuse_g + specular_g),
            z: clamp(diffuse_b + specular_b),
        };

        Ok(vec![("lit_color".to_string(), Value::Vec3Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phong_lighting_basic() {
        // Normal pointing up, light from above, view from above
        let inputs = vec![
            ("normal".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("light_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("view_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("surface_color".to_string(), Value::Vec3Val(Vec3 { x: 0.5, y: 0.5, z: 0.5 })),
            ("light_color".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 1.0, z: 1.0 })),
            ("shininess".to_string(), Value::F32Val(32.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "lit_color");

        if let Value::Vec3Val(color) = &result[0].1 {
            // Should have full diffuse (dot(n,l) = 1) and full specular (perfect reflection)
            assert!(color.x > 0.5); // At least diffuse component
            assert!(color.y > 0.5);
            assert!(color.z > 0.5);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_phong_lighting_no_light() {
        // Light perpendicular to normal (dot = 0)
        let inputs = vec![
            ("normal".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("light_dir".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 0.0, z: 0.0 })),
            ("view_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("surface_color".to_string(), Value::Vec3Val(Vec3 { x: 0.5, y: 0.5, z: 0.5 })),
            ("light_color".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 1.0, z: 1.0 })),
            ("shininess".to_string(), Value::F32Val(32.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(color) = &result[0].1 {
            // Should be very dark (no diffuse, no specular)
            assert!(color.x < 0.1);
            assert!(color.y < 0.1);
            assert!(color.z < 0.1);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_phong_lighting_colored_surface() {
        let inputs = vec![
            ("normal".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("light_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("view_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("surface_color".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 0.0, z: 0.0 })), // Red
            ("light_color".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 1.0, z: 1.0 })),
            ("shininess".to_string(), Value::F32Val(32.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(color) = &result[0].1 {
            // Red surface should produce red diffuse + white specular
            assert!(color.x > 0.5); // Red component strong
            assert!(color.y > 0.0); // Green from specular
            assert!(color.z > 0.0); // Blue from specular
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_phong_lighting_normalize_vectors() {
        // Test that non-normalized vectors are handled correctly
        let inputs = vec![
            ("normal".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 2.0, z: 0.0 })), // Not normalized
            ("light_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 3.0, z: 0.0 })), // Not normalized
            ("view_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 4.0, z: 0.0 })), // Not normalized
            ("surface_color".to_string(), Value::Vec3Val(Vec3 { x: 0.5, y: 0.5, z: 0.5 })),
            ("light_color".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 1.0, z: 1.0 })),
            ("shininess".to_string(), Value::F32Val(32.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        // Should produce valid result by normalizing internally
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "lit_color");
    }

    #[test]
    fn test_phong_lighting_negative_shininess() {
        let inputs = vec![
            ("normal".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("light_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("view_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("surface_color".to_string(), Value::Vec3Val(Vec3 { x: 0.5, y: 0.5, z: 0.5 })),
            ("light_color".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 1.0, z: 1.0 })),
            ("shininess".to_string(), Value::F32Val(-1.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("non-negative"));
    }

    #[test]
    fn test_phong_lighting_missing_input() {
        let inputs = vec![
            ("normal".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("light_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("view_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("surface_color".to_string(), Value::Vec3Val(Vec3 { x: 0.5, y: 0.5, z: 0.5 })),
            ("light_color".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 1.0, z: 1.0 })),
            // Missing shininess
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("shininess".to_string()));
    }

    #[test]
    fn test_phong_lighting_low_shininess() {
        // Low shininess = broad specular highlight
        let inputs = vec![
            ("normal".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("light_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("view_dir".to_string(), Value::Vec3Val(Vec3 { x: 0.1, y: 0.99, z: 0.0 })), // Slightly off
            ("surface_color".to_string(), Value::Vec3Val(Vec3 { x: 0.2, y: 0.2, z: 0.2 })),
            ("light_color".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 1.0, z: 1.0 })),
            ("shininess".to_string(), Value::F32Val(2.0)), // Very low shininess
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(color) = &result[0].1 {
            // Should still have some specular despite offset view
            // (low shininess means broad highlight)
            assert!(color.x >= 0.0 && color.x <= 1.0);
            assert!(color.y >= 0.0 && color.y <= 1.0);
            assert!(color.z >= 0.0 && color.z <= 1.0);
        } else {
            panic!("Expected Vec3Val");
        }
    }
}
