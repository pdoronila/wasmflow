wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

// ============================================================================
// Metadata Interface
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "To String".to_string(),
            version: "1.0.0".to_string(),
            description: "Converts any primitive value to its string representation".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "value".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "The value to convert to string (number, boolean, or string)".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "text".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "The string representation of the input value".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}


// ============================================================================
// Execution Interface
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract value input
        let value = inputs
            .iter()
            .find(|(name, _)| name == "value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: value".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Connect a value to this input".to_string()),
            })?;

        // Convert Value to string
        let text = match &value.1 {
            Value::U32Val(n) => n.to_string(),
            Value::I32Val(n) => n.to_string(),
            Value::F32Val(n) => n.to_string(),
            Value::StringVal(s) => s.clone(),
            Value::BoolVal(b) => b.to_string(),
            Value::BinaryVal(_) => {
                return Err(ExecutionError {
                    message: "Cannot convert binary data to string".to_string(),
                    input_name: Some("value".to_string()),
                    recovery_hint: Some("Use a primitive value (number, boolean, or string)".to_string()),
                });
            }
            Value::StringListVal(_) | Value::U32ListVal(_) | Value::F32ListVal(_) => {
                return Err(ExecutionError {
                    message: "Cannot convert list to string".to_string(),
                    input_name: Some("value".to_string()),
                    recovery_hint: Some("Use a primitive value (number, boolean, or string). For lists, use json-stringify or list-join".to_string()),
                });
            }
        };

        Ok(vec![("text".to_string(), Value::StringVal(text))])
    }
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_to_string() {
        let inputs = vec![("value".to_string(), Value::U32Val(42))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "text");
        assert_eq!(result[0].1, Value::StringVal("42".to_string()));
    }

    #[test]
    fn test_i32_to_string() {
        let inputs = vec![("value".to_string(), Value::I32Val(-123))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "text");
        assert_eq!(result[0].1, Value::StringVal("-123".to_string()));
    }

    #[test]
    fn test_f32_to_string() {
        let inputs = vec![("value".to_string(), Value::F32Val(3.14))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "text");
        assert_eq!(result[0].1, Value::StringVal("3.14".to_string()));
    }

    #[test]
    fn test_bool_to_string() {
        let inputs = vec![("value".to_string(), Value::BoolVal(true))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "text");
        assert_eq!(result[0].1, Value::StringVal("true".to_string()));

        let inputs_false = vec![("value".to_string(), Value::BoolVal(false))];
        let result_false = Component::execute(inputs_false).unwrap();
        assert_eq!(result_false[0].1, Value::StringVal("false".to_string()));
    }

    #[test]
    fn test_string_passthrough() {
        let inputs = vec![("value".to_string(), Value::StringVal("hello".to_string()))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "text");
        assert_eq!(result[0].1, Value::StringVal("hello".to_string()));
    }

    #[test]
    fn test_binary_error() {
        let inputs = vec![("value".to_string(), Value::BinaryVal(vec![1, 2, 3]))];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.message.contains("binary"));
    }

    #[test]
    fn test_list_error() {
        let inputs = vec![(
            "value".to_string(),
            Value::StringListVal(vec!["a".to_string(), "b".to_string()]),
        )];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.message.contains("list"));
    }

    #[test]
    fn test_zero_to_string() {
        let inputs = vec![("value".to_string(), Value::U32Val(0))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].1, Value::StringVal("0".to_string()));
    }
}

