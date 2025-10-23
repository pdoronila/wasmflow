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
            name: "List Get".to_string(),
            version: "1.0.0".to_string(),
            description: "Retrieves an element from a list at a specified index".to_string(),
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
                description: "The list to get element from".to_string(),
            },
            PortSpec {
                name: "index".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "The index of the element to retrieve (0-based)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "element".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "The element at the specified index".to_string(),
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

        // Extract index input
        let index_input = inputs
            .iter()
            .find(|(name, _)| name == "index")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: index".to_string(),
                input_name: Some("index".to_string()),
                recovery_hint: Some("Connect an index value to this input".to_string()),
            })?;

        let index = match &index_input.1 {
            Value::U32Val(i) => *i as usize,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected u32 for input 'index', got {:?}", index_input.1),
                    input_name: Some("index".to_string()),
                    recovery_hint: Some("Provide a positive integer value".to_string()),
                });
            }
        };

        // Check bounds
        if index >= list_values.len() {
            return Err(ExecutionError {
                message: format!(
                    "Index {} out of bounds for list of length {}",
                    index,
                    list_values.len()
                ),
                input_name: Some("index".to_string()),
                recovery_hint: Some(format!(
                    "Provide an index between 0 and {}",
                    list_values.len().saturating_sub(1)
                )),
            });
        }

        let element = list_values[index].clone();

        Ok(vec![("element".to_string(), element)])
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_element_at_index_0() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![
                    Value::StringVal("first".to_string()),
                    Value::StringVal("second".to_string()),
                    Value::StringVal("third".to_string()),
                ]),
            ),
            ("index".to_string(), Value::U32Val(0)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "element");
        assert_eq!(result[0].1, Value::StringVal("first".to_string()));
    }

    #[test]
    fn test_get_element_at_index_1() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![
                    Value::U32Val(10),
                    Value::U32Val(20),
                    Value::U32Val(30),
                ]),
            ),
            ("index".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "element");
        assert_eq!(result[0].1, Value::U32Val(20));
    }

    #[test]
    fn test_get_element_out_of_bounds() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![Value::U32Val(1), Value::U32Val(2)]),
            ),
            ("index".to_string(), Value::U32Val(5)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("out of bounds"));
    }
}
