//! JSON Extract Each Component - Extract a field from each JSON string in a list
//!
//! This component processes a list of JSON strings (JSONL format) and extracts
//! a specific field from each one, returning a list of extracted values.

wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

// ============================================================================
// METADATA INTERFACE
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "JSON Extract Each".to_string(),
            version: "1.0.0".to_string(),
            description: "Extract a field from each JSON string in a list (JSONL batch processing)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "json_strings".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "List of JSON strings to parse".to_string(),
            },
            PortSpec {
                name: "field_path".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Key path to extract (e.g., 'path', 'event.file', 'data[0]')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "values".to_string(),
                data_type: DataType::ListType,
                optional: false,
                description: "Extracted values (skips failed parses)".to_string(),
            },
            PortSpec {
                name: "error_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of items that failed to parse".to_string(),
            },
            PortSpec {
                name: "success_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of successful extractions".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

// ============================================================================
// EXECUTION INTERFACE
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract json_strings input
        let json_strings = inputs
            .iter()
            .find(|(name, _)| name == "json_strings")
            .and_then(|(_, val)| match val {
                Value::StringListVal(items) => Some(items.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'json_strings' input".to_string(),
                input_name: Some("json_strings".to_string()),
                recovery_hint: Some("Connect a StringListVal to the 'json_strings' port".to_string()),
            })?;

        // Extract field_path input
        let field_path = inputs
            .iter()
            .find(|(name, _)| name == "field_path")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'field_path' input".to_string(),
                input_name: Some("field_path".to_string()),
                recovery_hint: Some("Connect a String value to the 'field_path' port".to_string()),
            })?;

        // Parse key path once (reuse for all JSON objects)
        let tokens = tokenize(&field_path).map_err(|e| ExecutionError {
            message: format!("Invalid field path: {}", e),
            input_name: Some("field_path".to_string()),
            recovery_hint: Some("Use dot notation (e.g., 'event.path') or bracket notation (e.g., 'items[0]')".to_string()),
        })?;

        // Process each JSON string
        let mut extracted_values = Vec::new();
        let mut error_count = 0;

        for json_str in &json_strings {
            match parse_and_extract(json_str, &tokens) {
                Ok(value) => extracted_values.push(value),
                Err(_) => error_count += 1, // Skip failed parses
            }
        }

        let success_count = extracted_values.len();

        Ok(vec![
            ("values".to_string(), Value::StringListVal(extracted_values)),
            ("error_count".to_string(), Value::U32Val(error_count)),
            ("success_count".to_string(), Value::U32Val(success_count as u32)),
        ])
    }
}

// ============================================================================
// JSON PARSING LOGIC (adapted from json-parser component)
// ============================================================================

/// Parse JSON string and extract value at key path
fn parse_and_extract(json_str: &str, tokens: &[Token]) -> Result<String, String> {
    // Parse JSON
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    // Extract value
    let extracted_value = extract_value(&parsed, tokens)?;

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
                _ => Err(format!("Cannot access property '{}' on non-object", key)),
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
                _ => Err("Cannot index into non-array".to_string()),
            }
        }
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
    fn test_all_valid_json() {
        let inputs = vec![
            ("json_strings".to_string(), Value::StringListVal(vec![
                r#"{"path": "/home/user/file1.txt"}"#.to_string(),
                r#"{"path": "/home/user/file2.txt"}"#.to_string(),
                r#"{"path": "/home/user/file3.txt"}"#.to_string(),
            ])),
            ("field_path".to_string(), Value::StringVal("path".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        // Check extracted values
        match &result[0].1 {
            Value::StringListVal(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], "/home/user/file1.txt");
                assert_eq!(values[1], "/home/user/file2.txt");
                assert_eq!(values[2], "/home/user/file3.txt");
            }
            _ => panic!("Expected StringListVal output"),
        }

        // Check error count
        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }

        // Check success count
        match &result[2].1 {
            Value::U32Val(count) => assert_eq!(*count, 3),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_some_invalid_json() {
        let inputs = vec![
            ("json_strings".to_string(), Value::StringListVal(vec![
                r#"{"path": "file1.txt"}"#.to_string(),
                r#"invalid json"#.to_string(),  // malformed
                r#"{"path": "file3.txt"}"#.to_string(),
            ])),
            ("field_path".to_string(), Value::StringVal("path".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], "file1.txt");
                assert_eq!(values[1], "file3.txt");
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 1),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_field_missing_in_some() {
        let inputs = vec![
            ("json_strings".to_string(), Value::StringListVal(vec![
                r#"{"path": "file1.txt"}"#.to_string(),
                r#"{"name": "file2.txt"}"#.to_string(),  // wrong field
                r#"{"path": "file3.txt"}"#.to_string(),
            ])),
            ("field_path".to_string(), Value::StringVal("path".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(values) => {
                assert_eq!(values.len(), 2);
            }
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 1),
            _ => panic!("Expected U32Val output"),
        }
    }

    #[test]
    fn test_nested_field_extraction() {
        let inputs = vec![
            ("json_strings".to_string(), Value::StringListVal(vec![
                r#"{"event": {"path": "file1.txt"}}"#.to_string(),
                r#"{"event": {"path": "file2.txt"}}"#.to_string(),
            ])),
            ("field_path".to_string(), Value::StringVal("event.path".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], "file1.txt");
            }
            _ => panic!("Expected StringListVal output"),
        }
    }

    #[test]
    fn test_array_index_extraction() {
        let inputs = vec![
            ("json_strings".to_string(), Value::StringListVal(vec![
                r#"{"files": ["a.txt", "b.txt"]}"#.to_string(),
                r#"{"files": ["c.txt", "d.txt"]}"#.to_string(),
            ])),
            ("field_path".to_string(), Value::StringVal("files[0]".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], "a.txt");
                assert_eq!(values[1], "c.txt");
            }
            _ => panic!("Expected StringListVal output"),
        }
    }

    #[test]
    fn test_empty_input_list() {
        let inputs = vec![
            ("json_strings".to_string(), Value::StringListVal(vec![])),
            ("field_path".to_string(), Value::StringVal("path".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();

        match &result[0].1 {
            Value::StringListVal(values) => assert_eq!(values.len(), 0),
            _ => panic!("Expected StringListVal output"),
        }

        match &result[1].1 {
            Value::U32Val(count) => assert_eq!(*count, 0),
            _ => panic!("Expected U32Val output"),
        }
    }
}
