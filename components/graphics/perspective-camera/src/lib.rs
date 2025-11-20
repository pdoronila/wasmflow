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

use glam::{Mat4, Vec3};

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Perspective Camera".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculate view and projection matrices for a perspective camera".to_string(),
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
                description: "Camera position in world space".to_string(),
            },
            PortSpec {
                name: "target".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Point the camera is looking at".to_string(),
            },
            PortSpec {
                name: "up".to_string(),
                data_type: DataType::Vec3Type,
                optional: true,
                description: "Up vector (default: 0,1,0)".to_string(),
            },
            PortSpec {
                name: "fov".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Vertical field of view in degrees".to_string(),
            },
            PortSpec {
                name: "aspect_ratio".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Aspect ratio (width / height)".to_string(),
            },
            PortSpec {
                name: "near".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Near clipping plane distance".to_string(),
            },
            PortSpec {
                name: "far".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Far clipping plane distance".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "view_matrix".to_string(),
                data_type: DataType::Mat4Type,
                optional: false,
                description: "View matrix (world to camera space)".to_string(),
            },
            PortSpec {
                name: "projection_matrix".to_string(),
                data_type: DataType::Mat4Type,
                optional: false,
                description: "Projection matrix (camera to clip space)".to_string(),
            },
            PortSpec {
                name: "camera_position".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Camera position (pass-through)".to_string(),
            },
            PortSpec {
                name: "view_direction".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Normalized direction the camera is facing".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract camera position
        let position = extract_vec3(&inputs, "position")?;
        let pos = Vec3::new(position.x, position.y, position.z);

        // Extract target
        let target = extract_vec3(&inputs, "target")?;
        let tgt = Vec3::new(target.x, target.y, target.z);

        // Extract up vector (default to Y-up)
        let up_input = inputs.iter().find(|(name, _)| name == "up");
        let up = if let Some((_, Value::Vec3Val(v))) = up_input {
            Vec3::new(v.x, v.y, v.z)
        } else {
            Vec3::new(0.0, 1.0, 0.0) // Default Y-up
        };

        // Validate up vector
        if up.length_squared() < 1e-6 {
            return Err(ExecutionError {
                message: "Up vector must have non-zero length".to_string(),
                input_name: Some("up".to_string()),
                recovery_hint: Some("Provide a valid up vector (e.g., 0,1,0 for Y-up)".to_string()),
            });
        }

        // Extract FOV
        let fov_degrees = extract_f32(&inputs, "fov")?;
        if fov_degrees <= 0.0 || fov_degrees >= 180.0 {
            return Err(ExecutionError {
                message: format!("FOV must be between 0 and 180 degrees, got {}", fov_degrees),
                input_name: Some("fov".to_string()),
                recovery_hint: Some("Provide a valid FOV (typical: 45-90 degrees)".to_string()),
            });
        }

        // Extract aspect ratio
        let aspect_ratio = extract_f32(&inputs, "aspect_ratio")?;
        if aspect_ratio <= 0.0 {
            return Err(ExecutionError {
                message: format!("Aspect ratio must be positive, got {}", aspect_ratio),
                input_name: Some("aspect_ratio".to_string()),
                recovery_hint: Some("Provide a positive aspect ratio (e.g., 16/9 = 1.778)".to_string()),
            });
        }

        // Extract near plane
        let near = extract_f32(&inputs, "near")?;
        if near <= 0.0 {
            return Err(ExecutionError {
                message: format!("Near plane must be positive, got {}", near),
                input_name: Some("near".to_string()),
                recovery_hint: Some("Provide a positive near plane distance (e.g., 0.1)".to_string()),
            });
        }

        // Extract far plane
        let far = extract_f32(&inputs, "far")?;
        if far <= near {
            return Err(ExecutionError {
                message: format!("Far plane ({}) must be greater than near plane ({})", far, near),
                input_name: Some("far".to_string()),
                recovery_hint: Some("Provide a far plane distance greater than near plane".to_string()),
            });
        }

        // Calculate view matrix using look_at
        let view_matrix = Mat4::look_at_rh(pos, tgt, up);

        // Calculate projection matrix
        let fov_radians = fov_degrees.to_radians();
        let projection_matrix = Mat4::perspective_rh(fov_radians, aspect_ratio, near, far);

        // Calculate view direction
        let view_direction = (tgt - pos).normalize();

        // Convert matrices to WIT types
        let view_mat = mat4_to_wit(&view_matrix);
        let proj_mat = mat4_to_wit(&projection_matrix);
        let view_dir = wasmflow::node::types::Vec3 {
            x: view_direction.x,
            y: view_direction.y,
            z: view_direction.z,
        };

        Ok(vec![
            ("view_matrix".to_string(), Value::Mat4Val(view_mat)),
            ("projection_matrix".to_string(), Value::Mat4Val(proj_mat)),
            ("camera_position".to_string(), Value::Vec3Val(position)),
            ("view_direction".to_string(), Value::Vec3Val(view_dir)),
        ])
    }
}

// Helper functions
fn extract_vec3(inputs: &[(String, Value)], name: &str) -> Result<wasmflow::node::types::Vec3, ExecutionError> {
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

// Convert glam Mat4 to WIT Mat4 (column-major)
fn mat4_to_wit(mat: &Mat4) -> wasmflow::node::types::Mat4 {
    let cols = mat.to_cols_array();
    wasmflow::node::types::Mat4 {
        m00: cols[0],
        m01: cols[1],
        m02: cols[2],
        m03: cols[3],
        m10: cols[4],
        m11: cols[5],
        m12: cols[6],
        m13: cols[7],
        m20: cols[8],
        m21: cols[9],
        m22: cols[10],
        m23: cols[11],
        m30: cols[12],
        m31: cols[13],
        m32: cols[14],
        m33: cols[15],
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_basic() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 5.0,
                }),
            ),
            (
                "target".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            ("fov".to_string(), Value::F32Val(60.0)),
            ("aspect_ratio".to_string(), Value::F32Val(16.0 / 9.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 4);

        // Check output names
        assert_eq!(result[0].0, "view_matrix");
        assert_eq!(result[1].0, "projection_matrix");
        assert_eq!(result[2].0, "camera_position");
        assert_eq!(result[3].0, "view_direction");
    }

    #[test]
    fn test_camera_view_direction() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 5.0,
                }),
            ),
            (
                "target".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            ("fov".to_string(), Value::F32Val(60.0)),
            ("aspect_ratio".to_string(), Value::F32Val(1.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::Vec3Val(dir) = &result[3].1 {
            // View direction should point from camera (0,0,5) to target (0,0,0)
            // Normalized direction = (0,0,-1)
            assert!((dir.x - 0.0).abs() < 0.001);
            assert!((dir.y - 0.0).abs() < 0.001);
            assert!((dir.z - (-1.0)).abs() < 0.001);
        } else {
            panic!("Expected Vec3Val for view_direction");
        }
    }

    #[test]
    fn test_camera_position_passthrough() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 10.0,
                    y: 5.0,
                    z: 3.0,
                }),
            ),
            (
                "target".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            ("fov".to_string(), Value::F32Val(45.0)),
            ("aspect_ratio".to_string(), Value::F32Val(1.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::Vec3Val(pos) = &result[2].1 {
            assert_eq!(pos.x, 10.0);
            assert_eq!(pos.y, 5.0);
            assert_eq!(pos.z, 3.0);
        } else {
            panic!("Expected Vec3Val for camera_position");
        }
    }

    #[test]
    fn test_camera_custom_up_vector() {
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
                "target".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "up".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }),
            ),
            ("fov".to_string(), Value::F32Val(60.0)),
            ("aspect_ratio".to_string(), Value::F32Val(1.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_camera_invalid_fov() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 5.0,
                }),
            ),
            (
                "target".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            ("fov".to_string(), Value::F32Val(0.0)), // Invalid FOV
            ("aspect_ratio".to_string(), Value::F32Val(1.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("fov".to_string()));
    }

    #[test]
    fn test_camera_invalid_near_far() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 5.0,
                }),
            ),
            (
                "target".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            ("fov".to_string(), Value::F32Val(60.0)),
            ("aspect_ratio".to_string(), Value::F32Val(1.0)),
            ("near".to_string(), Value::F32Val(100.0)), // Near > far
            ("far".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("far".to_string()));
    }

    #[test]
    fn test_camera_matrices_are_mat4() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 5.0,
                }),
            ),
            (
                "target".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            ("fov".to_string(), Value::F32Val(60.0)),
            ("aspect_ratio".to_string(), Value::F32Val(1.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
        ];

        let result = Component::execute(inputs).unwrap();

        // Check that matrices are Mat4Val
        assert!(matches!(result[0].1, Value::Mat4Val(_)));
        assert!(matches!(result[1].1, Value::Mat4Val(_)));
    }

    #[test]
    fn test_camera_typical_game_setup() {
        let inputs = vec![
            (
                "position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 5.0,
                    y: 3.0,
                    z: 5.0,
                }),
            ),
            (
                "target".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            ("fov".to_string(), Value::F32Val(75.0)),
            ("aspect_ratio".to_string(), Value::F32Val(16.0 / 9.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(1000.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 4);

        // Should produce valid matrices
        assert!(matches!(result[0].1, Value::Mat4Val(_)));
        assert!(matches!(result[1].1, Value::Mat4Val(_)));
    }
}
