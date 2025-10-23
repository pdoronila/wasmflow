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
            name: "String Trim".to_string(),
            version: "1.0.0".to_string(),
            description: "Removes leading and trailing whitespace from a string".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Core".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "text".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "String to trim".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Trimmed string".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "String Trim executing");

        let text = inputs
            .iter()
            .find(|(n, _)| n == "text")
            .and_then(|(_, v)| {
                if let Value::StringVal(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide a string value".to_string()),
            })?;

        Ok(vec![("result".to_string(), Value::StringVal(text.trim().to_string()))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_spaces() {
        let inputs = vec![("text".to_string(), Value::StringVal("  hello  ".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_trim_tabs_newlines() {
        let inputs = vec![("text".to_string(), Value::StringVal("\t\nhello\t\n".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_trim_middle_whitespace() {
        let inputs = vec![("text".to_string(), Value::StringVal("  hello world  ".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string output"),
        }
    }
}
