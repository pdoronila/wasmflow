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
            name: "List Slice".to_string(),
            version: "1.0.0".to_string(),
            description: "Extracts a portion of a list from start index to end index".to_string(),
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
                description: "The list to slice".to_string(),
            },
            PortSpec {
                name: "start".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "The starting index (inclusive)".to_string(),
            },
            PortSpec {
                name: "end".to_string(),
                data_type: DataType::U32Type,
                optional: true,
                description: "The ending index (exclusive). If not provided, slices to end of list".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::ListType,
            optional: false,
            description: "The sliced portion of the list".to_string(),
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
                    recovery_hint: Some("Provide a list value".to_string()),
                });
            }
        };

        // Extract start input
        let start_input = inputs
            .iter()
            .find(|(name, _)| name == "start")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: start".to_string(),
                input_name: Some("start".to_string()),
                recovery_hint: Some("Connect a start index to this input".to_string()),
            })?;

        let start = match &start_input.1 {
            Value::U32Val(i) => *i as usize,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected u32 for input 'start', got {:?}", start_input.1),
                    input_name: Some("start".to_string()),
                    recovery_hint: Some("Provide a positive integer value".to_string()),
                });
            }
        };

        // Extract optional end input
        let end = if let Some(end_input) = inputs.iter().find(|(name, _)| name == "end") {
            match &end_input.1 {
                Value::U32Val(i) => *i as usize,
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected u32 for input 'end', got {:?}", end_input.1),
                        input_name: Some("end".to_string()),
                        recovery_hint: Some("Provide a positive integer value".to_string()),
                    });
                }
            }
        } else {
            list_values.len()
        };

        // Perform slicing
        let start_clamped = start.min(list_values.len());
        let end_clamped = end.min(list_values.len());

        let sliced = if start_clamped >= end_clamped {
            vec![]
        } else {
            list_values[start_clamped..end_clamped].to_vec()
        };

        Ok(vec![("result".to_string(), Value::StringListVal(sliced))])
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
    fn test_slice_with_start_and_end() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec![
                    "apple".to_string(),
                    "banana".to_string(),
                    "cherry".to_string(),
                    "date".to_string(),
                    "elderberry".to_string(),
                ]),
            ),
            ("start".to_string(), Value::U32Val(1)),
            ("end".to_string(), Value::U32Val(4)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");

        if let Value::StringListVal(list) = &result[0].1 {
            assert_eq!(list.len(), 3);
            assert_eq!(list[0], "banana");
            assert_eq!(list[1], "cherry");
            assert_eq!(list[2], "date");
        } else {
            panic!("Expected StringListVal");
        }
    }

    #[test]
    fn test_slice_from_start_to_end_of_list() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec![
                    "a".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                ]),
            ),
            ("start".to_string(), Value::U32Val(1)),
            // No end provided - should go to end of list
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::StringListVal(list) = &result[0].1 {
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], "b");
            assert_eq!(list[1], "c");
        } else {
            panic!("Expected StringListVal");
        }
    }

    #[test]
    fn test_slice_start_beyond_end() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec!["one".to_string(), "two".to_string()]),
            ),
            ("start".to_string(), Value::U32Val(5)),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::StringListVal(list) = &result[0].1 {
            assert_eq!(list.len(), 0); // Empty list
        } else {
            panic!("Expected StringListVal");
        }
    }

    #[test]
    fn test_slice_start_greater_than_end() {
        let inputs = vec![
            (
                "list".to_string(),
                Value::StringListVal(vec![
                    "one".to_string(),
                    "two".to_string(),
                    "three".to_string(),
                ]),
            ),
            ("start".to_string(), Value::U32Val(2)),
            ("end".to_string(), Value::U32Val(1)),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringListVal(list) = &result[0].1 {
            assert_eq!(list.len(), 0); // Empty list when start > end
        } else {
            panic!("Expected StringListVal");
        }
    }
}

