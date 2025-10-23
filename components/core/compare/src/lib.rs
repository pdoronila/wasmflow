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
            name: "Compare".to_string(),
            version: "1.0.0".to_string(),
            description: "Compares two values using various comparison operations".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Logic".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "left".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Left operand".to_string(),
            },
            PortSpec {
                name: "right".to_string(),
                data_type: DataType::AnyType,
                optional: false,
                description: "Right operand".to_string(),
            },
            PortSpec {
                name: "operation".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Comparison operation: equals/==, not-equals/!=, greater-than/>, less-than/<, greater-or-equal/>=, less-or-equal/<=".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::BoolType,
            optional: false,
            description: "Comparison result".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Compare component executing");

        let left = inputs
            .iter()
            .find(|(n, _)| n == "left")
            .map(|(_, v)| v)
            .ok_or_else(|| ExecutionError {
                message: "Missing 'left' input".to_string(),
                input_name: Some("left".to_string()),
                recovery_hint: Some("Provide a value for the left operand".to_string()),
            })?;

        let right = inputs
            .iter()
            .find(|(n, _)| n == "right")
            .map(|(_, v)| v)
            .ok_or_else(|| ExecutionError {
                message: "Missing 'right' input".to_string(),
                input_name: Some("right".to_string()),
                recovery_hint: Some("Provide a value for the right operand".to_string()),
            })?;

        let operation = inputs
            .iter()
            .find(|(n, _)| n == "operation")
            .and_then(|(_, v)| if let Value::StringVal(s) = v { Some(s.clone()) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'operation' input".to_string(),
                input_name: Some("operation".to_string()),
                recovery_hint: Some("Provide a valid operation string".to_string()),
            })?;

        let result = compare_values(left, right, &operation)?;

        Ok(vec![("result".to_string(), Value::BoolVal(result))])
    }
}

fn compare_values(left: &Value, right: &Value, operation: &str) -> Result<bool, ExecutionError> {
    match (left, right) {
        // Numeric comparisons - allow mixed numeric types
        (Value::U32Val(l), Value::U32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),
        (Value::I32Val(l), Value::I32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),
        (Value::F32Val(l), Value::F32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),

        // Mixed numeric types
        (Value::U32Val(l), Value::I32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),
        (Value::U32Val(l), Value::F32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),
        (Value::I32Val(l), Value::U32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),
        (Value::I32Val(l), Value::F32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),
        (Value::F32Val(l), Value::U32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),
        (Value::F32Val(l), Value::I32Val(r)) => compare_numbers(*l as f64, *r as f64, operation),

        // String comparisons
        (Value::StringVal(l), Value::StringVal(r)) => compare_strings(l, r, operation),

        // Boolean comparisons - only equality operations
        (Value::BoolVal(l), Value::BoolVal(r)) => compare_booleans(*l, *r, operation),

        // Type mismatch
        _ => Err(ExecutionError {
            message: format!("Cannot compare incompatible types: {:?} and {:?}", left, right),
            input_name: None,
            recovery_hint: Some("Ensure both operands are of compatible types (both numbers, both strings, or both booleans)".to_string()),
        }),
    }
}

fn compare_numbers(left: f64, right: f64, operation: &str) -> Result<bool, ExecutionError> {
    match operation {
        "equals" | "==" => Ok((left - right).abs() < f64::EPSILON),
        "not-equals" | "!=" => Ok((left - right).abs() >= f64::EPSILON),
        "greater-than" | ">" => Ok(left > right),
        "less-than" | "<" => Ok(left < right),
        "greater-or-equal" | ">=" => Ok(left >= right),
        "less-or-equal" | "<=" => Ok(left <= right),
        _ => Err(ExecutionError {
            message: format!("Invalid operation: {}", operation),
            input_name: Some("operation".to_string()),
            recovery_hint: Some("Use: equals/==, not-equals/!=, greater-than/>, less-than/<, greater-or-equal/>=, or less-or-equal/<=".to_string()),
        }),
    }
}

fn compare_strings(left: &str, right: &str, operation: &str) -> Result<bool, ExecutionError> {
    match operation {
        "equals" | "==" => Ok(left == right),
        "not-equals" | "!=" => Ok(left != right),
        "greater-than" | ">" => Ok(left > right),
        "less-than" | "<" => Ok(left < right),
        "greater-or-equal" | ">=" => Ok(left >= right),
        "less-or-equal" | "<=" => Ok(left <= right),
        _ => Err(ExecutionError {
            message: format!("Invalid operation: {}", operation),
            input_name: Some("operation".to_string()),
            recovery_hint: Some("Use: equals/==, not-equals/!=, greater-than/>, less-than/<, greater-or-equal/>=, or less-or-equal/<=".to_string()),
        }),
    }
}

fn compare_booleans(left: bool, right: bool, operation: &str) -> Result<bool, ExecutionError> {
    match operation {
        "equals" | "==" => Ok(left == right),
        "not-equals" | "!=" => Ok(left != right),
        "greater-than" | ">" | "less-than" | "<" | "greater-or-equal" | ">=" | "less-or-equal" | "<=" => {
            Err(ExecutionError {
                message: "Boolean values can only be compared for equality".to_string(),
                input_name: Some("operation".to_string()),
                recovery_hint: Some("Use 'equals'/'==' or 'not-equals'/'!=' for boolean comparisons".to_string()),
            })
        }
        _ => Err(ExecutionError {
            message: format!("Invalid operation: {}", operation),
            input_name: Some("operation".to_string()),
            recovery_hint: Some("Use: equals/== or not-equals/!= for booleans".to_string()),
        }),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_numbers_greater_than() {
        let inputs = vec![
            ("left".to_string(), Value::U32Val(10)),
            ("right".to_string(), Value::U32Val(5)),
            ("operation".to_string(), Value::StringVal("greater-than".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_compare_strings_less_than() {
        let inputs = vec![
            ("left".to_string(), Value::StringVal("apple".to_string())),
            ("right".to_string(), Value::StringVal("banana".to_string())),
            ("operation".to_string(), Value::StringVal("less-than".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_compare_mixed_numeric_types() {
        let inputs = vec![
            ("left".to_string(), Value::F32Val(10.5)),
            ("right".to_string(), Value::U32Val(10)),
            ("operation".to_string(), Value::StringVal("greater-than".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_compare_type_mismatch() {
        let inputs = vec![
            ("left".to_string(), Value::StringVal("hello".to_string())),
            ("right".to_string(), Value::U32Val(42)),
            ("operation".to_string(), Value::StringVal("equals".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_boolean_ordering_error() {
        let inputs = vec![
            ("left".to_string(), Value::BoolVal(true)),
            ("right".to_string(), Value::BoolVal(false)),
            ("operation".to_string(), Value::StringVal("greater-than".to_string())),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_symbolic_equals() {
        let inputs = vec![
            ("left".to_string(), Value::U32Val(5)),
            ("right".to_string(), Value::U32Val(5)),
            ("operation".to_string(), Value::StringVal("==".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_symbolic_greater_than() {
        let inputs = vec![
            ("left".to_string(), Value::U32Val(10)),
            ("right".to_string(), Value::U32Val(5)),
            ("operation".to_string(), Value::StringVal(">".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_symbolic_less_or_equal() {
        let inputs = vec![
            ("left".to_string(), Value::U32Val(5)),
            ("right".to_string(), Value::U32Val(10)),
            ("operation".to_string(), Value::StringVal("<=".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }

    #[test]
    fn test_symbolic_string_comparison() {
        let inputs = vec![
            ("left".to_string(), Value::StringVal("abc".to_string())),
            ("right".to_string(), Value::StringVal("xyz".to_string())),
            ("operation".to_string(), Value::StringVal("<".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::BoolVal(b) => assert_eq!(*b, true),
            _ => panic!("Expected bool output"),
        }
    }
}
