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

// Implement the metadata interface
impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "String Concat".to_string(),
            version: "1.0.0".to_string(),
            description: "Joins multiple strings into a single string".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Core".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "text1".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "First string".to_string(),
            },
            PortSpec {
                name: "text2".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Second string".to_string(),
            },
            PortSpec {
                name: "text3".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Third string (optional)".to_string(),
            },
            PortSpec {
                name: "text4".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Fourth string (optional)".to_string(),
            },
            PortSpec {
                name: "text5".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Fifth string (optional)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Concatenated string".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None // Pure computation - no special capabilities needed
    }
}

// Implement the execution interface
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "String Concat executing");

        // Extract all string inputs (required and optional)
        let mut strings = Vec::new();

        for name in &["text1", "text2", "text3", "text4", "text5"] {
            if let Some((_, Value::StringVal(s))) = inputs.iter().find(|(n, _)| n == name) {
                strings.push(s.clone());
            } else if name == &"text1" || name == &"text2" {
                // Required inputs
                return Err(ExecutionError {
                    message: format!("Missing required input '{}'", name),
                    input_name: Some(name.to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        }

        // Concatenate all strings
        let result = strings.join("");

        // Return output
        Ok(vec![("result".to_string(), Value::StringVal(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_two_strings() {
        let inputs = vec![
            ("text1".to_string(), Value::StringVal("Hello".to_string())),
            ("text2".to_string(), Value::StringVal(" World".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "result");

        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "Hello World"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_concat_multiple_strings() {
        let inputs = vec![
            ("text1".to_string(), Value::StringVal("a".to_string())),
            ("text2".to_string(), Value::StringVal("b".to_string())),
            ("text3".to_string(), Value::StringVal("c".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "abc"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_concat_with_empty_string() {
        let inputs = vec![
            ("text1".to_string(), Value::StringVal("".to_string())),
            ("text2".to_string(), Value::StringVal("test".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string output"),
        }
    }
}
