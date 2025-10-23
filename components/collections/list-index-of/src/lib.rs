wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

// ============================================================================
// Metadata Interface
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "List Index Of".to_string(),
            version: "1.0.0".to_string(),
            description: "Returns the index of the first occurrence of a value in a list, or -1 if not found".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Collections".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "list".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "The list to search in".to_string(),
            },
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "The value to search for".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "index".to_string(),
            data_type: DataType::I32Type,
            optional: false,
            description: "The index of the first occurrence (0-based), or -1 if not found".to_string(),
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
        // Extract list input
        let list = inputs
            .iter()
            .find(|(name, _)| name == "list")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: list".to_string(),
                input_name: Some("list".to_string()),
                recovery_hint: Some("Connect a list to this input".to_string()),
            })?;

        let list_values = match &list.1 {
            Value::ListVal(items) => items,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected list for input 'list', got {:?}", list.1),
                    input_name: Some("list".to_string()),
                    recovery_hint: Some("Provide a list value".to_string()),
                });
            }
        };

        // Extract value input
        let value_input = inputs
            .iter()
            .find(|(name, _)| name == "value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: value".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Connect a value to this input".to_string()),
            })?;

        let search_value = &value_input.1;

        // Find the index of the first occurrence
        let index = list_values
            .iter()
            .position(|item| values_equal(item, search_value))
            .map(|pos| pos as i32)
            .unwrap_or(-1);

        Ok(vec![("index".to_string(), Value::I32Val(index))])
    }
}

// Helper function to compare values for equality
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::U32Val(a), Value::U32Val(b)) => a == b,
        (Value::I32Val(a), Value::I32Val(b)) => a == b,
        (Value::F32Val(a), Value::F32Val(b)) => a == b,
        (Value::StringVal(a), Value::StringVal(b)) => a == b,
        (Value::BoolVal(a), Value::BoolVal(b)) => a == b,
        // Lists are equal if they have the same length and all elements are equal
        (Value::ListVal(a), Value::ListVal(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        // Different types are not equal
        _ => false,
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_of_value_found() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![
                    Value::StringVal("apple".to_string()),
                    Value::StringVal("banana".to_string()),
                    Value::StringVal("cherry".to_string()),
                ]),
            ),
            ("value".to_string(), Value::StringVal("banana".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "index");
        assert_eq!(result[0].1, Value::I32Val(1));
    }

    #[test]
    fn test_index_of_first_occurrence() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![
                    Value::U32Val(10),
                    Value::U32Val(20),
                    Value::U32Val(10),
                    Value::U32Val(30),
                ]),
            ),
            ("value".to_string(), Value::U32Val(10)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "index");
        assert_eq!(result[0].1, Value::I32Val(0)); // First occurrence at index 0
    }

    #[test]
    fn test_index_of_value_not_found() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![
                    Value::U32Val(1),
                    Value::U32Val(2),
                    Value::U32Val(3),
                ]),
            ),
            ("value".to_string(), Value::U32Val(99)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "index");
        assert_eq!(result[0].1, Value::I32Val(-1)); // Not found returns -1
    }

    #[test]
    fn test_index_of_empty_list() {
        let inputs = vec![
            ("list".to_string(), Value::ListVal(vec![])),
            ("value".to_string(), Value::StringVal("test".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].1, Value::I32Val(-1)); // Empty list returns -1
    }
}
