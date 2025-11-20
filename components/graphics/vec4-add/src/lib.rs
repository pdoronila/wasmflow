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
            name: "Vec4 Add".to_string(),
            version: "1.0.0".to_string(),
            description: "Add two 4D vectors component-wise".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "a".to_string(),
                data_type: DataType::Vec4Type,
                optional: false,
                description: "First vector".to_string(),
            },
            PortSpec {
                name: "b".to_string(),
                data_type: DataType::Vec4Type,
                optional: false,
                description: "Second vector".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::Vec4Type,
            optional: false,
            description: "Sum of vectors a + b".to_string(),
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
            recovery_hint: Some("Connect a vec4 to the 'a' input".to_string()),
        })?;
        let a = match &a_input.1 {
            Value::Vec4Val(v) => v,
            _ => return Err(ExecutionError {
                message: format!("Expected vec4 for 'a', got {:?}", a_input.1),
                input_name: Some("a".to_string()),
                recovery_hint: Some("Provide a vec4 value".to_string()),
            }),
        };

        // Extract vector b
        let b_input = inputs.iter().find(|(name, _)| name == "b").ok_or_else(|| ExecutionError {
            message: "Missing required input: b".to_string(),
            input_name: Some("b".to_string()),
            recovery_hint: Some("Connect a vec4 to the 'b' input".to_string()),
        })?;
        let b = match &b_input.1 {
            Value::Vec4Val(v) => v,
            _ => return Err(ExecutionError {
                message: format!("Expected vec4 for 'b', got {:?}", b_input.1),
                input_name: Some("b".to_string()),
                recovery_hint: Some("Provide a vec4 value".to_string()),
            }),
        };

        // Add vectors component-wise
        let result = Vec4 {
            x: a.x + b.x,
            y: a.y + b.y,
            z: a.z + b.z,
            w: a.w + b.w,
        };

        Ok(vec![("result".to_string(), Value::Vec4Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec4_add_basic() {
        let inputs = vec![
            ("a".to_string(), Value::Vec4Val(Vec4 { x: 1.0, y: 2.0, z: 3.0, w: 4.0 })),
            ("b".to_string(), Value::Vec4Val(Vec4 { x: 5.0, y: 6.0, z: 7.0, w: 8.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec4Val(vec) = &result[0].1 {
            assert_eq!(vec.x, 6.0);
            assert_eq!(vec.y, 8.0);
            assert_eq!(vec.z, 10.0);
            assert_eq!(vec.w, 12.0);
        } else {
            panic!("Expected Vec4Val");
        }
    }

    #[test]
    fn test_vec4_add_negative() {
        let inputs = vec![
            ("a".to_string(), Value::Vec4Val(Vec4 { x: 10.0, y: 20.0, z: 30.0, w: 40.0 })),
            ("b".to_string(), Value::Vec4Val(Vec4 { x: -5.0, y: -10.0, z: -15.0, w: -20.0 })),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec4Val(vec) = &result[0].1 {
            assert_eq!(vec.x, 5.0);
            assert_eq!(vec.y, 10.0);
            assert_eq!(vec.z, 15.0);
            assert_eq!(vec.w, 20.0);
        } else {
            panic!("Expected Vec4Val");
        }
    }
}
