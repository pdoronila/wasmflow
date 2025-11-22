wit_bindgen::generate!({
    path: "wit",
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
            name: "Spot Shadow".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculate perspective shadow matrix for spot lights".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "light_position".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Spot light world position".to_string(),
            },
            PortSpec {
                name: "light_direction".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Spot light direction (will be normalized)".to_string(),
            },
            PortSpec {
                name: "cone_angle".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Spot light cone angle in degrees (must be > 0 and < 180)".to_string(),
            },
            PortSpec {
                name: "near".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Shadow near plane distance".to_string(),
            },
            PortSpec {
                name: "far".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Shadow far plane distance (light range)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "shadow_matrix".to_string(),
            data_type: DataType::ListType,
            optional: false,
            description: "Shadow matrix (16 floats, column-major)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract light_position
        let light_position = extract_vec3(&inputs, "light_position")?;

        // Extract light_direction
        let light_direction = extract_vec3(&inputs, "light_direction")?;
        let direction_length = light_direction.length();
        if direction_length < 0.001 {
            return Err(ExecutionError {
                message: "Light direction vector is too small (near zero)".to_string(),
                input_name: Some("light_direction".to_string()),
                recovery_hint: Some("Provide a non-zero direction vector".to_string()),
            });
        }
        let light_direction = light_direction.normalize();

        // Extract cone_angle
        let cone_angle = extract_f32(&inputs, "cone_angle")?;
        if cone_angle <= 0.0 || cone_angle >= 180.0 {
            return Err(ExecutionError {
                message: format!("Cone angle must be between 0 and 180 degrees, got {}", cone_angle),
                input_name: Some("cone_angle".to_string()),
                recovery_hint: Some("Provide a cone angle between 0 and 180 (typical: 30-90 degrees)".to_string()),
            });
        }

        // Extract near
        let near = extract_f32(&inputs, "near")?;
        if near <= 0.0 {
            return Err(ExecutionError {
                message: "Near plane must be positive".to_string(),
                input_name: Some("near".to_string()),
                recovery_hint: Some("Provide a positive near plane distance (e.g., 0.1)".to_string()),
            });
        }

        // Extract far
        let far = extract_f32(&inputs, "far")?;
        if far <= near {
            return Err(ExecutionError {
                message: format!("Far plane ({}) must be greater than near plane ({})", far, near),
                input_name: Some("far".to_string()),
                recovery_hint: Some("Provide a far plane distance greater than near (light range)".to_string()),
            });
        }

        // Calculate shadow matrix
        let shadow_matrix = calculate_spot_shadow_matrix(
            light_position,
            light_direction,
            cone_angle,
            near,
            far,
        );

        // Convert to flat array
        let matrix_flat = shadow_matrix.to_cols_array().to_vec();

        let mut outputs = Vec::new();
        outputs.push(("shadow_matrix".to_string(), Value::F32ListVal(matrix_flat)));

        Ok(outputs)
    }
}

// Helper functions

fn extract_vec3(inputs: &[(String, Value)], name: &str) -> Result<Vec3, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(format!("Connect a vec3 value to '{}'", name)),
        })?;

    match &input.1 {
        Value::Vec3Val(v) => Ok(Vec3::new(v.x, v.y, v.z)),
        Value::F32ListVal(list) if list.len() == 3 => Ok(Vec3::new(list[0], list[1], list[2])),
        _ => Err(ExecutionError {
            message: format!("Expected vec3 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a 3-element vector [x, y, z]".to_string()),
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
            recovery_hint: Some(format!("Connect a number to '{}'", name)),
        })?;

    match input.1 {
        Value::F32Val(f) => Ok(f),
        Value::U32Val(u) => Ok(u as f32),
        Value::I32Val(i) => Ok(i as f32),
        _ => Err(ExecutionError {
            message: format!("Expected f32 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a number value".to_string()),
        }),
    }
}

/// Calculate spot light shadow matrix
///
/// Creates a perspective projection matrix that matches the spot light's cone.
/// The FOV is set to match the cone angle, ensuring the shadow map covers
/// the exact area illuminated by the spot light.
fn calculate_spot_shadow_matrix(
    light_position: Vec3,
    light_direction: Vec3,
    cone_angle: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    // Convert cone angle from degrees to radians
    let fov = cone_angle.to_radians();

    // Create perspective projection matching the cone
    let projection = Mat4::perspective_rh(
        fov,  // FOV matches cone angle
        1.0,  // Aspect ratio 1:1 (square shadow map)
        near,
        far,
    );

    // Create view matrix looking in the light direction
    // We need an up vector that's not parallel to the light direction
    let up = if light_direction.y.abs() < 0.999 {
        Vec3::Y
    } else {
        Vec3::X
    };

    let view = Mat4::look_at_rh(
        light_position,
        light_position + light_direction,
        up,
    );

    projection * view
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_shadow_matrix_valid() {
        let matrix = calculate_spot_shadow_matrix(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            45.0,
            0.1,
            10.0,
        );

        // Matrix should be invertible
        assert!(matrix.determinant().abs() > 0.0);
    }

    #[test]
    fn test_execute_valid() {
        let inputs = vec![
            (
                "light_position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 5.0,
                    z: 0.0,
                }),
            ),
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            ("cone_angle".to_string(), Value::F32Val(45.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs).unwrap();

        // Should have 1 output
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "shadow_matrix");

        // Output should be flattened f32 list with 16 elements
        if let Value::F32ListVal(matrix) = &result[0].1 {
            assert_eq!(matrix.len(), 16);
        } else {
            panic!("Expected F32ListVal for shadow_matrix");
        }
    }

    #[test]
    fn test_invalid_cone_angle_zero() {
        let inputs = vec![
            (
                "light_position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 5.0,
                    z: 0.0,
                }),
            ),
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            ("cone_angle".to_string(), Value::F32Val(0.0)), // Invalid: zero
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_cone_angle_too_large() {
        let inputs = vec![
            (
                "light_position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 5.0,
                    z: 0.0,
                }),
            ),
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            ("cone_angle".to_string(), Value::F32Val(180.0)), // Invalid: >= 180
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_direction() {
        let inputs = vec![
            (
                "light_position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 5.0,
                    z: 0.0,
                }),
            ),
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ), // Zero direction
            ("cone_angle".to_string(), Value::F32Val(45.0)),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_near() {
        let inputs = vec![
            (
                "light_position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 5.0,
                    z: 0.0,
                }),
            ),
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            ("cone_angle".to_string(), Value::F32Val(45.0)),
            ("near".to_string(), Value::F32Val(-0.1)), // Negative near
            ("far".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_near_far() {
        let inputs = vec![
            (
                "light_position".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 5.0,
                    z: 0.0,
                }),
            ),
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            ("cone_angle".to_string(), Value::F32Val(45.0)),
            ("near".to_string(), Value::F32Val(10.0)), // near > far
            ("far".to_string(), Value::F32Val(5.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_cone_angles() {
        // Test that different cone angles produce different matrices
        let matrix1 = calculate_spot_shadow_matrix(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            30.0,
            0.1,
            10.0,
        );

        let matrix2 = calculate_spot_shadow_matrix(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            60.0,
            0.1,
            10.0,
        );

        // Matrices should differ
        let cols1 = matrix1.to_cols_array();
        let cols2 = matrix2.to_cols_array();

        let all_same = cols1
            .iter()
            .zip(cols2.iter())
            .all(|(a, b)| (a - b).abs() < 0.001);

        assert!(!all_same, "Matrices should differ for different cone angles");
    }

    #[test]
    fn test_different_directions() {
        // Test that different directions produce different matrices
        let matrix1 = calculate_spot_shadow_matrix(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            45.0,
            0.1,
            10.0,
        );

        let matrix2 = calculate_spot_shadow_matrix(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            45.0,
            0.1,
            10.0,
        );

        // Matrices should differ
        let cols1 = matrix1.to_cols_array();
        let cols2 = matrix2.to_cols_array();

        let all_same = cols1
            .iter()
            .zip(cols2.iter())
            .all(|(a, b)| (a - b).abs() < 0.001);

        assert!(!all_same, "Matrices should differ for different directions");
    }
}
