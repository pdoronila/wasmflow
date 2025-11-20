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
            name: "Mat4 Constructor".to_string(),
            version: "1.0.0".to_string(),
            description: "Create a 4x4 matrix from 16 components or 4 column vectors".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            // Column-based input (Option B) - optional
            PortSpec {
                name: "col0".to_string(),
                data_type: DataType::Vec4Type,
                optional: true,
                description: "Column 0 vector (if using column mode)".to_string(),
            },
            PortSpec {
                name: "col1".to_string(),
                data_type: DataType::Vec4Type,
                optional: true,
                description: "Column 1 vector (if using column mode)".to_string(),
            },
            PortSpec {
                name: "col2".to_string(),
                data_type: DataType::Vec4Type,
                optional: true,
                description: "Column 2 vector (if using column mode)".to_string(),
            },
            PortSpec {
                name: "col3".to_string(),
                data_type: DataType::Vec4Type,
                optional: true,
                description: "Column 3 vector (if using column mode)".to_string(),
            },
            // Component-based input (Option A) - optional
            PortSpec { name: "m00".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [0,0]".to_string() },
            PortSpec { name: "m01".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [0,1]".to_string() },
            PortSpec { name: "m02".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [0,2]".to_string() },
            PortSpec { name: "m03".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [0,3]".to_string() },
            PortSpec { name: "m10".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [1,0]".to_string() },
            PortSpec { name: "m11".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [1,1]".to_string() },
            PortSpec { name: "m12".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [1,2]".to_string() },
            PortSpec { name: "m13".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [1,3]".to_string() },
            PortSpec { name: "m20".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [2,0]".to_string() },
            PortSpec { name: "m21".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [2,1]".to_string() },
            PortSpec { name: "m22".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [2,2]".to_string() },
            PortSpec { name: "m23".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [2,3]".to_string() },
            PortSpec { name: "m30".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [3,0]".to_string() },
            PortSpec { name: "m31".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [3,1]".to_string() },
            PortSpec { name: "m32".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [3,2]".to_string() },
            PortSpec { name: "m33".to_string(), data_type: DataType::F32Type, optional: true, description: "Matrix element [3,3]".to_string() },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "matrix".to_string(),
            data_type: DataType::Mat4Type,
            optional: false,
            description: "Resulting 4x4 matrix".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Check if column-based inputs are provided
        let col0 = inputs.iter().find(|(name, _)| name == "col0");
        let col1 = inputs.iter().find(|(name, _)| name == "col1");
        let col2 = inputs.iter().find(|(name, _)| name == "col2");
        let col3 = inputs.iter().find(|(name, _)| name == "col3");

        if col0.is_some() || col1.is_some() || col2.is_some() || col3.is_some() {
            // Column-based mode
            let c0 = extract_vec4(&inputs, "col0")?;
            let c1 = extract_vec4(&inputs, "col1")?;
            let c2 = extract_vec4(&inputs, "col2")?;
            let c3 = extract_vec4(&inputs, "col3")?;

            let matrix = Mat4 {
                m00: c0.x,
                m01: c0.y,
                m02: c0.z,
                m03: c0.w,
                m10: c1.x,
                m11: c1.y,
                m12: c1.z,
                m13: c1.w,
                m20: c2.x,
                m21: c2.y,
                m22: c2.z,
                m23: c2.w,
                m30: c3.x,
                m31: c3.y,
                m32: c3.z,
                m33: c3.w,
            };

            return Ok(vec![("matrix".to_string(), Value::Mat4Val(matrix))]);
        }

        // Component-based mode - extract all 16 components
        let m00 = extract_f32(&inputs, "m00")?;
        let m01 = extract_f32(&inputs, "m01")?;
        let m02 = extract_f32(&inputs, "m02")?;
        let m03 = extract_f32(&inputs, "m03")?;
        let m10 = extract_f32(&inputs, "m10")?;
        let m11 = extract_f32(&inputs, "m11")?;
        let m12 = extract_f32(&inputs, "m12")?;
        let m13 = extract_f32(&inputs, "m13")?;
        let m20 = extract_f32(&inputs, "m20")?;
        let m21 = extract_f32(&inputs, "m21")?;
        let m22 = extract_f32(&inputs, "m22")?;
        let m23 = extract_f32(&inputs, "m23")?;
        let m30 = extract_f32(&inputs, "m30")?;
        let m31 = extract_f32(&inputs, "m31")?;
        let m32 = extract_f32(&inputs, "m32")?;
        let m33 = extract_f32(&inputs, "m33")?;

        let matrix = Mat4 {
            m00,
            m01,
            m02,
            m03,
            m10,
            m11,
            m12,
            m13,
            m20,
            m21,
            m22,
            m23,
            m30,
            m31,
            m32,
            m33,
        };

        Ok(vec![("matrix".to_string(), Value::Mat4Val(matrix))])
    }
}

// Helper functions
fn extract_f32(inputs: &[(String, Value)], name: &str) -> Result<f32, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(
                "Provide either all 16 matrix components (m00-m33) or 4 column vectors (col0-col3)"
                    .to_string(),
            ),
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

fn extract_vec4(inputs: &[(String, Value)], name: &str) -> Result<Vec4, ExecutionError> {
    let input = inputs
        .iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("All 4 column vectors (col0-col3) are required in column mode".to_string()),
        })?;

    match &input.1 {
        Value::Vec4Val(v) => Ok(v.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected vec4 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a vec4 value".to_string()),
        }),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mat4_construct_from_components() {
        let inputs = vec![
            ("m00".to_string(), Value::F32Val(1.0)),
            ("m01".to_string(), Value::F32Val(0.0)),
            ("m02".to_string(), Value::F32Val(0.0)),
            ("m03".to_string(), Value::F32Val(0.0)),
            ("m10".to_string(), Value::F32Val(0.0)),
            ("m11".to_string(), Value::F32Val(1.0)),
            ("m12".to_string(), Value::F32Val(0.0)),
            ("m13".to_string(), Value::F32Val(0.0)),
            ("m20".to_string(), Value::F32Val(0.0)),
            ("m21".to_string(), Value::F32Val(0.0)),
            ("m22".to_string(), Value::F32Val(1.0)),
            ("m23".to_string(), Value::F32Val(0.0)),
            ("m30".to_string(), Value::F32Val(0.0)),
            ("m31".to_string(), Value::F32Val(0.0)),
            ("m32".to_string(), Value::F32Val(0.0)),
            ("m33".to_string(), Value::F32Val(1.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "matrix");

        if let Value::Mat4Val(mat) = &result[0].1 {
            // Check identity matrix
            assert_eq!(mat.m00, 1.0);
            assert_eq!(mat.m11, 1.0);
            assert_eq!(mat.m22, 1.0);
            assert_eq!(mat.m33, 1.0);
            assert_eq!(mat.m01, 0.0);
            assert_eq!(mat.m02, 0.0);
        } else {
            panic!("Expected Mat4Val");
        }
    }

    #[test]
    fn test_mat4_construct_from_columns() {
        let inputs = vec![
            (
                "col0".to_string(),
                Value::Vec4Val(Vec4 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0,
                }),
            ),
            (
                "col1".to_string(),
                Value::Vec4Val(Vec4 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                    w: 0.0,
                }),
            ),
            (
                "col2".to_string(),
                Value::Vec4Val(Vec4 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                    w: 0.0,
                }),
            ),
            (
                "col3".to_string(),
                Value::Vec4Val(Vec4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                }),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Mat4Val(mat) = &result[0].1 {
            // Check identity matrix
            assert_eq!(mat.m00, 1.0);
            assert_eq!(mat.m11, 1.0);
            assert_eq!(mat.m22, 1.0);
            assert_eq!(mat.m33, 1.0);
        } else {
            panic!("Expected Mat4Val");
        }
    }

    #[test]
    fn test_mat4_construct_missing_components() {
        let inputs = vec![
            ("m00".to_string(), Value::F32Val(1.0)),
            ("m01".to_string(), Value::F32Val(0.0)),
            // Missing other components
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_mat4_construct_custom_values() {
        let inputs = vec![
            ("m00".to_string(), Value::F32Val(2.0)),
            ("m01".to_string(), Value::F32Val(0.0)),
            ("m02".to_string(), Value::F32Val(0.0)),
            ("m03".to_string(), Value::F32Val(5.0)),
            ("m10".to_string(), Value::F32Val(0.0)),
            ("m11".to_string(), Value::F32Val(3.0)),
            ("m12".to_string(), Value::F32Val(0.0)),
            ("m13".to_string(), Value::F32Val(6.0)),
            ("m20".to_string(), Value::F32Val(0.0)),
            ("m21".to_string(), Value::F32Val(0.0)),
            ("m22".to_string(), Value::F32Val(4.0)),
            ("m23".to_string(), Value::F32Val(7.0)),
            ("m30".to_string(), Value::F32Val(0.0)),
            ("m31".to_string(), Value::F32Val(0.0)),
            ("m32".to_string(), Value::F32Val(0.0)),
            ("m33".to_string(), Value::F32Val(1.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Mat4Val(mat) = &result[0].1 {
            assert_eq!(mat.m00, 2.0);
            assert_eq!(mat.m11, 3.0);
            assert_eq!(mat.m22, 4.0);
            assert_eq!(mat.m03, 5.0);
            assert_eq!(mat.m13, 6.0);
            assert_eq!(mat.m23, 7.0);
        } else {
            panic!("Expected Mat4Val");
        }
    }
}
