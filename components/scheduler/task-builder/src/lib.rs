//! Task Builder Component
//!
//! Creates a properly formatted task configuration Record for the scheduler.
//! Takes individual inputs (component_id, priority, budget, etc.) and outputs a Record.

wit_bindgen::generate!({
    path: "./wit",
    world: "component",
    with: {
        "wasmflow:node/types@1.1.0": generate,
        "wasmflow:node/host@1.1.0": generate,
        "wasmflow:node/metadata@1.1.0": generate,
        "wasmflow:node/execution@1.1.0": generate,
    },
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use serde::Serialize;
use std::collections::BTreeMap;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Task Builder".to_string(),
            version: "1.0.0".to_string(),
            description: "Creates a scheduler task configuration from individual inputs. \
                         Outputs a Record that can be fed into the scheduler's task list."
                .to_string(),
            author: "WasmFlow Scheduler".to_string(),
            category: Some("Scheduler".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "component_id".to_string(),
                data_type: DataType::StringType,
                description: "Component ID to execute (e.g., 'user:math-adder')".to_string(),
                optional: false,
            },
            PortSpec {
                name: "priority".to_string(),
                data_type: DataType::U32Type,
                description: "Task priority (0-255, higher = more important). Default: 128"
                    .to_string(),
                optional: true,
            },
            PortSpec {
                name: "budget_ms".to_string(),
                data_type: DataType::U32Type,
                description: "Maximum execution time budget in milliseconds. Default: 100"
                    .to_string(),
                optional: true,
            },
            PortSpec {
                name: "period_ms".to_string(),
                data_type: DataType::U32Type,
                description: "Period for periodic tasks in milliseconds (0 = aperiodic). Default: 0"
                    .to_string(),
                optional: true,
            },
            PortSpec {
                name: "deadline_ms".to_string(),
                data_type: DataType::U32Type,
                description: "Relative deadline in milliseconds (0 = use period). Default: 0"
                    .to_string(),
                optional: true,
            },
            PortSpec {
                name: "display_name".to_string(),
                data_type: DataType::StringType,
                description: "Display name for visualization (defaults to component_id)".to_string(),
                optional: true,
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "task_json".to_string(),
            data_type: DataType::StringType,
            description: "Task configuration as JSON string (use with json-parser to create Record)"
                .to_string(),
            optional: false,
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

#[derive(Serialize)]
struct TaskConfig {
    component_id: String,
    priority: u32,
    budget_ms: u32,
    period_ms: u32,
    deadline_ms: u32,
    display_name: String,
    inputs: BTreeMap<String, serde_json::Value>,
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract required component_id
        let component_id = inputs
            .iter()
            .find(|(name, _)| name == "component_id")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing required input: component_id".to_string(),
                input_name: Some("component_id".to_string()),
                recovery_hint: Some("Connect a string value with the component ID to execute".to_string()),
            })?;

        // Extract optional inputs with defaults
        let priority = inputs
            .iter()
            .find(|(name, _)| name == "priority")
            .and_then(|(_, val)| match val {
                Value::U32Val(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(128);

        let budget_ms = inputs
            .iter()
            .find(|(name, _)| name == "budget_ms")
            .and_then(|(_, val)| match val {
                Value::U32Val(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(100);

        let period_ms = inputs
            .iter()
            .find(|(name, _)| name == "period_ms")
            .and_then(|(_, val)| match val {
                Value::U32Val(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0);

        let deadline_ms = inputs
            .iter()
            .find(|(name, _)| name == "deadline_ms")
            .and_then(|(_, val)| match val {
                Value::U32Val(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0);

        let display_name = inputs
            .iter()
            .find(|(name, _)| name == "display_name")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| component_id.clone());

        // Build task configuration
        let task = TaskConfig {
            component_id,
            priority,
            budget_ms,
            period_ms,
            deadline_ms,
            display_name,
            inputs: BTreeMap::new(),  // Empty for now
        };

        // Serialize to JSON
        let json = serde_json::to_string(&task).map_err(|e| ExecutionError {
            message: format!("Failed to serialize task to JSON: {}", e),
            input_name: None,
            recovery_hint: Some("This is an internal error. Please report it.".to_string()),
        })?;

        Ok(vec![("task_json".to_string(), Value::StringVal(json))])
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_builder_minimal() {
        let inputs = vec![(
            "component_id".to_string(),
            Value::StringVal("user:math-adder".to_string()),
        )];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "task_json");

        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"component_id\":\"user:math-adder\""));
            assert!(json.contains("\"priority\":128")); // Default
            assert!(json.contains("\"budget_ms\":100")); // Default
        } else {
            panic!("Expected String output");
        }
    }

    #[test]
    fn test_task_builder_full() {
        let inputs = vec![
            (
                "component_id".to_string(),
                Value::StringVal("user:sensor-reader".to_string()),
            ),
            ("priority".to_string(), Value::U32Val(200)),
            ("budget_ms".to_string(), Value::U32Val(50)),
            ("period_ms".to_string(), Value::U32Val(1000)),
            ("deadline_ms".to_string(), Value::U32Val(900)),
            (
                "display_name".to_string(),
                Value::StringVal("Sensor Task".to_string()),
            ),
        ];

        let result = Component::execute(inputs).unwrap();
        if let Value::StringVal(json) = &result[0].1 {
            assert!(json.contains("\"priority\":200"));
            assert!(json.contains("\"budget_ms\":50"));
            assert!(json.contains("\"period_ms\":1000"));
            assert!(json.contains("\"display_name\":\"Sensor Task\""));
        }
    }

    #[test]
    fn test_task_builder_missing_component_id() {
        let inputs = vec![("priority".to_string(), Value::U32Val(100))];

        let result = Component::execute(inputs);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("Missing required input: component_id"));
    }
}
