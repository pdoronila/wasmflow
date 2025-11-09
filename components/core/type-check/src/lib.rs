// Generate bindings from WIT files
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

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Type Check".to_string(),
            version: "1.0.0".to_string(),
            description: "Validates that a value matches an expected type at runtime. Returns validation result and actual type.".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Value to check".to_string(),
            },
            PortSpec {
                name: "expected-type".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Expected type: u32, i32, f32, string, bool, binary, string-list, u32-list, or f32-list".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "is-valid".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if value matches expected type".to_string(),
            },
            PortSpec {
                name: "actual-type".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Actual type of the value".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

/// Get the type name for a value
fn get_type_name(value: &Value) -> &'static str {
    match value {
        Value::U32Val(_) => "u32",
        Value::I32Val(_) => "i32",
        Value::F32Val(_) => "f32",
        Value::StringVal(_) => "string",
        Value::BoolVal(_) => "bool",
        Value::BinaryVal(_) => "binary",
        Value::StringListVal(_) => "string-list",
        Value::U32ListVal(_) => "u32-list",
        Value::F32ListVal(_) => "f32-list",
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Type Check component executing");

        // Extract value
        let value = inputs
            .iter()
            .find(|(n, _)| n == "value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: value".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Connect a value to check its type".to_string()),
            })?;

        // Extract expected type
        let expected_type = inputs
            .iter()
            .find(|(n, _)| n == "expected-type")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.as_str()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'expected-type' input".to_string(),
                input_name: Some("expected-type".to_string()),
                recovery_hint: Some("Provide a type string: u32, i32, f32, string, bool, binary, string-list, u32-list, or f32-list".to_string()),
            })?;

        // Get actual type
        let actual_type = get_type_name(&value.1);

        // Check if types match
        let is_valid = actual_type == expected_type;

        host::log("debug", &format!("Expected: {}, Actual: {}, Valid: {}", expected_type, actual_type, is_valid));

        Ok(vec![
            ("is-valid".to_string(), Value::BoolVal(is_valid)),
            ("actual-type".to_string(), Value::StringVal(actual_type.to_string())),
        ])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_u32() {
        let inputs = vec![
            ("value".to_string(), Value::U32Val(42)),
            ("expected-type".to_string(), Value::StringVal("u32".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 2);

        let is_valid = result.iter().find(|(n, _)| n == "is-valid").unwrap();
        match &is_valid.1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool"),
        }

        let actual_type = result.iter().find(|(n, _)| n == "actual-type").unwrap();
        match &actual_type.1 {
            Value::StringVal(s) => assert_eq!(s, "u32"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_invalid_type() {
        let inputs = vec![
            ("value".to_string(), Value::StringVal("hello".to_string())),
            ("expected-type".to_string(), Value::StringVal("u32".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        let is_valid = result.iter().find(|(n, _)| n == "is-valid").unwrap();
        match &is_valid.1 {
            Value::BoolVal(b) => assert_eq!(*b, false),
            _ => panic!("Expected bool"),
        }

        let actual_type = result.iter().find(|(n, _)| n == "actual-type").unwrap();
        match &actual_type.1 {
            Value::StringVal(s) => assert_eq!(s, "string"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_valid_string_list() {
        let inputs = vec![
            ("value".to_string(), Value::StringListVal(vec!["a".to_string(), "b".to_string()])),
            ("expected-type".to_string(), Value::StringVal("string-list".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        let is_valid = result.iter().find(|(n, _)| n == "is-valid").unwrap();
        match &is_valid.1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool"),
        }
    }

    #[test]
    fn test_all_types() {
        let test_cases = vec![
            (Value::U32Val(42), "u32"),
            (Value::I32Val(-42), "i32"),
            (Value::F32Val(3.14), "f32"),
            (Value::StringVal("test".to_string()), "string"),
            (Value::BoolVal(true), "bool"),
            (Value::BinaryVal(vec![1, 2, 3]), "binary"),
            (Value::StringListVal(vec!["a".to_string()]), "string-list"),
            (Value::U32ListVal(vec![1, 2, 3]), "u32-list"),
            (Value::F32ListVal(vec![1.0, 2.0]), "f32-list"),
        ];

        for (value, expected_type) in test_cases {
            let inputs = vec![
                ("value".to_string(), value),
                ("expected-type".to_string(), Value::StringVal(expected_type.to_string())),
            ];
            let result = Component::execute(inputs).unwrap();

            let is_valid = result.iter().find(|(n, _)| n == "is-valid").unwrap();
            match &is_valid.1 {
                Value::BoolVal(b) => assert_eq!(*b, true, "Failed for type {}", expected_type),
                _ => panic!("Expected bool"),
            }
        }
    }

    #[test]
    fn test_missing_value() {
        let inputs = vec![
            ("expected-type".to_string(), Value::StringVal("u32".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_expected_type() {
        let inputs = vec![
            ("value".to_string(), Value::U32Val(42)),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
