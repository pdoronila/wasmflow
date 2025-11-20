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
            name: "Vector Cross Product".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculate cross product of two 3D vectors".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "a".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "First 3D vector".to_string(),
            },
            PortSpec {
                name: "b".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Second 3D vector".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::Vec3Type,
            optional: false,
            description: "Cross product (perpendicular vector)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract vector a
        let a_input = inputs.iter().find(|(name, _)| name == "a").ok_or_else(|| ExecutionError {
            message: "Missing required input: a".to_string(),
            input_name: Some("a".to_string()),
            recovery_hint: Some("Connect a vec3 to the 'a' input".to_string()),
        })?;
        let a = match &a_input.1 {
            Value::Vec3Val(v) => v,
            _ => return Err(ExecutionError {
                message: format!("Expected vec3 for 'a', got {:?}", a_input.1),
                input_name: Some("a".to_string()),
                recovery_hint: Some("Provide a vec3 value".to_string()),
            }),
        };

        // Extract vector b
        let b_input = inputs.iter().find(|(name, _)| name == "b").ok_or_else(|| ExecutionError {
            message: "Missing required input: b".to_string(),
            input_name: Some("b".to_string()),
            recovery_hint: Some("Connect a vec3 to the 'b' input".to_string()),
        })?;
        let b = match &b_input.1 {
            Value::Vec3Val(v) => v,
            _ => return Err(ExecutionError {
                message: format!("Expected vec3 for 'b', got {:?}", b_input.1),
                input_name: Some("b".to_string()),
                recovery_hint: Some("Provide a vec3 value".to_string()),
            }),
        };

        // Calculate cross product: a × b
        // Formula: (a.y*b.z - a.z*b.y, a.z*b.x - a.x*b.z, a.x*b.y - a.y*b.x)
        let result = Vec3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        };

        Ok(vec![("result".to_string(), Value::Vec3Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_product_basic() {
        // X × Y = Z
        let inputs = vec![
            ("a".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 0.0, z: 0.0 })),
            ("b".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(vec) = &result[0].1 {
            assert_eq!(vec.x, 0.0);
            assert_eq!(vec.y, 0.0);
            assert_eq!(vec.z, 1.0);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_cross_product_reverse() {
        // Y × X = -Z
        let inputs = vec![
            ("a".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            ("b".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 0.0, z: 0.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(vec) = &result[0].1 {
            assert_eq!(vec.x, 0.0);
            assert_eq!(vec.y, 0.0);
            assert_eq!(vec.z, -1.0);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_cross_product_parallel() {
        // Parallel vectors have zero cross product
        let inputs = vec![
            ("a".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 2.0, z: 3.0 })),
            ("b".to_string(), Value::Vec3Val(Vec3 { x: 2.0, y: 4.0, z: 6.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(vec) = &result[0].1 {
            assert!((vec.x).abs() < 0.001);
            assert!((vec.y).abs() < 0.001);
            assert!((vec.z).abs() < 0.001);
        } else {
            panic!("Expected Vec3Val");
        }
    }
}
