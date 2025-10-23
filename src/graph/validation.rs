//! Graph connectivity validation
//!
//! This module provides functions for validating graph properties,
//! particularly for checking if selected nodes form a connected subgraph.

use crate::graph::NodeGraph;
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Dfs;
use std::collections::HashMap;

/// Node identifier type (re-export from graph module)
pub type NodeId = uuid::Uuid;

/// Check if a set of nodes forms a connected subgraph
///
/// A connected subgraph means all selected nodes are reachable from each other
/// through edges that only traverse other selected nodes.
///
/// This is required for composition - we cannot compose disconnected nodes
/// as they would not be able to communicate via ports.
///
/// # Arguments
/// * `graph` - The node graph to validate
/// * `node_ids` - The set of nodes to check for connectivity
///
/// # Returns
/// * `Ok(true)` if nodes form a connected subgraph
/// * `Ok(false)` if nodes are disconnected, less than 2 nodes, or some nodes don't exist
/// * `Err(_)` if validation fails
///
/// # Implementation
/// Uses petgraph DFS to check reachability treating edges as undirected.
pub fn is_connected_subgraph(graph: &NodeGraph, node_ids: &[NodeId]) -> Result<bool> {
    // Validate we have at least 2 nodes
    if node_ids.len() < 2 {
        return Ok(false);
    }

    // Build a petgraph from the selected nodes
    // We treat the graph as undirected for connectivity purposes
    let mut pg: DiGraph<NodeId, ()> = DiGraph::new();
    let mut uuid_to_index: HashMap<NodeId, NodeIndex> = HashMap::new();

    // Add nodes to petgraph
    for &node_id in node_ids {
        // Verify node exists in graph
        if !graph.nodes.contains_key(&node_id) {
            log::warn!("Node {} not found in graph", node_id);
            return Ok(false);
        }

        let idx = pg.add_node(node_id);
        uuid_to_index.insert(node_id, idx);
    }

    // Add edges between selected nodes (only edges where both endpoints are selected)
    // We add edges in both directions to treat as undirected graph
    for conn in &graph.connections {
        // Check if both endpoints are in our selected set
        if uuid_to_index.contains_key(&conn.from_node)
            && uuid_to_index.contains_key(&conn.to_node) {

            let from_idx = uuid_to_index[&conn.from_node];
            let to_idx = uuid_to_index[&conn.to_node];

            // Add edge in both directions (treating as undirected)
            pg.add_edge(from_idx, to_idx, ());
            pg.add_edge(to_idx, from_idx, ());
        }
    }

    // Perform DFS from the first node
    let start_idx = uuid_to_index[&node_ids[0]];
    let mut dfs = Dfs::new(&pg, start_idx);

    let mut visited_count = 0;
    while dfs.next(&pg).is_some() {
        visited_count += 1;
    }

    // All nodes must be reachable from the first node
    Ok(visited_count == node_ids.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::graph::NodeGraph;

    #[test]
    fn test_validation_rejects_single_node() {
        let graph = NodeGraph::new("test".to_string(), "test_author".to_string());
        let node_id = uuid::Uuid::new_v4();

        let result = is_connected_subgraph(&graph, &[node_id]).unwrap();
        assert!(!result, "Single node should not be considered connected");
    }

    #[test]
    fn test_validation_rejects_empty_selection() {
        let graph = NodeGraph::new("test".to_string(), "test_author".to_string());

        let result = is_connected_subgraph(&graph, &[]).unwrap();
        assert!(!result, "Empty selection should not be considered connected");
    }

    #[test]
    fn test_validation_rejects_nonexistent_nodes() {
        let graph = NodeGraph::new("test".to_string(), "test_author".to_string());
        let node_id1 = uuid::Uuid::new_v4();
        let node_id2 = uuid::Uuid::new_v4();

        let result = is_connected_subgraph(&graph, &[node_id1, node_id2]).unwrap();
        assert!(!result, "Nonexistent nodes should not be considered connected");
    }
}
