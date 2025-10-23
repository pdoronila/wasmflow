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
            name: "List Append".to_string(),
            version: "1.0.0".to_string(),
            description: "Appends a string value to the end of a string list, creating a new list".to_string(),
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
                description: "The string list to append to".to_string(),
            },
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "The string value to append to the list".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::ListType,
            optional: false,
            description: "The new string list with the value appended".to_string(),
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

        let mut list_values = match &list.1 {
            Value::StringListVal(items) => items.clone(),
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

        let value = match &value_input.1 {
            Value::StringVal(s) => s.clone(),
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for input 'value', got {:?}", value_input.1),
                    input_name: Some("value".to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        };

        // Append value to list (immutable operation - creates new list)
        list_values.push(value);

        Ok(vec![("result".to_string(), Value::StringListVal(list_values))])
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_to_existing_list() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec![
                    "apple".to_string(),
                    "banana".to_string(),
                    "cherry".to_string(),
                ]),
            ),
            ("value".to_string(), Value::StringVal("date".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");

        if let Value::StringListVal(list) = &result[0].1 {
            assert_eq!(list.len(), 4);
            assert_eq!(list[0], "apple");
            assert_eq!(list[1], "banana");
            assert_eq!(list[2], "cherry");
            assert_eq!(list[3], "date");
        } else {
            panic!("Expected StringListVal");
        }
    }

    #[test]
    fn test_append_to_empty_list() {
        let inputs = vec![
            ("list".to_string(), Value::StringListVal(vec![])),
            ("value".to_string(), Value::StringVal("first".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");

        if let Value::StringListVal(list) = &result[0].1 {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0], "first");
        } else {
            panic!("Expected StringListVal");
        }
    }

    #[test]
    fn test_append_multiple_times() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec!["one".to_string(), "two".to_string()]),
            ),
            ("value".to_string(), Value::StringVal("three".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::StringListVal(list) = &result[0].1 {
            assert_eq!(list.len(), 3);
            assert_eq!(list[2], "three");
        } else {
            panic!("Expected StringListVal");
        }
    }
}
