//! Built-in mathematical operation nodes

use crate::graph::node::{ComponentSpec, DataType, NodeValue};
use crate::ComponentError;
use std::collections::HashMap;

/// Execute a mathematical operation on inputs
pub trait MathOperation {
    fn execute(&self, inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError>;
    fn spec(&self) -> ComponentSpec;
}

/// Add node: adds two numbers
pub struct AddNode;

impl MathOperation for AddNode {
    fn execute(&self, inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let a = inputs
            .get("a")
            .ok_or_else(|| ComponentError::ExecutionError("Missing input 'a'".to_string()))?;
        let b = inputs
            .get("b")
            .ok_or_else(|| ComponentError::ExecutionError("Missing input 'b'".to_string()))?;

        let result = match (a, b) {
            (NodeValue::F32(a_val), NodeValue::F32(b_val)) => NodeValue::F32(a_val + b_val),
            (NodeValue::I32(a_val), NodeValue::I32(b_val)) => NodeValue::I32(a_val + b_val),
            (NodeValue::U32(a_val), NodeValue::U32(b_val)) => NodeValue::U32(a_val + b_val),
            _ => return Err(ComponentError::ExecutionError(
                "Type mismatch: inputs must be the same numeric type".to_string()
            )),
        };

        let mut outputs = HashMap::new();
        outputs.insert("sum".to_string(), result);
        Ok(outputs)
    }

    fn spec(&self) -> ComponentSpec {
        ComponentSpec::new_builtin(
            "builtin:math:add".to_string(),
            "Add".to_string(),
            "Adds two numbers and returns the sum".to_string(),
            Some("Math".to_string()),
        )
        .with_input("a".to_string(), DataType::F32, "First number".to_string())
        .with_input("b".to_string(), DataType::F32, "Second number".to_string())
        .with_output("sum".to_string(), DataType::F32, "Sum of inputs".to_string())
    }
}

/// Subtract node: subtracts two numbers
pub struct SubtractNode;

impl MathOperation for SubtractNode {
    fn execute(&self, inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let a = inputs
            .get("a")
            .ok_or_else(|| ComponentError::ExecutionError("Missing input 'a'".to_string()))?;
        let b = inputs
            .get("b")
            .ok_or_else(|| ComponentError::ExecutionError("Missing input 'b'".to_string()))?;

        let result = match (a, b) {
            (NodeValue::F32(a_val), NodeValue::F32(b_val)) => NodeValue::F32(a_val - b_val),
            (NodeValue::I32(a_val), NodeValue::I32(b_val)) => NodeValue::I32(a_val - b_val),
            (NodeValue::U32(a_val), NodeValue::U32(b_val)) => {
                if a_val >= b_val {
                    NodeValue::U32(a_val - b_val)
                } else {
                    return Err(ComponentError::ExecutionError(
                        "Underflow: cannot subtract larger value from smaller for unsigned integers".to_string()
                    ));
                }
            }
            _ => return Err(ComponentError::ExecutionError(
                "Type mismatch: inputs must be the same numeric type".to_string()
            )),
        };

        let mut outputs = HashMap::new();
        outputs.insert("difference".to_string(), result);
        Ok(outputs)
    }

    fn spec(&self) -> ComponentSpec {
        ComponentSpec::new_builtin(
            "builtin:math:subtract".to_string(),
            "Subtract".to_string(),
            "Subtracts second number from first".to_string(),
            Some("Math".to_string()),
        )
        .with_input("a".to_string(), DataType::F32, "First number".to_string())
        .with_input("b".to_string(), DataType::F32, "Second number".to_string())
        .with_output("difference".to_string(), DataType::F32, "Difference (a - b)".to_string())
    }
}

/// Multiply node: multiplies two numbers
pub struct MultiplyNode;

impl MathOperation for MultiplyNode {
    fn execute(&self, inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let a = inputs
            .get("a")
            .ok_or_else(|| ComponentError::ExecutionError("Missing input 'a'".to_string()))?;
        let b = inputs
            .get("b")
            .ok_or_else(|| ComponentError::ExecutionError("Missing input 'b'".to_string()))?;

        let result = match (a, b) {
            (NodeValue::F32(a_val), NodeValue::F32(b_val)) => NodeValue::F32(a_val * b_val),
            (NodeValue::I32(a_val), NodeValue::I32(b_val)) => NodeValue::I32(a_val * b_val),
            (NodeValue::U32(a_val), NodeValue::U32(b_val)) => NodeValue::U32(a_val * b_val),
            _ => return Err(ComponentError::ExecutionError(
                "Type mismatch: inputs must be the same numeric type".to_string()
            )),
        };

        let mut outputs = HashMap::new();
        outputs.insert("product".to_string(), result);
        Ok(outputs)
    }

    fn spec(&self) -> ComponentSpec {
        ComponentSpec::new_builtin(
            "builtin:math:multiply".to_string(),
            "Multiply".to_string(),
            "Multiplies two numbers".to_string(),
            Some("Math".to_string()),
        )
        .with_input("a".to_string(), DataType::F32, "First number".to_string())
        .with_input("b".to_string(), DataType::F32, "Second number".to_string())
        .with_output("product".to_string(), DataType::F32, "Product (a × b)".to_string())
    }
}

/// Divide node: divides two numbers
pub struct DivideNode;

impl MathOperation for DivideNode {
    fn execute(&self, inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let a = inputs
            .get("a")
            .ok_or_else(|| ComponentError::ExecutionError("Missing input 'a'".to_string()))?;
        let b = inputs
            .get("b")
            .ok_or_else(|| ComponentError::ExecutionError("Missing input 'b'".to_string()))?;

        let result = match (a, b) {
            (NodeValue::F32(a_val), NodeValue::F32(b_val)) => {
                if *b_val == 0.0 {
                    return Err(ComponentError::ExecutionError(
                        "Division by zero".to_string()
                    ));
                }
                NodeValue::F32(a_val / b_val)
            }
            (NodeValue::I32(a_val), NodeValue::I32(b_val)) => {
                if *b_val == 0 {
                    return Err(ComponentError::ExecutionError(
                        "Division by zero".to_string()
                    ));
                }
                NodeValue::I32(a_val / b_val)
            }
            (NodeValue::U32(a_val), NodeValue::U32(b_val)) => {
                if *b_val == 0 {
                    return Err(ComponentError::ExecutionError(
                        "Division by zero".to_string()
                    ));
                }
                NodeValue::U32(a_val / b_val)
            }
            _ => return Err(ComponentError::ExecutionError(
                "Type mismatch: inputs must be the same numeric type".to_string()
            )),
        };

        let mut outputs = HashMap::new();
        outputs.insert("quotient".to_string(), result);
        Ok(outputs)
    }

    fn spec(&self) -> ComponentSpec {
        ComponentSpec::new_builtin(
            "builtin:math:divide".to_string(),
            "Divide".to_string(),
            "Divides first number by second (with division-by-zero handling)".to_string(),
            Some("Math".to_string()),
        )
        .with_input("a".to_string(), DataType::F32, "Dividend".to_string())
        .with_input("b".to_string(), DataType::F32, "Divisor".to_string())
        .with_output("quotient".to_string(), DataType::F32, "Quotient (a ÷ b)".to_string())
    }
}

/// Register all math operations
pub fn register_math_nodes(registry: &mut crate::graph::node::ComponentRegistry) {
    // T030-T033: Register math nodes with custom footer view
    let footer_view = crate::builtin::views::MathNodeFooterView::new();

    registry.register_builtin(AddNode.spec().with_footer_view(footer_view.clone()));
    registry.register_builtin(SubtractNode.spec().with_footer_view(footer_view.clone()));
    registry.register_builtin(MultiplyNode.spec().with_footer_view(footer_view.clone()));
    registry.register_builtin(DivideNode.spec().with_footer_view(footer_view));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_f32() {
        let add = AddNode;
        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), NodeValue::F32(5.0));
        inputs.insert("b".to_string(), NodeValue::F32(3.0));

        let result = add.execute(&inputs).unwrap();
        assert_eq!(result.get("sum"), Some(&NodeValue::F32(8.0)));
    }

    #[test]
    fn test_subtract_f32() {
        let sub = SubtractNode;
        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), NodeValue::F32(10.0));
        inputs.insert("b".to_string(), NodeValue::F32(3.0));

        let result = sub.execute(&inputs).unwrap();
        assert_eq!(result.get("difference"), Some(&NodeValue::F32(7.0)));
    }

    #[test]
    fn test_multiply_f32() {
        let mul = MultiplyNode;
        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), NodeValue::F32(4.0));
        inputs.insert("b".to_string(), NodeValue::F32(3.0));

        let result = mul.execute(&inputs).unwrap();
        assert_eq!(result.get("product"), Some(&NodeValue::F32(12.0)));
    }

    #[test]
    fn test_divide_f32() {
        let div = DivideNode;
        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), NodeValue::F32(12.0));
        inputs.insert("b".to_string(), NodeValue::F32(3.0));

        let result = div.execute(&inputs).unwrap();
        assert_eq!(result.get("quotient"), Some(&NodeValue::F32(4.0)));
    }

    #[test]
    fn test_divide_by_zero() {
        let div = DivideNode;
        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), NodeValue::F32(12.0));
        inputs.insert("b".to_string(), NodeValue::F32(0.0));

        let result = div.execute(&inputs);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ComponentError::ExecutionError(_)));
    }
}
