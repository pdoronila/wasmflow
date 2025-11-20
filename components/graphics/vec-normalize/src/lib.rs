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
            name: "Vector Normalize".to_string(),
            version: "1.0.0".to_string(),
            description: "Normalize a vector to unit length (vec2, vec3, or vec4)".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "vector".to_string(),
            data_type: DataType::Vec3Type,  // Accept vec2, vec3, or vec4
            optional: false,
            description: "Vector to normalize (vec2, vec3, or vec4)".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::Vec3Type,  // Same type as input
            optional: false,
            description: "Normalized vector (unit length)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract input vector
        let vec_input = inputs.iter().find(|(name, _)| name == "vector").ok_or_else(|| ExecutionError {
            message: "Missing required input: vector".to_string(),
            input_name: Some("vector".to_string()),
            recovery_hint: Some("Connect a vec2, vec3, or vec4 to the vector input".to_string()),
        })?;

        // Normalize based on type
        match &vec_input.1 {
            Value::Vec2Val(v) => {
                let length = (v.x * v.x + v.y * v.y).sqrt();
                if length < 1e-10 {
                    return Err(ExecutionError {
                        message: "Cannot normalize zero vector".to_string(),
                        input_name: Some("vector".to_string()),
                        recovery_hint: Some("Provide a non-zero vector".to_string()),
                    });
                }
                let result = Vec2 {
                    x: v.x / length,
                    y: v.y / length,
                };
                Ok(vec![("result".to_string(), Value::Vec2Val(result))])
            }
            Value::Vec3Val(v) => {
                let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
                if length < 1e-10 {
                    return Err(ExecutionError {
                        message: "Cannot normalize zero vector".to_string(),
                        input_name: Some("vector".to_string()),
                        recovery_hint: Some("Provide a non-zero vector".to_string()),
                    });
                }
                let result = Vec3 {
                    x: v.x / length,
                    y: v.y / length,
                    z: v.z / length,
                };
                Ok(vec![("result".to_string(), Value::Vec3Val(result))])
            }
            Value::Vec4Val(v) => {
                let length = (v.x * v.x + v.y * v.y + v.z * v.z + v.w * v.w).sqrt();
                if length < 1e-10 {
                    return Err(ExecutionError {
                        message: "Cannot normalize zero vector".to_string(),
                        input_name: Some("vector".to_string()),
                        recovery_hint: Some("Provide a non-zero vector".to_string()),
                    });
                }
                let result = Vec4 {
                    x: v.x / length,
                    y: v.y / length,
                    z: v.z / length,
                    w: v.w / length,
                };
                Ok(vec![("result".to_string(), Value::Vec4Val(result))])
            }
            _ => Err(ExecutionError {
                message: format!("Expected vec2, vec3, or vec4, got {:?}", vec_input.1),
                input_name: Some("vector".to_string()),
                recovery_hint: Some("Provide a vector value".to_string()),
            }),
        }
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_vec2() {
        let inputs = vec![
            ("vector".to_string(), Value::Vec2Val(Vec2 { x: 3.0, y: 4.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec2Val(vec) = &result[0].1 {
            assert!((vec.x - 0.6).abs() < 0.001);
            assert!((vec.y - 0.8).abs() < 0.001);
            // Verify unit length
            let length = (vec.x * vec.x + vec.y * vec.y).sqrt();
            assert!((length - 1.0).abs() < 0.001);
        } else {
            panic!("Expected Vec2Val");
        }
    }

    #[test]
    fn test_normalize_vec3() {
        let inputs = vec![
            ("vector".to_string(), Value::Vec3Val(Vec3 { x: 1.0, y: 2.0, z: 2.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec3Val(vec) = &result[0].1 {
            let length = (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt();
            assert!((length - 1.0).abs() < 0.001);
        } else {
            panic!("Expected Vec3Val");
        }
    }

    #[test]
    fn test_normalize_zero_vector() {
        let inputs = vec![
            ("vector".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 0.0, z: 0.0 })),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
