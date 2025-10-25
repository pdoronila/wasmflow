wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

// ============================================================================
// Metadata Interface
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "HTML Escape".to_string(),
            version: "1.0.0".to_string(),
            description: "Escapes HTML special characters to prevent XSS attacks and ensure safe rendering".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "text".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Text to escape (e.g., '<script>alert(1)</script>')".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "escaped".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "HTML-safe escaped text (e.g., '&lt;script&gt;alert(1)&lt;/script&gt;')".to_string(),
            },
            PortSpec {
                name: "char_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of characters escaped".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}


// ============================================================================
// Execution Interface
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract text
        let text = inputs
            .iter()
            .find(|(name, _)| name == "text")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: text".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Connect text to escape to this input".to_string()),
            })?;

        let text_str = match &text.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'text', got {:?}", text.1),
                    input_name: Some("text".to_string()),
                    recovery_hint: Some("Provide a string value".to_string()),
                });
            }
        };

        // Escape HTML
        let (escaped, char_count) = escape_html(text_str);

        Ok(vec![
            ("escaped".to_string(), Value::StringVal(escaped)),
            ("char_count".to_string(), Value::U32Val(char_count)),
        ])
    }
}

// ============================================================================
// HTML Escaping Logic
// ============================================================================

/// Escape HTML special characters
/// Returns: (escaped_string, count_of_escaped_chars)
fn escape_html(text: &str) -> (String, u32) {
    let mut result = String::with_capacity(text.len());
    let mut char_count = 0;

    for c in text.chars() {
        match c {
            '<' => {
                result.push_str("&lt;");
                char_count += 1;
            }
            '>' => {
                result.push_str("&gt;");
                char_count += 1;
            }
            '&' => {
                result.push_str("&amp;");
                char_count += 1;
            }
            '"' => {
                result.push_str("&quot;");
                char_count += 1;
            }
            '\'' => {
                result.push_str("&#x27;");
                char_count += 1;
            }
            '/' => {
                // Escape forward slash to prevent </script> injection
                result.push_str("&#x2F;");
                char_count += 1;
            }
            _ => {
                result.push(c);
            }
        }
    }

    (result, char_count)
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_html_escape() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("<script>alert('XSS')</script>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(
            result[0].1,
            Value::StringVal("&lt;script&gt;alert(&#x27;XSS&#x27;)&lt;&#x2F;script&gt;".to_string())
        );
        assert_eq!(result[1].1, Value::U32Val(7)); // < > ' ' < / >
    }

    #[test]
    fn test_ampersand_escape() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Tom & Jerry".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Tom &amp; Jerry".to_string()));
        assert_eq!(result[1].1, Value::U32Val(1));
    }

    #[test]
    fn test_quote_escape() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("He said \"Hello\"".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("He said &quot;Hello&quot;".to_string()));
        assert_eq!(result[1].1, Value::U32Val(2));
    }

    #[test]
    fn test_single_quote_escape() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("It's a test".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("It&#x27;s a test".to_string()));
        assert_eq!(result[1].1, Value::U32Val(1));
    }

    #[test]
    fn test_no_escape_needed() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Plain text with no special chars".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Plain text with no special chars".to_string()));
        assert_eq!(result[1].1, Value::U32Val(0));
    }

    #[test]
    fn test_empty_string() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("".to_string()));
        assert_eq!(result[1].1, Value::U32Val(0));
    }

    #[test]
    fn test_all_special_chars() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("<>&\"'/".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("&lt;&gt;&amp;&quot;&#x27;&#x2F;".to_string()));
        assert_eq!(result[1].1, Value::U32Val(6));
    }

    #[test]
    fn test_script_tag_injection() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("<img src=x onerror=alert(1)>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(
            result[0].1,
            Value::StringVal("&lt;img src=x onerror=alert(1)&gt;".to_string())
        );
    }

    #[test]
    fn test_javascript_url() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("<a href=\"javascript:alert('XSS')\">Click</a>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        // All dangerous characters escaped
        if let Value::StringVal(escaped) = &result[0].1 {
            assert!(escaped.contains("&lt;a"));
            assert!(escaped.contains("&quot;"));
            assert!(escaped.contains("&#x27;"));
            assert!(escaped.contains("&gt;"));
        }
    }

    #[test]
    fn test_html_entity_passthrough() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Already &amp; escaped".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        // Double-escapes existing entities (which is correct for safety)
        assert_eq!(result[0].1, Value::StringVal("Already &amp;amp; escaped".to_string()));
    }

    #[test]
    fn test_multiline_html() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("<div>\n<script>alert(1)</script>\n</div>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(escaped) = &result[0].1 {
            assert!(escaped.contains("&lt;div&gt;"));
            assert!(escaped.contains("&lt;script&gt;"));
            assert!(escaped.contains("\n")); // Newlines preserved
        }
    }

    #[test]
    fn test_unicode_characters() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("Hello 世界 <b>bold</b>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(escaped) = &result[0].1 {
            assert!(escaped.contains("世界")); // Unicode preserved
            assert!(escaped.contains("&lt;b&gt;")); // HTML escaped
        }
    }

    #[test]
    fn test_closing_tag_protection() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("</script><script>alert(1)</script>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(escaped) = &result[0].1 {
            // / is escaped to prevent closing tags
            assert!(escaped.contains("&#x2F;"));
        }
    }

    #[test]
    fn test_data_attribute_injection() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("<div data-value=\"\" onload=alert(1)\">".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(escaped) = &result[0].1 {
            assert!(!escaped.contains("<div")); // No raw tags
            assert!(escaped.contains("&lt;"));
            assert!(escaped.contains("&quot;"));
        }
    }

    #[test]
    fn test_missing_input() {
        let inputs = vec![];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("text".to_string()));
    }

    #[test]
    fn test_css_injection() {
        let inputs = vec![
            ("text".to_string(), Value::StringVal("<style>body { background: url('javascript:alert(1)'); }</style>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(escaped) = &result[0].1 {
            assert!(escaped.contains("&lt;style&gt;"));
            assert!(escaped.contains("&#x27;")); // Quotes escaped
        }
    }

    #[test]
    fn test_long_text() {
        let long_text = "<script>".repeat(1000);
        let inputs = vec![
            ("text".to_string(), Value::StringVal(long_text)),
        ];

        let result = Component::execute(inputs).unwrap();

        // Should handle long strings without issues
        assert_eq!(result[1].1, Value::U32Val(2000)); // 1000 < and 1000 >
    }
}
