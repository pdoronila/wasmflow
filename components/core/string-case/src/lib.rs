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
            name: "String Case".to_string(),
            version: "1.0.0".to_string(),
            description: "Converts string case (uppercase, lowercase, titlecase)".to_string(),
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
                name: "operation".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Operation: uppercase, lowercase, or titlecase".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Transformed string".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "String Case executing");

        let text = inputs
            .iter()
            .find(|(n, _)| n == "text")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide a string value".to_string()),
            })?;

        let operation = inputs
            .iter()
            .find(|(n, _)| n == "operation")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'operation' input".to_string(),
                input_name: Some("operation".to_string()),
                recovery_hint: Some("Provide 'uppercase', 'lowercase', or 'titlecase'".to_string()),
            })?;

        let result = match operation.as_str() {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "titlecase" => titlecase(&text),
            _ => {
                return Err(ExecutionError {
                    message: format!("Invalid operation: '{}'", operation),
                    input_name: Some("operation".to_string()),
                    recovery_hint: Some("Use 'uppercase', 'lowercase', or 'titlecase'".to_string()),
                });
            }
        };

        Ok(vec![("result".to_string(), Value::StringVal(result))])
    }
}

export!(Component);

fn titlecase(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c.is_whitespace() {
            result.push(c);
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.extend(c.to_lowercase());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uppercase() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("hello".to_string())),
            ("operation".to_string(), Value::StringVal("uppercase".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "HELLO"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_lowercase() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("HELLO".to_string())),
            ("operation".to_string(), Value::StringVal("lowercase".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_titlecase() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("hello world".to_string())),
            ("operation".to_string(), Value::StringVal("titlecase".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "Hello World"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_invalid_operation() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("hello".to_string())),
            ("operation".to_string(), Value::StringVal("invalid".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
