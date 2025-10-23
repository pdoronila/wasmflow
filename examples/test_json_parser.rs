//! Simple test program to verify JSON parser implementation
//!
//! Run with: cargo run --example test_json_parser

use wasmflow::builtin::json_parser::{parse, JsonValue, JsonParserError};

fn main() {
    println!("JSON Parser Test Suite\n");
    println!("======================\n");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Simple number extraction
    println!("Test 1: Simple number extraction");
    match parse(r#"{"version": 1}"#, "version") {
        Ok(JsonValue::Number(1.0)) => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 2: Simple string extraction
    println!("Test 2: Simple string extraction");
    match parse(r#"{"author": "me"}"#, "author") {
        Ok(JsonValue::String(s)) if s == "me" => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 3: Nested property
    println!("Test 3: Nested property extraction");
    match parse(r#"{"metadata": {"author": "me"}}"#, "metadata.author") {
        Ok(JsonValue::String(s)) if s == "me" => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 4: Array indexing
    println!("Test 4: Array indexing");
    match parse(r#"{"values": [10, 20, 30]}"#, "values[1]") {
        Ok(JsonValue::Number(20.0)) => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 5: Combined notation
    println!("Test 5: Combined notation (array[index].property)");
    match parse(
        r#"{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}"#,
        "runs[1].time",
    ) {
        Ok(JsonValue::Number(1000.0)) => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 6: Invalid JSON
    println!("Test 6: Invalid JSON error handling");
    match parse("{invalid", "version") {
        Err(JsonParserError::InvalidJson(_)) => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 7: Path not found
    println!("Test 7: Path not found error handling");
    match parse(r#"{"version": 1}"#, "nonexistent") {
        Err(JsonParserError::PathNotFound(_)) => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 8: Index out of bounds
    println!("Test 8: Index out of bounds error handling");
    match parse(r#"{"runs": [1, 2]}"#, "runs[999]") {
        Err(JsonParserError::IndexOutOfBounds(999, 2)) => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 9: Deep nesting (10 levels)
    println!("Test 9: Deep nesting (10 levels)");
    match parse(
        r#"{"a": {"b": {"c": {"d": {"e": {"f": {"g": {"h": {"i": {"j": "deep"}}}}}}}}}}"#,
        "a.b.c.d.e.f.g.h.i.j",
    ) {
        Ok(JsonValue::String(s)) if s == "deep" => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Test 10: Null value
    println!("Test 10: Null value handling");
    match parse(r#"{"value": null}"#, "value") {
        Ok(JsonValue::Null) => {
            println!("✓ PASS\n");
            passed += 1;
        }
        _ => {
            println!("✗ FAIL\n");
            failed += 1;
        }
    }

    // Summary
    println!("======================");
    println!("Results: {} passed, {} failed", passed, failed);

    if failed == 0 {
        println!("\n✅ All tests passed!");
        std::process::exit(0);
    } else {
        println!("\n❌ Some tests failed!");
        std::process::exit(1);
    }
}
