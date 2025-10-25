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
            name: "Simple Template Render".to_string(),
            version: "1.0.0".to_string(),
            description: "Renders templates by replacing {{key}} placeholders with values from JSON data".to_string(),
            author: "WasmFlow Web Server Library".to_string(),
            category: Some("HTTP".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "template".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Template string with {{key}} placeholders (e.g., 'Hello {{name}}!')".to_string(),
            },
            PortSpec {
                name: "data".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Data as JSON object (e.g., '{\"name\":\"Alice\"}')".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "rendered".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Rendered template with placeholders replaced".to_string(),
            },
            PortSpec {
                name: "placeholder_count".to_string(),
                data_type: DataType::U32Type,
                optional: false,
                description: "Number of placeholders found and replaced".to_string(),
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
        // Extract template
        let template = inputs
            .iter()
            .find(|(name, _)| name == "template")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: template".to_string(),
                input_name: Some("template".to_string()),
                recovery_hint: Some("Connect a template string to this input".to_string()),
            })?;

        let template_str = match &template.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'template', got {:?}", template.1),
                    input_name: Some("template".to_string()),
                    recovery_hint: Some("Provide a template string".to_string()),
                });
            }
        };

        // Extract data
        let data = inputs
            .iter()
            .find(|(name, _)| name == "data")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: data".to_string(),
                input_name: Some("data".to_string()),
                recovery_hint: Some("Connect JSON data to this input".to_string()),
            })?;

        let data_str = match &data.1 {
            Value::StringVal(s) => s,
            _ => {
                return Err(ExecutionError {
                    message: format!("Expected string for 'data', got {:?}", data.1),
                    input_name: Some("data".to_string()),
                    recovery_hint: Some("Provide data as JSON string".to_string()),
                });
            }
        };

        // Parse data JSON
        let data_map = parse_json_data(data_str).map_err(|e| ExecutionError {
            message: format!("Failed to parse data JSON: {}", e),
            input_name: Some("data".to_string()),
            recovery_hint: Some("Provide valid JSON object (e.g., {{\"key\":\"value\"}})".to_string()),
        })?;

        // Render template
        let (rendered, placeholder_count) = render_template(template_str, &data_map);

        Ok(vec![
            ("rendered".to_string(), Value::StringVal(rendered)),
            ("placeholder_count".to_string(), Value::U32Val(placeholder_count)),
        ])
    }
}

// ============================================================================
// Template Rendering Logic
// ============================================================================

/// Render template by replacing {{key}} placeholders
fn render_template(template: &str, data: &[(String, String)]) -> (String, u32) {
    let mut result = template.to_string();
    let mut placeholder_count = 0;

    // Replace each placeholder
    for (key, value) in data {
        let placeholder = format!("{{{{{}}}}}", key);

        // Count occurrences before replacement
        let count = result.matches(&placeholder).count();
        placeholder_count += count as u32;

        // Replace all occurrences
        result = result.replace(&placeholder, value);
    }

    (result, placeholder_count)
}

/// Parse JSON data into key-value pairs
fn parse_json_data(json_str: &str) -> Result<Vec<(String, String)>, String> {
    let trimmed = json_str.trim();

    if trimmed.is_empty() || trimmed == "{}" {
        return Ok(Vec::new());
    }

    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err("Data must be a JSON object (start with { and end with })".to_string());
    }

    let content = &trimmed[1..trimmed.len() - 1]; // Remove { and }

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut data = Vec::new();

    // Split by commas (simple parser)
    for pair in content.split(',') {
        let pair = pair.trim();

        if let Some(colon_pos) = pair.find(':') {
            let key_part = pair[..colon_pos].trim();
            let value_part = pair[colon_pos + 1..].trim();

            // Remove quotes from key and value
            let key = key_part.trim_matches('"').to_string();
            let value = value_part.trim_matches('"').to_string();

            data.push((key, value));
        } else {
            return Err(format!("Invalid key-value pair in JSON: {}", pair));
        }
    }

    Ok(data)
}


// ============================================================================
export!(Component);

// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_replacement() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Hello {{name}}!".to_string())),
            ("data".to_string(), Value::StringVal("{\"name\":\"Alice\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Hello Alice!".to_string()));
        assert_eq!(result[1].1, Value::U32Val(1)); // 1 placeholder replaced
    }

    #[test]
    fn test_multiple_placeholders() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("{{greeting}} {{name}}, you are {{age}} years old!".to_string())),
            ("data".to_string(), Value::StringVal("{\"greeting\":\"Hello\",\"name\":\"Bob\",\"age\":\"30\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Hello Bob, you are 30 years old!".to_string()));
        assert_eq!(result[1].1, Value::U32Val(3)); // 3 placeholders
    }

    #[test]
    fn test_repeated_placeholder() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("{{name}} said {{name}} again!".to_string())),
            ("data".to_string(), Value::StringVal("{\"name\":\"Echo\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Echo said Echo again!".to_string()));
        assert_eq!(result[1].1, Value::U32Val(2)); // 2 occurrences
    }

    #[test]
    fn test_no_placeholders() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("This is plain text".to_string())),
            ("data".to_string(), Value::StringVal("{\"name\":\"Alice\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("This is plain text".to_string()));
        assert_eq!(result[1].1, Value::U32Val(0)); // No placeholders
    }

    #[test]
    fn test_unused_data_keys() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Hello {{name}}!".to_string())),
            ("data".to_string(), Value::StringVal("{\"name\":\"Alice\",\"age\":\"25\",\"city\":\"NYC\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Hello Alice!".to_string()));
        // Unused keys (age, city) don't cause errors
    }

    #[test]
    fn test_missing_data_key() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Hello {{name}}, from {{city}}!".to_string())),
            ("data".to_string(), Value::StringVal("{\"name\":\"Alice\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        // Missing key leaves placeholder unchanged
        assert_eq!(result[0].1, Value::StringVal("Hello Alice, from {{city}}!".to_string()));
        assert_eq!(result[1].1, Value::U32Val(1)); // Only name was replaced
    }

    #[test]
    fn test_empty_data() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Hello {{name}}!".to_string())),
            ("data".to_string(), Value::StringVal("{}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Hello {{name}}!".to_string())); // Unchanged
        assert_eq!(result[1].1, Value::U32Val(0));
    }

    #[test]
    fn test_html_template() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("<h1>{{title}}</h1><p>{{content}}</p>".to_string())),
            ("data".to_string(), Value::StringVal("{\"title\":\"Welcome\",\"content\":\"Hello World\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("<h1>Welcome</h1><p>Hello World</p>".to_string()));
    }

    #[test]
    fn test_numeric_values() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Price: ${{price}}".to_string())),
            ("data".to_string(), Value::StringVal("{\"price\":\"99.99\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Price: $99.99".to_string()));
    }

    #[test]
    fn test_empty_value() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Value: {{value}}".to_string())),
            ("data".to_string(), Value::StringVal("{\"value\":\"\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Value: ".to_string()));
    }

    #[test]
    fn test_special_characters_in_value() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Message: {{msg}}".to_string())),
            ("data".to_string(), Value::StringVal("{\"msg\":\"Hello & goodbye!\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("Message: Hello & goodbye!".to_string()));
    }

    #[test]
    fn test_multiline_template() {
        let template = "Name: {{name}}\nAge: {{age}}\nCity: {{city}}";
        let inputs = vec![
            ("template".to_string(), Value::StringVal(template.to_string())),
            ("data".to_string(), Value::StringVal("{\"name\":\"Alice\",\"age\":\"30\",\"city\":\"NYC\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        let expected = "Name: Alice\nAge: 30\nCity: NYC";
        assert_eq!(result[0].1, Value::StringVal(expected.to_string()));
    }

    #[test]
    fn test_invalid_json_data() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Hello {{name}}!".to_string())),
            ("data".to_string(), Value::StringVal("not valid json".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_template_input() {
        let inputs = vec![
            ("data".to_string(), Value::StringVal("{\"name\":\"Alice\"}".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("template".to_string()));
    }

    #[test]
    fn test_missing_data_input() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("Hello {{name}}!".to_string())),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("data".to_string()));
    }

    #[test]
    fn test_url_in_template() {
        let inputs = vec![
            ("template".to_string(), Value::StringVal("<a href=\"{{url}}\">{{text}}</a>".to_string())),
            ("data".to_string(), Value::StringVal("{\"url\":\"https://example.com\",\"text\":\"Click here\"}".to_string())),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result[0].1, Value::StringVal("<a href=\"https://example.com\">Click here</a>".to_string()));
    }
}
