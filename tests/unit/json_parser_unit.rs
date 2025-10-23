//! Unit tests for JSON parser node
//!
//! Tests cover:
//! - Simple property extraction (T013)
//! - Error handling (T014)
//! - Nested properties (T018-T019)
//! - Array indexing (T023-T024)
//! - Combined notation (T028-T029)
//! - Edge cases (T041)

use wasmflow::builtin::json_parser::*;

// ============================================================================
// T013: Simple property extraction tests
// ============================================================================

#[test]
fn test_simple_number_extraction() {
    let result = parse(r#"{"version": 1}"#, "version");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Number(1.0));
}

#[test]
fn test_simple_string_extraction() {
    let result = parse(r#"{"author": "me"}"#, "author");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::String("me".to_string()));
}

#[test]
fn test_simple_boolean_extraction() {
    let result = parse(r#"{"enabled": true}"#, "enabled");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Boolean(true));
}

#[test]
fn test_simple_null_extraction() {
    let result = parse(r#"{"value": null}"#, "value");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Null);
}

// ============================================================================
// T014: Error handling tests
// ============================================================================

#[test]
fn test_invalid_json() {
    let result = parse("{invalid", "version");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::InvalidJson(_)) => (),
        _ => panic!("Expected InvalidJson error"),
    }
}

#[test]
fn test_empty_key_path() {
    let result = parse(r#"{"version": 1}"#, "");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::MalformedPath(_)) => (),
        _ => panic!("Expected MalformedPath error"),
    }
}

#[test]
fn test_nonexistent_key() {
    let result = parse("{}", "missing");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::PathNotFound(_)) => (),
        _ => panic!("Expected PathNotFound error"),
    }
}

// ============================================================================
// T018: Nested property extraction tests
// ============================================================================

#[test]
fn test_nested_property_two_levels() {
    let result = parse(r#"{"metadata": {"author": "me"}}"#, "metadata.author");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::String("me".to_string()));
}

#[test]
fn test_nested_property_three_levels() {
    let result = parse(
        r#"{"config": {"server": {"port": 8080}}}"#,
        "config.server.port",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Number(8080.0));
}

#[test]
fn test_deep_nesting_four_levels() {
    let result = parse(
        r#"{"a": {"b": {"c": {"d": "deep"}}}}"#,
        "a.b.c.d",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::String("deep".to_string()));
}

// ============================================================================
// T019: Nested error cases
// ============================================================================

#[test]
fn test_nested_path_not_found() {
    let result = parse(r#"{"metadata": {}}"#, "metadata.missing");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::PathNotFound(_)) => (),
        _ => panic!("Expected PathNotFound error"),
    }
}

#[test]
fn test_property_on_number_type_mismatch() {
    let result = parse(r#"{"version": 1}"#, "version.property");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::TypeMismatch(_)) => (),
        _ => panic!("Expected TypeMismatch error"),
    }
}

// ============================================================================
// T023: Array index extraction tests
// ============================================================================

#[test]
fn test_array_index_object() {
    let result = parse(r#"{"runs": [{"id": 1}, {"id": 2}]}"#, "runs[1]");
    assert!(result.is_ok());
    // Should be an object (serialized as JSON string)
    match result.unwrap() {
        JsonValue::Object(s) => {
            assert!(s.contains("\"id\""));
            assert!(s.contains("2"));
        }
        _ => panic!("Expected Object variant"),
    }
}

#[test]
fn test_array_index_number() {
    let result = parse(r#"{"values": [10, 20, 30]}"#, "values[0]");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Number(10.0));
}

#[test]
fn test_array_index_string() {
    let result = parse(r#"{"items": ["first", "second", "third"]}"#, "items[2]");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::String("third".to_string()));
}

// ============================================================================
// T024: Array error cases
// ============================================================================

#[test]
fn test_array_index_out_of_bounds() {
    let result = parse(r#"{"runs": [{"id": 1}, {"id": 2}]}"#, "runs[999]");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::IndexOutOfBounds(idx, len)) => {
            assert_eq!(idx, 999);
            assert_eq!(len, 2);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_index_on_non_array_type_mismatch() {
    let result = parse(r#"{"metadata": {"author": "me"}}"#, "metadata[0]");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::TypeMismatch(_)) => (),
        _ => panic!("Expected TypeMismatch error"),
    }
}

#[test]
fn test_malformed_array_index_non_numeric() {
    let result = parse(r#"{"runs": [1, 2]}"#, "runs[abc]");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::MalformedPath(_)) => (),
        _ => panic!("Expected MalformedPath error"),
    }
}

// ============================================================================
// T028: Combined notation extraction tests
// ============================================================================

#[test]
fn test_combined_array_then_property() {
    let result = parse(
        r#"{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}"#,
        "runs[1].time",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Number(1000.0));
}

#[test]
fn test_combined_property_then_array() {
    let result = parse(
        r#"{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}"#,
        "users[0].name",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::String("Alice".to_string()));
}

#[test]
fn test_complex_deep_mixed_path() {
    let result = parse(
        r#"{"data": {"items": [{"value": {"score": 95}}]}}"#,
        "data.items[0].value.score",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Number(95.0));
}

// ============================================================================
// T029: Deep nesting and large arrays tests
// ============================================================================

#[test]
fn test_ten_level_deep_nesting() {
    let json = r#"{"a": {"b": {"c": {"d": {"e": {"f": {"g": {"h": {"i": {"j": "deep"}}}}}}}}}}"#;
    let result = parse(json, "a.b.c.d.e.f.g.h.i.j");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::String("deep".to_string()));
}

#[test]
fn test_large_array_access() {
    // Create an array with 1000+ elements
    let mut arr = Vec::new();
    for i in 0..1001 {
        arr.push(i);
    }
    let json_str = format!(r#"{{"values": {}}}"#, serde_json::to_string(&arr).unwrap());

    let result = parse(&json_str, "values[1000]");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Number(1000.0));
}

// ============================================================================
// Tokenizer tests (supporting T009)
// ============================================================================

#[test]
fn test_tokenize_simple_property() {
    let tokens = tokenize("version").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Ident("version".to_string()));
}

#[test]
fn test_tokenize_nested_property() {
    let tokens = tokenize("metadata.author").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], Token::Ident("metadata".to_string()));
    assert_eq!(tokens[1], Token::Ident("author".to_string()));
}

#[test]
fn test_tokenize_array_index() {
    let tokens = tokenize("runs[1]").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], Token::Ident("runs".to_string()));
    assert_eq!(tokens[1], Token::Index(1));
}

#[test]
fn test_tokenize_combined_notation() {
    let tokens = tokenize("runs[1].time").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Token::Ident("runs".to_string()));
    assert_eq!(tokens[1], Token::Index(1));
    assert_eq!(tokens[2], Token::Ident("time".to_string()));
}

#[test]
fn test_tokenize_malformed_double_dot() {
    let result = tokenize("metadata..author");
    assert!(result.is_err());
}

#[test]
fn test_tokenize_malformed_leading_digit() {
    let result = tokenize("123invalid");
    assert!(result.is_err());
}

// ============================================================================
// T041: Comprehensive edge case tests from data-model.md
// ============================================================================

/// Edge Case 1: Empty key path
#[test]
fn test_edge_case_empty_key_path() {
    let result = parse(r#"{"version": 1}"#, "");
    assert!(result.is_err());
    match result {
        Err(JsonParserError::MalformedPath(_)) => (),
        _ => panic!("Expected MalformedPath error for empty path"),
    }
}

/// Edge Case 2: Null values in JSON (should return JsonValue::Null)
#[test]
fn test_edge_case_null_value() {
    let result = parse(r#"{"value": null}"#, "value");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Null);
}

/// Edge Case 3: Missing vs Null (distinct behaviors)
#[test]
fn test_edge_case_missing_vs_null() {
    // Null value exists
    let result_null = parse(r#"{"value": null}"#, "value");
    assert!(result_null.is_ok());
    assert_eq!(result_null.unwrap(), JsonValue::Null);

    // Missing value
    let result_missing = parse(r#"{}"#, "value");
    assert!(result_missing.is_err());
    assert!(matches!(
        result_missing,
        Err(JsonParserError::PathNotFound(_))
    ));
}

/// Edge Case 4: Deep nesting (10+ levels)
#[test]
fn test_edge_case_deep_nesting() {
    let json = r#"{"a":{"b":{"c":{"d":{"e":{"f":{"g":{"h":{"i":{"j":"deep"}}}}}}}}}}"#;
    let result = parse(json, "a.b.c.d.e.f.g.h.i.j");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::String("deep".to_string()));
}

/// Edge Case 5: Large arrays (1000+ elements)
#[test]
fn test_edge_case_large_array() {
    let arr: Vec<i32> = (0..1001).collect();
    let json = format!(r#"{{"values": {}}}"#, serde_json::to_string(&arr).unwrap());

    let result = parse(&json, "values[1000]");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Number(1000.0));
}

/// Edge Case 6: Zero index (first element)
#[test]
fn test_edge_case_zero_index() {
    let result = parse(r#"{"array": [10, 20, 30]}"#, "array[0]");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::Number(10.0));
}

/// Edge Case 7: Negative indices (should error)
#[test]
fn test_edge_case_negative_index() {
    // Negative indices are not valid syntax and should be rejected during tokenization
    let result = parse(r#"{"array": [1, 2, 3]}"#, "array[-1]");
    assert!(result.is_err());
    // The tokenizer will reject the '-' character in the index
    assert!(matches!(result, Err(JsonParserError::MalformedPath(_))));
}

/// Edge Case 8: Property access on primitive (type mismatch)
#[test]
fn test_edge_case_property_on_primitive() {
    let result = parse(r#"{"version": 1}"#, "version.property");
    assert!(result.is_err());
    assert!(matches!(result, Err(JsonParserError::TypeMismatch(_))));
}

/// Edge Case 9: Index on non-array (type mismatch)
#[test]
fn test_edge_case_index_on_object() {
    let result = parse(r#"{"metadata": {"author": "me"}}"#, "metadata[0]");
    assert!(result.is_err());
    assert!(matches!(result, Err(JsonParserError::TypeMismatch(_))));
}

/// Edge Case 10: Special characters in keys (out of scope but test behavior)
#[test]
fn test_edge_case_literal_dot_in_key() {
    // JSON with literal "author.name" key
    let json = r#"{"author.name": "me"}"#;
    // Path "author.name" looks for nested structure {"author": {"name": "me"}}
    let result = parse(json, "author.name");
    // Should fail because it looks for nested structure, not literal key
    assert!(result.is_err());
    assert!(matches!(result, Err(JsonParserError::PathNotFound(_))));
}

/// Additional edge case: Empty object
#[test]
fn test_edge_case_empty_object() {
    let result = parse(r#"{"obj": {}}"#, "obj");
    assert!(result.is_ok());
    match result.unwrap() {
        JsonValue::Object(s) => assert_eq!(s, "{}"),
        _ => panic!("Expected Object variant"),
    }
}

/// Additional edge case: Empty array
#[test]
fn test_edge_case_empty_array() {
    let result = parse(r#"{"arr": []}"#, "arr");
    assert!(result.is_ok());
    match result.unwrap() {
        JsonValue::Array(s) => assert_eq!(s, "[]"),
        _ => panic!("Expected Array variant"),
    }
}

/// Additional edge case: Empty array indexing (out of bounds)
#[test]
fn test_edge_case_empty_array_index() {
    let result = parse(r#"{"arr": []}"#, "arr[0]");
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(JsonParserError::IndexOutOfBounds(0, 0))
    ));
}

/// Additional edge case: Path starting with dot
#[test]
fn test_edge_case_path_starts_with_dot() {
    let result = parse(r#"{"field": "value"}"#, ".field");
    assert!(result.is_err());
    assert!(matches!(result, Err(JsonParserError::MalformedPath(_))));
}

/// Additional edge case: Double dots in path
#[test]
fn test_edge_case_double_dots() {
    let result = parse(r#"{"a": {"b": "value"}}"#, "a..b");
    assert!(result.is_err());
    assert!(matches!(result, Err(JsonParserError::MalformedPath(_))));
}

/// Additional edge case: Complex mixed types
#[test]
fn test_edge_case_complex_mixed_types() {
    let json = r#"{
        "data": {
            "items": [
                {
                    "id": 1,
                    "values": [10, 20, 30],
                    "metadata": {
                        "tags": ["a", "b", "c"]
                    }
                }
            ]
        }
    }"#;

    let result = parse(json, "data.items[0].metadata.tags[1]");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsonValue::String("b".to_string()));
}
