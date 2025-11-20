//! Node duplication functionality
//!
//! This module handles cloning selected nodes and their internal connections.
//! Users can duplicate nodes via UI button or Ctrl+D keyboard shortcut.

use super::WasmFlowApp;
use crate::graph::command::Command;
use crate::graph::connection::Connection;
use crate::graph::node::{GraphNode, Port};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

/// Position offset for cloned nodes (pixels)
const CLONE_OFFSET: egui::Vec2 = egui::Vec2::new(50.0, 50.0);

/// Suffix appended to cloned node display names
const CLONE_NAME_SUFFIX: &str = " (Clone)";

impl WasmFlowApp {
    /// Handle clone action - duplicate selected nodes and their internal connections
    ///
    /// This method:
    /// 1. Gets all selected nodes
    /// 2. Clones each node with new UUIDs
    /// 3. Clones internal connections (both endpoints selected)
    /// 4. Creates a Command::CloneNodes and executes via history (for undo/redo)
    /// 5. Clears original selection and selects clones
    pub(super) fn handle_clone_action(&mut self) {
        // Get selected nodes
        let selected_nodes = get_selected_node_ids(&self.graph);

        if selected_nodes.is_empty() {
            self.status_message = "No nodes selected to clone".to_string();
            return;
        }

        log::info!("Cloning {} selected node(s)", selected_nodes.len());

        // Clone nodes and build mappings
        let mut node_mapping = HashMap::new(); // old_id → new_id
        let mut port_mapping = HashMap::new(); // old_port_id → new_port_id
        let mut cloned_nodes = Vec::new();

        for node_id in &selected_nodes {
            if let Some(original) = self.graph.nodes.get(node_id) {
                let (cloned, old_id, new_id, ports) =
                    clone_node(original, CLONE_OFFSET, CLONE_NAME_SUFFIX);
                node_mapping.insert(old_id, new_id);
                port_mapping.extend(ports);
                cloned_nodes.push(cloned);
            }
        }

        // Clone internal connections
        let cloned_connections = clone_internal_connections(
            &self.graph.connections,
            &selected_nodes,
            &node_mapping,
            &port_mapping,
        );

        log::info!(
            "Cloned {} node(s) with {} internal connection(s)",
            cloned_nodes.len(),
            cloned_connections.len()
        );

        // Create command for undo/redo
        let command = Command::CloneNodes {
            cloned_nodes: cloned_nodes.clone(),
            cloned_connections: cloned_connections.clone(),
        };

        // Execute command via history
        if let Err(e) = self.history.execute(command, &mut self.graph) {
            self.error_message = Some(format!("Failed to clone nodes: {}", e));
            return;
        }

        // Clear original selection
        for node in self.graph.nodes.values_mut() {
            node.selected = false;
        }

        // Select cloned nodes
        for cloned in &cloned_nodes {
            if let Some(node) = self.graph.nodes.get_mut(&cloned.id) {
                node.selected = true;
            }
        }

        // Mark graph as dirty
        self.dirty = true;

        // Status message
        self.status_message = format!(
            "Cloned {} node(s) with {} connection(s)",
            cloned_nodes.len(),
            cloned_connections.len()
        );
    }
}

/// Clone a single port with a new UUID
///
/// Returns: (cloned_port, original_port_id, new_port_id)
fn clone_port(original: &Port) -> (Port, Uuid, Uuid) {
    let new_port_id = Uuid::new_v4();
    let cloned_port = Port {
        id: new_port_id,
        name: original.name.clone(),
        data_type: original.data_type.clone(),
        direction: original.direction,
        optional: original.optional,
        current_value: original.current_value.clone(),
    };
    (cloned_port, original.id, new_port_id)
}

/// Clone a node with new UUIDs for node and all ports
///
/// Returns: (cloned_node, original_node_id, new_node_id, port_mapping)
/// where port_mapping: old_port_id → new_port_id
fn clone_node(
    original: &GraphNode,
    offset: egui::Vec2,
    name_suffix: &str,
) -> (GraphNode, Uuid, Uuid, HashMap<Uuid, Uuid>) {
    let new_node_id = Uuid::new_v4();
    let mut port_mapping = HashMap::new();

    // Clone inputs with new port UUIDs
    let cloned_inputs: Vec<Port> = original
        .inputs
        .iter()
        .map(|port| {
            let (cloned, old_id, new_id) = clone_port(port);
            port_mapping.insert(old_id, new_id);
            cloned
        })
        .collect();

    // Clone outputs with new port UUIDs
    let cloned_outputs: Vec<Port> = original
        .outputs
        .iter()
        .map(|port| {
            let (cloned, old_id, new_id) = clone_port(port);
            port_mapping.insert(old_id, new_id);
            cloned
        })
        .collect();

    let cloned_node = GraphNode {
        id: new_node_id,
        component_id: original.component_id.clone(),
        display_name: format!("{}{}", original.display_name, name_suffix),
        position: original.position + offset,
        inputs: cloned_inputs,
        outputs: cloned_outputs,
        metadata: original.metadata.clone(),
        capabilities: original.capabilities.clone(),
        creator_data: original.creator_data.clone(), // Clone WASM Creator data
        composition_data: original.composition_data.clone(), // Clone composition binary
        continuous_config: original.continuous_config.clone(), // Clone continuous config
        shader_editor_data: original.shader_editor_data.clone(), // Clone GLSL shader editor data
        shader_preview_data: original.shader_preview_data.clone(), // Clone shader preview data
        needs_component_refresh: original.needs_component_refresh,
        // Reset UI/runtime state
        selected: true, // Clone is selected
        execution_state: crate::graph::node::ExecutionState::Idle,
        dirty: false,
        cached_footer_view: None, // Don't clone cached view
        execution_started_at: None,
        execution_completed_at: None,
    };

    (cloned_node, original.id, new_node_id, port_mapping)
}

/// Check if a connection is internal (both endpoints in selected set)
fn is_internal_connection(conn: &Connection, selected_nodes: &HashSet<Uuid>) -> bool {
    selected_nodes.contains(&conn.from_node) && selected_nodes.contains(&conn.to_node)
}

/// Clone all internal connections (both endpoints selected)
///
/// External connections (one endpoint outside selection) are NOT cloned.
fn clone_internal_connections(
    connections: &[Connection],
    selected_nodes: &HashSet<Uuid>,
    node_mapping: &HashMap<Uuid, Uuid>,
    port_mapping: &HashMap<Uuid, Uuid>,
) -> Vec<Connection> {
    connections
        .iter()
        .filter(|conn| is_internal_connection(conn, selected_nodes))
        .map(|conn| Connection {
            id: Uuid::new_v4(),
            from_node: *node_mapping.get(&conn.from_node).expect("Node mapping should exist"),
            from_port: *port_mapping.get(&conn.from_port).expect("Port mapping should exist"),
            to_node: *node_mapping.get(&conn.to_node).expect("Node mapping should exist"),
            to_port: *port_mapping.get(&conn.to_port).expect("Port mapping should exist"),
        })
        .collect()
}

/// Get all selected node IDs from the graph
fn get_selected_node_ids(graph: &crate::graph::graph::NodeGraph) -> HashSet<Uuid> {
    graph
        .nodes
        .iter()
        .filter(|(_, node)| node.selected)
        .map(|(id, _)| *id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::{DataType, PortDirection};

    #[test]
    fn test_clone_port_generates_new_uuid() {
        let original = Port::new(
            "test_port".to_string(),
            DataType::F32,
            PortDirection::Input,
            false,
        );
        let original_id = original.id;

        let (cloned, old_id, new_id) = clone_port(&original);

        assert_eq!(old_id, original_id);
        assert_ne!(new_id, original_id);
        assert_eq!(cloned.id, new_id);
    }

    #[test]
    fn test_clone_port_preserves_properties() {
        let original = Port::new(
            "test_port".to_string(),
            DataType::String,
            PortDirection::Output,
            true,
        );

        let (cloned, _, _) = clone_port(&original);

        assert_eq!(cloned.name, original.name);
        assert_eq!(cloned.data_type, original.data_type);
        assert_eq!(cloned.direction, original.direction);
        assert_eq!(cloned.optional, original.optional);
    }

    #[test]
    fn test_clone_node_generates_new_uuid() {
        let original = GraphNode::new(
            "test_component".to_string(),
            "Test Node".to_string(),
            egui::Pos2::new(100.0, 100.0),
        );
        let original_id = original.id;

        let (cloned, old_id, new_id, _) =
            clone_node(&original, CLONE_OFFSET, CLONE_NAME_SUFFIX);

        assert_eq!(old_id, original_id);
        assert_ne!(new_id, original_id);
        assert_eq!(cloned.id, new_id);
    }

    #[test]
    fn test_clone_node_offsets_position() {
        let original = GraphNode::new(
            "test_component".to_string(),
            "Test Node".to_string(),
            egui::Pos2::new(100.0, 200.0),
        );

        let (cloned, _, _, _) = clone_node(&original, CLONE_OFFSET, CLONE_NAME_SUFFIX);

        assert_eq!(cloned.position.x, 150.0); // 100 + 50
        assert_eq!(cloned.position.y, 250.0); // 200 + 50
    }

    #[test]
    fn test_clone_node_appends_name_suffix() {
        let original = GraphNode::new(
            "test_component".to_string(),
            "Test Node".to_string(),
            egui::Pos2::new(0.0, 0.0),
        );

        let (cloned, _, _, _) = clone_node(&original, CLONE_OFFSET, CLONE_NAME_SUFFIX);

        assert_eq!(cloned.display_name, "Test Node (Clone)");
    }

    #[test]
    fn test_clone_node_resets_state_fields() {
        let mut original = GraphNode::new(
            "test_component".to_string(),
            "Test Node".to_string(),
            egui::Pos2::new(0.0, 0.0),
        );
        original.selected = false;
        original.execution_state = crate::graph::node::ExecutionState::Running;
        original.dirty = true;

        let (cloned, _, _, _) = clone_node(&original, CLONE_OFFSET, CLONE_NAME_SUFFIX);

        assert_eq!(cloned.selected, true); // Clone is selected
        assert_eq!(
            cloned.execution_state,
            crate::graph::node::ExecutionState::Idle
        );
        assert_eq!(cloned.dirty, false);
    }

    #[test]
    fn test_is_internal_connection_both_selected() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let port1 = Uuid::new_v4();
        let port2 = Uuid::new_v4();

        let conn = Connection::new(node1, port1, node2, port2);
        let mut selected = HashSet::new();
        selected.insert(node1);
        selected.insert(node2);

        assert!(is_internal_connection(&conn, &selected));
    }

    #[test]
    fn test_is_internal_connection_one_external() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let port1 = Uuid::new_v4();
        let port2 = Uuid::new_v4();

        let conn = Connection::new(node1, port1, node2, port2);
        let mut selected = HashSet::new();
        selected.insert(node1); // Only one node selected

        assert!(!is_internal_connection(&conn, &selected));
    }

    #[test]
    fn test_clone_internal_connections_filters_external() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let node3 = Uuid::new_v4();
        let port1 = Uuid::new_v4();
        let port2 = Uuid::new_v4();
        let port3 = Uuid::new_v4();

        // Internal connection (both selected)
        let conn1 = Connection::new(node1, port1, node2, port2);
        // External connection (node3 not selected)
        let conn2 = Connection::new(node2, port2, node3, port3);

        let connections = vec![conn1, conn2];
        let mut selected = HashSet::new();
        selected.insert(node1);
        selected.insert(node2);

        // Create mappings
        let mut node_mapping = HashMap::new();
        let new_node1 = Uuid::new_v4();
        let new_node2 = Uuid::new_v4();
        node_mapping.insert(node1, new_node1);
        node_mapping.insert(node2, new_node2);

        let mut port_mapping = HashMap::new();
        let new_port1 = Uuid::new_v4();
        let new_port2 = Uuid::new_v4();
        port_mapping.insert(port1, new_port1);
        port_mapping.insert(port2, new_port2);

        let cloned = clone_internal_connections(
            &connections,
            &selected,
            &node_mapping,
            &port_mapping,
        );

        // Should only clone internal connection
        assert_eq!(cloned.len(), 1);
        assert_eq!(cloned[0].from_node, new_node1);
        assert_eq!(cloned[0].to_node, new_node2);
    }

    #[test]
    fn test_clone_internal_connections_generates_new_ids() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let port1 = Uuid::new_v4();
        let port2 = Uuid::new_v4();

        let conn = Connection::new(node1, port1, node2, port2);
        let original_conn_id = conn.id;

        let connections = vec![conn];
        let mut selected = HashSet::new();
        selected.insert(node1);
        selected.insert(node2);

        let mut node_mapping = HashMap::new();
        node_mapping.insert(node1, Uuid::new_v4());
        node_mapping.insert(node2, Uuid::new_v4());

        let mut port_mapping = HashMap::new();
        port_mapping.insert(port1, Uuid::new_v4());
        port_mapping.insert(port2, Uuid::new_v4());

        let cloned = clone_internal_connections(
            &connections,
            &selected,
            &node_mapping,
            &port_mapping,
        );

        assert_eq!(cloned.len(), 1);
        assert_ne!(cloned[0].id, original_conn_id);
    }
}
