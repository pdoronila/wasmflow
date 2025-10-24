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
            name: "String Length".to_string(),
            version: "1.0.0".to_string(),
            description: "Counts the number of characters in a string (Unicode-aware)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Core".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "text".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "String to measure".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::U32Type,
            optional: false,
            description: "Number of characters".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "String Length executing");

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

        let length = text.chars().count() as u32;
        Ok(vec![("result".to_string(), Value::U32Val(length))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_string() {
        let inputs = vec![("text".to_string(), Value::StringVal("hello".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 5),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_unicode_emojis() {
        let inputs = vec![("text".to_string(), Value::StringVal("🚀🌟".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 2), // 2 characters, not bytes
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_empty_string() {
        let inputs = vec![("text".to_string(), Value::StringVal("".to_string()))];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 0),
            _ => panic!("Expected u32 output"),
        }
    }
}
