//! Integration tests for JSON parser node in graph context
//!
//! Tests T017, T022, T027, T032, T033

use wasmflow::builtin::json_parser::{parse, JsonValue, JsonParserError};

// ============================================================================
// T017: Integration test - Simple property extraction
// ============================================================================

#[test]
fn test_integration_simple_property() {
    let json_string = r#"{"version": 1}"#;
    let key_path = "version";

    let result = parse(json_string, key_path);

    assert!(result.is_ok(), "Expected successful parse");
    assert_eq!(result.unwrap(), JsonValue::Number(1.0));
}

// ============================================================================
// T022: Integration test - Nested property extraction
// ============================================================================

#[test]
fn test_integration_nested_property() {
    let json_string = r#"{"metadata": {"author": "me"}}"#;
    let key_path = "metadata.author";

    let result = parse(json_string, key_path);

    assert!(result.is_ok(), "Expected successful parse");
    assert_eq!(result.unwrap(), JsonValue::String("me".to_string()));
}

// ============================================================================
// T027: Integration test - Array indexing
// ============================================================================

#[test]
fn test_integration_array_indexing() {
    let json_string = r#"{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}"#;
    let key_path = "runs[1].time";

    let result = parse(json_string, key_path);

    assert!(result.is_ok(), "Expected successful parse");
    assert_eq!(result.unwrap(), JsonValue::Number(1000.0));
}

// ============================================================================
// T032: Integration test - Complex mixed notation
// ============================================================================

#[test]
fn test_integration_complex_path() {
    let json_string = r#"{"data": {"items": [{"value": {"score": 95}}]}}"#;
    let key_path = "data.items[0].value.score";

    let result = parse(json_string, key_path);

    assert!(result.is_ok(), "Expected successful parse");
    assert_eq!(result.unwrap(), JsonValue::Number(95.0));
}

// ============================================================================
// T033: Integration test - Multiple parsers on same JSON (chaining)
// ============================================================================

#[test]
fn test_integration_multiple_parsers_same_source() {
    let json_source = r#"{
        "version": 1,
        "metadata": {"author": "me"},
        "runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]
    }"#;

    // Parser 1: Extract version
    let result1 = parse(json_source, "version");
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), JsonValue::Number(1.0));

    // Parser 2: Extract author
    let result2 = parse(json_source, "metadata.author");
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), JsonValue::String("me".to_string()));

    // Parser 3: Extract time from second run
    let result3 = parse(json_source, "runs[1].time");
    assert!(result3.is_ok());
    assert_eq!(result3.unwrap(), JsonValue::Number(1000.0));
}

// ============================================================================
// Error handling integration tests
// ============================================================================

#[test]
fn test_integration_error_invalid_json() {
    let result = parse("{invalid json", "version");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::InvalidJson(_)) => (),
        _ => panic!("Expected InvalidJson error"),
    }
}

#[test]
fn test_integration_error_path_not_found() {
    let result = parse(r#"{"version": 1}"#, "nonexistent");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::PathNotFound(_)) => (),
        _ => panic!("Expected PathNotFound error"),
    }
}

#[test]
fn test_integration_error_index_out_of_bounds() {
    let result = parse(r#"{"runs": [1, 2]}"#, "runs[999]");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::IndexOutOfBounds(idx, len)) => {
            assert_eq!(idx, 999);
            assert_eq!(len, 2);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}
