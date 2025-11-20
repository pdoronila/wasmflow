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
            name: "Cube Primitive".to_string(),
            version: "1.0.0".to_string(),
            description: "Generate cube mesh data with customizable dimensions".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "size".to_string(),
            data_type: DataType::Vec3Type,
            optional: false,
            description: "Cube dimensions (width, height, depth)".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "vertices".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Vertex positions as JSON strings (format: \"x,y,z\")".to_string(),
            },
            PortSpec {
                name: "normals".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Vertex normals as JSON strings (format: \"x,y,z\")".to_string(),
            },
            PortSpec {
                name: "uvs".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "UV coordinates as JSON strings (format: \"u,v\")".to_string(),
            },
            PortSpec {
                name: "indices".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Triangle indices (3 indices per triangle)".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract size
        let size = extract_vec3(&inputs, "size")?;

        // Validate dimensions
        if size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
            return Err(ExecutionError {
                message: format!(
                    "All dimensions must be positive, got ({}, {}, {})",
                    size.x, size.y, size.z
                ),
                input_name: Some("size".to_string()),
                recovery_hint: Some("Provide positive values for width, height, and depth".to_string()),
            });
        }

        // Generate cube mesh
        let (vertices, normals, uvs, indices) = generate_cube(size.x, size.y, size.z);

        // Convert to string representations
        let vertex_strings: Vec<String> = vertices
            .iter()
            .map(|v| format!("{},{},{}", v.x, v.y, v.z))
            .collect();

        let normal_strings: Vec<String> = normals
            .iter()
            .map(|n| format!("{},{},{}", n.x, n.y, n.z))
            .collect();

        let uv_strings: Vec<String> = uvs
            .iter()
            .map(|(u, v)| format!("{},{}", u, v))
            .collect();

        Ok(vec![
            ("vertices".to_string(), Value::StringListVal(vertex_strings)),
            ("normals".to_string(), Value::StringListVal(normal_strings)),
            ("uvs".to_string(), Value::StringListVal(uv_strings)),
            ("indices".to_string(), Value::U32ListVal(indices)),
        ])
    }
}

/// Generate a cube mesh with separate vertices per face (for proper normals)
fn generate_cube(width: f32, height: f32, depth: f32) -> (Vec<Vec3>, Vec<Vec3>, Vec<(f32, f32)>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let half_w = width / 2.0;
    let half_h = height / 2.0;
    let half_d = depth / 2.0;

    // Define 6 faces of the cube
    // Each face has 4 vertices with the same normal

    // Front face (+Z)
    add_face(
        &mut vertices,
        &mut normals,
        &mut uvs,
        &mut indices,
        [
            Vec3::new(-half_w, -half_h, half_d),
            Vec3::new(half_w, -half_h, half_d),
            Vec3::new(half_w, half_h, half_d),
            Vec3::new(-half_w, half_h, half_d),
        ],
        Vec3::new(0.0, 0.0, 1.0),
    );

    // Back face (-Z)
    add_face(
        &mut vertices,
        &mut normals,
        &mut uvs,
        &mut indices,
        [
            Vec3::new(half_w, -half_h, -half_d),
            Vec3::new(-half_w, -half_h, -half_d),
            Vec3::new(-half_w, half_h, -half_d),
            Vec3::new(half_w, half_h, -half_d),
        ],
        Vec3::new(0.0, 0.0, -1.0),
    );

    // Right face (+X)
    add_face(
        &mut vertices,
        &mut normals,
        &mut uvs,
        &mut indices,
        [
            Vec3::new(half_w, -half_h, half_d),
            Vec3::new(half_w, -half_h, -half_d),
            Vec3::new(half_w, half_h, -half_d),
            Vec3::new(half_w, half_h, half_d),
        ],
        Vec3::new(1.0, 0.0, 0.0),
    );

    // Left face (-X)
    add_face(
        &mut vertices,
        &mut normals,
        &mut uvs,
        &mut indices,
        [
            Vec3::new(-half_w, -half_h, -half_d),
            Vec3::new(-half_w, -half_h, half_d),
            Vec3::new(-half_w, half_h, half_d),
            Vec3::new(-half_w, half_h, -half_d),
        ],
        Vec3::new(-1.0, 0.0, 0.0),
    );

    // Top face (+Y)
    add_face(
        &mut vertices,
        &mut normals,
        &mut uvs,
        &mut indices,
        [
            Vec3::new(-half_w, half_h, half_d),
            Vec3::new(half_w, half_h, half_d),
            Vec3::new(half_w, half_h, -half_d),
            Vec3::new(-half_w, half_h, -half_d),
        ],
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Bottom face (-Y)
    add_face(
        &mut vertices,
        &mut normals,
        &mut uvs,
        &mut indices,
        [
            Vec3::new(-half_w, -half_h, -half_d),
            Vec3::new(half_w, -half_h, -half_d),
            Vec3::new(half_w, -half_h, half_d),
            Vec3::new(-half_w, -half_h, half_d),
        ],
        Vec3::new(0.0, -1.0, 0.0),
    );

    (vertices, normals, uvs, indices)
}

/// Add a face to the mesh
fn add_face(
    vertices: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    uvs: &mut Vec<(f32, f32)>,
    indices: &mut Vec<u32>,
    corners: [Vec3; 4],
    normal: Vec3,
) {
    let base_index = vertices.len() as u32;

    // Add 4 vertices for the quad
    vertices.extend_from_slice(&corners);

    // All vertices on this face have the same normal
    normals.push(normal);
    normals.push(normal);
    normals.push(normal);
    normals.push(normal);

    // Standard quad UVs
    uvs.push((0.0, 0.0));
    uvs.push((1.0, 0.0));
    uvs.push((1.0, 1.0));
    uvs.push((0.0, 1.0));

    // Two triangles per face
    // Triangle 1
    indices.push(base_index);
    indices.push(base_index + 1);
    indices.push(base_index + 2);

    // Triangle 2
    indices.push(base_index);
    indices.push(base_index + 2);
    indices.push(base_index + 3);
}

// Helper function
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

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_basic_generation() {
        let inputs = vec![(
            "size".to_string(),
            Value::Vec3Val(wasmflow::node::types::Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }),
        )];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 4);

        // Extract outputs
        let vertices = if let Value::StringListVal(v) = &result[0].1 {
            v
        } else {
            panic!("Expected StringListVal for vertices");
        };

        let indices = if let Value::U32ListVal(i) = &result[3].1 {
            i
        } else {
            panic!("Expected U32ListVal for indices");
        };

        // Cube has 6 faces * 4 vertices per face = 24 vertices
        assert_eq!(vertices.len(), 24);

        // Cube has 6 faces * 2 triangles per face * 3 indices per triangle = 36 indices
        assert_eq!(indices.len(), 36);
    }

    #[test]
    fn test_cube_custom_dimensions() {
        let inputs = vec![(
            "size".to_string(),
            Value::Vec3Val(wasmflow::node::types::Vec3 {
                x: 2.0,
                y: 3.0,
                z: 4.0,
            }),
        )];

        let result = Component::execute(inputs).unwrap();

        let vertices = if let Value::StringListVal(v) = &result[0].1 {
            v
        } else {
            panic!("Expected StringListVal");
        };

        // Still 24 vertices regardless of size
        assert_eq!(vertices.len(), 24);

        // Check that vertices actually use the dimensions (half of 2.0 = 1.0)
        let first_vertex = &vertices[0];
        assert!(first_vertex.contains("-1") || first_vertex.contains("1")); // Contains dimension values
    }

    #[test]
    fn test_cube_normals() {
        let inputs = vec![(
            "size".to_string(),
            Value::Vec3Val(wasmflow::node::types::Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }),
        )];

        let result = Component::execute(inputs).unwrap();

        let normals = if let Value::StringListVal(n) = &result[1].1 {
            n
        } else {
            panic!("Expected StringListVal for normals");
        };

        // 24 normals (one per vertex)
        assert_eq!(normals.len(), 24);

        // First 4 normals should be the same (front face)
        assert_eq!(normals[0], normals[1]);
        assert_eq!(normals[1], normals[2]);
        assert_eq!(normals[2], normals[3]);
    }

    #[test]
    fn test_cube_invalid_dimensions() {
        let inputs = vec![(
            "size".to_string(),
            Value::Vec3Val(wasmflow::node::types::Vec3 {
                x: -1.0,
                y: 1.0,
                z: 1.0,
            }),
        )];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("size".to_string()));
    }

    #[test]
    fn test_cube_uv_coordinates() {
        let inputs = vec![(
            "size".to_string(),
            Value::Vec3Val(wasmflow::node::types::Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }),
        )];

        let result = Component::execute(inputs).unwrap();

        let uvs = if let Value::StringListVal(u) = &result[2].1 {
            u
        } else {
            panic!("Expected StringListVal for uvs");
        };

        // 24 UV coordinates (one per vertex)
        assert_eq!(uvs.len(), 24);

        // Each face should have standard quad UVs (0,0), (1,0), (1,1), (0,1)
        assert_eq!(uvs[0], "0,0");
        assert_eq!(uvs[1], "1,0");
        assert_eq!(uvs[2], "1,1");
        assert_eq!(uvs[3], "0,1");
    }

    #[test]
    fn test_cube_vertex_format() {
        let inputs = vec![(
            "size".to_string(),
            Value::Vec3Val(wasmflow::node::types::Vec3 {
                x: 2.0,
                y: 2.0,
                z: 2.0,
            }),
        )];

        let result = Component::execute(inputs).unwrap();

        let vertices = if let Value::StringListVal(v) = &result[0].1 {
            v
        } else {
            panic!("Expected StringListVal");
        };

        // Check that first vertex is parseable
        let first_vertex = &vertices[0];
        let parts: Vec<&str> = first_vertex.split(',').collect();
        assert_eq!(parts.len(), 3);

        // Each part should be a valid float
        for part in parts {
            part.parse::<f32>().expect("Should be valid f32");
        }
    }
}
