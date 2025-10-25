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
            name: "Static File Response".to_string(),
            version: "1.0.0".to_string(),
            description: "Prepares complete HTTP response data for serving static files (headers, status, MIME type)".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "file_path".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "File path to serve (e.g., '/var/www/index.html')".to_string(),
            },
            PortSpec {
                name: "file_content".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "File content to serve (from file-reader component)".to_string(),
            },
            PortSpec {
                name: "cache_control".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Cache-Control header value (e.g., 'public, max-age=3600')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "status".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "HTTP status code (200 for success)".to_string(),
            },
            PortSpec {
                name: "headers".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Complete headers as JSON object with Content-Type, Content-Length, etc.".to_string(),
            },
            PortSpec {
                name: "body".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "File content (same as input file_content)".to_string(),
            },
            PortSpec {
                name: "mime_type".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Detected MIME type (e.g., 'text/html')".to_string(),
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
        // Extract file_path
        let file_path = inputs
            .iter()
            .find(|(name, _)| name == "file_path")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: file_path".to_string(),
                input_name: Some("file_path".to_string()),
                recovery_hint: Some("Connect a file path to this input".to_string()),
            })?;

        let path_str = match &file_path.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'file_path', got {:?}", file_path.1),
                    input_name: Some("file_path".to_string()),
                    recovery_hint: Some("Provide a string file path".to_string()),
                });
            }
        };

        // Extract file_content
        let file_content = inputs
            .iter()
            .find(|(name, _)| name == "file_content")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: file_content".to_string(),
                input_name: Some("file_content".to_string()),
                recovery_hint: Some("Connect file content (from file-reader) to this input".to_string()),
            })?;

        let content_str = match &file_content.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'file_content', got {:?}", file_content.1),
                    input_name: Some("file_content".to_string()),
                    recovery_hint: Some("Provide file content as string".to_string()),
                });
            }
        };

        // Extract cache_control (optional)
        let cache_control = if let Some(cache_input) = inputs.iter().find(|(name, _)| name == "cache_control") {
            match &cache_input.1 {
                Value::StringVal(s) => Some(s.clone()),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string for 'cache_control', got {:?}", cache_input.1),
                        input_name: Some("cache_control".to_string()),
                        recovery_hint: Some("Provide a string cache control value".to_string()),
                    });
                }
            }
        } else {
            None
        };

        // Detect MIME type from file path
        let (mime_type, is_text, charset) = detect_mime_type(path_str);

        // Build headers
        let headers = build_headers(content_str, &mime_type, &charset, cache_control.as_deref());

        Ok(vec![
            ("status".to_string(), Value::U32Val(200)),
            ("headers".to_string(), Value::StringVal(headers)),
            ("body".to_string(), Value::StringVal(content_str.clone())),
            ("mime_type".to_string(), Value::StringVal(mime_type)),
        ])
    }
}

// ============================================================================
// MIME Type Detection and Header Building Logic
// ============================================================================

/// Detect MIME type from file path
/// Returns: (mime_type, is_text, charset)
fn detect_mime_type(file_path: &str) -> (String, bool, String) {
    let extension = get_file_extension(file_path);

    let mime_type = match extension.as_str() {
        // Text/HTML/XML
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "text/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "md" => "text/markdown",

        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",

        // Fonts
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",

        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",

        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",

        // Documents
        "pdf" => "application/pdf",

        // Archives
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",

        // Application
        "wasm" => "application/wasm",

        // Default
        _ => "application/octet-stream",
    };

    let is_text = mime_type.starts_with("text/")
        || mime_type == "application/json"
        || mime_type == "application/xml"
        || mime_type == "image/svg+xml";

    let charset = if is_text {
        "utf-8".to_string()
    } else {
        String::new()
    };

    (mime_type.to_string(), is_text, charset)
}

/// Extract file extension from path
fn get_file_extension(path: &str) -> String {
    if let Some(dot_pos) = path.rfind('.') {
        let ext = &path[dot_pos + 1..];
        ext.to_lowercase()
    } else {
        String::new()
    }
}

/// Build headers for static file response
fn build_headers(content: &str, mime_type: &str, charset: &str, cache_control: Option<&str>) -> String {
    let mut header_pairs = Vec::new();

    // Content-Type
    let content_type = if charset.is_empty() {
        mime_type.to_string()
    } else {
        format!("{}; charset={}", mime_type, charset)
    };
    header_pairs.push(format!("\"content-type\":\"{}\"", content_type));

    // Content-Length
    header_pairs.push(format!("\"content-length\":\"{}\"", content.len()));

    // Cache-Control (if provided)
    if let Some(cache) = cache_control {
        if !cache.is_empty() {
            let escaped_cache = escape_json_string(cache);
            header_pairs.push(format!("\"cache-control\":\"{}\"", escaped_cache));
        }
    }

    format!("{{{}}}", header_pairs.join(","))
}

/// Escape special characters for JSON string values
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }

    result
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("/var/www/index.html".to_string())),
            ("file_content".to_string(), Value::StringVal("<html><body>Hello</body></html>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::U32Val(200)); // status
        assert_eq!(result[2].1, Value::StringVal("<html><body>Hello</body></html>".to_string())); // body
        assert_eq!(result[3].1, Value::StringVal("text/html".to_string())); // mime_type

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"content-type\":\"text/html; charset=utf-8\""));
            assert!(headers.contains("\"content-length\":\"30\""));
        }
    }

    #[test]
    fn test_css_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("style.css".to_string())),
            ("file_content".to_string(), Value::StringVal("body { margin: 0; }".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[3].1, Value::StringVal("text/css".to_string()));

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"content-type\":\"text/css; charset=utf-8\""));
        }
    }

    #[test]
    fn test_javascript_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("app.js".to_string())),
            ("file_content".to_string(), Value::StringVal("console.log('test');".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[3].1, Value::StringVal("text/javascript".to_string()));
    }

    #[test]
    fn test_image_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("logo.png".to_string())),
            ("file_content".to_string(), Value::StringVal("PNG_BINARY_DATA".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[3].1, Value::StringVal("image/png".to_string()));

        if let Value::StringVal(headers) = &result[1].1 {
            // Binary files don't include charset
            assert!(headers.contains("\"content-type\":\"image/png\""));
            assert!(!headers.contains("charset"));
        }
    }

    #[test]
    fn test_with_cache_control() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("index.html".to_string())),
            ("file_content".to_string(), Value::StringVal("<html></html>".to_string())),
            ("cache_control".to_string(), Value::StringVal("public, max-age=3600".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"cache-control\":\"public, max-age=3600\""));
        }
    }

    #[test]
    fn test_without_cache_control() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("index.html".to_string())),
            ("file_content".to_string(), Value::StringVal("<html></html>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(!headers.contains("cache-control"));
        }
    }

    #[test]
    fn test_json_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("data.json".to_string())),
            ("file_content".to_string(), Value::StringVal("{\"key\":\"value\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[3].1, Value::StringVal("application/json".to_string()));

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"content-type\":\"application/json; charset=utf-8\""));
        }
    }

    #[test]
    fn test_pdf_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("document.pdf".to_string())),
            ("file_content".to_string(), Value::StringVal("PDF_DATA".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[3].1, Value::StringVal("application/pdf".to_string()));
    }

    #[test]
    fn test_font_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("font.woff2".to_string())),
            ("file_content".to_string(), Value::StringVal("FONT_DATA".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[3].1, Value::StringVal("font/woff2".to_string()));
    }

    #[test]
    fn test_empty_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("empty.txt".to_string())),
            ("file_content".to_string(), Value::StringVal("".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"content-length\":\"0\""));
        }
    }

    #[test]
    fn test_missing_file_path() {
        let inputs = vec![
            ("file_content".to_string(), Value::StringVal("content".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("file_path".to_string()));
    }

    #[test]
    fn test_missing_file_content() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("index.html".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("file_content".to_string()));
    }

    #[test]
    fn test_svg_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("icon.svg".to_string())),
            ("file_content".to_string(), Value::StringVal("<svg></svg>".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[3].1, Value::StringVal("image/svg+xml".to_string()));

        if let Value::StringVal(headers) = &result[1].1 {
            // SVG is text-based
            assert!(headers.contains("\"content-type\":\"image/svg+xml; charset=utf-8\""));
        }
    }

    #[test]
    fn test_content_length_calculation() {
        let content = "This is a test file with some content.";
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("test.txt".to_string())),
            ("file_content".to_string(), Value::StringVal(content.to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains(&format!("\"content-length\":\"{}\"", content.len())));
        }
    }

    #[test]
    fn test_no_cache_directive() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("api.json".to_string())),
            ("file_content".to_string(), Value::StringVal("{}".to_string())),
            ("cache_control".to_string(), Value::StringVal("no-cache, no-store, must-revalidate".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(headers) = &result[1].1 {
            assert!(headers.contains("\"cache-control\":\"no-cache, no-store, must-revalidate\""));
        }
    }
}
