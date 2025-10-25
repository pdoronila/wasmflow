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
            NodeValue::Bool(_) => ("Bool", DataType::Bool),
            NodeValue::Binary(_) => ("Binary", DataType::Binary),
            NodeValue::List(items) => {
                // Detect homogeneous list type from first element
                if items.is_empty() {
                    // Default to string list for empty lists
                    ("String List", DataType::List(Box::new(DataType::String)))
                } else {
                    match &items[0] {
                        NodeValue::String(_) => ("String List", DataType::List(Box::new(DataType::String))),
                        NodeValue::U32(_) => ("U32 List", DataType::List(Box::new(DataType::U32))),
                        NodeValue::F32(_) => ("F32 List", DataType::List(Box::new(DataType::F32))),
                        NodeValue::I32(_) => ("I32 List", DataType::List(Box::new(DataType::I32))),
                        _ => ("List", DataType::List(Box::new(DataType::Any))),
                    }
                }
            }
            NodeValue::Record(_) => ("Record", DataType::Record(vec![])),
        };

        ComponentSpec::new_builtin(
            format!("builtin:constant:{}", type_name.to_lowercase().replace(" ", "-")),
            format!("Constant ({})", type_name),
            format!("Outputs a constant {} value", type_name.to_lowercase()),
            Some("Builtin".to_string()),
        )
        .with_output("value".to_string(), data_type, format!("Constant {} value", type_name.to_lowercase()))
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

    /// Create a String list constant
    pub fn string_list(values: Vec<String>) -> Self {
        Self::new(NodeValue::List(
            values.into_iter().map(NodeValue::String).collect()
        ))
    }

    /// Create a U32 list constant
    pub fn u32_list(values: Vec<u32>) -> Self {
        Self::new(NodeValue::List(
            values.into_iter().map(NodeValue::U32).collect()
        ))
    }

    /// Create an F32 list constant
    pub fn f32_list(values: Vec<f32>) -> Self {
        Self::new(NodeValue::List(
            values.into_iter().map(NodeValue::F32).collect()
        ))
    }
}

/// Register all constant node types
pub fn register_constant_nodes(registry: &mut crate::graph::node::ComponentRegistry) {
    // Register constants with custom footer view
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
            .with_footer_view(footer_view.clone())
    );

    // Register list constant nodes with initial values so spec() can detect type
    registry.register_builtin(
        ConstantNode::string_list(vec!["".to_string()])
            .spec()
            .with_footer_view(footer_view.clone())
    );
    registry.register_builtin(
        ConstantNode::u32_list(vec![0])
            .spec()
            .with_footer_view(footer_view.clone())
    );
    registry.register_builtin(
        ConstantNode::f32_list(vec![0.0])
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

    #[test]
    fn test_string_list_constant() {
        let constant = ConstantNode::string_list(vec![
            "hello".to_string(),
            "world".to_string(),
        ]);
        let inputs = HashMap::new();
        let result = constant.execute(&inputs).unwrap();

        match result.get("value") {
            Some(NodeValue::List(items)) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    NodeValue::String(s) => assert_eq!(s, "hello"),
                    _ => panic!("Expected string item"),
                }
                match &items[1] {
                    NodeValue::String(s) => assert_eq!(s, "world"),
                    _ => panic!("Expected string item"),
                }
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_u32_list_constant() {
        let constant = ConstantNode::u32_list(vec![1, 2, 3, 4, 5]);
        let inputs = HashMap::new();
        let result = constant.execute(&inputs).unwrap();

        match result.get("value") {
            Some(NodeValue::List(items)) => {
                assert_eq!(items.len(), 5);
                match &items[0] {
                    NodeValue::U32(n) => assert_eq!(*n, 1),
                    _ => panic!("Expected u32 item"),
                }
                match &items[4] {
                    NodeValue::U32(n) => assert_eq!(*n, 5),
                    _ => panic!("Expected u32 item"),
                }
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_f32_list_constant() {
        let constant = ConstantNode::f32_list(vec![1.5, 2.5, 3.5]);
        let inputs = HashMap::new();
        let result = constant.execute(&inputs).unwrap();

        match result.get("value") {
            Some(NodeValue::List(items)) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    NodeValue::F32(n) => assert_eq!(*n, 1.5),
                    _ => panic!("Expected f32 item"),
                }
                match &items[2] {
                    NodeValue::F32(n) => assert_eq!(*n, 3.5),
                    _ => panic!("Expected f32 item"),
                }
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_empty_list_constant() {
        let constant = ConstantNode::string_list(vec![]);
        let inputs = HashMap::new();
        let result = constant.execute(&inputs).unwrap();

        match result.get("value") {
            Some(NodeValue::List(items)) => {
                assert_eq!(items.len(), 0);
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_string_list_spec() {
        let constant = ConstantNode::string_list(vec!["test".to_string()]);
        let spec = constant.spec();
        assert_eq!(spec.id, "builtin:constant:string-list");
        assert_eq!(spec.name, "Constant (String List)");
        assert_eq!(spec.output_spec.len(), 1);
        assert_eq!(spec.output_spec[0].name, "value");

        // Check that the output type is a list of strings
        match &spec.output_spec[0].data_type {
            crate::graph::node::DataType::List(inner) => {
                match **inner {
                    crate::graph::node::DataType::String => (),
                    _ => panic!("Expected List<String> type"),
                }
            }
            _ => panic!("Expected List type"),
        }
    }

    #[test]
    fn test_u32_list_spec() {
        let constant = ConstantNode::u32_list(vec![42]);
        let spec = constant.spec();
        assert_eq!(spec.id, "builtin:constant:u32-list");
        assert_eq!(spec.name, "Constant (U32 List)");

        // Check that the output type is a list of u32
        match &spec.output_spec[0].data_type {
            crate::graph::node::DataType::List(inner) => {
                match **inner {
                    crate::graph::node::DataType::U32 => (),
                    _ => panic!("Expected List<U32> type"),
                }
            }
            _ => panic!("Expected List type"),
        }
    }

    #[test]
    fn test_f32_list_spec() {
        let constant = ConstantNode::f32_list(vec![3.14]);
        let spec = constant.spec();
        assert_eq!(spec.id, "builtin:constant:f32-list");
        assert_eq!(spec.name, "Constant (F32 List)");

        // Check that the output type is a list of f32
        match &spec.output_spec[0].data_type {
            crate::graph::node::DataType::List(inner) => {
                match **inner {
                    crate::graph::node::DataType::F32 => (),
                    _ => panic!("Expected List<F32> type"),
                }
            }
            _ => panic!("Expected List type"),
        }
    }

    #[test]
    fn test_empty_list_defaults_to_string_list() {
        let constant = ConstantNode::string_list(vec![]);
        let spec = constant.spec();
        assert_eq!(spec.id, "builtin:constant:string-list");
        assert_eq!(spec.name, "Constant (String List)");

        // Check that empty list defaults to string list type
        match &spec.output_spec[0].data_type {
            crate::graph::node::DataType::List(inner) => {
                match **inner {
                    crate::graph::node::DataType::String => (),
                    _ => panic!("Expected List<String> type for empty list"),
                }
            }
            _ => panic!("Expected List type"),
        }
    }
}
