//! Example continuous execution nodes
//!
//! Demonstrates continuous execution with various input patterns.

use crate::graph::node::{ComponentSpec, DataType, NodeValue};
use crate::runtime::engine::NodeExecutor;
use crate::ComponentError;
use std::collections::HashMap;

/// Continuous timer executor
pub struct ContinuousTimerExecutor;

impl NodeExecutor for ContinuousTimerExecutor {
    fn execute(&self, inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let mut outputs = HashMap::new();

        // Get interval from input (default to 100ms)
        let _interval_ms = if let Some(NodeValue::U32(interval)) = inputs.get("interval") {
            *interval
        } else {
            100
        };

        // For regular (non-continuous) execution, just return initial values
        // The continuous execution will update these values over time
        outputs.insert("counter".to_string(), NodeValue::U32(0));
        outputs.insert("elapsed_seconds".to_string(), NodeValue::F32(0.0));

        Ok(outputs)
    }
}

/// T050: Continuous node that combines multiple inputs
/// This demonstrates reactive input processing for continuous nodes
pub struct ContinuousCombinerExecutor;

impl NodeExecutor for ContinuousCombinerExecutor {
    fn execute(&self, inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let mut outputs = HashMap::new();

        // Get input values (with defaults)
        let input_a = if let Some(NodeValue::String(s)) = inputs.get("input_a") {
            s.clone()
        } else {
            "A".to_string()
        };

        let input_b = if let Some(NodeValue::String(s)) = inputs.get("input_b") {
            s.clone()
        } else {
            "B".to_string()
        };

        let separator = if let Some(NodeValue::String(s)) = inputs.get("separator") {
            s.clone()
        } else {
            " ".to_string()
        };

        // Combine inputs
        let combined = format!("{}{}{}", input_a, separator, input_b);
        outputs.insert("combined".to_string(), NodeValue::String(combined));

        // Also output individual lengths for monitoring
        outputs.insert("length_a".to_string(), NodeValue::U32(input_a.len() as u32));
        outputs.insert("length_b".to_string(), NodeValue::U32(input_b.len() as u32));

        Ok(outputs)
    }
}

/// Register the continuous example nodes in the component registry
pub fn register_continuous_example(registry: &mut crate::graph::node::ComponentRegistry) {
    // Continuous timer node
    let timer_spec = ComponentSpec::new_builtin(
        "builtin:continuous:timer".to_string(),
        "Continuous Timer".to_string(),
        "A timer that continuously increments a counter. Example of continuous execution.".to_string(),
        Some("Example".to_string()),
    )
    .with_input("interval".to_string(), DataType::U32, "Interval in milliseconds (default: 100)".to_string())
    .with_output("counter".to_string(), DataType::U32, "Number of iterations".to_string())
    .with_output("elapsed_seconds".to_string(), DataType::F32, "Elapsed time in seconds".to_string());

    registry.register_builtin(timer_spec);

    // T050: Continuous combiner node (demonstrates reactive input processing)
    let combiner_spec = ComponentSpec::new_builtin(
        "builtin:continuous:combiner".to_string(),
        "Continuous Combiner".to_string(),
        "Combines two string inputs continuously. Demonstrates reactive input processing.".to_string(),
        Some("Example".to_string()),
    )
    .with_input("input_a".to_string(), DataType::String, "First input string".to_string())
    .with_input("input_b".to_string(), DataType::String, "Second input string".to_string())
    .with_input("separator".to_string(), DataType::String, "Separator between inputs (default: space)".to_string())
    .with_output("combined".to_string(), DataType::String, "Combined result".to_string())
    .with_output("length_a".to_string(), DataType::U32, "Length of input_a".to_string())
    .with_output("length_b".to_string(), DataType::U32, "Length of input_b".to_string());

    registry.register_builtin(combiner_spec);
}

