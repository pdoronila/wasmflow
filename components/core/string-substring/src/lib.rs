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
            name: "String Substring".to_string(),
            version: "1.0.0".to_string(),
            description: "Extracts a substring from a string (Unicode-aware)".to_string(),
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
                description: "Input string".to_string(),
            },
            PortSpec {
                name: "start".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Start index (character position)".to_string(),
            },
            PortSpec {
                name: "length".to_string(),
                data_type: DataType::U32Type,
                optional: true,
                description: "Number of characters to extract (to end if omitted)".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Extracted substring".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "String Substring executing");

        let text = inputs
            .iter()
            .find(|(n, _)| n == "text")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide a string value".to_string()),
            })?;

        let start = inputs
            .iter()
            .find(|(n, _)| n == "start")
            .and_then(|(_, v)| if let Value::U32Val(n) = v { Some(*n) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'start' input".to_string(),
                input_name: Some("start".to_string()),
                recovery_hint: Some("Provide a u32 start index".to_string()),
            })?;

        let length = inputs
            .iter()
            .find(|(n, _)| n == "length")
            .and_then(|(_, v)| if let Value::U32Val(n) = v { Some(*n) } else { None });

        // Convert to character array for Unicode safety
        let chars: Vec<char> = text.chars().collect();
        let start = start as usize;

        // If start is beyond the string length, return empty string
        if start >= chars.len() {
            return Ok(vec![("result".to_string(), Value::StringVal(String::new()))]);
        }

        // Calculate end index
        let end = if let Some(len) = length {
            (start + len as usize).min(chars.len())
        } else {
            chars.len()
        };

        // Extract substring
        let result: String = chars[start..end].iter().collect();

        Ok(vec![("result".to_string(), Value::StringVal(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substring_with_length() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Hello World".to_string())),
            ("start".to_string(), Value::U32Val(0)),
            ("length".to_string(), Value::U32Val(5)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "Hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_substring_to_end() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Hello World".to_string())),
            ("start".to_string(), Value::U32Val(6)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "World"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_substring_start_beyond_end() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Hello".to_string())),
            ("start".to_string(), Value::U32Val(100)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, ""),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_substring_unicode() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("🚀🌟✨".to_string())),
            ("start".to_string(), Value::U32Val(1)),
            ("length".to_string(), Value::U32Val(2)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "🌟✨"),
            _ => panic!("Expected string output"),
        }
    }
}
