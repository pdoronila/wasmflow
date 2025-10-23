//! Contract tests for JSON Parser WIT interface compliance
//!
//! Tests T034, T035

use wasmflow::builtin::json_parser::{parse, JsonValue, JsonParserError};

// ============================================================================
// T034: WIT interface validation tests
// ============================================================================

/// Test that all JsonValue variants map correctly to WIT types
#[test]
fn test_json_value_variants_complete() {
    // String variant
    let result = parse(r#"{"str": "hello"}"#, "str");
    assert!(matches!(result, Ok(JsonValue::String(_))));

    // Number variant (f64)
    let result = parse(r#"{"num": 42.5}"#, "num");
    assert!(matches!(result, Ok(JsonValue::Number(_))));

    // Boolean variant
    let result = parse(r#"{"bool": true}"#, "bool");
    assert!(matches!(result, Ok(JsonValue::Boolean(_))));

    // Object variant (serialized JSON)
    let result = parse(r#"{"obj": {"key": "value"}}"#, "obj");
    assert!(matches!(result, Ok(JsonValue::Object(_))));

    // Array variant (serialized JSON)
    let result = parse(r#"{"arr": [1, 2, 3]}"#, "arr");
    assert!(matches!(result, Ok(JsonValue::Array(_))));

    // Null variant
    let result = parse(r#"{"null": null}"#, "null");
    assert!(matches!(result, Ok(JsonValue::Null)));
}

/// Test that error kinds match WIT enum definition
#[test]
fn test_error_kind_variants_complete() {
    // invalid-json
    let result = parse("{invalid", "key");
    assert!(matches!(result, Err(JsonParserError::InvalidJson(_))));

    // path-not-found
    let result = parse("{}", "missing");
    assert!(matches!(result, Err(JsonParserError::PathNotFound(_))));

    // malformed-path
    let result = parse("{}", "");
    assert!(matches!(result, Err(JsonParserError::MalformedPath(_))));

    // index-out-of-bounds
    let result = parse(r#"{"arr": [1, 2]}"#, "arr[999]");
    assert!(matches!(
        result,
        Err(JsonParserError::IndexOutOfBounds(_, _))
    ));

    // type-mismatch
    let result = parse(r#"{"num": 1}"#, "num.property");
    assert!(matches!(result, Err(JsonParserError::TypeMismatch(_))));
}

/// Test that parse function signature matches WIT interface
/// WIT: parse: func(json-string: string, key-path: string) -> result<json-value, parse-error>
#[test]
fn test_parse_signature() {
    // Success case - returns Ok(JsonValue)
    let result = parse(r#"{"version": 1}"#, "version");
    assert!(result.is_ok());

    // Error case - returns Err(JsonParserError)
    let result = parse("{invalid", "key");
    assert!(result.is_err());
}

// ============================================================================
// T035: Type conversion correctness tests
// ============================================================================

/// Test all JSON type → JsonValue conversions
#[test]
fn test_type_conversion_string() {
    let result = parse(r#"{"str": "hello world"}"#, "str").unwrap();
    assert_eq!(result, JsonValue::String("hello world".to_string()));
}

#[test]
fn test_type_conversion_number_integer() {
    let result = parse(r#"{"num": 42}"#, "num").unwrap();
    assert_eq!(result, JsonValue::Number(42.0));
}

#[test]
fn test_type_conversion_number_float() {
    let result = parse(r#"{"num": 3.14159}"#, "num").unwrap();
    assert_eq!(result, JsonValue::Number(3.14159));
}

#[test]
fn test_type_conversion_number_negative() {
    let result = parse(r#"{"num": -273.15}"#, "num").unwrap();
    assert_eq!(result, JsonValue::Number(-273.15));
}

#[test]
fn test_type_conversion_boolean_true() {
    let result = parse(r#"{"bool": true}"#, "bool").unwrap();
    assert_eq!(result, JsonValue::Boolean(true));
}

#[test]
fn test_type_conversion_boolean_false() {
    let result = parse(r#"{"bool": false}"#, "bool").unwrap();
    assert_eq!(result, JsonValue::Boolean(false));
}

/// Test that object serialization produces valid JSON
#[test]
fn test_type_conversion_object_valid_json() {
    let result = parse(r#"{"obj": {"id": 1, "name": "test"}}"#, "obj").unwrap();

    match result {
        JsonValue::Object(json_str) => {
            // Verify it's valid JSON by parsing it again
            let reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            assert!(reparsed.is_object());
            assert_eq!(reparsed["id"], 1);
            assert_eq!(reparsed["name"], "test");
        }
        _ => panic!("Expected Object variant"),
    }
}

/// Test that array serialization produces valid JSON
#[test]
fn test_type_conversion_array_valid_json() {
    let result = parse(r#"{"arr": [1, 2, 3, 4, 5]}"#, "arr").unwrap();

    match result {
        JsonValue::Array(json_str) => {
            // Verify it's valid JSON by parsing it again
            let reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            assert!(reparsed.is_array());
            assert_eq!(reparsed.as_array().unwrap().len(), 5);
            assert_eq!(reparsed[0], 1);
        }
        _ => panic!("Expected Array variant"),
    }
}

/// Test null handling (FR-012)
#[test]
fn test_type_conversion_null() {
    let result = parse(r#"{"value": null}"#, "value").unwrap();
    assert_eq!(result, JsonValue::Null);
}

/// Test that null is distinct from missing
#[test]
fn test_null_vs_missing() {
    // Null value exists and returns JsonValue::Null
    let result_null = parse(r#"{"value": null}"#, "value");
    assert!(result_null.is_ok());
    assert_eq!(result_null.unwrap(), JsonValue::Null);

    // Missing value returns PathNotFound error
    let result_missing = parse(r#"{}"#, "value");
    assert!(result_missing.is_err());
    assert!(matches!(
        result_missing,
        Err(JsonParserError::PathNotFound(_))
    ));
}

/// Test empty object and array handling
#[test]
fn test_empty_object() {
    let result = parse(r#"{"obj": {}}"#, "obj").unwrap();
    match result {
        JsonValue::Object(s) => assert_eq!(s, "{}"),
        _ => panic!("Expected Object variant"),
    }
}

#[test]
fn test_empty_array() {
    let result = parse(r#"{"arr": []}"#, "arr").unwrap();
    match result {
        JsonValue::Array(s) => assert_eq!(s, "[]"),
        _ => panic!("Expected Array variant"),
    }
}

/// Test number precision (f64)
#[test]
fn test_number_precision() {
    let result = parse(r#"{"num": 1.7976931348623157e+308}"#, "num").unwrap();
    match result {
        JsonValue::Number(n) => {
            // Verify f64 precision is maintained
            assert!(n > 1.0e308);
        }
        _ => panic!("Expected Number variant"),
    }
}

/// Test special number values
#[test]
fn test_number_zero() {
    let result = parse(r#"{"num": 0}"#, "num").unwrap();
    assert_eq!(result, JsonValue::Number(0.0));
}

#[test]
fn test_number_scientific_notation() {
    let result = parse(r#"{"num": 1.23e10}"#, "num").unwrap();
    assert_eq!(result, JsonValue::Number(1.23e10));
}
