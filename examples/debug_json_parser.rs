//! Debug program for Test 5
//!
//! Run with: cargo run --example debug_json_parser

use wasmflow::builtin::json_parser::{parse, JsonValue};

fn main() {
    let json = r#"{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}"#;
    let path = "runs[1].time";

    println!("JSON: {}", json);
    println!("Path: {}", path);
    println!();

    match parse(json, path) {
        Ok(value) => {
            println!("Success!");
            match value {
                JsonValue::Number(n) => println!("Got number: {}", n),
                JsonValue::String(s) => println!("Got string: {}", s),
                JsonValue::Boolean(b) => println!("Got boolean: {}", b),
                JsonValue::Object(o) => println!("Got object: {}", o),
                JsonValue::Array(a) => println!("Got array: {}", a),
                JsonValue::Null => println!("Got null"),
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
