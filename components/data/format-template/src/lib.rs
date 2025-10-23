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
            name: "Format Template".to_string(),
            version: "1.0.0".to_string(),
            description: "Formats a template string by replacing placeholders like {0}, {1}, {2} with values from a list".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "template".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "The template string with placeholders like {0}, {1}, {2}".to_string(),
            },
            PortSpec {
                name: "values".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "The list of string values to substitute into the template".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "The formatted string with placeholders replaced by values".to_string(),
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
        // Extract template input
        let template_input = inputs
            .iter()
            .find(|(name, _)| name == "template")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: template".to_string(),
                input_name: Some("template".to_string()),
                recovery_hint: Some("Connect a template string to this input".to_string()),
            })?;

        let template = match &template_input.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for input 'template', got {:?}", template_input.1),
                    input_name: Some("template".to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        };

        // Extract values input
        let values_input = inputs
            .iter()
            .find(|(name, _)| name == "values")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: values".to_string(),
                input_name: Some("values".to_string()),
                recovery_hint: Some("Connect a list of values to this input".to_string()),
            })?;

        let values = match &values_input.1 {
            Value::StringListVal(items) => items,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string list for input 'values', got {:?}", values_input.1),
                    input_name: Some("values".to_string()),
                    recovery_hint: Some("Provide a string list value".to_string()),
                });
            }
        };

        // Replace placeholders {0}, {1}, {2}, etc. with values
        let mut result = template.clone();
        for (i, value) in values.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            result = result.replace(&placeholder, value);
        }

        Ok(vec![("result".to_string(), Value::StringVal(result))])
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
    fn test_format_simple_template() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Hello {0}!".to_string())),
            (
                "values".to_string(),
                Value::StringListVal(vec!["World".to_string()]),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");
        assert_eq!(result[0].1, Value::StringVal("Hello World!".to_string()));
    }

    #[test]
    fn test_format_multiple_placeholders() {
        let inputs = vec![
            (
                "template".to_string(),
                Value::StringVal("Name: {0}, Age: {1}, City: {2}".to_string()),
            ),
            (
                "values".to_string(),
                Value::StringListVal(vec![
                    "Alice".to_string(),
                    "30".to_string(),
                    "New York".to_string(),
                ]),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].1, Value::StringVal("Name: Alice, Age: 30, City: New York".to_string()));
    }

    #[test]
    fn test_format_unused_placeholders() {
        let inputs = vec![
            (
                "template".to_string(),
                Value::StringVal("Hello {0}, you are {1} years old. {2}".to_string()),
            ),
            (
                "values".to_string(),
                Value::StringListVal(vec!["Bob".to_string(), "25".to_string()]),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        // {2} should remain since there's no third value
        assert_eq!(
            result[0].1,
            Value::StringVal("Hello Bob, you are 25 years old. {2}".to_string())
        );
    }

    #[test]
    fn test_format_extra_values() {
        let inputs = vec![
            (
                "template".to_string(),
                Value::StringVal("Hello {0}!".to_string()),
            ),
            (
                "values".to_string(),
                Value::StringListVal(vec![
                    "World".to_string(),
                    "Extra".to_string(),
                    "Unused".to_string(),
                ]),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        // Extra values should be ignored
        assert_eq!(result[0].1, Value::StringVal("Hello World!".to_string()));
    }

    #[test]
    fn test_format_no_placeholders() {
        let inputs = vec![
            (
                "template".to_string(),
                Value::StringVal("No placeholders here".to_string()),
            ),
            (
                "values".to_string(),
                Value::StringListVal(vec!["ignored".to_string()]),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].1, Value::StringVal("No placeholders here".to_string()));
    }

    #[test]
    fn test_format_empty_values() {
        let inputs = vec![
            (
                "template".to_string(),
                Value::StringVal("Hello {0}!".to_string()),
            ),
            ("values".to_string(), Value::StringListVal(vec![])),
        ];

        let result = Component::execute(inputs).unwrap();
        // Placeholder remains since no values provided
        assert_eq!(result[0].1, Value::StringVal("Hello {0}!".to_string()));
    }

    #[test]
    fn test_format_repeated_placeholders() {
        let inputs = vec![
            (
                "template".to_string(),
                Value::StringVal("{0} and {0} and {0}".to_string()),
            ),
            (
                "values".to_string(),
                Value::StringListVal(vec!["test".to_string()]),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        // All instances of {0} should be replaced
        assert_eq!(result[0].1, Value::StringVal("test and test and test".to_string()));
    }

    #[test]
    fn test_format_sequential_placeholders() {
        let inputs = vec![
            (
                "template".to_string(),
                Value::StringVal("{0}{1}{2}".to_string()),
            ),
            (
                "values".to_string(),
                Value::StringListVal(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].1, Value::StringVal("abc".to_string()));
    }
}

