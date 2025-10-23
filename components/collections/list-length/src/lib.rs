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
            name: "List Length".to_string(),
            version: "1.0.0".to_string(),
            description: "Returns the number of elements in a list".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Collections".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "list".to_string(),
            data_type: DataType::ListType,
            optional: false,
            description: "The list to get length of".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "length".to_string(),
            data_type: DataType::U32Type,
            optional: false,
            description: "The number of elements in the list".to_string(),
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

        // Extract list value
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

        let length = list_values.len() as u32;

        Ok(vec![("length".to_string(), Value::U32Val(length))])
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_length_with_elements() {
        let inputs = vec![(
            "list".to_string(),
            Value::ListVal(vec![
                Value::U32Val(1),
                Value::U32Val(2),
                Value::U32Val(3),
                Value::U32Val(4),
            ]),
        )];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "length");
        assert_eq!(result[0].1, Value::U32Val(4));
    }

    #[test]
    fn test_list_length_empty() {
        let inputs = vec![("list".to_string(), Value::ListVal(vec![]))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "length");
        assert_eq!(result[0].1, Value::U32Val(0));
    }

    #[test]
    fn test_list_length_single_element() {
        let inputs = vec![(
            "list".to_string(),
            Value::ListVal(vec![Value::StringVal("hello".to_string())]),
        )];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "length");
        assert_eq!(result[0].1, Value::U32Val(1));
    }
}
