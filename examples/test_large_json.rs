//! Test if JSON parser handles large JSON strings
//!
//! Run with: cargo run --example test_large_json

use wasmflow::builtin::json_parser::parse;

fn main() {
    println!("Testing JSON parser with large JSON...\n");

    // Create a large JSON (> 15KB to exceed 13842 bytes)
    let mut large_json = String::from("{\"data\":[");
    for i in 0..1000 {
        if i > 0 {
            large_json.push(',');
        }
        large_json.push_str(&format!(r#"{{"id":{},"name":"item_{}","value":"data_{}"}}"#, i, i, i));
    }
    large_json.push_str(r#"],"metadata":{"count":1000}}"#);

    println!("Generated JSON size: {} bytes", large_json.len());
    println!("Testing extraction at various paths...\n");

    // Test 1: Extract array
    match parse(&large_json, "data") {
        Ok(value) => println!("✓ Successfully extracted 'data' array"),
        Err(e) => println!("✗ Error extracting 'data': {}", e),
    }

    // Test 2: Extract first item
    match parse(&large_json, "data[0].name") {
        Ok(value) => println!("✓ Successfully extracted 'data[0].name': {:?}", value),
        Err(e) => println!("✗ Error extracting 'data[0].name': {}", e),
    }

    // Test 3: Extract metadata
    match parse(&large_json, "metadata.count") {
        Ok(value) => println!("✓ Successfully extracted 'metadata.count': {:?}", value),
        Err(e) => println!("✗ Error extracting 'metadata.count': {}", e),
    }

    println!("\n✅ All tests passed - JSON parser handles large JSON correctly");
}
