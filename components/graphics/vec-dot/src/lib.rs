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
            name: "Vector Dot Product".to_string(),
            version: "1.0.0".to_string(),
            description: "Calculate dot product of two vectors (vec2, vec3, or vec4)".to_string(),
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
                description: "First vector".to_string(),
            },
            PortSpec {
                name: "b".to_string(),
                data_type: DataType::Vec3Type,
                optional: false,
                description: "Second vector".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Dot product (scalar)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract vectors
        let a_input = inputs.iter().find(|(name, _)| name == "a").ok_or_else(|| ExecutionError {
            message: "Missing required input: a".to_string(),
            input_name: Some("a".to_string()),
            recovery_hint: Some("Connect a vector to the 'a' input".to_string()),
        })?;

        let b_input = inputs.iter().find(|(name, _)| name == "b").ok_or_else(|| ExecutionError {
            message: "Missing required input: b".to_string(),
            input_name: Some("b".to_string()),
            recovery_hint: Some("Connect a vector to the 'b' input".to_string()),
        })?;

        // Calculate dot product based on type
        let result = match (&a_input.1, &b_input.1) {
            (Value::Vec2Val(a), Value::Vec2Val(b)) => {
                a.x * b.x + a.y * b.y
            }
            (Value::Vec3Val(a), Value::Vec3Val(b)) => {
                a.x * b.x + a.y * b.y + a.z * b.z
            }
            (Value::Vec4Val(a), Value::Vec4Val(b)) => {
                a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
            }
            _ => {
                return Err(ExecutionError {
                    message: "Vectors must be the same type (both vec2, vec3, or vec4)".to_string(),
                    input_name: Some("a".to_string()),
                    recovery_hint: Some("Ensure both inputs are the same vector type".to_string()),
                });
            }
        };

        Ok(vec![("result".to_string(), Value::F32Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_vec2() {
        let inputs = vec![
            ("a".to_string(), Value::Vec2Val(Vec2 { x: 1.0, y: 2.0 })),
            ("b".to_string(), Value::Vec2Val(Vec2 { x: 3.0, y: 4.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::F32Val(dot) = result[0].1 {
            assert_eq!(dot, 11.0); // 1*3 + 2*4 = 11
        } else {
            panic!("Expected F32Val");
        }
    }

    #[test]
    fn test_dot_vec3() {
        let inputs = vec![
            ("a".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 0.0, z: 0.0 })),
            ("b".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::F32Val(dot) = result[0].1 {
            assert_eq!(dot, 0.0); // Perpendicular vectors
        } else {
            panic!("Expected F32Val");
        }
    }

    #[test]
    fn test_dot_type_mismatch() {
        let inputs = vec![
            ("a".to_string(), Value::Vec2Val(Vec2 { x: 1.0, y: 2.0 })),
            ("b".to_string(), Value::Vec3Val(Vec3 { x: 3.0, y: 4.0, z: 5.0 })),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
