//! Node graph data structure and operations

use crate::graph::connection::Connection;
use crate::graph::node::{DataType, GraphNode, PortDirection};
use crate::runtime::capabilities::CapabilityGrant;
use crate::GraphError;
use petgraph::algo;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use uuid::Uuid;

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub created_at: String,
    pub modified_at: String,
    pub author: String,
    #[serde(default)]
    pub description: String,
}

impl GraphMetadata {
    pub fn new(author: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            created_at: now.clone(),
            modified_at: now,
            author,
            description: String::new(),
        }
    }

    pub fn touch(&mut self) {
        self.modified_at = chrono::Utc::now().to_rfc3339();
    }
}

/// Container for the complete visual programming graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraph {
    /// Unique graph identifier
    pub id: Uuid,
    /// User-assigned graph name
    pub name: String,
    /// All nodes in graph (BTreeMap for deterministic serialization)
    pub nodes: BTreeMap<Uuid, GraphNode>,
    /// All edges between nodes
    pub connections: Vec<Connection>,
    /// Graph metadata
    pub metadata: GraphMetadata,
    /// Schema version for serialization compatibility
    pub version: u32,
    /// T074: Capability grants for nodes (persisted with graph)
    /// Maps node_id to approved capability grant
    pub capability_grants: BTreeMap<Uuid, CapabilityGrant>,
    /// Cached topological sort result (invalidated on structural changes)
    #[serde(skip)]
    execution_order_cache: Option<Vec<Uuid>>,
}

impl NodeGraph {
    /// Create a new empty graph
    pub fn new(name: String, author: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            nodes: BTreeMap::new(),
            connections: Vec::new(),
            metadata: GraphMetadata::new(author),
            version: 1,
            capability_grants: BTreeMap::new(),
            execution_order_cache: None,
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: GraphNode) -> Uuid {
        let node_id = node.id;
        self.nodes.insert(node_id, node);
        self.invalidate_cache();
        self.metadata.touch();
        node_id
    }

    /// Remove a node and all connected edges
    pub fn remove_node(&mut self, node_id: Uuid) -> Result<GraphNode, GraphError> {
        // Remove all connections involving this node
        self.connections.retain(|conn| !conn.involves_node(node_id));

        // Remove the node
        let node = self.nodes.remove(&node_id).ok_or_else(|| {
            GraphError::InvalidConnection(format!("Node {} not found", node_id))
        })?;

        self.invalidate_cache();
        self.metadata.touch();
        Ok(node)
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: Uuid) -> Option<&GraphNode> {
        self.nodes.get(&node_id)
    }

    /// T059: Mark all nodes using a component as needing refresh
    /// Call this when a component is recompiled/updated
    pub fn mark_component_users_for_refresh(&mut self, component_id: &str) {
        let mut count = 0;
        for node in self.nodes.values_mut() {
            if node.component_id == component_id {
                node.needs_component_refresh = true;
                count += 1;
            }
        }
        if count > 0 {
            log::info!("Marked {} nodes using component '{}' for refresh", count, component_id);
            self.metadata.touch();
        }
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, node_id: Uuid) -> Option<&mut GraphNode> {
        self.nodes.get_mut(&node_id)
    }

    /// Add a connection between two ports
    pub fn add_connection(
        &mut self,
        from_node: Uuid,
        from_port: Uuid,
        to_node: Uuid,
        to_port: Uuid,
    ) -> Result<Uuid, GraphError> {
        // Validate nodes exist
        let source_node = self
            .nodes
            .get(&from_node)
            .ok_or_else(|| GraphError::InvalidConnection("Source node not found".to_string()))?;
        let target_node = self
            .nodes
            .get(&to_node)
            .ok_or_else(|| GraphError::InvalidConnection("Target node not found".to_string()))?;

        // Find ports
        let source_port = source_node
            .outputs
            .iter()
            .find(|p| p.id == from_port)
            .ok_or_else(|| GraphError::InvalidConnection("Source port not found".to_string()))?;
        let target_port = target_node
            .inputs
            .iter()
            .find(|p| p.id == to_port)
            .ok_or_else(|| GraphError::InvalidConnection("Target port not found".to_string()))?;

        // Validate port directions
        if source_port.direction != PortDirection::Output {
            return Err(GraphError::InvalidConnection(
                "Source port must be an output".to_string(),
            ));
        }
        if target_port.direction != PortDirection::Input {
            return Err(GraphError::InvalidConnection(
                "Target port must be an input".to_string(),
            ));
        }

        // Validate no self-connections
        if from_node == to_node {
            return Err(GraphError::InvalidConnection(
                "Self-connections are not allowed".to_string(),
            ));
        }

        // Check type compatibility
        if !Self::types_compatible(&source_port.data_type, &target_port.data_type) {
            return Err(GraphError::TypeMismatch {
                from: source_port.data_type.name(),
                to: target_port.data_type.name(),
            });
        }

        // Check if target port already has a connection
        if self
            .connections
            .iter()
            .any(|conn| conn.to_port == to_port)
        {
            return Err(GraphError::InvalidConnection(
                "Input port already has a connection".to_string(),
            ));
        }

        // Create connection
        let connection = Connection::new(from_node, from_port, to_node, to_port);
        let connection_id = connection.id;
        self.connections.push(connection);

        // Check for cycles
        if self.has_cycle()? {
            // Remove the connection we just added
            self.connections.pop();
            return Err(GraphError::CycleDetected(vec![from_node, to_node]));
        }

        self.invalidate_cache();
        self.metadata.touch();
        Ok(connection_id)
    }

    /// Remove a connection by ID
    pub fn remove_connection(&mut self, connection_id: Uuid) -> Result<Connection, GraphError> {
        let index = self
            .connections
            .iter()
            .position(|conn| conn.id == connection_id)
            .ok_or_else(|| GraphError::InvalidConnection("Connection not found".to_string()))?;

        let connection = self.connections.remove(index);
        self.invalidate_cache();
        self.metadata.touch();
        Ok(connection)
    }

    /// Check if data types are compatible for connection
    pub fn types_compatible(from: &DataType, to: &DataType) -> bool {
        // Any type matches everything
        if matches!(from, DataType::Any) || matches!(to, DataType::Any) {
            return true;
        }

        // Exact match
        if from == to {
            return true;
        }

        // List compatibility
        if let (DataType::List(from_inner), DataType::List(to_inner)) = (from, to) {
            return Self::types_compatible(from_inner, to_inner);
        }

        false
    }

    /// Build a petgraph DiGraph for analysis
    fn build_digraph(&self) -> DiGraph<Uuid, ()> {
        let mut graph = DiGraph::new();
        let mut node_indices: HashMap<Uuid, NodeIndex> = HashMap::new();

        // Add nodes
        for node_id in self.nodes.keys() {
            let index = graph.add_node(*node_id);
            node_indices.insert(*node_id, index);
        }

        // Add edges
        for connection in &self.connections {
            if let (Some(&from_idx), Some(&to_idx)) = (
                node_indices.get(&connection.from_node),
                node_indices.get(&connection.to_node),
            ) {
                graph.add_edge(from_idx, to_idx, ());
            }
        }

        graph
    }

    /// Check if the graph contains cycles
    pub fn has_cycle(&self) -> Result<bool, GraphError> {
        let graph = self.build_digraph();
        Ok(algo::is_cyclic_directed(&graph))
    }

    /// Get the execution order using topological sort
    pub fn execution_order(&mut self) -> Result<Vec<Uuid>, GraphError> {
        // Return cached result if available
        if let Some(cached) = &self.execution_order_cache {
            return Ok(cached.clone());
        }

        let graph = self.build_digraph();

        // Check for cycles
        if algo::is_cyclic_directed(&graph) {
            return Err(GraphError::CycleDetected(
                self.nodes.keys().copied().collect(),
            ));
        }

        // Perform topological sort
        let sorted = algo::toposort(&graph, None).map_err(|cycle| {
            GraphError::CycleDetected(vec![*graph.node_weight(cycle.node_id()).unwrap()])
        })?;

        // Convert NodeIndex back to Uuid
        let order: Vec<Uuid> = sorted
            .into_iter()
            .map(|idx| *graph.node_weight(idx).unwrap())
            .collect();

        // Cache the result
        self.execution_order_cache = Some(order.clone());

        Ok(order)
    }

    /// Validate the entire graph
    pub fn validate(&self) -> Result<ValidationReport, GraphError> {
        let mut report = ValidationReport::default();

        // Check for cycles
        let graph = self.build_digraph();
        if algo::is_cyclic_directed(&graph) {
            report.errors.push("Graph contains cycles".to_string());
        }

        // Validate all connections
        for connection in &self.connections {
            // Check nodes exist
            if !self.nodes.contains_key(&connection.from_node) {
                report.errors.push(format!(
                    "Connection {} references non-existent source node",
                    connection.id
                ));
            }
            if !self.nodes.contains_key(&connection.to_node) {
                report.errors.push(format!(
                    "Connection {} references non-existent target node",
                    connection.id
                ));
            }
        }

        // Check required inputs
        for (node_id, node) in &self.nodes {
            for input in &node.inputs {
                if !input.optional {
                    let has_connection = self
                        .connections
                        .iter()
                        .any(|conn| conn.to_node == *node_id && conn.to_port == input.id);
                    if !has_connection {
                        report.warnings.push(format!(
                            "Node {} has unconnected required input '{}'",
                            node.display_name, input.name
                        ));
                    }
                }
            }
        }

        Ok(report)
    }

    /// Invalidate cached execution order
    fn invalidate_cache(&mut self) {
        self.execution_order_cache = None;
    }

    /// Get all connections involving a specific node
    pub fn node_connections(&self, node_id: Uuid) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|conn| conn.involves_node(node_id))
            .collect()
    }

    /// Get incoming connections for a node
    pub fn incoming_connections(&self, node_id: Uuid) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|conn| conn.to_node == node_id)
            .collect()
    }

    /// Get outgoing connections for a node
    pub fn outgoing_connections(&self, node_id: Uuid) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|conn| conn.from_node == node_id)
            .collect()
    }

    /// T074: Grant capability to a node
    pub fn grant_capability(&mut self, grant: CapabilityGrant) {
        self.capability_grants.insert(grant.node_id, grant);
        self.metadata.touch();
    }

    /// T074: Revoke capability from a node
    pub fn revoke_capability(&mut self, node_id: Uuid) -> Option<CapabilityGrant> {
        let grant = self.capability_grants.remove(&node_id);
        if grant.is_some() {
            self.metadata.touch();
        }
        grant
    }

    /// T074: Get capability grant for a node
    pub fn get_capability_grant(&self, node_id: Uuid) -> Option<&CapabilityGrant> {
        self.capability_grants.get(&node_id)
    }

    /// T074: Check if a node has a capability grant
    pub fn has_capability_grant(&self, node_id: Uuid) -> bool {
        self.capability_grants.contains_key(&node_id)
    }
}

/// Validation report
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::{ComponentSpec, DataType};

    #[test]
    fn test_graph_creation() {
        let graph = NodeGraph::new("Test Graph".to_string(), "Test Author".to_string());
        assert_eq!(graph.name, "Test Graph");
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.connections.len(), 0);
    }

    #[test]
    fn test_add_remove_node() {
        let mut graph = NodeGraph::new("Test".to_string(), "Author".to_string());
        let spec = ComponentSpec::new_builtin(
            "test:node".to_string(),
            "Test Node".to_string(),
            "A test node".to_string(),
            None,
        );
        let node = spec.create_node(egui::Pos2::new(0.0, 0.0));
        let node_id = node.id;

        graph.add_node(node);
        assert_eq!(graph.nodes.len(), 1);

        let removed = graph.remove_node(node_id).unwrap();
        assert_eq!(removed.id, node_id);
        assert_eq!(graph.nodes.len(), 0);
    }

    #[test]
    fn test_type_compatibility() {
        assert!(NodeGraph::types_compatible(
            &DataType::F32,
            &DataType::F32
        ));
        assert!(NodeGraph::types_compatible(&DataType::Any, &DataType::F32));
        assert!(NodeGraph::types_compatible(&DataType::F32, &DataType::Any));
        assert!(!NodeGraph::types_compatible(
            &DataType::F32,
            &DataType::I32
        ));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = NodeGraph::new("Test".to_string(), "Author".to_string());

        // Create three nodes
        let spec = ComponentSpec::new_builtin(
            "test:node".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            None,
        )
        .with_input("in".to_string(), DataType::F32, "Input".to_string())
        .with_output("out".to_string(), DataType::F32, "Output".to_string());

        let node1 = spec.create_node(egui::Pos2::new(0.0, 0.0));
        let node2 = spec.create_node(egui::Pos2::new(100.0, 0.0));
        let node3 = spec.create_node(egui::Pos2::new(200.0, 0.0));

        let id1 = node1.id;
        let id2 = node2.id;
        let id3 = node3.id;

        let port1_out = node1.outputs[0].id;
        let port2_in = node2.inputs[0].id;
        let port2_out = node2.outputs[0].id;
        let port3_in = node3.inputs[0].id;
        let port3_out = node3.outputs[0].id;
        let port1_in = node1.inputs[0].id;

        graph.add_node(node1);
        graph.add_node(node2);
        graph.add_node(node3);

        // Create chain: node1 -> node2 -> node3
        graph
            .add_connection(id1, port1_out, id2, port2_in)
            .unwrap();
        graph
            .add_connection(id2, port2_out, id3, port3_in)
            .unwrap();

        // Try to create cycle: node3 -> node1
        let result = graph.add_connection(id3, port3_out, id1, port1_in);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphError::CycleDetected(_)));
    }
}
