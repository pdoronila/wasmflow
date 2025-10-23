//! Graph data structures and operations
//!
//! This module contains the core graph types: nodes, connections, ports, and values.

pub mod command;
pub mod connection;
pub mod drill_down; // T005: Drill-down view context management
pub mod execution;
#[allow(clippy::module_inception)]
pub mod graph;
pub mod node;
pub mod serialization;
pub mod state;
pub mod validation; // T003: Graph connectivity validation

pub use command::{Command, CommandHistory};
pub use connection::Connection;
pub use graph::NodeGraph;
pub use node::{ComponentSpec, GraphNode, NodeValue, Port};
pub use state::{can_start, can_stop, validate_transition};
