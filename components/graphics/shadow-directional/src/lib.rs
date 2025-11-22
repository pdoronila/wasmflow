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

use glam::{Mat4, Vec3, Vec4};

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Directional Shadow".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculate cascaded shadow map matrices for directional lights".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "light_direction".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Light direction (normalized)".to_string(),
            },
            PortSpec {
                name: "view_matrix".to_string(),
                data_type: DataType::Mat4Type,
                optional: false,
                description: "Camera view matrix".to_string(),
            },
            PortSpec {
                name: "projection_matrix".to_string(),
                data_type: DataType::Mat4Type,
                optional: false,
                description: "Camera projection matrix".to_string(),
            },
            PortSpec {
                name: "near".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Camera near plane distance".to_string(),
            },
            PortSpec {
                name: "far".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Camera far plane distance".to_string(),
            },
            PortSpec {
                name: "cascade_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of cascades (1-4, typical: 4)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "shadow_matrices".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Shadow matrices (flattened f32 list: cascade_count * 16 floats)".to_string(),
            },
            PortSpec {
                name: "cascade_splits".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Cascade split distances (f32 list)".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract light_direction
        let light_direction = extract_vec3(&inputs, "light_direction")?;

        // Extract view_matrix
        let view_matrix = extract_mat4(&inputs, "view_matrix")?;

        // Extract projection_matrix
        let projection_matrix = extract_mat4(&inputs, "projection_matrix")?;

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
                recovery_hint: Some("Provide a far plane distance greater than near".to_string()),
            });
        }

        // Extract cascade_count
        let cascade_count = extract_u32(&inputs, "cascade_count")?;
        if cascade_count < 1 || cascade_count > 4 {
            return Err(ExecutionError {
                message: format!("Cascade count must be 1-4, got {}", cascade_count),
                input_name: Some("cascade_count".to_string()),
                recovery_hint: Some("Use 4 cascades for best quality, 2-3 for performance".to_string()),
            });
        }

        // Normalize light direction
        let light_dir = light_direction.normalize_or_zero();
        if light_dir.length() < 0.001 {
            return Err(ExecutionError {
                message: "Light direction is zero or too small".to_string(),
                input_name: Some("light_direction".to_string()),
                recovery_hint: Some("Provide a non-zero light direction vector".to_string()),
            });
        }

        // Calculate cascade splits using practical split scheme (lambda = 0.5)
        let cascade_splits = calculate_cascade_splits(near, far, cascade_count, 0.5);

        // Calculate shadow matrix for each cascade
        let mut shadow_matrices = Vec::new();
        for i in 0..cascade_count as usize {
            let near_dist = cascade_splits[i];
            let far_dist = cascade_splits[i + 1];

            let shadow_matrix = calculate_directional_shadow_matrix(
                light_dir,
                view_matrix,
                projection_matrix,
                near_dist,
                far_dist,
            );

            shadow_matrices.push(shadow_matrix);
        }

        // Convert to output format
        let mut outputs = Vec::new();

        // Shadow matrices as flattened f32 list (cascade_count * 16 floats)
        // Each matrix is 16 floats in column-major order
        let mut matrices_flat = Vec::with_capacity(shadow_matrices.len() * 16);
        for matrix in &shadow_matrices {
            matrices_flat.extend_from_slice(&matrix.to_cols_array());
        }
        outputs.push(("shadow_matrices".to_string(), Value::F32ListVal(matrices_flat)));

        // Cascade splits as f32 list
        outputs.push((
            "cascade_splits".to_string(),
            Value::F32ListVal(cascade_splits),
        ));

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

fn extract_mat4(inputs: &[(String, Value)], name: &str) -> Result<Mat4, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(format!("Connect a mat4 value to '{}'", name)),
        })?;

    match &input.1 {
        Value::Mat4Val(m) => {
            // Mat4Val has individual m00-m33 fields (column-major)
            let arr = [
                m.m00, m.m10, m.m20, m.m30, // Column 0
                m.m01, m.m11, m.m21, m.m31, // Column 1
                m.m02, m.m12, m.m22, m.m32, // Column 2
                m.m03, m.m13, m.m23, m.m33, // Column 3
            ];
            Ok(Mat4::from_cols_array(&arr))
        }
        Value::F32ListVal(list) if list.len() == 16 => {
            let arr: [f32; 16] = list.as_slice().try_into().unwrap();
            Ok(Mat4::from_cols_array(&arr))
        }
        _ => Err(ExecutionError {
            message: format!("Expected mat4 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a 4x4 matrix (16 elements)".to_string()),
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

fn extract_u32(inputs: &[(String, Value)], name: &str) -> Result<u32, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(format!("Connect an integer to '{}'", name)),
        })?;

    match input.1 {
        Value::U32Val(u) => Ok(u),
        Value::I32Val(i) if i >= 0 => Ok(i as u32),
        _ => Err(ExecutionError {
            message: format!("Expected u32 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a positive integer".to_string()),
        }),
    }
}

/// Calculate cascade splits using practical split scheme
fn calculate_cascade_splits(near: f32, far: f32, cascade_count: u32, lambda: f32) -> Vec<f32> {
    let mut splits = vec![near];

    for i in 1..=cascade_count {
        let i_f = i as f32;
        let count_f = cascade_count as f32;

        // Logarithmic split
        let log_split = near * (far / near).powf(i_f / count_f);

        // Uniform split
        let uniform_split = near + (far - near) * (i_f / count_f);

        // Practical split (blend of log and uniform)
        let split = lambda * log_split + (1.0 - lambda) * uniform_split;

        splits.push(split);
    }

    splits
}

/// Calculate directional shadow matrix for a cascade
fn calculate_directional_shadow_matrix(
    light_direction: Vec3,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    near_distance: f32,
    far_distance: f32,
) -> Mat4 {
    // Get frustum corners in world space
    let frustum_corners = calculate_frustum_corners_world_space(
        view_matrix,
        projection_matrix,
        near_distance,
        far_distance,
    );

    // Calculate frustum center
    let mut center = Vec3::ZERO;
    for corner in &frustum_corners {
        center += *corner;
    }
    center /= frustum_corners.len() as f32;

    // Create light view matrix (look at frustum center from light direction)
    let light_view = Mat4::look_at_rh(
        center - light_direction * 10.0, // Position light far back
        center,
        Vec3::Y, // Up vector
    );

    // Calculate AABB of frustum in light space
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for corner in &frustum_corners {
        let light_space = light_view.transform_point3(*corner);
        min = min.min(light_space);
        max = max.max(light_space);
    }

    // Create orthographic projection for light (directional lights use ortho)
    let light_projection = Mat4::orthographic_rh(
        min.x, max.x, // left, right
        min.y, max.y, // bottom, top
        min.z, max.z, // near, far
    );

    light_projection * light_view
}

/// Calculate frustum corners in world space
fn calculate_frustum_corners_world_space(
    view_matrix: Mat4,
    projection_matrix: Mat4,
    _near_distance: f32,
    _far_distance: f32,
) -> Vec<Vec3> {
    let inv_view_proj = (projection_matrix * view_matrix).inverse();

    let mut corners = Vec::with_capacity(8);

    // NDC coordinates of frustum corners
    let ndc_corners = [
        // Near plane
        Vec4::new(-1.0, -1.0, -1.0, 1.0), // Bottom-left
        Vec4::new(1.0, -1.0, -1.0, 1.0),  // Bottom-right
        Vec4::new(1.0, 1.0, -1.0, 1.0),   // Top-right
        Vec4::new(-1.0, 1.0, -1.0, 1.0),  // Top-left
        // Far plane
        Vec4::new(-1.0, -1.0, 1.0, 1.0), // Bottom-left
        Vec4::new(1.0, -1.0, 1.0, 1.0),  // Bottom-right
        Vec4::new(1.0, 1.0, 1.0, 1.0),   // Top-right
        Vec4::new(-1.0, 1.0, 1.0, 1.0),  // Top-left
    ];

    for ndc in ndc_corners {
        let world_pos = inv_view_proj * ndc;
        let world_pos = world_pos / world_pos.w; // Perspective divide
        corners.push(world_pos.truncate());
    }

    corners
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to convert glam Mat4 to WIT Mat4
    fn glam_to_wit_mat4(m: Mat4) -> wasmflow::node::types::Mat4 {
        wasmflow::node::types::Mat4 {
            m00: m.col(0).x,
            m10: m.col(0).y,
            m20: m.col(0).z,
            m30: m.col(0).w,
            m01: m.col(1).x,
            m11: m.col(1).y,
            m21: m.col(1).z,
            m31: m.col(1).w,
            m02: m.col(2).x,
            m12: m.col(2).y,
            m22: m.col(2).z,
            m32: m.col(2).w,
            m03: m.col(3).x,
            m13: m.col(3).y,
            m23: m.col(3).z,
            m33: m.col(3).w,
        }
    }

    #[test]
    fn test_cascade_splits_practical() {
        let splits = calculate_cascade_splits(0.1, 100.0, 4, 0.5);
        assert_eq!(splits.len(), 5); // 4 cascades + 1 = 5 splits
        assert_eq!(splits[0], 0.1); // Near
        assert_eq!(splits[4], 100.0); // Far
        // Middle splits should be between near and far
        for i in 1..4 {
            assert!(splits[i] > splits[i - 1]);
            assert!(splits[i] < splits[i + 1]);
        }
    }

    #[test]
    fn test_cascade_splits_uniform() {
        let splits = calculate_cascade_splits(0.1, 100.0, 4, 0.0);
        // Uniform splits with lambda=0: near + (far-near) * (i/count)
        // 0.1 + 99.9 * 0.25 = 25.075
        // 0.1 + 99.9 * 0.5 = 50.05
        // 0.1 + 99.9 * 0.75 = 75.025
        assert!((splits[1] - 25.075).abs() < 0.01);
        assert!((splits[2] - 50.05).abs() < 0.01);
        assert!((splits[3] - 75.025).abs() < 0.01);
    }

    #[test]
    fn test_execute_valid() {
        let inputs = vec![
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "view_matrix".to_string(),
                Value::Mat4Val(glam_to_wit_mat4(Mat4::IDENTITY)),
            ),
            (
                "projection_matrix".to_string(),
                Value::Mat4Val(glam_to_wit_mat4(Mat4::perspective_rh(
                    std::f32::consts::FRAC_PI_4,
                    16.0 / 9.0,
                    0.1,
                    100.0,
                ))),
            ),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
            ("cascade_count".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs).unwrap();

        // Should have 2 outputs
        assert_eq!(result.len(), 2);

        // Check shadow_matrices
        let matrices = result.iter().find(|(name, _)| name == "shadow_matrices");
        assert!(matrices.is_some());

        // Check cascade_splits
        let splits = result.iter().find(|(name, _)| name == "cascade_splits");
        assert!(splits.is_some());
        if let Some((_, Value::F32ListVal(splits))) = splits {
            assert_eq!(splits.len(), 5); // 4 cascades + 1 = 5 splits
        } else {
            panic!("cascade_splits should be F32ListVal");
        }
    }

    #[test]
    fn test_invalid_cascade_count() {
        let inputs = vec![
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "view_matrix".to_string(),
                Value::Mat4Val(glam_to_wit_mat4(Mat4::IDENTITY)),
            ),
            (
                "projection_matrix".to_string(),
                Value::Mat4Val(glam_to_wit_mat4(Mat4::IDENTITY)),
            ),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
            ("cascade_count".to_string(), Value::U32Val(5)), // Invalid: > 4
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_near_far() {
        let inputs = vec![
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                }),
            ),
            (
                "view_matrix".to_string(),
                Value::Mat4Val(glam_to_wit_mat4(Mat4::IDENTITY)),
            ),
            (
                "projection_matrix".to_string(),
                Value::Mat4Val(glam_to_wit_mat4(Mat4::IDENTITY)),
            ),
            ("near".to_string(), Value::F32Val(100.0)), // near > far
            ("far".to_string(), Value::F32Val(10.0)),
            ("cascade_count".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_light_direction() {
        let inputs = vec![
            (
                "light_direction".to_string(),
                Value::Vec3Val(wasmflow::node::types::Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ),
            (
                "view_matrix".to_string(),
                Value::Mat4Val(glam_to_wit_mat4(Mat4::IDENTITY)),
            ),
            (
                "projection_matrix".to_string(),
                Value::Mat4Val(glam_to_wit_mat4(Mat4::IDENTITY)),
            ),
            ("near".to_string(), Value::F32Val(0.1)),
            ("far".to_string(), Value::F32Val(100.0)),
            ("cascade_count".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
