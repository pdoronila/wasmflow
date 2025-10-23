//! Graph connections linking node outputs to inputs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A directed edge connecting one node's output port to another node's input port
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    /// Unique connection identifier
    pub id: Uuid,
    /// Source node ID
    pub from_node: Uuid,
    /// Source output port ID
    pub from_port: Uuid,
    /// Target node ID
    pub to_node: Uuid,
    /// Target input port ID
    pub to_port: Uuid,
}

impl Connection {
    /// Create a new connection between ports
    pub fn new(
        from_node: Uuid,
        from_port: Uuid,
        to_node: Uuid,
        to_port: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_node,
            from_port,
            to_node,
            to_port,
        }
    }

    /// Check if this connection involves a specific node
    pub fn involves_node(&self, node_id: Uuid) -> bool {
        self.from_node == node_id || self.to_node == node_id
    }

    /// Check if this connection uses a specific port
    pub fn involves_port(&self, port_id: Uuid) -> bool {
        self.from_port == port_id || self.to_port == port_id
    }

    /// Get the source (from) endpoint
    pub fn source(&self) -> (Uuid, Uuid) {
        (self.from_node, self.from_port)
    }

    /// Get the target (to) endpoint
    pub fn target(&self) -> (Uuid, Uuid) {
        (self.to_node, self.to_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_creation() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let port1 = Uuid::new_v4();
        let port2 = Uuid::new_v4();

        let conn = Connection::new(node1, port1, node2, port2);

        assert_eq!(conn.from_node, node1);
        assert_eq!(conn.from_port, port1);
        assert_eq!(conn.to_node, node2);
        assert_eq!(conn.to_port, port2);
    }

    #[test]
    fn test_involves_node() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let node3 = Uuid::new_v4();
        let port1 = Uuid::new_v4();
        let port2 = Uuid::new_v4();

        let conn = Connection::new(node1, port1, node2, port2);

        assert!(conn.involves_node(node1));
        assert!(conn.involves_node(node2));
        assert!(!conn.involves_node(node3));
    }

    #[test]
    fn test_involves_port() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let port1 = Uuid::new_v4();
        let port2 = Uuid::new_v4();
        let port3 = Uuid::new_v4();

        let conn = Connection::new(node1, port1, node2, port2);

        assert!(conn.involves_port(port1));
        assert!(conn.involves_port(port2));
        assert!(!conn.involves_port(port3));
    }

    #[test]
    fn test_source_target() {
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let port1 = Uuid::new_v4();
        let port2 = Uuid::new_v4();

        let conn = Connection::new(node1, port1, node2, port2);

        assert_eq!(conn.source(), (node1, port1));
        assert_eq!(conn.target(), (node2, port2));
    }
}
