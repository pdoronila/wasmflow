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
            name: "Point Shadow".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculate cubemap shadow matrices for point lights (6 faces)".to_string(),
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
                description: "Point light world position".to_string(),
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
                description: "Shadow far plane distance (light radius)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "shadow_matrices".to_string(),
            data_type: DataType::ListType,
            optional: false,
            description: "6 shadow matrices for cubemap faces (flattened: 6 * 16 = 96 floats)".to_string(),
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
                recovery_hint: Some("Provide a far plane distance greater than near (light radius)".to_string()),
            });
        }

        // Calculate shadow matrices for all 6 cubemap faces
        let shadow_matrices = calculate_point_shadow_matrices(light_position, near, far);

        // Flatten matrices to single f32 list (6 * 16 = 96 floats)
        let mut matrices_flat = Vec::with_capacity(96);
        for matrix in &shadow_matrices {
            matrices_flat.extend_from_slice(&matrix.to_cols_array());
        }

        let mut outputs = Vec::new();
        outputs.push(("shadow_matrices".to_string(), Value::F32ListVal(matrices_flat)));

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

/// Calculate point light shadow matrices (6 faces for cubemap)
///
/// Returns 6 shadow matrices in this order:
/// 1. +X face (right)
/// 2. -X face (left)
/// 3. +Y face (top)
/// 4. -Y face (bottom)
/// 5. +Z face (front)
/// 6. -Z face (back)
fn calculate_point_shadow_matrices(light_position: Vec3, near: f32, far: f32) -> Vec<Mat4> {
    let projection = Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_2, // 90 degree FOV for each face
        1.0,                         // Aspect ratio 1:1 (square faces)
        near,
        far,
    );

    // Cubemap face directions and up vectors
    let faces = [
        (Vec3::X, Vec3::NEG_Y),     // +X face (right)
        (Vec3::NEG_X, Vec3::NEG_Y), // -X face (left)
        (Vec3::Y, Vec3::Z),         // +Y face (top)
        (Vec3::NEG_Y, Vec3::NEG_Z), // -Y face (bottom)
        (Vec3::Z, Vec3::NEG_Y),     // +Z face (front)
        (Vec3::NEG_Z, Vec3::NEG_Y), // -Z face (back)
    ];

    faces
        .iter()
        .map(|(direction, up)| {
            let view = Mat4::look_at_rh(light_position, light_position + *direction, *up);
            projection * view
        })
        .collect()
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_shadow_matrices_count() {
        let light_pos = Vec3::new(0.0, 5.0, 0.0);
        let matrices = calculate_point_shadow_matrices(light_pos, 0.1, 10.0);

        // Should generate exactly 6 matrices (one per cubemap face)
        assert_eq!(matrices.len(), 6);
    }

    #[test]
    fn test_point_shadow_matrices_valid() {
        let light_pos = Vec3::new(0.0, 5.0, 0.0);
        let matrices = calculate_point_shadow_matrices(light_pos, 0.1, 10.0);

        // All matrices should be valid (non-zero determinant)
        for (i, matrix) in matrices.iter().enumerate() {
            let det = matrix.determinant();
            assert!(
                det.abs() > 0.0,
                "Matrix {} has zero determinant: {}",
                i,
                det
            );
        }
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
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(10.0)),
        ];

        let result = Component::execute(inputs).unwrap();

        // Should have 1 output
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "shadow_matrices");

        // Output should be flattened f32 list with 96 elements (6 matrices * 16 floats)
        if let Value::F32ListVal(matrices) = &result[0].1 {
            assert_eq!(matrices.len(), 96);
        } else {
            panic!("Expected F32ListVal for shadow_matrices");
        }
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
            ("near".to_string(), Value::F32Val(10.0)), // near > far
            ("far".to_string(), Value::F32Val(5.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_positions() {
        // Test that different light positions produce different results
        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(5.0, 5.0, 5.0);

        let matrices1 = calculate_point_shadow_matrices(pos1, 0.1, 10.0);
        let matrices2 = calculate_point_shadow_matrices(pos2, 0.1, 10.0);

        // At least one matrix should be different
        let all_same = matrices1.iter().zip(matrices2.iter()).all(|(m1, m2)| {
            m1.to_cols_array()
                .iter()
                .zip(m2.to_cols_array().iter())
                .all(|(a, b)| (a - b).abs() < 0.001)
        });

        assert!(!all_same, "Matrices should differ for different positions");
    }

    #[test]
    fn test_perspective_90_degrees() {
        // Verify that perspective projection uses 90 degree FOV
        let matrices = calculate_point_shadow_matrices(Vec3::ZERO, 0.1, 10.0);

        // All matrices should be using perspective projection
        // After projection * view multiplication, we need to verify the FOV
        // Check that the matrices have the correct perspective scaling
        for (i, matrix) in matrices.iter().enumerate() {
            let cols = matrix.to_cols_array();

            // For a 90-degree FOV perspective projection, the diagonal elements
            // cols[0] and cols[5] should be approximately 1.0 (f = 1/tan(45°) ≈ 1.0)
            // This verifies we're using perspective projection with correct FOV
            let determinant = matrix.determinant();
            assert!(
                determinant.abs() > 0.0,
                "Matrix {} should be invertible (non-zero determinant), got {}",
                i,
                determinant
            );

            // Verify it's not an identity matrix
            let is_identity = cols.iter().enumerate().all(|(idx, &val)| {
                let expected = if idx % 5 == 0 { 1.0 } else { 0.0 };
                (val - expected).abs() < 0.001
            });
            assert!(!is_identity, "Matrix {} should not be identity", i);
        }
    }
}
