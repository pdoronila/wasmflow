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
            name: "Parse Number".to_string(),
            version: "1.0.0".to_string(),
            description: "Parses a string into a floating-point number (f32)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "text".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "The string to parse as a number (supports integers, decimals, and scientific notation)".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "number".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "The parsed number as f32".to_string(),
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
        // Extract text input
        let text_input = inputs
            .iter()
            .find(|(name, _)| name == "text")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: text".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Connect a text value to this input".to_string()),
            })?;

        let text = match &text_input.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for input 'text', got {:?}", text_input.1),
                    input_name: Some("text".to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        };

        // Parse the string to f32
        let number = text.trim().parse::<f32>().map_err(|e| ExecutionError {
            message: format!("Failed to parse '{}' as a number: {}", text, e),
            input_name: Some("text".to_string()),
            recovery_hint: Some("Provide a valid number string (e.g., '42', '3.14', '1.5e2')".to_string()),
        })?;

        Ok(vec![("number".to_string(), Value::F32Val(number))])
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
    fn test_parse_integer() {
        let inputs = vec![("text".to_string(), Value::StringVal("42".to_string()))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "number");
        assert_eq!(result[0].1, Value::F32Val(42.0));
    }

    #[test]
    fn test_parse_negative() {
        let inputs = vec![("text".to_string(), Value::StringVal("-123".to_string()))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "number");
        assert_eq!(result[0].1, Value::F32Val(-123.0));
    }

    #[test]
    fn test_parse_decimal() {
        let inputs = vec![("text".to_string(), Value::StringVal("3.14".to_string()))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "number");

        if let Value::F32Val(n) = result[0].1 {
            assert!((n - 3.14).abs() < 0.001);
        } else {
            panic!("Expected F32Val");
        }
    }

    #[test]
    fn test_parse_scientific_notation() {
        let inputs = vec![("text".to_string(), Value::StringVal("1.5e2".to_string()))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::F32Val(n) = result[0].1 {
            assert!((n - 150.0).abs() < 0.001);
        } else {
            panic!("Expected F32Val");
        }
    }

    #[test]
    fn test_parse_with_whitespace() {
        let inputs = vec![("text".to_string(), Value::StringVal("  42.5  ".to_string()))];

        let result = Component::execute(inputs).unwrap();

        if let Value::F32Val(n) = result[0].1 {
            assert!((n - 42.5).abs() < 0.001);
        } else {
            panic!("Expected F32Val");
        }
    }

    #[test]
    fn test_parse_zero() {
        let inputs = vec![("text".to_string(), Value::StringVal("0".to_string()))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].1, Value::F32Val(0.0));
    }

    #[test]
    fn test_parse_invalid_string() {
        let inputs = vec![("text".to_string(), Value::StringVal("not a number".to_string()))];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.message.contains("Failed to parse"));
    }

    #[test]
    fn test_parse_empty_string() {
        let inputs = vec![("text".to_string(), Value::StringVal("".to_string()))];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_partial_number() {
        let inputs = vec![("text".to_string(), Value::StringVal("42abc".to_string()))];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}

