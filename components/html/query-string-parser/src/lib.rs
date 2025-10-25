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
            name: "Query String Parser".to_string(),
            version: "1.0.0".to_string(),
            description: "Parses URL query strings into JSON object (e.g., '?name=John&age=30')".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "query_string".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Query string to parse (with or without leading '?')".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "params".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Parsed parameters as JSON object (e.g., {\"name\":\"John\",\"age\":\"30\"})".to_string(),
            },
            PortSpec {
                name: "param_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of parameters parsed".to_string(),
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
        // Extract query_string
        let query_input = inputs
            .iter()
            .find(|(name, _)| name == "query_string")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: query_string".to_string(),
                input_name: Some("query_string".to_string()),
                recovery_hint: Some("Connect a query string to this input".to_string()),
            })?;

        let query_str = match &query_input.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'query_string', got {:?}", query_input.1),
                    input_name: Some("query_string".to_string()),
                    recovery_hint: Some("Provide a string query string".to_string()),
                });
            }
        };

        // Parse the query string
        let params = parse_query_string(query_str);

        // Build JSON object
        let params_json = build_params_json(&params);
        let param_count = params.len() as u32;

        Ok(vec![
            ("params".to_string(), Value::StringVal(params_json)),
            ("param_count".to_string(), Value::U32Val(param_count)),
        ])
    }
}

// ============================================================================
// Query String Parsing Logic
// ============================================================================

/// Parse a query string into key-value pairs
fn parse_query_string(query: &str) -> Vec<(String, String)> {
    let mut params = Vec::new();

    // Remove leading '?' if present
    let query = query.strip_prefix('?').unwrap_or(query);

    if query.is_empty() {
        return params;
    }

    // Split by '&' to get individual parameters
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }

        // Split by '=' to get key and value
        if let Some(equals_pos) = pair.find('=') {
            let key = &pair[..equals_pos];
            let value = &pair[equals_pos + 1..];

            // URL decode both key and value
            let decoded_key = url_decode(key);
            let decoded_value = url_decode(value);

            params.push((decoded_key, decoded_value));
        } else {
            // No '=' found - treat as key with empty value
            let decoded_key = url_decode(pair);
            params.push((decoded_key, String::new()));
        }
    }

    params
}

/// URL decode a string (decode %XX sequences and + as space)
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                // Try to parse next two characters as hex
                let hex1 = chars.next();
                let hex2 = chars.next();

                if let (Some(h1), Some(h2)) = (hex1, hex2) {
                    if let Some(decoded) = decode_hex_pair(h1, h2) {
                        result.push(decoded);
                    } else {
                        // Invalid hex sequence - keep as-is
                        result.push('%');
                        result.push(h1);
                        result.push(h2);
                    }
                } else {
                    // Incomplete hex sequence - keep '%'
                    result.push('%');
                    if let Some(h1) = hex1 {
                        result.push(h1);
                    }
                }
            }
            _ => result.push(c),
        }
    }

    result
}

/// Decode a hex pair (e.g., "20" -> ' ')
fn decode_hex_pair(h1: char, h2: char) -> Option<char> {
    let hex_str = format!("{}{}", h1, h2);
    u8::from_str_radix(&hex_str, 16)
        .ok()
        .and_then(|byte| {
            // Only decode valid ASCII characters for simplicity
            if byte.is_ascii() {
                Some(byte as char)
            } else {
                None
            }
        })
}

/// Build JSON object from parameter key-value pairs
fn build_params_json(params: &[(String, String)]) -> String {
    if params.is_empty() {
        return "{}".to_string();
    }

    let pairs: Vec<String> = params
        .iter()
        .map(|(key, value)| {
            let escaped_key = escape_json_string(key);
            let escaped_value = escape_json_string(value);
            format!("\"{}\":\"{}\"", escaped_key, escaped_value)
        })
        .collect();

    format!("{{{}}}", pairs.join(","))
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
    fn test_simple_query_string() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("name=John&age=30".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::U32Val(2)); // param_count

        if let Value::StringVal(params) = &result[0].1 {
            assert!(params.contains("\"name\":\"John\""));
            assert!(params.contains("\"age\":\"30\""));
        } else {
            panic!("Expected StringVal for params");
        }
    }

    #[test]
    fn test_query_string_with_leading_question_mark() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("?search=rust&page=2".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::U32Val(2));

        if let Value::StringVal(params) = &result[0].1 {
            assert!(params.contains("\"search\":\"rust\""));
            assert!(params.contains("\"page\":\"2\""));
        }
    }

    #[test]
    fn test_empty_query_string() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("{}".to_string()));
        assert_eq!(result[1].1, Value::U32Val(0));
    }

    #[test]
    fn test_query_string_with_only_question_mark() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("?".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("{}".to_string()));
        assert_eq!(result[1].1, Value::U32Val(0));
    }

    #[test]
    fn test_url_encoded_values() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("message=Hello%20World&email=user%40example.com".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(params) = &result[0].1 {
            assert!(params.contains("\"message\":\"Hello World\""));
            assert!(params.contains("\"email\":\"user@example.com\""));
        }
    }

    #[test]
    fn test_plus_as_space() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("name=John+Doe&city=New+York".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(params) = &result[0].1 {
            assert!(params.contains("\"name\":\"John Doe\""));
            assert!(params.contains("\"city\":\"New York\""));
        }
    }

    #[test]
    fn test_parameter_without_value() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("debug&verbose=true".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::U32Val(2));

        if let Value::StringVal(params) = &result[0].1 {
            assert!(params.contains("\"debug\":\"\""));
            assert!(params.contains("\"verbose\":\"true\""));
        }
    }

    #[test]
    fn test_empty_parameter_value() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("name=&age=30".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(params) = &result[0].1 {
            assert!(params.contains("\"name\":\"\""));
            assert!(params.contains("\"age\":\"30\""));
        }
    }

    #[test]
    fn test_duplicate_keys_last_wins() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("id=1&id=2&id=3".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        // All three should be in the JSON (we don't dedupe - last one wins in most parsers)
        assert_eq!(result[1].1, Value::U32Val(3));
    }

    #[test]
    fn test_special_characters_in_values() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("data=%7B%22key%22%3A%22value%22%7D".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(params) = &result[0].1 {
            // %7B = {, %22 = ", %3A = :, %7D = }
            assert!(params.contains("\"data\":\"{\\\"key\\\":\\\"value\\\"}\""));
        }
    }

    #[test]
    fn test_ampersand_edge_cases() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("a=1&&b=2&&&c=3".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        // Should skip empty pairs from consecutive ampersands
        assert_eq!(result[1].1, Value::U32Val(3));
    }

    #[test]
    fn test_invalid_percent_encoding() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("test=%ZZ&valid=%20".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(params) = &result[0].1 {
            // Invalid hex should be kept as-is
            assert!(params.contains("\"test\":\"%ZZ\""));
            assert!(params.contains("\"valid\":\" \""));
        }
    }

    #[test]
    fn test_missing_input() {
        let inputs = vec![];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("query_string".to_string()));
    }

    #[test]
    fn test_common_search_query() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("?q=rust+programming&sort=relevance&filter=recent".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[1].1, Value::U32Val(3));

        if let Value::StringVal(params) = &result[0].1 {
            assert!(params.contains("\"q\":\"rust programming\""));
            assert!(params.contains("\"sort\":\"relevance\""));
            assert!(params.contains("\"filter\":\"recent\""));
        }
    }

    #[test]
    fn test_numeric_values() {
        let inputs = vec![
            ("query_string".to_string(), Value::StringVal("page=1&limit=10&offset=50".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        if let Value::StringVal(params) = &result[0].1 {
            // Values are still strings in JSON
            assert!(params.contains("\"page\":\"1\""));
            assert!(params.contains("\"limit\":\"10\""));
            assert!(params.contains("\"offset\":\"50\""));
        }
    }
}
