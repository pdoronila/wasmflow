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
            name: "Status Code Mapper".to_string(),
            version: "1.0.0".to_string(),
            description: "Maps HTTP status codes to their standard reason phrases and categories".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "code".to_string(),
            data_type: DataType::U32Type,
            optional: false,
            description: "HTTP status code (100-599)".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "message".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Standard HTTP reason phrase (e.g., 'OK', 'Not Found', 'Internal Server Error')".to_string(),
            },
            PortSpec {
                name: "category".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Status code category: 'Informational', 'Success', 'Redirection', 'Client Error', 'Server Error', or 'Unknown'".to_string(),
            },
            PortSpec {
                name: "is_error".to_string(),
                data_type: DataType::BoolType,
                optional: false,
                description: "True if status code indicates an error (4xx or 5xx)".to_string(),
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
        // Extract status code
        let code_input = inputs
            .iter()
            .find(|(name, _)| name == "code")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: code".to_string(),
                input_name: Some("code".to_string()),
                recovery_hint: Some("Connect a status code (e.g., 200, 404, 500) to this input".to_string()),
            })?;

        let code = match &code_input.1 {
            Value::U32Val(n) => *n,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected U32 for 'code', got {:?}", code_input.1),
                    input_name: Some("code".to_string()),
                    recovery_hint: Some("Provide a numeric status code between 100 and 599".to_string()),
                });
            }
        };

        // Validate status code range
        if code < 100 || code >= 600 {
            return Err(ExecutionError {
                message: format!("Invalid HTTP status code: {}. Must be between 100 and 599", code),
                input_name: Some("code".to_string()),
                recovery_hint: Some("Use a standard HTTP status code (1xx-5xx range)".to_string()),
            });
        }

        let message = get_status_message(code);
        let category = get_status_category(code);
        let is_error = code >= 400 && code < 600;

        Ok(vec![
            ("message".to_string(), Value::StringVal(message)),
            ("category".to_string(), Value::StringVal(category)),
            ("is_error".to_string(), Value::BoolVal(is_error)),
        ])
    }
}

// ============================================================================
// Status Code Mapping Logic
// ============================================================================

/// Get the standard HTTP reason phrase for a status code
fn get_status_message(code: u32) -> String {
    match code {
        // 1xx Informational
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",

        // 2xx Success
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",

        // 3xx Redirection
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",

        // 4xx Client Error
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",

        // 5xx Server Error
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",

        // Unknown status codes
        _ => "Unknown Status Code",
    }
    .to_string()
}

/// Get the category name for a status code based on its first digit
fn get_status_category(code: u32) -> String {
    match code / 100 {
        1 => "Informational",
        2 => "Success",
        3 => "Redirection",
        4 => "Client Error",
        5 => "Server Error",
        _ => "Unknown",
    }
    .to_string()
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_200_ok() {
        let inputs = vec![("code".to_string(), Value::U32Val(200))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 3);

        assert_eq!(result[0].0, "message");
        assert_eq!(result[0].1, Value::StringVal("OK".to_string()));

        assert_eq!(result[1].0, "category");
        assert_eq!(result[1].1, Value::StringVal("Success".to_string()));

        assert_eq!(result[2].0, "is_error");
        assert_eq!(result[2].1, Value::BoolVal(false));
    }

    #[test]
    fn test_404_not_found() {
        let inputs = vec![("code".to_string(), Value::U32Val(404))];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Not Found".to_string()));
        assert_eq!(result[1].1, Value::StringVal("Client Error".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(true));
    }

    #[test]
    fn test_500_internal_server_error() {
        let inputs = vec![("code".to_string(), Value::U32Val(500))];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Internal Server Error".to_string()));
        assert_eq!(result[1].1, Value::StringVal("Server Error".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(true));
    }

    #[test]
    fn test_301_moved_permanently() {
        let inputs = vec![("code".to_string(), Value::U32Val(301))];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Moved Permanently".to_string()));
        assert_eq!(result[1].1, Value::StringVal("Redirection".to_string()));
        assert_eq!(result[2].1, Value::BoolVal(false));
    }

    #[test]
    fn test_informational_codes() {
        let test_cases = vec![
            (100, "Continue"),
            (101, "Switching Protocols"),
            (103, "Early Hints"),
        ];

        for (code, expected_msg) in test_cases {
            let inputs = vec![("code".to_string(), Value::U32Val(code))];
            let result = Component::execute(inputs).unwrap();

            assert_eq!(result[0].1, Value::StringVal(expected_msg.to_string()));
            assert_eq!(result[1].1, Value::StringVal("Informational".to_string()));
            assert_eq!(result[2].1, Value::BoolVal(false));
        }
    }

    #[test]
    fn test_all_2xx_success() {
        let test_cases = vec![200, 201, 202, 204, 206];

        for code in test_cases {
            let inputs = vec![("code".to_string(), Value::U32Val(code))];
            let result = Component::execute(inputs).unwrap();

            assert_eq!(result[1].1, Value::StringVal("Success".to_string()));
            assert_eq!(result[2].1, Value::BoolVal(false));
        }
    }

    #[test]
    fn test_all_4xx_client_errors() {
        let test_cases = vec![400, 401, 403, 404, 405, 429];

        for code in test_cases {
            let inputs = vec![("code".to_string(), Value::U32Val(code))];
            let result = Component::execute(inputs).unwrap();

            assert_eq!(result[1].1, Value::StringVal("Client Error".to_string()));
            assert_eq!(result[2].1, Value::BoolVal(true));
        }
    }

    #[test]
    fn test_all_5xx_server_errors() {
        let test_cases = vec![500, 501, 502, 503, 504];

        for code in test_cases {
            let inputs = vec![("code".to_string(), Value::U32Val(code))];
            let result = Component::execute(inputs).unwrap();

            assert_eq!(result[1].1, Value::StringVal("Server Error".to_string()));
            assert_eq!(result[2].1, Value::BoolVal(true));
        }
    }

    #[test]
    fn test_teapot() {
        // RFC 2324 - Hyper Text Coffee Pot Control Protocol
        let inputs = vec![("code".to_string(), Value::U32Val(418))];
        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("I'm a teapot".to_string()));
    }

    #[test]
    fn test_unknown_status_code() {
        let inputs = vec![("code".to_string(), Value::U32Val(299))];
        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Unknown Status Code".to_string()));
        assert_eq!(result[1].1, Value::StringVal("Success".to_string())); // Still 2xx category
    }

    #[test]
    fn test_invalid_code_too_low() {
        let inputs = vec![("code".to_string(), Value::U32Val(99))];
        let result = Component::execute(inputs);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid HTTP status code"));
    }

    #[test]
    fn test_invalid_code_too_high() {
        let inputs = vec![("code".to_string(), Value::U32Val(600))];
        let result = Component::execute(inputs);

        assert!(result.is_err());
    }

    #[test]
    fn test_missing_input() {
        let inputs = vec![];
        let result = Component::execute(inputs);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("code".to_string()));
    }
}
