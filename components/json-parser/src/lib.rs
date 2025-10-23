//! JSON Parser Component - Extracts values from JSON using key paths
//!
//! This component extracts values from JSON strings using key path notation.
//! Supports dot notation (metadata.author) and bracket notation (runs[1]).

// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use exports::wasmflow::node::ui::Guest as UiGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

// UI types
use exports::wasmflow::node::ui::{
    ColoredText, FooterView, KeyValuePair, UiElement,
};

struct Component;

// ============================================================================
// METADATA INTERFACE
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "JSON Parser".to_string(),
            version: "1.0.0".to_string(),
            description: "Extract values from JSON using key path notation (dot and bracket syntax)".to_string(),
            author: "WasmFlow".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "json_string".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "JSON string to parse".to_string(),
            },
            PortSpec {
                name: "key_path".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Key path (e.g., 'metadata.author' or 'runs[1].time')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Extracted value (type preserved as string representation)".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None // No system access required
    }
}

// ============================================================================
// EXECUTION INTERFACE
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "JSON Parser component executing");

        // Extract inputs
        let json_string = extract_string(&inputs, "json_string")?;
        let key_path = extract_string(&inputs, "key_path")?;

        host::log("debug", &format!("Parsing JSON ({} bytes) with path: {}", json_string.len(), key_path));

        // Parse JSON and extract value
        match parse_json(&json_string, &key_path) {
            Ok(value) => {
                host::log("info", &format!("Successfully extracted value: {}",
                    if value.len() > 100 { format!("{}... ({} bytes)", &value[..100], value.len()) }
                    else { value.clone() }
                ));

                Ok(vec![
                    ("value".to_string(), Value::StringVal(value)),
                ])
            }
            Err(err_msg) => {
                host::log("error", &format!("JSON parsing failed: {}", err_msg));
                Err(ExecutionError {
                    message: err_msg,
                    input_name: Some("json_string".to_string()),
                    recovery_hint: Some("Verify JSON is valid and key path exists".to_string()),
                })
            }
        }
    }
}

// ============================================================================
// UI INTERFACE
// ============================================================================

impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        let mut elements = Vec::new();

        // Header with icon
        elements.push(UiElement::ColoredLabel(ColoredText {
            text: "📋 Extracted Value".to_string(),
            r: 150,
            g: 200,
            b: 255,
        }));

        elements.push(UiElement::Separator);

        // Extract value output
        for (name, value) in &outputs {
            if name == "value" {
                if let Value::StringVal(s) = value {
                    let display_value = if s.len() > 200 {
                        format!("{}... ({} bytes)", &s[..200], s.len())
                    } else {
                        s.clone()
                    };

                    elements.push(UiElement::KeyValue(KeyValuePair {
                        key: "Value".to_string(),
                        value: display_value,
                    }));
                }
            }
        }

        Some(FooterView { elements })
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn extract_string(inputs: &[(String, Value)], name: &str) -> Result<String, ExecutionError> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, val)| match val {
            Value::StringVal(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| ExecutionError {
            message: format!("Missing or invalid '{}' input", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(format!("Connect a String value to the '{}' port", name)),
        })
}

// ============================================================================
// JSON PARSING LOGIC
// ============================================================================

/// Parse JSON and extract value at key path
fn parse_json(json_str: &str, key_path: &str) -> Result<String, String> {
    // Parse JSON
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    // Tokenize key path
    let tokens = tokenize(key_path)?;

    // Extract value
    let extracted_value = extract_value(&parsed, &tokens)?;

    // Convert to string representation
    Ok(value_to_string(&extracted_value))
}

/// Token types for key path parsing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Index(usize),
}

/// Tokenize a key path
fn tokenize(key_path: &str) -> Result<Vec<Token>, String> {
    if key_path.is_empty() {
        return Err("Key path cannot be empty".to_string());
    }

    let mut tokens = Vec::new();
    let mut chars = key_path.chars().peekable();
    let mut current_ident = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current_ident.is_empty() {
                    validate_identifier(&current_ident)?;
                    tokens.push(Token::Ident(current_ident.clone()));
                    current_ident.clear();
                } else if tokens.is_empty() {
                    return Err("Key path cannot start with '.'".to_string());
                }
            }
            '[' => {
                if !current_ident.is_empty() {
                    validate_identifier(&current_ident)?;
                    tokens.push(Token::Ident(current_ident.clone()));
                    current_ident.clear();
                }

                let mut index_str = String::new();
                loop {
                    match chars.peek() {
                        Some(']') => {
                            chars.next();
                            break;
                        }
                        Some(digit) if digit.is_ascii_digit() => {
                            index_str.push(*digit);
                            chars.next();
                        }
                        Some(other) => {
                            return Err(format!("Invalid character '{}' in array index", other));
                        }
                        None => {
                            return Err("Unclosed bracket in array index".to_string());
                        }
                    }
                }

                if index_str.is_empty() {
                    return Err("Empty array index".to_string());
                }

                let index = index_str.parse::<usize>()
                    .map_err(|_| format!("Invalid array index: {}", index_str))?;

                tokens.push(Token::Index(index));
            }
            _ => {
                current_ident.push(ch);
            }
        }
    }

    if !current_ident.is_empty() {
        validate_identifier(&current_ident)?;
        tokens.push(Token::Ident(current_ident));
    }

    if tokens.is_empty() {
        return Err("Key path resulted in no tokens".to_string());
    }

    Ok(tokens)
}

/// Validate identifier syntax
fn validate_identifier(ident: &str) -> Result<(), String> {
    if ident.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    let mut chars = ident.chars();
    if let Some(first) = chars.next() {
        if !first.is_alphabetic() && first != '_' {
            return Err(format!("Identifier '{}' must start with letter or underscore", ident));
        }
    }

    for ch in chars {
        if !ch.is_alphanumeric() && ch != '_' {
            return Err(format!("Identifier '{}' contains invalid character '{}'", ident, ch));
        }
    }

    Ok(())
}

/// Extract value from JSON using tokens
fn extract_value(json: &serde_json::Value, tokens: &[Token]) -> Result<serde_json::Value, String> {
    if tokens.is_empty() {
        return Ok(json.clone());
    }

    let token = &tokens[0];
    let remaining = &tokens[1..];

    match token {
        Token::Ident(key) => {
            match json {
                serde_json::Value::Object(map) => {
                    match map.get(key) {
                        Some(value) => extract_value(value, remaining),
                        None => Err(format!("Key '{}' not found in JSON object", key)),
                    }
                }
                _ => Err(format!("Cannot access property '{}' on non-object (found {})",
                    key, type_name(json))),
            }
        }
        Token::Index(idx) => {
            match json {
                serde_json::Value::Array(arr) => {
                    if *idx >= arr.len() {
                        return Err(format!("Array index {} out of bounds (array has {} elements)",
                            idx, arr.len()));
                    }
                    extract_value(&arr[*idx], remaining)
                }
                _ => Err(format!("Cannot index into non-array (found {})", type_name(json))),
            }
        }
    }
}

/// Get type name for error messages
fn type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::String(_) => "string",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Object(_) => "object",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Null => "null",
    }
}

/// Convert JSON value to string representation
fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

export!(Component);

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("version").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Ident("version".to_string()));
    }

    #[test]
    fn test_tokenize_nested() {
        let tokens = tokenize("metadata.author").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Ident("metadata".to_string()));
        assert_eq!(tokens[1], Token::Ident("author".to_string()));
    }

    #[test]
    fn test_tokenize_array() {
        let tokens = tokenize("runs[1]").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Ident("runs".to_string()));
        assert_eq!(tokens[1], Token::Index(1));
    }

    #[test]
    fn test_tokenize_combined() {
        let tokens = tokenize("runs[1].time").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Ident("runs".to_string()));
        assert_eq!(tokens[1], Token::Index(1));
        assert_eq!(tokens[2], Token::Ident("time".to_string()));
    }

    #[test]
    fn test_parse_simple() {
        let result = parse_json(r#"{"version": 1}"#, "version");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1");
    }

    #[test]
    fn test_parse_nested() {
        let result = parse_json(r#"{"metadata": {"author": "me"}}"#, "metadata.author");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "me");
    }

    #[test]
    fn test_parse_array() {
        let result = parse_json(r#"{"values": [10, 20, 30]}"#, "values[1]");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "20");
    }

    #[test]
    fn test_parse_combined() {
        let result = parse_json(
            r#"{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}"#,
            "runs[1].time"
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1000");
    }
}
