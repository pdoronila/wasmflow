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

use glam::{Vec3, Vec4};

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Plane Primitive".to_string(),
            version: "1.0.0".to_string(),
            description: "Generate subdivided plane mesh data (XZ plane, facing +Y)".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "width".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Plane width (X dimension)".to_string(),
            },
            PortSpec {
                name: "height".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Plane height (Z dimension)".to_string(),
            },
            PortSpec {
                name: "subdivisions".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of subdivisions per axis (1 = single quad, 2 = 2x2 grid, etc.)".to_string(),
            },
        ]
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
                name: "tangents".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Tangent vectors as JSON strings (format: \"x,y,z,w\" where w=handedness)".to_string(),
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
        // Extract width
        let width = extract_f32(&inputs, "width")?;
        if width <= 0.0 {
            return Err(ExecutionError {
                message: format!("Width must be positive, got {}", width),
                input_name: Some("width".to_string()),
                recovery_hint: Some("Provide a positive width value".to_string()),
            });
        }

        // Extract height
        let height = extract_f32(&inputs, "height")?;
        if height <= 0.0 {
            return Err(ExecutionError {
                message: format!("Height must be positive, got {}", height),
                input_name: Some("height".to_string()),
                recovery_hint: Some("Provide a positive height value".to_string()),
            });
        }

        // Extract subdivisions
        let subdivisions = extract_u32(&inputs, "subdivisions")?;
        if subdivisions < 1 {
            return Err(ExecutionError {
                message: format!("Subdivisions must be at least 1, got {}", subdivisions),
                input_name: Some("subdivisions".to_string()),
                recovery_hint: Some("Provide at least 1 subdivision".to_string()),
            });
        }

        // Generate plane mesh
        let (vertices, normals, uvs, tangents, indices) = generate_plane(width, height, subdivisions);

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

        let tangent_strings: Vec<String> = tangents
            .iter()
            .map(|t| format!("{},{},{},{}", t.x, t.y, t.z, t.w))
            .collect();

        Ok(vec![
            ("vertices".to_string(), Value::StringListVal(vertex_strings)),
            ("normals".to_string(), Value::StringListVal(normal_strings)),
            ("uvs".to_string(), Value::StringListVal(uv_strings)),
            ("tangents".to_string(), Value::StringListVal(tangent_strings)),
            ("indices".to_string(), Value::U32ListVal(indices)),
        ])
    }
}

/// Generate a subdivided plane mesh (XZ plane, facing +Y)
fn generate_plane(width: f32, height: f32, subdivisions: u32) -> (Vec<Vec3>, Vec<Vec3>, Vec<(f32, f32)>, Vec<Vec4>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut tangents = Vec::new();
    let mut indices = Vec::new();

    let half_w = width / 2.0;
    let half_h = height / 2.0;

    // Generate vertices, normals, UVs, and tangents
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let u = (x as f32) / (subdivisions as f32);
            let v = (z as f32) / (subdivisions as f32);

            // Map UV [0,1] to world space [-half_w, half_w] x [-half_h, half_h]
            let pos_x = -half_w + (u * width);
            let pos_z = -half_h + (v * height);

            vertices.push(Vec3::new(pos_x, 0.0, pos_z));
            normals.push(Vec3::new(0.0, 1.0, 0.0)); // All normals point up (+Y)
            tangents.push(Vec4::new(1.0, 0.0, 0.0, 1.0)); // Tangent points along +X (U direction)
            uvs.push((u, v));
        }
    }

    // Generate indices for triangles
    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let i0 = z * (subdivisions + 1) + x;
            let i1 = i0 + 1;
            let i2 = (z + 1) * (subdivisions + 1) + x;
            let i3 = i2 + 1;

            // Two triangles per quad
            // Triangle 1
            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            // Triangle 2
            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    (vertices, normals, uvs, tangents, indices)
}

// Helper functions
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

fn extract_u32(inputs: &[(String, Value)], name: &str) -> Result<u32, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.1 {
        Value::U32Val(v) => Ok(*v),
        _ => Err(ExecutionError {
            message: format!("Expected u32 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a u32 value".to_string()),
        }),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_basic_generation() {
        let inputs = vec![
            ("width".to_string(), Value::F32Val(10.0)),
            ("height".to_string(), Value::F32Val(10.0)),
            ("subdivisions".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 5);

        // Extract outputs
        let vertices = if let Value::StringListVal(v) = &result[0].1 {
            v
        } else {
            panic!("Expected StringListVal for vertices");
        };

        let tangents = if let Value::StringListVal(t) = &result[3].1 {
            t
        } else {
            panic!("Expected StringListVal for tangents");
        };

        let indices = if let Value::U32ListVal(i) = &result[4].1 {
            i
        } else {
            panic!("Expected U32ListVal for indices");
        };

        // Single subdivision: (1+1) * (1+1) = 4 vertices
        assert_eq!(vertices.len(), 4);
        assert_eq!(tangents.len(), 4);

        // Single subdivision: 1 * 1 * 2 triangles * 3 indices = 6 indices
        assert_eq!(indices.len(), 6);
    }

    #[test]
    fn test_plane_subdivided() {
        let inputs = vec![
            ("width".to_string(), Value::F32Val(10.0)),
            ("height".to_string(), Value::F32Val(10.0)),
            ("subdivisions".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs).unwrap();

        let vertices = if let Value::StringListVal(v) = &result[0].1 {
            v
        } else {
            panic!("Expected StringListVal");
        };

        let indices = if let Value::U32ListVal(i) = &result[4].1 {
            i
        } else {
            panic!("Expected U32ListVal");
        };

        // 4 subdivisions: (4+1) * (4+1) = 25 vertices
        assert_eq!(vertices.len(), 25);

        // 4 subdivisions: 4 * 4 * 2 triangles * 3 indices = 96 indices
        assert_eq!(indices.len(), 96);
    }

    #[test]
    fn test_plane_normals() {
        let inputs = vec![
            ("width".to_string(), Value::F32Val(10.0)),
            ("height".to_string(), Value::F32Val(10.0)),
            ("subdivisions".to_string(), Value::U32Val(2)),
        ];

        let result = Component::execute(inputs).unwrap();

        let normals = if let Value::StringListVal(n) = &result[1].1 {
            n
        } else {
            panic!("Expected StringListVal for normals");
        };

        // All normals should point up (+Y)
        for normal in normals {
            assert_eq!(normal, "0,1,0");
        }
    }

    #[test]
    fn test_plane_uv_coordinates() {
        let inputs = vec![
            ("width".to_string(), Value::F32Val(10.0)),
            ("height".to_string(), Value::F32Val(10.0)),
            ("subdivisions".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs).unwrap();

        let uvs = if let Value::StringListVal(u) = &result[2].1 {
            u
        } else {
            panic!("Expected StringListVal for uvs");
        };

        // Corners should have standard UVs
        assert_eq!(uvs[0], "0,0"); // Top-left
        assert_eq!(uvs[1], "1,0"); // Top-right
        assert_eq!(uvs[2], "0,1"); // Bottom-left
        assert_eq!(uvs[3], "1,1"); // Bottom-right
    }

    #[test]
    fn test_plane_invalid_width() {
        let inputs = vec![
            ("width".to_string(), Value::F32Val(-1.0)),
            ("height".to_string(), Value::F32Val(10.0)),
            ("subdivisions".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("width".to_string()));
    }

    #[test]
    fn test_plane_invalid_subdivisions() {
        let inputs = vec![
            ("width".to_string(), Value::F32Val(10.0)),
            ("height".to_string(), Value::F32Val(10.0)),
            ("subdivisions".to_string(), Value::U32Val(0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("subdivisions".to_string()));
    }

    #[test]
    fn test_plane_high_subdivisions() {
        let inputs = vec![
            ("width".to_string(), Value::F32Val(10.0)),
            ("height".to_string(), Value::F32Val(10.0)),
            ("subdivisions".to_string(), Value::U32Val(16)),
        ];

        let result = Component::execute(inputs).unwrap();

        let vertices = if let Value::StringListVal(v) = &result[0].1 {
            v
        } else {
            panic!("Expected StringListVal");
        };

        // 16 subdivisions: (16+1) * (16+1) = 289 vertices
        assert_eq!(vertices.len(), 289);
    }
}
