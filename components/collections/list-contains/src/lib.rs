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
            name: "List Contains".to_string(),
            version: "1.0.0".to_string(),
            description: "Checks if a string list contains a specific value".to_string(),
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
                description: "The string list to search in".to_string(),
            },
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "The string value to search for".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "True if the value is found in the list, false otherwise".to_string(),
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
            Value::StringListVal(items) => items,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string list for input 'list', got {:?}", list.1),
                    input_name: Some("list".to_string()),
                    recovery_hint: Some("Provide a string list value".to_string()),
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

        let search_value = match &value_input.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for input 'value', got {:?}", value_input.1),
                    input_name: Some("value".to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        };

        // Check if value is in list
        let contains = list_values.iter().any(|item| item == search_value);

        Ok(vec![("result".to_string(), Value::BoolVal(contains))])
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_value_found() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec![
                    "apple".to_string(),
                    "banana".to_string(),
                    "cherry".to_string(),
                ]),
            ),
            ("value".to_string(), Value::StringVal("banana".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");
        assert_eq!(result[0].1, Value::BoolVal(true));
    }

    #[test]
    fn test_contains_value_not_found() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec![
                    "apple".to_string(),
                    "banana".to_string(),
                ]),
            ),
            ("value".to_string(), Value::StringVal("cherry".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");
        assert_eq!(result[0].1, Value::BoolVal(false));
    }

    #[test]
    fn test_contains_empty_list() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![])),
            ("value".to_string(), Value::StringVal("test".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, Value::BoolVal(false));
    }

    #[test]
    fn test_contains_first_element() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec![
                    "first".to_string(),
                    "second".to_string(),
                    "third".to_string(),
                ]),
            ),
            ("value".to_string(), Value::StringVal("first".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].1, Value::BoolVal(true));
    }
}
