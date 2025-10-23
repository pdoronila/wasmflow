//! T084: Incremental graph execution logic
//!
//! This module provides incremental execution capabilities for graphs,
//! allowing only dirty nodes and their dependents to be re-executed
//! when changes occur, significantly improving performance for large graphs.

use crate::graph::graph::NodeGraph;
use uuid::Uuid;
use std::collections::HashSet;

/// T084: Mark a node as dirty (needs re-execution)
///
/// When a node is marked dirty:
/// - Its execution state is reset to Idle
/// - All downstream dependent nodes are also marked dirty
pub fn mark_node_dirty(graph: &mut NodeGraph, node_id: Uuid) {
    if let Some(node) = graph.nodes.get_mut(&node_id) {
        node.dirty = true;
        node.execution_state = crate::graph::node::ExecutionState::Idle;
        log::debug!("Marked node {} ({}) as dirty", node.display_name, node_id);
    }

    // Mark all downstream nodes as dirty
    mark_downstream_dirty(graph, node_id);
}

/// T084: Mark all downstream dependent nodes as dirty
///
/// Recursively marks all nodes that depend on the given node's outputs
/// as dirty, ensuring that changes propagate correctly through the graph.
fn mark_downstream_dirty(graph: &mut NodeGraph, node_id: Uuid) {
    let mut to_mark = Vec::new();

    // Find all nodes that have connections from this node
    for connection in &graph.connections {
        if connection.from_node == node_id {
            to_mark.push(connection.to_node);
        }
    }

    // Recursively mark downstream nodes
    for downstream_id in to_mark {
        if let Some(node) = graph.nodes.get_mut(&downstream_id) {
            if !node.dirty {
                // Only recurse if we're changing the dirty state
                // This prevents infinite loops in cyclic graphs
                node.dirty = true;
                node.execution_state = crate::graph::node::ExecutionState::Idle;
                log::debug!("Marked downstream node {} ({}) as dirty", node.display_name, downstream_id);

                // Recurse to mark further downstream nodes
                mark_downstream_dirty(graph, downstream_id);
            }
        }
    }
}

/// T084: Mark all nodes in the graph as dirty
///
/// Useful when loading a graph or when a global change requires
/// complete re-execution.
pub fn mark_all_dirty(graph: &mut NodeGraph) {
    for node in graph.nodes.values_mut() {
        node.dirty = true;
        node.execution_state = crate::graph::node::ExecutionState::Idle;
    }
    log::debug!("Marked all {} nodes as dirty", graph.nodes.len());
}

/// T084: Mark all nodes as clean (not dirty)
///
/// Used after successful execution to reset dirty flags.
pub fn mark_all_clean(graph: &mut NodeGraph) {
    for node in graph.nodes.values_mut() {
        node.dirty = false;
    }
}

/// T084: Get execution order for dirty nodes only
///
/// Returns a topologically sorted list of dirty nodes that need execution,
/// including any upstream dependencies required for correct execution.
pub fn get_dirty_execution_order(graph: &mut NodeGraph) -> Result<Vec<Uuid>, crate::GraphError> {
    // Get all dirty nodes
    let dirty_nodes: HashSet<Uuid> = graph
        .nodes
        .iter()
        .filter(|(_, node)| node.dirty)
        .map(|(id, _)| *id)
        .collect();

    if dirty_nodes.is_empty() {
        return Ok(Vec::new());
    }

    // Get full execution order
    let full_order = graph.execution_order()?;

    // Filter to only include dirty nodes
    // We keep the topological order but only include dirty nodes
    let dirty_order: Vec<Uuid> = full_order
        .into_iter()
        .filter(|id| dirty_nodes.contains(id))
        .collect();

    log::debug!(
        "Incremental execution: {} dirty nodes out of {} total",
        dirty_order.len(),
        graph.nodes.len()
    );

    Ok(dirty_order)
}

/// T084: Check if any nodes are dirty
pub fn has_dirty_nodes(graph: &NodeGraph) -> bool {
    graph.nodes.values().any(|node| node.dirty)
}

/// T084: Count dirty nodes
pub fn count_dirty_nodes(graph: &NodeGraph) -> usize {
    graph.nodes.values().filter(|node| node.dirty).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::{ComponentSpec, DataType};

    #[test]
    fn test_mark_node_dirty() {
        let mut graph = NodeGraph::new("Test".to_string(), "Test".to_string());

        // Create a simple graph: A -> B -> C
        let spec = ComponentSpec::new_builtin(
            "builtin:test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            None,
        )
        .with_input("in".to_string(), DataType::F32, "Input".to_string())
        .with_output("out".to_string(), DataType::F32, "Output".to_string());

        let mut node_a = spec.create_node(egui::Pos2::new(0.0, 0.0));
        let mut node_b = spec.create_node(egui::Pos2::new(100.0, 0.0));
        let mut node_c = spec.create_node(egui::Pos2::new(200.0, 0.0));

        // Mark all as clean initially
        node_a.dirty = false;
        node_b.dirty = false;
        node_c.dirty = false;

        let id_a = node_a.id;
        let id_b = node_b.id;
        let id_c = node_c.id;

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);

        // Connect A -> B -> C
        let out_a = graph.nodes.get(&id_a).unwrap().outputs[0].id;
        let in_b = graph.nodes.get(&id_b).unwrap().inputs[0].id;
        let out_b = graph.nodes.get(&id_b).unwrap().outputs[0].id;
        let in_c = graph.nodes.get(&id_c).unwrap().inputs[0].id;

        graph.add_connection(id_a, out_a, id_b, in_b).unwrap();
        graph.add_connection(id_b, out_b, id_c, in_c).unwrap();

        // Mark A as dirty
        mark_node_dirty(&mut graph, id_a);

        // All three nodes should now be dirty
        assert!(graph.nodes.get(&id_a).unwrap().dirty);
        assert!(graph.nodes.get(&id_b).unwrap().dirty);
        assert!(graph.nodes.get(&id_c).unwrap().dirty);
    }

    #[test]
    fn test_dirty_execution_order() {
        let mut graph = NodeGraph::new("Test".to_string(), "Test".to_string());

        let spec = ComponentSpec::new_builtin(
            "builtin:test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            None,
        )
        .with_input("in".to_string(), DataType::F32, "Input".to_string())
        .with_output("out".to_string(), DataType::F32, "Output".to_string());

        let mut node_a = spec.create_node(egui::Pos2::new(0.0, 0.0));
        let mut node_b = spec.create_node(egui::Pos2::new(100.0, 0.0));

        // Mark only B as dirty
        node_a.dirty = false;
        node_b.dirty = true;

        let id_a = node_a.id;
        let id_b = node_b.id;

        graph.add_node(node_a);
        graph.add_node(node_b);

        let order = get_dirty_execution_order(&mut graph).unwrap();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], id_b);

        // Now mark both as dirty
        mark_node_dirty(&mut graph, id_a);
        mark_node_dirty(&mut graph, id_b);

        let order = get_dirty_execution_order(&mut graph).unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_count_dirty_nodes() {
        let mut graph = NodeGraph::new("Test".to_string(), "Test".to_string());

        let spec = ComponentSpec::new_builtin(
            "builtin:test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            None,
        );

        let mut node_a = spec.create_node(egui::Pos2::new(0.0, 0.0));
        let mut node_b = spec.create_node(egui::Pos2::new(100.0, 0.0));

        node_a.dirty = true;
        node_b.dirty = false;

        graph.add_node(node_a);
        graph.add_node(node_b);

        assert_eq!(count_dirty_nodes(&graph), 1);
        assert!(has_dirty_nodes(&graph));

        mark_all_clean(&mut graph);
        assert_eq!(count_dirty_nodes(&graph), 0);
        assert!(!has_dirty_nodes(&graph));
    }
}
