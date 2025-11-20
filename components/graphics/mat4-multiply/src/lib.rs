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
            name: "Mat4 Multiply".to_string(),
            version: "1.0.0".to_string(),
            description: "Multiply two 4x4 matrices".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "a".to_string(),
                data_type: DataType::Mat4Type,
                optional: false,
                description: "First matrix".to_string(),
            },
            PortSpec {
                name: "b".to_string(),
                data_type: DataType::Mat4Type,
                optional: false,
                description: "Second matrix".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::Mat4Type,
            optional: false,
            description: "Product matrix (a * b)".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract matrix a
        let a = extract_mat4(&inputs, "a")?;

        // Extract matrix b
        let b = extract_mat4(&inputs, "b")?;

        // Perform matrix multiplication: result = a * b
        // Standard matrix multiplication: C[i,j] = sum(A[i,k] * B[k,j] for k in 0..4)
        let result = Mat4 {
            // Row 0
            m00: a.m00 * b.m00 + a.m01 * b.m10 + a.m02 * b.m20 + a.m03 * b.m30,
            m01: a.m00 * b.m01 + a.m01 * b.m11 + a.m02 * b.m21 + a.m03 * b.m31,
            m02: a.m00 * b.m02 + a.m01 * b.m12 + a.m02 * b.m22 + a.m03 * b.m32,
            m03: a.m00 * b.m03 + a.m01 * b.m13 + a.m02 * b.m23 + a.m03 * b.m33,

            // Row 1
            m10: a.m10 * b.m00 + a.m11 * b.m10 + a.m12 * b.m20 + a.m13 * b.m30,
            m11: a.m10 * b.m01 + a.m11 * b.m11 + a.m12 * b.m21 + a.m13 * b.m31,
            m12: a.m10 * b.m02 + a.m11 * b.m12 + a.m12 * b.m22 + a.m13 * b.m32,
            m13: a.m10 * b.m03 + a.m11 * b.m13 + a.m12 * b.m23 + a.m13 * b.m33,

            // Row 2
            m20: a.m20 * b.m00 + a.m21 * b.m10 + a.m22 * b.m20 + a.m23 * b.m30,
            m21: a.m20 * b.m01 + a.m21 * b.m11 + a.m22 * b.m21 + a.m23 * b.m31,
            m22: a.m20 * b.m02 + a.m21 * b.m12 + a.m22 * b.m22 + a.m23 * b.m32,
            m23: a.m20 * b.m03 + a.m21 * b.m13 + a.m22 * b.m23 + a.m23 * b.m33,

            // Row 3
            m30: a.m30 * b.m00 + a.m31 * b.m10 + a.m32 * b.m20 + a.m33 * b.m30,
            m31: a.m30 * b.m01 + a.m31 * b.m11 + a.m32 * b.m21 + a.m33 * b.m31,
            m32: a.m30 * b.m02 + a.m31 * b.m12 + a.m32 * b.m22 + a.m33 * b.m32,
            m33: a.m30 * b.m03 + a.m31 * b.m13 + a.m32 * b.m23 + a.m33 * b.m33,
        };

        Ok(vec![("result".to_string(), Value::Mat4Val(result))])
    }
}

// Helper function
fn extract_mat4(inputs: &[(String, Value)], name: &str) -> Result<Mat4, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a mat4 value to this input".to_string()),
        })?;

    match &input.1 {
        Value::Mat4Val(v) => Ok(v.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected mat4 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a mat4 value".to_string()),
        }),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_matrix() -> Mat4 {
        Mat4 {
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m03: 0.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        }
    }

    #[test]
    fn test_mat4_multiply_identity() {
        let identity = identity_matrix();

        let inputs = vec![
            ("a".to_string(), Value::Mat4Val(identity.clone())),
            ("b".to_string(), Value::Mat4Val(identity.clone())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");

        if let Value::Mat4Val(mat) = &result[0].1 {
            assert_eq!(mat.m00, 1.0);
            assert_eq!(mat.m11, 1.0);
            assert_eq!(mat.m22, 1.0);
            assert_eq!(mat.m33, 1.0);
            assert_eq!(mat.m01, 0.0);
            assert_eq!(mat.m10, 0.0);
        } else {
            panic!("Expected Mat4Val");
        }
    }

    #[test]
    fn test_mat4_multiply_scale() {
        let identity = identity_matrix();

        let scale = Mat4 {
            m00: 2.0,
            m01: 0.0,
            m02: 0.0,
            m03: 0.0,
            m10: 0.0,
            m11: 3.0,
            m12: 0.0,
            m13: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 4.0,
            m23: 0.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        };

        let inputs = vec![
            ("a".to_string(), Value::Mat4Val(identity)),
            ("b".to_string(), Value::Mat4Val(scale.clone())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::Mat4Val(mat) = &result[0].1 {
            assert_eq!(mat.m00, 2.0);
            assert_eq!(mat.m11, 3.0);
            assert_eq!(mat.m22, 4.0);
            assert_eq!(mat.m33, 1.0);
        } else {
            panic!("Expected Mat4Val");
        }
    }

    #[test]
    fn test_mat4_multiply_translation() {
        let identity = identity_matrix();

        let translation = Mat4 {
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m03: 5.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
            m13: 10.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 15.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        };

        let inputs = vec![
            ("a".to_string(), Value::Mat4Val(identity)),
            ("b".to_string(), Value::Mat4Val(translation.clone())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::Mat4Val(mat) = &result[0].1 {
            assert_eq!(mat.m03, 5.0);
            assert_eq!(mat.m13, 10.0);
            assert_eq!(mat.m23, 15.0);
            assert_eq!(mat.m33, 1.0);
        } else {
            panic!("Expected Mat4Val");
        }
    }

    #[test]
    fn test_mat4_multiply_missing_input() {
        let identity = identity_matrix();

        let inputs = vec![("a".to_string(), Value::Mat4Val(identity))];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("b".to_string()));
    }

    #[test]
    fn test_mat4_multiply_composition() {
        // Test that (scale * translation) is computed correctly
        let scale = Mat4 {
            m00: 2.0,
            m01: 0.0,
            m02: 0.0,
            m03: 0.0,
            m10: 0.0,
            m11: 2.0,
            m12: 0.0,
            m13: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 2.0,
            m23: 0.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        };

        let translation = Mat4 {
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m03: 10.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
            m13: 20.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 30.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        };

        let inputs = vec![
            ("a".to_string(), Value::Mat4Val(scale)),
            ("b".to_string(), Value::Mat4Val(translation)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::Mat4Val(mat) = &result[0].1 {
            // Scale matrix * translation matrix
            assert_eq!(mat.m00, 2.0); // Scale preserved
            assert_eq!(mat.m11, 2.0);
            assert_eq!(mat.m22, 2.0);
            assert_eq!(mat.m03, 20.0); // Translation scaled: 2 * 10
            assert_eq!(mat.m13, 40.0); // 2 * 20
            assert_eq!(mat.m23, 60.0); // 2 * 30
        } else {
            panic!("Expected Mat4Val");
        }
    }
}
