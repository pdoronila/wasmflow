// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "./wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "String Contains".to_string(),
            version: "1.0.0".to_string(),
            description: "Checks if a string contains a substring (case-sensitive)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Core".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "text".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Input string to search in".to_string(),
            },
            PortSpec {
                name: "substring".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Substring to search for".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "True if substring is found, false otherwise".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "String Contains executing");

        let text = inputs
            .iter()
            .find(|(n, _)| n == "text")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide a string value".to_string()),
            })?;

        let substring = inputs
            .iter()
            .find(|(n, _)| n == "substring")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'substring' input".to_string(),
                input_name: Some("substring".to_string()),
                recovery_hint: Some("Provide a string value to search for".to_string()),
            })?;

        let result = text.contains(&substring);

        Ok(vec![("result".to_string(), Value::BoolVal(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_found() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Hello World".to_string())),
            ("substring".to_string(), Value::StringVal("World".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_contains_case_sensitive() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Hello World".to_string())),
            ("substring".to_string(), Value::StringVal("world".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, false), // Case sensitive
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_contains_empty_substring() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Hello".to_string())),
            ("substring".to_string(), Value::StringVal("".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true), // Empty string is always contained
            _ => panic!("Expected bool output"),
        }
    }
}
