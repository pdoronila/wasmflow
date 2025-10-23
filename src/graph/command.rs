//! Command pattern for undo/redo functionality

use super::connection::Connection;
use super::graph::NodeGraph;
use super::node::{GraphNode, NodeValue};
use uuid::Uuid;

/// A command that can be executed and undone
#[derive(Debug, Clone)]
pub enum Command {
    /// Add a node to the graph
    AddNode {
        node: GraphNode,
    },
    /// Remove a node from the graph
    RemoveNode {
        node_id: Uuid,
        node: GraphNode,
        connections: Vec<Connection>,
    },
    /// Move a node
    MoveNode {
        node_id: Uuid,
        old_position: egui::Pos2,
        new_position: egui::Pos2,
    },
    /// Add a connection
    AddConnection {
        from_node: Uuid,
        from_port: Uuid,
        to_node: Uuid,
        to_port: Uuid,
        connection_id: Option<Uuid>,
    },
    /// Remove a connection
    RemoveConnection {
        connection: Connection,
    },
    /// Change a node's constant value
    ChangeConstantValue {
        node_id: Uuid,
        port_index: usize,
        old_value: NodeValue,
        new_value: NodeValue,
    },
}

impl Command {
    /// Execute the command
    pub fn execute(&mut self, graph: &mut NodeGraph) -> Result<(), String> {
        match self {
            Command::AddNode { node } => {
                graph.add_node(node.clone());
                Ok(())
            }
            Command::RemoveNode {
                node_id,
                node,
                connections,
            } => {
                // Store the node and its connections before removal
                if let Some(removed_node) = graph.nodes.get(node_id) {
                    *node = removed_node.clone();
                    *connections = graph.node_connections(*node_id).into_iter().cloned().collect();
                }
                graph.remove_node(*node_id).map(|_| ()).map_err(|e| e.to_string())
            }
            Command::MoveNode {
                node_id,
                old_position: _,
                new_position,
            } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    node.position = *new_position;
                    Ok(())
                } else {
                    Err("Node not found".to_string())
                }
            }
            Command::AddConnection {
                from_node,
                from_port,
                to_node,
                to_port,
                connection_id,
            } => {
                let conn_id = graph
                    .add_connection(*from_node, *from_port, *to_node, *to_port)
                    .map_err(|e| e.to_string())?;
                *connection_id = Some(conn_id);
                Ok(())
            }
            Command::RemoveConnection { connection } => {
                graph
                    .remove_connection(connection.id)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            Command::ChangeConstantValue {
                node_id,
                port_index,
                old_value: _,
                new_value,
            } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    if let Some(port) = node.outputs.get_mut(*port_index) {
                        port.current_value = Some(new_value.clone());
                        Ok(())
                    } else {
                        Err("Port not found".to_string())
                    }
                } else {
                    Err("Node not found".to_string())
                }
            }
        }
    }

    /// Undo the command
    pub fn undo(&self, graph: &mut NodeGraph) -> Result<(), String> {
        match self {
            Command::AddNode { node } => {
                graph.remove_node(node.id).map(|_| ()).map_err(|e| e.to_string())
            }
            Command::RemoveNode {
                node,
                connections,
                ..
            } => {
                graph.add_node(node.clone());
                // Restore connections
                for conn in connections {
                    let _ = graph.add_connection(
                        conn.from_node,
                        conn.from_port,
                        conn.to_node,
                        conn.to_port,
                    );
                }
                Ok(())
            }
            Command::MoveNode {
                node_id,
                old_position,
                new_position: _,
            } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    node.position = *old_position;
                    Ok(())
                } else {
                    Err("Node not found".to_string())
                }
            }
            Command::AddConnection { connection_id, .. } => {
                if let Some(conn_id) = connection_id {
                    graph.remove_connection(*conn_id).map(|_| ()).map_err(|e| e.to_string())
                } else {
                    Err("Connection ID not set".to_string())
                }
            }
            Command::RemoveConnection { connection } => {
                graph
                    .add_connection(
                        connection.from_node,
                        connection.from_port,
                        connection.to_node,
                        connection.to_port,
                    )
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            Command::ChangeConstantValue {
                node_id,
                port_index,
                old_value,
                new_value: _,
            } => {
                if let Some(node) = graph.nodes.get_mut(node_id) {
                    if let Some(port) = node.outputs.get_mut(*port_index) {
                        port.current_value = Some(old_value.clone());
                        Ok(())
                    } else {
                        Err("Port not found".to_string())
                    }
                } else {
                    Err("Node not found".to_string())
                }
            }
        }
    }
}

/// Command history for undo/redo
#[derive(Debug, Default)]
pub struct CommandHistory {
    /// Stack of executed commands (for undo)
    undo_stack: Vec<Command>,
    /// Stack of undone commands (for redo)
    redo_stack: Vec<Command>,
    /// Maximum history size
    max_size: usize,
}

impl CommandHistory {
    /// Create a new command history with default max size
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size: 100,
        }
    }

    /// Create a new command history with specified max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Execute a command and add it to history
    pub fn execute(&mut self, mut command: Command, graph: &mut NodeGraph) -> Result<(), String> {
        command.execute(graph)?;

        // Clear redo stack when new command is executed
        self.redo_stack.clear();

        // Add to undo stack
        self.undo_stack.push(command);

        // Limit stack size
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }

        Ok(())
    }

    /// Undo the last command
    pub fn undo(&mut self, graph: &mut NodeGraph) -> Result<(), String> {
        if let Some(command) = self.undo_stack.pop() {
            command.undo(graph)?;
            self.redo_stack.push(command);
            Ok(())
        } else {
            Err("Nothing to undo".to_string())
        }
    }

    /// Redo the last undone command
    pub fn redo(&mut self, graph: &mut NodeGraph) -> Result<(), String> {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(graph)?;
            self.undo_stack.push(command);
            Ok(())
        } else {
            Err("Nothing to redo".to_string())
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::ComponentSpec;

    #[test]
    fn test_add_remove_node() {
        let mut graph = NodeGraph::new("Test".to_string(), "Author".to_string());
        let mut history = CommandHistory::new();

        let spec = ComponentSpec::new_builtin(
            "test:node".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            None,
        );
        let node = spec.create_node(egui::Pos2::new(0.0, 0.0));
        let _node_id = node.id;

        // Add node
        let cmd = Command::AddNode { node: node.clone() };
        history.execute(cmd, &mut graph).unwrap();
        assert_eq!(graph.nodes.len(), 1);

        // Undo
        history.undo(&mut graph).unwrap();
        assert_eq!(graph.nodes.len(), 0);

        // Redo
        history.redo(&mut graph).unwrap();
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_move_node() {
        let mut graph = NodeGraph::new("Test".to_string(), "Author".to_string());
        let mut history = CommandHistory::new();

        let spec = ComponentSpec::new_builtin(
            "test:node".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            None,
        );
        let node = spec.create_node(egui::Pos2::new(0.0, 0.0));
        let node_id = node.id;
        graph.add_node(node);

        // Move node
        let cmd = Command::MoveNode {
            node_id,
            old_position: egui::Pos2::new(0.0, 0.0),
            new_position: egui::Pos2::new(100.0, 100.0),
        };
        history.execute(cmd, &mut graph).unwrap();

        assert_eq!(graph.nodes.get(&node_id).unwrap().position, egui::Pos2::new(100.0, 100.0));

        // Undo
        history.undo(&mut graph).unwrap();
        assert_eq!(graph.nodes.get(&node_id).unwrap().position, egui::Pos2::new(0.0, 0.0));
    }
}
