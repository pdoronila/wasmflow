//! Node data structures for egui-snarl integration
//!
//! This module defines the data structures used by the canvas to represent
//! nodes and ports in the egui-snarl node editor.

use crate::graph::node::{DataType, ExecutionState};
use uuid::Uuid;

/// Data stored in snarl nodes
#[derive(Clone)]
pub struct SnarlNodeData {
    pub uuid: Uuid,
    pub display_name: String,
    pub component_id: String,
    pub inputs: Vec<SnarlPort>,
    pub outputs: Vec<SnarlPort>,
    #[allow(dead_code)]
    pub execution_state: ExecutionState,
    /// T059: Flag indicating component was updated
    pub needs_component_refresh: bool,
    /// Custom width for resizable nodes (e.g., WASM Creator)
    pub custom_width: Option<f32>,
    /// T040: Flag indicating this is a composite node (has internal structure)
    pub is_composite: bool,
    /// T048: Port mapping info for composite nodes (external_port_name -> (internal_node_name, internal_port_name))
    pub input_mappings: std::collections::BTreeMap<String, (String, String)>,
    pub output_mappings: std::collections::BTreeMap<String, (String, String)>,
}

/// Port data for snarl rendering
#[derive(Clone)]
pub struct SnarlPort {
    #[allow(dead_code)]
    pub uuid: Uuid,
    pub name: String,
    pub data_type: DataType,
    #[allow(dead_code)]
    pub current_value: Option<String>, // Formatted value for display
}
