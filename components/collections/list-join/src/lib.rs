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
            name: "List Join".to_string(),
            version: "1.0.0".to_string(),
            description: "Joins a list of strings into a single string with a delimiter".to_string(),
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
                description: "The list of strings to join".to_string(),
            },
            PortSpec {
                name: "delimiter".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "The delimiter to insert between elements".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "The joined string".to_string(),
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

        // Extract delimiter input
        let delimiter_input = inputs
            .iter()
            .find(|(name, _)| name == "delimiter")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: delimiter".to_string(),
                input_name: Some("delimiter".to_string()),
                recovery_hint: Some("Connect a delimiter string to this input".to_string()),
            })?;

        let delimiter = match &delimiter_input.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!(
                        "Expected string for input 'delimiter', got {:?}",
                        delimiter_input.1
                    ),
                    input_name: Some("delimiter".to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        };

        // Convert all list elements to strings
        let mut strings = Vec::new();
        for (i, value) in list_values.iter().enumerate() {
            match value {
                Value::StringVal(s) => strings.push(s.clone()),
                _ => {
                    return Err(ExecutionError {
                        message: format!("List element at index {} is not a string: {:?}", i, value),
                        input_name: Some("list".to_string()),
                        recovery_hint: Some("Ensure all list elements are strings".to_string()),
                    });
                }
            }
        }

        let result = strings.join(delimiter);

        Ok(vec![("result".to_string(), Value::StringVal(result))])
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_with_comma() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![
                    Value::StringVal("apple".to_string()),
                    Value::StringVal("banana".to_string()),
                    Value::StringVal("cherry".to_string()),
                ]),
            ),
            ("delimiter".to_string(), Value::StringVal(", ".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");
        assert_eq!(
            result[0].1,
            Value::StringVal("apple, banana, cherry".to_string())
        );
    }

    #[test]
    fn test_join_with_space() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![
                    Value::StringVal("hello".to_string()),
                    Value::StringVal("world".to_string()),
                ]),
            ),
            ("delimiter".to_string(), Value::StringVal(" ".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].1,
            Value::StringVal("hello world".to_string())
        );
    }

    #[test]
    fn test_join_empty_list() {
        let inputs = vec![
            ("list".to_string(), Value::ListVal(vec![])),
            ("delimiter".to_string(), Value::StringVal(", ".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, Value::StringVal("".to_string()));
    }

    #[test]
    fn test_join_single_element() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::ListVal(vec![Value::StringVal("only".to_string())]),
            ),
            ("delimiter".to_string(), Value::StringVal(", ".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, Value::StringVal("only".to_string()));
    }
}
