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
            name: "Content-Type Header".to_string(),
            version: "1.0.0".to_string(),
            description: "Builds properly formatted Content-Type header with MIME type, charset, and boundary".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "mime_type".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "MIME type (e.g., 'text/html', 'application/json')".to_string(),
            },
            PortSpec {
                name: "charset".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Character encoding (e.g., 'utf-8', 'iso-8859-1')".to_string(),
            },
            PortSpec {
                name: "boundary".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Boundary string for multipart content (e.g., '----WebKitFormBoundary')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "header_value".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Complete Content-Type header value (e.g., 'text/html; charset=utf-8')".to_string(),
            },
            PortSpec {
                name: "is_text".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if MIME type is text-based".to_string(),
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
        // Extract mime_type
        let mime_type = inputs
            .iter()
            .find(|(name, _)| name == "mime_type")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: mime_type".to_string(),
                input_name: Some("mime_type".to_string()),
                recovery_hint: Some("Connect a MIME type to this input".to_string()),
            })?;

        let mime_type_str = match &mime_type.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'mime_type', got {:?}", mime_type.1),
                    input_name: Some("mime_type".to_string()),
                    recovery_hint: Some("Provide a MIME type string".to_string()),
                });
            }
        };

        // Extract charset (optional)
        let charset = if let Some(charset_input) = inputs.iter().find(|(name, _)| name == "charset") {
            match &charset_input.1 {
                Value::StringVal(s) => Some(s.clone()),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'charset', got {:?}", charset_input.1),
                        input_name: Some("charset".to_string()),
                        recovery_hint: Some("Provide a charset string".to_string()),
                    });
                }
            }
        } else {
            None
        };

        // Extract boundary (optional)
        let boundary = if let Some(boundary_input) = inputs.iter().find(|(name, _)| name == "boundary") {
            match &boundary_input.1 {
                Value::StringVal(s) => Some(s.clone()),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'boundary', got {:?}", boundary_input.1),
                        input_name: Some("boundary".to_string()),
                        recovery_hint: Some("Provide a boundary string".to_string()),
                    });
                }
            }
        } else {
            None
        };

        // Build Content-Type header value
        let (header_value, is_text) = build_content_type(mime_type_str, charset.as_deref(), boundary.as_deref());

        Ok(vec![
            ("header_value".to_string(), Value::StringVal(header_value)),
            ("is_text".to_string(), Value::BoolVal(is_text)),
        ])
    }
}

// ============================================================================
// Content-Type Building Logic
// ============================================================================

/// Build Content-Type header value
/// Returns: (header_value, is_text)
fn build_content_type(mime_type: &str, charset: Option<&str>, boundary: Option<&str>) -> (String, bool) {
    let mut parts = vec![mime_type.to_string()];

    // Add charset if provided and not empty
    if let Some(cs) = charset {
        if !cs.is_empty() {
            parts.push(format!("charset={}", cs));
        }
    }

    // Add boundary if provided and not empty
    if let Some(b) = boundary {
        if !b.is_empty() {
            parts.push(format!("boundary={}", b));
        }
    }

    let header_value = parts.join("; ");

    // Determine if text-based
    let is_text = is_text_mime_type(mime_type);

    (header_value, is_text)
}

/// Check if MIME type is text-based
fn is_text_mime_type(mime_type: &str) -> bool {
    let mime_lower = mime_type.to_lowercase();

    mime_lower.starts_with("text/")
        || mime_lower == "application/json"
        || mime_lower == "application/xml"
        || mime_lower == "application/xhtml+xml"
        || mime_lower == "application/javascript"
        || mime_lower == "application/ecmascript"
        || mime_lower.contains("+xml")
        || mime_lower.contains("+json")
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mime_type() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("text/html".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("text/html".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true)); // is_text
    }

    #[test]
    fn test_mime_type_with_charset() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("text/html".to_string())),
            ("charset".to_string(), Value::StringVal("utf-8".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("text/html; charset=utf-8".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true));
    }

    #[test]
    fn test_json_with_charset() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("application/json".to_string())),
            ("charset".to_string(), Value::StringVal("utf-8".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/json; charset=utf-8".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true)); // JSON is text
    }

    #[test]
    fn test_multipart_with_boundary() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("multipart/form-data".to_string())),
            ("boundary".to_string(), Value::StringVal("----WebKitFormBoundary7MA4YWxkTrZu0gW".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(
            result[0].1,
            Value::StringVal("multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW".to_string())
        );
        assert_eq!(result[1].1, Value::BoolVal(false)); // multipart is not text
    }

    #[test]
    fn test_all_parameters() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("text/plain".to_string())),
            ("charset".to_string(), Value::StringVal("iso-8859-1".to_string())),
            ("boundary".to_string(), Value::StringVal("boundary123".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(
            result[0].1,
            Value::StringVal("text/plain; charset=iso-8859-1; boundary=boundary123".to_string())
        );
    }

    #[test]
    fn test_binary_mime_type() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("image/png".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("image/png".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false)); // Images are not text
    }

    #[test]
    fn test_xml_mime_type() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("application/xml".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::BoolVal(true)); // XML is text
    }

    #[test]
    fn test_svg_xml_mime_type() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("image/svg+xml".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::BoolVal(true)); // SVG (+xml) is text
    }

    #[test]
    fn test_empty_charset_ignored() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("text/html".to_string())),
            ("charset".to_string(), Value::StringVal("".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        // Empty charset should not be added
        assert_eq!(result[0].1, Value::StringVal("text/html".to_string()));
    }

    #[test]
    fn test_empty_boundary_ignored() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("multipart/form-data".to_string())),
            ("boundary".to_string(), Value::StringVal("".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("multipart/form-data".to_string()));
    }

    #[test]
    fn test_pdf_mime_type() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("application/pdf".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::BoolVal(false)); // PDF is not text
    }

    #[test]
    fn test_javascript_mime_type() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("application/javascript".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::BoolVal(true)); // JavaScript is text
    }

    #[test]
    fn test_css_mime_type() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("text/css".to_string())),
            ("charset".to_string(), Value::StringVal("utf-8".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("text/css; charset=utf-8".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true));
    }

    #[test]
    fn test_custom_json_variant() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("application/vnd.api+json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::BoolVal(true)); // +json suffix means text
    }

    #[test]
    fn test_missing_mime_type() {
        let inputs = vec![
            ("charset".to_string(), Value::StringVal("utf-8".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("mime_type".to_string()));
    }

    #[test]
    fn test_case_insensitive_text_detection() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("TEXT/HTML".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::BoolVal(true)); // Case-insensitive detection
    }

    #[test]
    fn test_octet_stream() {
        let inputs = vec![
            ("mime_type".to_string(), Value::StringVal("application/octet-stream".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::BoolVal(false)); // Binary type
    }

    #[test]
    fn test_various_charsets() {
        let charsets = vec!["utf-8", "iso-8859-1", "windows-1252", "shift_jis"];

        for charset in charsets {
            let inputs = vec![
                ("mime_type".to_string(), Value::StringVal("text/plain".to_string())),
                ("charset".to_string(), Value::StringVal(charset.to_string())),
            ];

            let result = Component::execute(inputs).unwrap();

            if let Value::StringVal(header) = &result[0].1 {
                assert!(header.contains(&format!("charset={}", charset)));
            }
        }
    }
}
