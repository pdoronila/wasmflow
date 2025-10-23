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
            name: "JSON Stringify".to_string(),
            version: "1.0.0".to_string(),
            description: "Serializes any data value to a JSON string representation".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Data".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "data".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "The data to serialize to JSON (any primitive or list type)".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "json".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "The JSON string representation of the input data".to_string(),
        }]
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
        // Extract data input
        let data = inputs
            .iter()
            .find(|(name, _)| name == "data")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: data".to_string(),
                input_name: Some("data".to_string()),
                recovery_hint: Some("Connect a value to this input".to_string()),
            })?;

        // Convert Value to JSON
        let json_string = match &data.1 {
            Value::U32Val(n) => serde_json::to_string(n),
            Value::I32Val(n) => serde_json::to_string(n),
            Value::F32Val(n) => serde_json::to_string(n),
            Value::StringVal(s) => serde_json::to_string(s),
            Value::BoolVal(b) => serde_json::to_string(b),
            Value::BinaryVal(bytes) => serde_json::to_string(bytes),
            Value::StringListVal(items) => serde_json::to_string(items),
            Value::U32ListVal(items) => serde_json::to_string(items),
            Value::F32ListVal(items) => serde_json::to_string(items),
        }
        .map_err(|e| ExecutionError {
            message: format!("Failed to serialize to JSON: {}", e),
            input_name: Some("data".to_string()),
            recovery_hint: Some("Ensure the input data is valid for JSON serialization".to_string()),
        })?;

        Ok(vec![("json".to_string(), Value::StringVal(json_string))])
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
    fn test_serialize_number() {
        let inputs = vec![("data".to_string(), Value::U32Val(42))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "json");
        assert_eq!(result[0].1, Value::StringVal("42".to_string()));
    }

    #[test]
    fn test_serialize_string() {
        let inputs = vec![("data".to_string(), Value::StringVal("hello world".to_string()))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "json");
        // String should be JSON-quoted
        assert_eq!(result[0].1, Value::StringVal("\"hello world\"".to_string()));
    }

    #[test]
    fn test_serialize_boolean() {
        let inputs = vec![("data".to_string(), Value::BoolVal(true))];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "json");
        assert_eq!(result[0].1, Value::StringVal("true".to_string()));
    }

    #[test]
    fn test_serialize_string_list() {
        let inputs = vec![(
            "data".to_string(),
            Value::StringListVal(vec![
                "apple".to_string(),
                "banana".to_string(),
                "cherry".to_string(),
            ]),
        )];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "json");

        if let Value::StringVal(json) = &result[0].1 {
            // Verify it's valid JSON array
            assert_eq!(json, "[\"apple\",\"banana\",\"cherry\"]");
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_serialize_u32_list() {
        let inputs = vec![(
            "data".to_string(),
            Value::U32ListVal(vec![1, 2, 3, 4, 5]),
        )];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::StringVal(json) = &result[0].1 {
            assert_eq!(json, "[1,2,3,4,5]");
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_serialize_f32_list() {
        let inputs = vec![(
            "data".to_string(),
            Value::F32ListVal(vec![1.5, 2.5, 3.5]),
        )];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::StringVal(json) = &result[0].1 {
            // serde_json represents f32 without trailing zeros
            assert_eq!(json, "[1.5,2.5,3.5]");
        } else {
            panic!("Expected StringVal");
        }
    }

    #[test]
    fn test_serialize_empty_list() {
        let inputs = vec![(
            "data".to_string(),
            Value::StringListVal(vec![]),
        )];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        if let Value::StringVal(json) = &result[0].1 {
            assert_eq!(json, "[]");
        } else {
            panic!("Expected StringVal");
        }
    }
}

