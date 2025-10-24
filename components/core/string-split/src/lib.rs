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
            name: "String Split".to_string(),
            version: "1.0.0".to_string(),
            description: "Splits a string on delimiter into a list of substrings".to_string(),
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
                description: "Input string to split".to_string(),
            },
            PortSpec {
                name: "delimiter".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Delimiter to split on (empty splits into characters)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::ListType,
            optional: false,
            description: "List of substrings".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "String Split executing");

        let text = inputs
            .iter()
            .find(|(n, _)| n == "text")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide a string value".to_string()),
            })?;

        let delimiter = inputs
            .iter()
            .find(|(n, _)| n == "delimiter")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'delimiter' input".to_string(),
                input_name: Some("delimiter".to_string()),
                recovery_hint: Some("Provide a string delimiter".to_string()),
            })?;

        // Split string - handle empty delimiter (split into characters)
        let parts: Vec<String> = if delimiter.is_empty() {
            text.chars()
                .map(|c| c.to_string())
                .collect()
        } else {
            text.split(&delimiter)
                .map(|s| s.to_string())
                .collect()
        };

        Ok(vec![("result".to_string(), Value::StringListVal(parts))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_on_comma() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("a,b,c".to_string())),
            ("delimiter".to_string(), Value::StringVal(",".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringListVal(parts) => {
                assert_eq!(parts.len(), 3);
                assert_eq!(parts[0], "a");
            },
            _ => panic!("Expected string list output"),
        }
    }

    #[test]
    fn test_split_on_empty_delimiter() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("hello".to_string())),
            ("delimiter".to_string(), Value::StringVal("".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringListVal(parts) => {
                assert_eq!(parts.len(), 5); // h, e, l, l, o
            },
            _ => panic!("Expected string list output"),
        }
    }

    #[test]
    fn test_split_consecutive_delimiters() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("a,,b".to_string())),
            ("delimiter".to_string(), Value::StringVal(",".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringListVal(parts) => {
                assert_eq!(parts.len(), 3); // "a", "", "b"
            },
            _ => panic!("Expected string list output"),
        }
    }
}
