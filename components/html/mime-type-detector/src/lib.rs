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
            name: "MIME Type Detector".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects MIME type from file path or extension for HTTP Content-Type headers".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "file_path".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "File path or filename (e.g., 'index.html' or '/static/style.css')".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "mime_type".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "MIME type (e.g., 'text/html', 'application/json')".to_string(),
            },
            PortSpec {
                name: "is_text".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if MIME type is text-based (starts with 'text/' or is JSON/XML)".to_string(),
            },
            PortSpec {
                name: "charset".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Recommended charset (e.g., 'utf-8' for text, empty for binary)".to_string(),
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
                recovery_hint: Some("Connect a file path or filename to this input".to_string()),
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

        // Detect MIME type from file extension
        let (mime_type, is_text, charset) = detect_mime_type(path_str);

        Ok(vec![
            ("mime_type".to_string(), Value::StringVal(mime_type)),
            ("is_text".to_string(), Value::BoolVal(is_text)),
            ("charset".to_string(), Value::StringVal(charset)),
        ])
    }
}

// ============================================================================
// MIME Type Detection Logic
// ============================================================================

/// Detect MIME type from file path
/// Returns: (mime_type, is_text, charset)
fn detect_mime_type(file_path: &str) -> (String, bool, String) {
    // Extract file extension
    let extension = get_file_extension(file_path);

    // Map extension to MIME type
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
        "yaml" | "yml" => "text/yaml",

        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",

        // Fonts
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",

        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" | "ogv" => "video/ogg",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",

        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "oga" => "audio/ogg",
        "m4a" => "audio/mp4",

        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "gzip" => "application/gzip",
        "bz2" => "application/x-bzip2",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",

        // Application
        "wasm" => "application/wasm",
        "bin" => "application/octet-stream",
        "exe" => "application/x-msdownload",
        "dll" => "application/x-msdownload",

        // Default
        _ => "application/octet-stream",
    };

    // Determine if text-based
    let is_text = mime_type.starts_with("text/")
        || mime_type == "application/json"
        || mime_type == "application/xml"
        || mime_type == "image/svg+xml"
        || mime_type == "text/yaml";

    // Determine charset
    let charset = if is_text {
        "utf-8".to_string()
    } else {
        String::new()
    };

    (mime_type.to_string(), is_text, charset)
}

/// Extract file extension from path (returns lowercase)
fn get_file_extension(path: &str) -> String {
    // Find last dot
    if let Some(dot_pos) = path.rfind('.') {
        let ext = &path[dot_pos + 1..];
        ext.to_lowercase()
    } else {
        String::new()
    }
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
            ("file_path".to_string(), Value::StringVal("index.html".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("text/html".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true)); // is_text
        assert_eq!(result[2].1, Value::StringVal("utf-8".to_string()));
    }

    #[test]
    fn test_css_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("style.css".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("text/css".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true));
    }

    #[test]
    fn test_javascript_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("app.js".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("text/javascript".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true));
    }

    #[test]
    fn test_json_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("data.json".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/json".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true)); // JSON is text
        assert_eq!(result[2].1, Value::StringVal("utf-8".to_string()));
    }

    #[test]
    fn test_image_png() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("logo.png".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("image/png".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false)); // is_text = false
        assert_eq!(result[2].1, Value::StringVal("".to_string())); // no charset for binary
    }

    #[test]
    fn test_image_jpeg() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("photo.jpg".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("image/jpeg".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false));
    }

    #[test]
    fn test_svg_image() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("icon.svg".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("image/svg+xml".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true)); // SVG is text-based
        assert_eq!(result[2].1, Value::StringVal("utf-8".to_string()));
    }

    #[test]
    fn test_pdf_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("document.pdf".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/pdf".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false));
    }

    #[test]
    fn test_zip_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("archive.zip".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/zip".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false));
    }

    #[test]
    fn test_wasm_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("component.wasm".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/wasm".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false));
    }

    #[test]
    fn test_full_path() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("/var/www/static/css/main.css".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("text/css".to_string()));
    }

    #[test]
    fn test_case_insensitive_extension() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("DOCUMENT.PDF".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/pdf".to_string()));
    }

    #[test]
    fn test_no_extension() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("README".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/octet-stream".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false));
    }

    #[test]
    fn test_unknown_extension() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("file.unknownext".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/octet-stream".to_string()));
    }

    #[test]
    fn test_video_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("video.mp4".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("video/mp4".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false));
    }

    #[test]
    fn test_audio_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("song.mp3".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("audio/mpeg".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false));
    }

    #[test]
    fn test_font_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("font.woff2".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("font/woff2".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(false));
    }

    #[test]
    fn test_missing_input() {
        let inputs = vec![];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("file_path".to_string()));
    }

    #[test]
    fn test_multiple_dots_in_filename() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("my.app.min.js".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("text/javascript".to_string()));
    }

    #[test]
    fn test_xml_file() {
        let inputs = vec![
            ("file_path".to_string(), Value::StringVal("config.xml".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("application/xml".to_string()));
        assert_eq!(result[1].1, Value::BoolVal(true)); // XML is text
    }
}
