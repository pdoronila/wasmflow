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
            name: "Vec2 Constructor".to_string(),
            version: "1.0.0".to_string(),
            description: "Create a 2D vector from X and Y components".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "x".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "X component".to_string(),
            },
            PortSpec {
                name: "y".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Y component".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "vector".to_string(),
            data_type: DataType::Vec2Type,
            optional: false,
            description: "Resulting 2D vector".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract x component
        let x = inputs
            .iter()
            .find(|(name, _)| name == "x")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: x".to_string(),
                input_name: Some("x".to_string()),
                recovery_hint: Some("Connect an f32 value to the x input".to_string()),
            })?;

        let x_val = match &x.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'x', got {:?}", x.1),
                    input_name: Some("x".to_string()),
                    recovery_hint: Some("Provide an f32 value".to_string()),
                });
            }
        };

        // Extract y component
        let y = inputs
            .iter()
            .find(|(name, _)| name == "y")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: y".to_string(),
                input_name: Some("y".to_string()),
                recovery_hint: Some("Connect an f32 value to the y input".to_string()),
            })?;

        let y_val = match &y.1 {
            Value::F32Val(v) => *v,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected f32 for 'y', got {:?}", y.1),
                    input_name: Some("y".to_string()),
                    recovery_hint: Some("Provide an f32 value".to_string()),
                });
            }
        };

        // Construct vec2
        let result = Vec2 {
            x: x_val,
            y: y_val,
        };

        Ok(vec![("vector".to_string(), Value::Vec2Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_construct_basic() {
        let inputs = vec![
            ("x".to_string(), Value::F32Val(3.0)),
            ("y".to_string(), Value::F32Val(4.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "vector");

        if let Value::Vec2Val(vec) = &result[0].1 {
            assert_eq!(vec.x, 3.0);
            assert_eq!(vec.y, 4.0);
        } else {
            panic!("Expected Vec2Val");
        }
    }

    #[test]
    fn test_vec2_construct_negative() {
        let inputs = vec![
            ("x".to_string(), Value::F32Val(-1.5)),
            ("y".to_string(), Value::F32Val(-2.5)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec2Val(vec) = &result[0].1 {
            assert_eq!(vec.x, -1.5);
            assert_eq!(vec.y, -2.5);
        } else {
            panic!("Expected Vec2Val");
        }
    }

    #[test]
    fn test_vec2_construct_zero() {
        let inputs = vec![
            ("x".to_string(), Value::F32Val(0.0)),
            ("y".to_string(), Value::F32Val(0.0)),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::Vec2Val(vec) = &result[0].1 {
            assert_eq!(vec.x, 0.0);
            assert_eq!(vec.y, 0.0);
        } else {
            panic!("Expected Vec2Val");
        }
    }

    #[test]
    fn test_vec2_construct_missing_x() {
        let inputs = vec![("y".to_string(), Value::F32Val(4.0))];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("x".to_string()));
    }

    #[test]
    fn test_vec2_construct_missing_y() {
        let inputs = vec![("x".to_string(), Value::F32Val(3.0))];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("y".to_string()));
    }

    #[test]
    fn test_vec2_construct_wrong_type() {
        let inputs = vec![
            ("x".to_string(), Value::U32Val(3)),
            ("y".to_string(), Value::F32Val(4.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
