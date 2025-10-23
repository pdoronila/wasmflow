//! Built-in constant value nodes

use crate::graph::node::{ComponentSpec, DataType, NodeValue};
use crate::ComponentError;
use std::collections::HashMap;

/// Constant node: outputs a user-configured constant value
/// Supports U32, I32, F32, and String types
pub struct ConstantNode {
    pub value: NodeValue,
}

impl ConstantNode {
    /// Create a new constant node with the specified value
    pub fn new(value: NodeValue) -> Self {
        Self { value }
    }

    /// Execute the constant node (always returns the configured value)
    pub fn execute(&self, _inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let mut outputs = HashMap::new();
        outputs.insert("value".to_string(), self.value.clone());
        Ok(outputs)
    }

    /// Get the component specification for this constant type
    pub fn spec(&self) -> ComponentSpec {
        let (type_name, data_type) = match &self.value {
            NodeValue::U32(_) => ("U32", DataType::U32),
            NodeValue::I32(_) => ("I32", DataType::I32),
            NodeValue::F32(_) => ("F32", DataType::F32),
            NodeValue::String(_) => ("String", DataType::String),
            NodeValue::Binary(_) => ("Binary", DataType::Binary),
            NodeValue::List(_) => ("List", DataType::List(Box::new(DataType::Any))),
            NodeValue::Record(_) => ("Record", DataType::Record(vec![])),
        };

        ComponentSpec::new_builtin(
            format!("builtin:constant:{}", type_name.to_lowercase()),
            format!("Constant ({})", type_name),
            format!("Outputs a constant {} value", type_name),
            Some("Constants".to_string()),
        )
        .with_output("value".to_string(), data_type, format!("Constant {} value", type_name))
    }

    /// Get the component specification for the unified constant node
    pub fn unified_spec() -> ComponentSpec {
        ComponentSpec::new_builtin(
            "builtin:constant:unified".to_string(),
            "Constant".to_string(),
            "Outputs a constant value with configurable type".to_string(),
            Some("Constants".to_string()),
        )
        .with_output("value".to_string(), DataType::Any, "Constant value".to_string())
    }
}

/// Helper functions to create common constant nodes
impl ConstantNode {
    /// Create a U32 constant
    pub fn u32(value: u32) -> Self {
        Self::new(NodeValue::U32(value))
    }

    /// Create an I32 constant
    pub fn i32(value: i32) -> Self {
        Self::new(NodeValue::I32(value))
    }

    /// Create an F32 constant
    pub fn f32(value: f32) -> Self {
        Self::new(NodeValue::F32(value))
    }

    /// Create a String constant
    pub fn string(value: String) -> Self {
        Self::new(NodeValue::String(value))
    }

    /// Create a Binary constant
    pub fn binary(value: Vec<u8>) -> Self {
        Self::new(NodeValue::Binary(value))
    }
}

/// Register all constant node types
pub fn register_constant_nodes(registry: &mut crate::graph::node::ComponentRegistry) {
    // Register unified constant with custom footer view
    let unified_footer_view = crate::builtin::views::UnifiedConstantFooterView::new();

    registry.register_builtin(
        ConstantNode::unified_spec()
            .with_footer_view(unified_footer_view)
    );

    // Keep the old individual constant types for backward compatibility
    let footer_view = crate::builtin::views::ConstantNodeFooterView::new();

    // Register constants with footer view (footer contains both value display and editing)
    registry.register_builtin(
        ConstantNode::f32(0.0)
            .spec()
            .with_footer_view(footer_view.clone())
    );
    registry.register_builtin(
        ConstantNode::i32(0)
            .spec()
            .with_footer_view(footer_view.clone())
    );
    registry.register_builtin(
        ConstantNode::u32(0)
            .spec()
            .with_footer_view(footer_view.clone())
    );
    registry.register_builtin(
        ConstantNode::string("".to_string())
            .spec()
            .with_footer_view(footer_view)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_constant() {
        let constant = ConstantNode::f32(42.5);
        let inputs = HashMap::new();
        let result = constant.execute(&inputs).unwrap();
        assert_eq!(result.get("value"), Some(&NodeValue::F32(42.5)));
    }

    #[test]
    fn test_i32_constant() {
        let constant = ConstantNode::i32(-100);
        let inputs = HashMap::new();
        let result = constant.execute(&inputs).unwrap();
        assert_eq!(result.get("value"), Some(&NodeValue::I32(-100)));
    }

    #[test]
    fn test_u32_constant() {
        let constant = ConstantNode::u32(999);
        let inputs = HashMap::new();
        let result = constant.execute(&inputs).unwrap();
        assert_eq!(result.get("value"), Some(&NodeValue::U32(999)));
    }

    #[test]
    fn test_string_constant() {
        let constant = ConstantNode::string("Hello, World!".to_string());
        let inputs = HashMap::new();
        let result = constant.execute(&inputs).unwrap();
        assert_eq!(result.get("value"), Some(&NodeValue::String("Hello, World!".to_string())));
    }

    #[test]
    fn test_constant_spec() {
        let constant = ConstantNode::f32(5.0);
        let spec = constant.spec();
        assert_eq!(spec.id, "builtin:constant:f32");
        assert_eq!(spec.name, "Constant (F32)");
        assert_eq!(spec.output_spec.len(), 1);
        assert_eq!(spec.output_spec[0].name, "value");
    }
}
