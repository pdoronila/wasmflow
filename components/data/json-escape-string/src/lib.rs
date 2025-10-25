wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "JSON Escape String".to_string(),
            version: "1.0.0".to_string(),
            description: "Escape special characters for JSON strings".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "text".to_string(),
                data_type: DataType::String,
                description: "Text to escape for JSON".to_string(),
                required: true,
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "escaped".to_string(),
                data_type: DataType::String,
                description: "JSON-safe escaped text (without surrounding quotes)".to_string(),
                required: true,
            },
            PortSpec {
                name: "escape_count".to_string(),
                data_type: DataType::U32,
                description: "Number of characters escaped".to_string(),
                required: true,
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract required input
        let text = inputs
            .iter()
            .find(|(name, _)| name == "text")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: text".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Provide text to escape for JSON".to_string()),
            })?;

        let input_text = match &text.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'text', got {:?}", text.1),
                    input_name: Some("text".to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        };

        // Escape JSON
        let (escaped, count) = json_escape(input_text);

        Ok(vec![
            ("escaped".to_string(), Value::StringVal(escaped)),
            ("escape_count".to_string(), Value::U32Val(count)),
        ])
    }
}

/// Escape special characters for JSON strings
///
/// Escapes:
/// - " → \"
/// - \ → \\
/// - \n → \n
/// - \r → \r
/// - \t → \t
/// - \b → \b (backspace)
/// - \f → \f (form feed)
fn json_escape(text: &str) -> (String, u32) {
    let mut escaped = String::with_capacity(text.len() + text.len() / 10);
    let mut escape_count = 0u32;

    for ch in text.chars() {
        match ch {
            '"' => {
                escaped.push_str("\\\"");
                escape_count += 1;
            }
            '\\' => {
                escaped.push_str("\\\\");
                escape_count += 1;
            }
            '\n' => {
                escaped.push_str("\\n");
                escape_count += 1;
            }
            '\r' => {
                escaped.push_str("\\r");
                escape_count += 1;
            }
            '\t' => {
                escaped.push_str("\\t");
                escape_count += 1;
            }
            '\u{0008}' => {
                // Backspace
                escaped.push_str("\\b");
                escape_count += 1;
            }
            '\u{000C}' => {
                // Form feed
                escaped.push_str("\\f");
                escape_count += 1;
            }
            _ => {
                escaped.push(ch);
            }
        }
    }

    (escaped, escape_count)
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_no_special_chars() {
        let (escaped, count) = json_escape("hello world");
        assert_eq!(escaped, "hello world");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_escape_quotes() {
        let (escaped, count) = json_escape(r#"He said "hello""#);
        assert_eq!(escaped, r#"He said \"hello\""#);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_escape_backslash() {
        let (escaped, count) = json_escape(r"C:\Users\test");
        assert_eq!(escaped, r"C:\\Users\\test");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_escape_newline() {
        let (escaped, count) = json_escape("line1\nline2");
        assert_eq!(escaped, r"line1\nline2");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_carriage_return() {
        let (escaped, count) = json_escape("line1\rline2");
        assert_eq!(escaped, r"line1\rline2");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_tab() {
        let (escaped, count) = json_escape("col1\tcol2");
        assert_eq!(escaped, r"col1\tcol2");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_backspace() {
        let (escaped, count) = json_escape("test\u{0008}end");
        assert_eq!(escaped, r"test\bend");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_form_feed() {
        let (escaped, count) = json_escape("test\u{000C}end");
        assert_eq!(escaped, r"test\fend");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escape_multiple() {
        let (escaped, count) = json_escape("\"quoted\"\n\ttab");
        assert_eq!(escaped, r#"\"quoted\"\n\ttab"#);
        assert_eq!(count, 4);
    }

    #[test]
    fn test_escape_empty_string() {
        let (escaped, count) = json_escape("");
        assert_eq!(escaped, "");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_escape_mixed_content() {
        let (escaped, count) = json_escape("He said \"hello\\world\"\nNext line");
        assert_eq!(escaped, r#"He said \"hello\\world\"\nNext line"#);
        assert_eq!(count, 4);
    }

    #[test]
    fn test_escape_unicode() {
        let (escaped, count) = json_escape("hello 🚀 world");
        assert_eq!(escaped, "hello 🚀 world");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_escape_json_value() {
        let (escaped, count) = json_escape(r#"{"name":"value"}"#);
        assert_eq!(escaped, r#"{\"name\":\"value\"}"#);
        assert_eq!(count, 4);
    }

    #[test]
    fn test_escape_all_special_chars() {
        let text = "\"\\\n\r\t\u{0008}\u{000C}";
        let (escaped, count) = json_escape(text);
        assert_eq!(escaped, r#"\"\\\n\r\t\b\f"#);
        assert_eq!(count, 7);
    }

    #[test]
    fn test_escape_consecutive_escapes() {
        let (escaped, count) = json_escape("\n\n\n");
        assert_eq!(escaped, r"\n\n\n");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_escape_path() {
        let (escaped, count) = json_escape(r"/api/users/John Doe");
        assert_eq!(escaped, r"/api/users/John Doe");
        assert_eq!(count, 0);
    }
}
