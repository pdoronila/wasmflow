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
            name: "Power".to_string(),
            version: "1.0.0".to_string(),
            description: "Raises a base to an exponent (base^exponent)".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "base".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Base number".to_string(),
            },
            PortSpec {
                name: "exponent".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Exponent".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "base raised to the power of exponent".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Power component executing");

        let base = inputs
            .iter()
            .find(|(n, _)| n == "base")
            .and_then(|(_, v)| if let Value::F32Val(f) = v { Some(*f) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'base' input".to_string(),
                input_name: Some("base".to_string()),
                recovery_hint: Some("Provide a numeric value for the base".to_string()),
            })?;

        let exponent = inputs
            .iter()
            .find(|(n, _)| n == "exponent")
            .and_then(|(_, v)| if let Value::F32Val(f) = v { Some(*f) } else { None })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'exponent' input".to_string(),
                input_name: Some("exponent".to_string()),
                recovery_hint: Some("Provide a numeric value for the exponent".to_string()),
            })?;

        let result = base.powf(exponent);

        if result.is_nan() {
            return Err(ExecutionError {
                message: format!(
                    "Invalid operation: {}^{} results in NaN (negative base with fractional exponent)",
                    base, exponent
                ),
                input_name: None,
                recovery_hint: Some("Use positive base for fractional exponents, or use integer exponents for negative bases".to_string()),
            });
        }

        if result.is_infinite() {
            return Err(ExecutionError {
                message: format!("Result overflow: {}^{} is infinite", base, exponent),
                input_name: None,
                recovery_hint: Some("Use smaller values to avoid overflow".to_string()),
            });
        }

        Ok(vec![("result".to_string(), Value::F32Val(result))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_basic() {
        let inputs = vec![
            ("base".to_string(), Value::F32Val(2.0)),
            ("exponent".to_string(), Value::F32Val(3.0)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 8.0).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_power_negative_exponent() {
        let inputs = vec![
            ("base".to_string(), Value::F32Val(10.0)),
            ("exponent".to_string(), Value::F32Val(-2.0)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 0.01).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_power_fractional_exponent() {
        let inputs = vec![
            ("base".to_string(), Value::F32Val(4.0)),
            ("exponent".to_string(), Value::F32Val(0.5)),
        ];
        let result = Component::execute(inputs).unwrap();
        match &result[0].1 {
            Value::F32Val(f) => assert!((f - 2.0).abs() < 0.001),
            _ => panic!("Expected f32 output"),
        }
    }

    #[test]
    fn test_power_negative_base_fractional_exponent() {
        let inputs = vec![
            ("base".to_string(), Value::F32Val(-2.0)),
            ("exponent".to_string(), Value::F32Val(0.5)),
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
