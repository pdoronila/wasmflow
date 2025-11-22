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
use std::f32::consts::PI;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Sphere Primitive".to_string(),
            version: "1.0.0".to_string(),
            description: "Generate UV sphere mesh data with customizable radius and tessellation".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "radius".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Sphere radius".to_string(),
            },
            PortSpec {
                name: "segments".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of horizontal segments (longitude divisions)".to_string(),
            },
            PortSpec {
                name: "rings".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of vertical rings (latitude divisions)".to_string(),
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
        // Extract radius
        let radius = extract_f32(&inputs, "radius")?;
        if radius <= 0.0 {
            return Err(ExecutionError {
                message: format!("Radius must be positive, got {}", radius),
                input_name: Some("radius".to_string()),
                recovery_hint: Some("Provide a positive radius value".to_string()),
            });
        }

        // Extract segments
        let segments = extract_u32(&inputs, "segments")?;
        if segments < 3 {
            return Err(ExecutionError {
                message: format!("Segments must be at least 3, got {}", segments),
                input_name: Some("segments".to_string()),
                recovery_hint: Some("Provide at least 3 segments for a valid sphere".to_string()),
            });
        }

        // Extract rings
        let rings = extract_u32(&inputs, "rings")?;
        if rings < 2 {
            return Err(ExecutionError {
                message: format!("Rings must be at least 2, got {}", rings),
                input_name: Some("rings".to_string()),
                recovery_hint: Some("Provide at least 2 rings for a valid sphere".to_string()),
            });
        }

        // Generate sphere mesh using UV sphere algorithm
        let (vertices, normals, uvs, tangents, indices) = generate_uv_sphere(radius, segments, rings);

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

/// Generate a UV sphere mesh
fn generate_uv_sphere(
    radius: f32,
    segments: u32,
    rings: u32,
) -> (Vec<Vec3>, Vec<Vec3>, Vec<(f32, f32)>, Vec<Vec4>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut tangents = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices, normals, UVs, and tangents
    for ring in 0..=rings {
        let phi = PI * (ring as f32) / (rings as f32); // Latitude angle (0 to π)
        let y = radius * phi.cos();
        let ring_radius = radius * phi.sin();

        for segment in 0..=segments {
            let theta = 2.0 * PI * (segment as f32) / (segments as f32); // Longitude angle (0 to 2π)

            // Calculate position
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();
            let position = Vec3::new(x, y, z);

            // Normal is the normalized position for a sphere centered at origin
            let normal = position.normalize();

            // Tangent is the derivative with respect to theta (longitude)
            // ∂P/∂θ = (-r·sin(φ)·sin(θ), 0, r·sin(φ)·cos(θ))
            // Normalized: (-sin(θ), 0, cos(θ))
            let tangent = Vec3::new(-theta.sin(), 0.0, theta.cos()).normalize();
            let handedness = 1.0; // Right-handed coordinate system
            let tangent_vec4 = Vec4::new(tangent.x, tangent.y, tangent.z, handedness);

            // UV coordinates
            let u = (segment as f32) / (segments as f32);
            let v = (ring as f32) / (rings as f32);

            vertices.push(position);
            normals.push(normal);
            uvs.push((u, v));
            tangents.push(tangent_vec4);
        }
    }

    // Generate indices for triangles
    for ring in 0..rings {
        for segment in 0..segments {
            let current_row = ring * (segments + 1);
            let next_row = (ring + 1) * (segments + 1);

            let i0 = current_row + segment;
            let i1 = next_row + segment;
            let i2 = current_row + segment + 1;
            let i3 = next_row + segment + 1;

            // Two triangles per quad
            // Triangle 1
            indices.push(i0);
            indices.push(i1);
            indices.push(i2);

            // Triangle 2
            indices.push(i2);
            indices.push(i1);
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
    fn test_sphere_basic_generation() {
        let inputs = vec![
            ("radius".to_string(), Value::F32Val(1.0)),
            ("segments".to_string(), Value::U32Val(8)),
            ("rings".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 5);

        // Extract outputs
        let vertices = if let Value::StringListVal(v) = &result[0].1 {
            v
        } else {
            panic!("Expected StringListVal for vertices");
        };

        let normals = if let Value::StringListVal(n) = &result[1].1 {
            n
        } else {
            panic!("Expected StringListVal for normals");
        };

        let uvs = if let Value::StringListVal(u) = &result[2].1 {
            u
        } else {
            panic!("Expected StringListVal for uvs");
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

        // Verify vertex count: (segments + 1) * (rings + 1)
        let expected_vertices = (8 + 1) * (4 + 1);
        assert_eq!(vertices.len(), expected_vertices);
        assert_eq!(normals.len(), expected_vertices);
        assert_eq!(uvs.len(), expected_vertices);
        assert_eq!(tangents.len(), expected_vertices);

        // Verify triangle count: segments * rings * 2 triangles * 3 indices
        let expected_indices = 8 * 4 * 2 * 3;
        assert_eq!(indices.len(), expected_indices);
    }

    #[test]
    fn test_sphere_vertex_format() {
        let inputs = vec![
            ("radius".to_string(), Value::F32Val(2.0)),
            ("segments".to_string(), Value::U32Val(4)),
            ("rings".to_string(), Value::U32Val(2)),
        ];

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

    #[test]
    fn test_sphere_poles() {
        let inputs = vec![
            ("radius".to_string(), Value::F32Val(1.0)),
            ("segments".to_string(), Value::U32Val(8)),
            ("rings".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs).unwrap();

        let vertices = if let Value::StringListVal(v) = &result[0].1 {
            v
        } else {
            panic!("Expected StringListVal");
        };

        // First ring should be top pole (y = radius)
        let top_vertex = &vertices[0];
        let parts: Vec<&str> = top_vertex.split(',').collect();
        let y: f32 = parts[1].parse().unwrap();
        assert!((y - 1.0).abs() < 0.001, "Top pole should be at y=1.0");

        // Last ring should be bottom pole (y = -radius)
        let bottom_vertex = &vertices[vertices.len() - 1];
        let parts: Vec<&str> = bottom_vertex.split(',').collect();
        let y: f32 = parts[1].parse().unwrap();
        assert!((y - (-1.0)).abs() < 0.001, "Bottom pole should be at y=-1.0");
    }

    #[test]
    fn test_sphere_invalid_radius() {
        let inputs = vec![
            ("radius".to_string(), Value::F32Val(-1.0)),
            ("segments".to_string(), Value::U32Val(8)),
            ("rings".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("radius".to_string()));
    }

    #[test]
    fn test_sphere_invalid_segments() {
        let inputs = vec![
            ("radius".to_string(), Value::F32Val(1.0)),
            ("segments".to_string(), Value::U32Val(2)), // Too few
            ("rings".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("segments".to_string()));
    }

    #[test]
    fn test_sphere_high_tessellation() {
        let inputs = vec![
            ("radius".to_string(), Value::F32Val(1.0)),
            ("segments".to_string(), Value::U32Val(32)),
            ("rings".to_string(), Value::U32Val(16)),
        ];

        let result = Component::execute(inputs).unwrap();

        let indices = if let Value::U32ListVal(i) = &result[4].1 {
            i
        } else {
            panic!("Expected U32ListVal");
        };

        // Should generate many triangles
        let expected_indices = 32 * 16 * 2 * 3;
        assert_eq!(indices.len(), expected_indices);
    }
}
