// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "./wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Switch Number".to_string(),
            version: "1.0.0".to_string(),
            description: "Routes data based on numeric matching. Compares value against up to 4 case numbers and returns corresponding output, or default if no match. Handles u32, i32, and f32.".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::AnyType,  // Accept u32, i32, or f32
                optional: false,
                description: "Numeric value to match against cases (u32, i32, or f32)".to_string(),
            },
            PortSpec {
                name: "case1".to_string(),
                data_type: DataType::F32Type,
                optional: true,
                description: "Number to match for case 1".to_string(),
            },
            PortSpec {
                name: "output1".to_string(),
                data_type: DataType::AnyType,
                optional: true,
                description: "Value to return if case1 matches".to_string(),
            },
            PortSpec {
                name: "case2".to_string(),
                data_type: DataType::F32Type,
                optional: true,
                description: "Number to match for case 2".to_string(),
            },
            PortSpec {
                name: "output2".to_string(),
                data_type: DataType::AnyType,
                optional: true,
                description: "Value to return if case2 matches".to_string(),
            },
            PortSpec {
                name: "case3".to_string(),
                data_type: DataType::F32Type,
                optional: true,
                description: "Number to match for case 3".to_string(),
            },
            PortSpec {
                name: "output3".to_string(),
                data_type: DataType::AnyType,
                optional: true,
                description: "Value to return if case3 matches".to_string(),
            },
            PortSpec {
                name: "case4".to_string(),
                data_type: DataType::F32Type,
                optional: true,
                description: "Number to match for case 4".to_string(),
            },
            PortSpec {
                name: "output4".to_string(),
                data_type: DataType::AnyType,
                optional: true,
                description: "Value to return if case4 matches".to_string(),
            },
            PortSpec {
                name: "default".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Default value to return if no cases match".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "Matched output value or default".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

/// Convert a numeric Value to f32 for comparison
fn value_to_f32(value: &Value) -> Result<f32, String> {
    match value {
        Value::U32Val(n) => Ok(*n as f32),
        Value::I32Val(n) => Ok(*n as f32),
        Value::F32Val(n) => Ok(*n),
        _ => Err(format!("Expected numeric value (u32, i32, or f32), got {:?}", value)),
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Switch Number component executing");

        // Extract value to match
        let value_input = inputs
            .iter()
            .find(|(n, _)| n == "value")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: value".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Provide a numeric value to match (u32, i32, or f32)".to_string()),
            })?;

        let value = value_to_f32(&value_input.1).map_err(|e| ExecutionError {
            message: e,
            input_name: Some("value".to_string()),
            recovery_hint: Some("Provide a numeric value (u32, i32, or f32)".to_string()),
        })?;

        // Extract default value
        let default = inputs
            .iter()
            .find(|(n, _)| n == "default")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: default".to_string(),
                input_name: Some("default".to_string()),
                recovery_hint: Some("Provide a default value to return when no cases match".to_string()),
            })?;

        // Try to match against each case
        for i in 1..=4 {
            let case_name = format!("case{}", i);
            let output_name = format!("output{}", i);

            // Check if this case is provided
            if let Some((_, case_val)) = inputs.iter().find(|(n, _)| n == &case_name) {
                if let Value::F32Val(case_number) = case_val {
                    // Check if value matches this case (with floating point tolerance)
                    if (value - case_number).abs() < f32::EPSILON {
                        // Find corresponding output
                        if let Some((_, output_val)) = inputs.iter().find(|(n, _)| n == &output_name) {
                            host::log("debug", &format!("Matched case {}: {}", i, case_number));
                            return Ok(vec![("result".to_string(), output_val.clone())]);
                        } else {
                            return Err(ExecutionError {
                                message: format!("Case{} matched but output{} is not provided", i, i),
                                input_name: Some(output_name),
                                recovery_hint: Some(format!("Provide an output{} value to return when case{} matches", i, i)),
                            });
                        }
                    }
                }
            }
        }

        // No cases matched, return default
        host::log("debug", "No cases matched, returning default");
        Ok(vec![("result".to_string(), default.1.clone())])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_case1_u32() {
        let inputs = vec![
            ("value".to_string(), Value::U32Val(42)),
            ("case1".to_string(), Value::F32Val(42.0)),
            ("output1".to_string(), Value::StringVal("matched".to_string())),
            ("default".to_string(), Value::StringVal("default".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "matched"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_match_case2_i32() {
        let inputs = vec![
            ("value".to_string(), Value::I32Val(-10)),
            ("case1".to_string(), Value::F32Val(5.0)),
            ("output1".to_string(), Value::U32Val(1)),
            ("case2".to_string(), Value::F32Val(-10.0)),
            ("output2".to_string(), Value::U32Val(2)),
            ("default".to_string(), Value::U32Val(0)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 2),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_match_case3_f32() {
        let inputs = vec![
            ("value".to_string(), Value::F32Val(3.14)),
            ("case1".to_string(), Value::F32Val(1.0)),
            ("output1".to_string(), Value::U32Val(1)),
            ("case2".to_string(), Value::F32Val(2.0)),
            ("output2".to_string(), Value::U32Val(2)),
            ("case3".to_string(), Value::F32Val(3.14)),
            ("output3".to_string(), Value::U32Val(3)),
            ("default".to_string(), Value::U32Val(0)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 3),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_no_match_returns_default() {
        let inputs = vec![
            ("value".to_string(), Value::U32Val(999)),
            ("case1".to_string(), Value::F32Val(1.0)),
            ("output1".to_string(), Value::U32Val(1)),
            ("case2".to_string(), Value::F32Val(2.0)),
            ("output2".to_string(), Value::U32Val(2)),
            ("default".to_string(), Value::StringVal("no match".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::StringVal(s) => assert_eq!(s, "no match"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_first_match_wins() {
        let inputs = vec![
            ("value".to_string(), Value::U32Val(10)),
            ("case1".to_string(), Value::F32Val(10.0)),
            ("output1".to_string(), Value::U32Val(1)),
            ("case2".to_string(), Value::F32Val(10.0)),
            ("output2".to_string(), Value::U32Val(2)),
            ("default".to_string(), Value::U32Val(0)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::U32Val(n) => assert_eq!(*n, 1, "First matching case should win"),
            _ => panic!("Expected u32 output"),
        }
    }

    #[test]
    fn test_missing_value() {
        let inputs = vec![
            ("case1".to_string(), Value::F32Val(1.0)),
            ("output1".to_string(), Value::U32Val(1)),
            ("default".to_string(), Value::U32Val(0)),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_value_type() {
        let inputs = vec![
            ("value".to_string(), Value::StringVal("not a number".to_string())),
            ("case1".to_string(), Value::F32Val(1.0)),
            ("output1".to_string(), Value::U32Val(1)),
            ("default".to_string(), Value::U32Val(0)),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_case_without_output() {
        let inputs = vec![
            ("value".to_string(), Value::U32Val(42)),
            ("case1".to_string(), Value::F32Val(42.0)),
            // Missing output1
            ("default".to_string(), Value::U32Val(0)),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
