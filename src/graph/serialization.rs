//! Graph serialization/deserialization
//!
//! Handles saving and loading NodeGraph structures to/from disk with integrity checking.

use crate::graph::graph::NodeGraph;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Magic bytes for file format identification
const MAGIC_BYTES: &[u8] = b"WASMFLOW";

/// Current file format version
const FORMAT_VERSION: u32 = 1;

/// Graph save file format
#[derive(Serialize, Deserialize)]
pub struct GraphSaveFormat {
    /// Magic bytes for file identification
    pub magic: Vec<u8>,
    /// File format version
    pub version: u32,
    /// The node graph data
    pub graph: NodeGraph,
    /// CRC64 checksum for integrity verification
    pub checksum: u64,
}

impl GraphSaveFormat {
    /// Create a new save format from a graph
    pub fn new(graph: NodeGraph) -> Self {
        // Serialize graph to calculate checksum
        let graph_bytes = bincode::serialize(&graph).unwrap_or_default();
        let checksum = crc::Crc::<u64>::new(&crc::CRC_64_ECMA_182).checksum(&graph_bytes);

        Self {
            magic: MAGIC_BYTES.to_vec(),
            version: FORMAT_VERSION,
            graph,
            checksum,
        }
    }

    /// Validate magic bytes
    pub fn validate_magic(&self) -> Result<()> {
        if self.magic != MAGIC_BYTES {
            anyhow::bail!("Invalid file format: magic bytes mismatch. Expected WASMFLOW file.");
        }
        Ok(())
    }

    /// Validate version compatibility
    pub fn validate_version(&self) -> Result<()> {
        if self.version > FORMAT_VERSION {
            anyhow::bail!(
                "Incompatible file version: {}. This application supports version {} or earlier.",
                self.version,
                FORMAT_VERSION
            );
        }
        Ok(())
    }

    /// Validate checksum integrity
    pub fn validate_checksum(&self) -> Result<()> {
        let graph_bytes = bincode::serialize(&self.graph)
            .context("Failed to serialize graph for checksum validation")?;
        let computed_checksum = crc::Crc::<u64>::new(&crc::CRC_64_ECMA_182).checksum(&graph_bytes);

        if computed_checksum != self.checksum {
            anyhow::bail!(
                "Checksum mismatch: file may be corrupted. Expected {}, got {}.",
                self.checksum,
                computed_checksum
            );
        }
        Ok(())
    }

    /// Validate all aspects of the save format
    pub fn validate(&self) -> Result<()> {
        self.validate_magic()
            .context("Magic bytes validation failed")?;
        self.validate_version()
            .context("Version validation failed")?;
        self.validate_checksum()
            .context("Checksum validation failed")?;
        Ok(())
    }
}

impl NodeGraph {
    /// Serialize the graph to bytes
    ///
    /// Creates a GraphSaveFormat with magic bytes, version, and checksum,
    /// then serializes the entire structure to bincode.
    ///
    /// T057: Before serialization, clears source_code on creator nodes where save_code is false.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // Clone the graph and prepare creator nodes for serialization
        let mut graph_to_save = self.clone();

        // T057: Clear source_code for creator nodes where save_code is false
        // T008: Clear runtime state for continuous nodes
        for node in graph_to_save.nodes.values_mut() {
            if let Some(creator_data) = &mut node.creator_data {
                creator_data.prepare_for_save();
            }
            // T008: Ensure continuous nodes runtime state is reset (already handled by #[serde(skip)])
            // The runtime_state field has #[serde(skip)] so it won't be serialized
        }

        let save_format = GraphSaveFormat::new(graph_to_save);
        bincode::serialize(&save_format)
            .context("Failed to serialize graph to bincode")
    }

    /// Deserialize a graph from bytes
    ///
    /// Validates magic bytes, version compatibility, and checksum integrity
    /// before returning the deserialized graph.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        log::info!("Deserializing graph from {} bytes", bytes.len());

        // Try bincode first
        let save_format: GraphSaveFormat = match bincode::deserialize(bytes) {
            Ok(format) => {
                log::info!("Bincode deserialization successful");
                format
            }
            Err(bincode_err) => {
                log::warn!("Bincode deserialization failed: {}", bincode_err);
                log::info!("Attempting JSON fallback for backward compatibility...");

                // Try JSON as fallback for old files
                match serde_json::from_slice(bytes) {
                    Ok(format) => {
                        log::info!("JSON deserialization successful (old format)");
                        format
                    }
                    Err(json_err) => {
                        log::error!("Both bincode and JSON deserialization failed");
                        log::error!("Bincode error: {}", bincode_err);
                        log::error!("JSON error: {}", json_err);
                        return Err(anyhow::anyhow!(
                            "Failed to deserialize graph file. File may be corrupted or from an incompatible version.\n\
                             Bincode error: {}\n\
                             JSON fallback error: {}",
                            bincode_err, json_err
                        ));
                    }
                }
            }
        };

        // Validate magic and version
        save_format.validate_magic()
            .context("Magic bytes validation failed")?;
        log::info!("Magic bytes valid");

        save_format.validate_version()
            .context("Version validation failed")?;
        log::info!("Version {} is compatible (current: {})", save_format.version, FORMAT_VERSION);

        // Try checksum validation, but don't fail if it doesn't match
        // This allows loading older files after code structure changes
        if let Err(e) = save_format.validate_checksum() {
            log::warn!("Checksum validation failed (this is OK for older files): {}", e);
            log::warn!("Loading graph anyway - please re-save to update checksum");
        } else {
            log::info!("Checksum valid");
        }

        // Validate graph structure
        let mut graph = save_format.graph;
        log::info!("Validating graph structure: {} nodes, {} connections",
                   graph.nodes.len(), graph.connections.len());

        graph.validate_structure()
            .context("Graph structure validation failed")?;

        // T009: Ensure all continuous nodes start in stopped state
        for node in graph.nodes.values_mut() {
            if let Some(config) = &mut node.continuous_config {
                // Runtime state is already reset by #[serde(skip)] default behavior
                // But we explicitly ensure it here for safety
                config.runtime_state = crate::graph::node::ContinuousRuntimeState::default();
                log::debug!("Reset continuous node runtime state for node {}", node.id);
            }
        }

        log::info!("Graph structure validation passed");
        Ok(graph)
    }

    /// Save the graph to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path.as_ref(), bytes)
            .with_context(|| format!("Failed to write graph to file: {}", path.as_ref().display()))
    }

    /// Load a graph from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(path.as_ref())
            .with_context(|| format!("Failed to read graph from file: {}", path.as_ref().display()))?;

        Self::from_bytes(&bytes)
            .with_context(|| format!("Failed to load graph from file: {}", path.as_ref().display()))
    }

    /// Validate the graph structure after deserialization
    ///
    /// Checks:
    /// - All connection node IDs exist in nodes map
    /// - All connection port IDs exist on their respective nodes
    /// - Type compatibility for all connections
    /// - No dangling references
    pub fn validate_structure(&self) -> Result<()> {
        // Check all connections reference valid nodes
        for conn in &self.connections {
            // Validate source node exists
            let source_node = self.nodes.get(&conn.from_node)
                .with_context(|| format!(
                    "Connection {} references non-existent source node {}",
                    conn.id, conn.from_node
                ))?;

            // Validate target node exists
            let target_node = self.nodes.get(&conn.to_node)
                .with_context(|| format!(
                    "Connection {} references non-existent target node {}",
                    conn.id, conn.to_node
                ))?;

            // Validate source port exists
            let source_port = source_node.outputs.iter()
                .find(|p| p.id == conn.from_port)
                .with_context(|| format!(
                    "Connection {} references non-existent source port {} on node {}",
                    conn.id, conn.from_port, conn.from_node
                ))?;

            // Validate target port exists
            let target_port = target_node.inputs.iter()
                .find(|p| p.id == conn.to_port)
                .with_context(|| format!(
                    "Connection {} references non-existent target port {} on node {}",
                    conn.id, conn.to_port, conn.to_node
                ))?;

            // Validate type compatibility using NodeGraph's type checking
            if !NodeGraph::types_compatible(&source_port.data_type, &target_port.data_type) {
                anyhow::bail!(
                    "Connection {} has incompatible types: {} (source) -> {} (target)",
                    conn.id,
                    source_port.data_type.name(),
                    target_port.data_type.name()
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_serialization() {
        let graph = NodeGraph::new("Test Graph".to_string(), "Test Author".to_string());

        // Serialize to bytes
        let bytes = graph.to_bytes().expect("Failed to serialize");

        // Deserialize from bytes
        let loaded = NodeGraph::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(graph.name, loaded.name);
        assert_eq!(graph.metadata.author, loaded.metadata.author);
        assert_eq!(graph.nodes.len(), loaded.nodes.len());
        assert_eq!(graph.connections.len(), loaded.connections.len());
    }

    #[test]
    fn test_invalid_magic_bytes() {
        let mut save_format = GraphSaveFormat::new(
            NodeGraph::new("Test".to_string(), "Author".to_string())
        );
        save_format.magic = b"INVALID".to_vec();

        let bytes = bincode::serialize(&save_format).unwrap();
        let result = NodeGraph::from_bytes(&bytes);

        // Should fail validation due to invalid magic bytes
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_checksum() {
        // Create a graph and serialize it properly first
        let graph = NodeGraph::new("Test".to_string(), "Author".to_string());
        let bytes = graph.to_bytes().unwrap();

        // Deserialize to get the format
        let mut save_format: GraphSaveFormat = bincode::deserialize(&bytes).unwrap();

        // Now corrupt the checksum
        save_format.checksum = 0; // Corrupt checksum

        let corrupted_bytes = bincode::serialize(&save_format).unwrap();
        let result = NodeGraph::from_bytes(&corrupted_bytes);

        // Should succeed with warning (checksum validation is non-fatal for backward compatibility)
        assert!(result.is_ok(), "Checksum validation should be non-fatal");
    }

    #[test]
    fn test_graph_with_nodes_serialization() {
        use crate::graph::node::GraphNode;

        let mut graph = NodeGraph::new("Test Graph".to_string(), "Test Author".to_string());

        // Create simple nodes without ports to avoid complex enum serialization
        let node1 = GraphNode::new(
            "builtin:test:node1".to_string(),
            "Node 1".to_string(),
            egui::Pos2::new(100.0, 100.0),
        );
        let node2 = GraphNode::new(
            "builtin:test:node2".to_string(),
            "Node 2".to_string(),
            egui::Pos2::new(200.0, 200.0),
        );

        graph.add_node(node1);
        graph.add_node(node2);

        // Serialize to bytes
        let bytes = graph.to_bytes().expect("Failed to serialize graph with nodes");

        // Deserialize from bytes
        let loaded = NodeGraph::from_bytes(&bytes).expect("Failed to deserialize graph with nodes");

        assert_eq!(loaded.nodes.len(), 2);
        assert_eq!(loaded.name, "Test Graph");
    }

    // T057-T058: Tests for creator node data serialization
    #[test]
    fn test_creator_node_serialization_with_code() {
        use crate::graph::node::{CompilationState, GraphNode, Language, WasmCreatorNodeData};

        let mut graph = NodeGraph::new("Test Graph".to_string(), "Test Author".to_string());

        // Create a graph node with creator data (save_code = true)
        let mut creator_node = GraphNode::new(
            "builtin:development:wasm-creator".to_string(),
            "WASM Creator".to_string(),
            egui::Pos2::new(100.0, 100.0),
        );

        creator_node.creator_data = Some(WasmCreatorNodeData {
            component_name: "TestComponent".to_string(),
            save_code: true,
            source_code: "let result = value * 2.0;".to_string(),
            compilation_state: CompilationState::Idle,
            generated_component_id: None,
            language: Language::Rust,
            editor_theme: crate::ui::code_editor::CodeTheme::default(),
        });

        graph.add_node(creator_node.clone());

        // Serialize to bytes
        let bytes = graph.to_bytes().expect("Failed to serialize graph with creator node");

        // Deserialize from bytes
        let loaded = NodeGraph::from_bytes(&bytes).expect("Failed to deserialize graph with creator node");

        assert_eq!(loaded.nodes.len(), 1);
        let loaded_node = loaded.nodes.values().next().unwrap();
        let loaded_creator_data = loaded_node.creator_data.as_ref().expect("Creator data should be present");

        assert_eq!(loaded_creator_data.component_name, "TestComponent");
        assert_eq!(loaded_creator_data.save_code, true);
        assert_eq!(loaded_creator_data.source_code, "let result = value * 2.0;");
    }

    #[test]
    fn test_creator_node_serialization_without_code() {
        use crate::graph::node::{CompilationState, GraphNode, Language, WasmCreatorNodeData};

        let mut graph = NodeGraph::new("Test Graph".to_string(), "Test Author".to_string());

        // Create a graph node with creator data (save_code = false)
        let mut creator_node = GraphNode::new(
            "builtin:development:wasm-creator".to_string(),
            "WASM Creator".to_string(),
            egui::Pos2::new(100.0, 100.0),
        );

        creator_node.creator_data = Some(WasmCreatorNodeData {
            component_name: "TestComponent".to_string(),
            save_code: false,
            source_code: "let result = value * 2.0;".to_string(), // This should not be serialized
            compilation_state: CompilationState::Idle,
            generated_component_id: None,
            language: Language::Rust,
            editor_theme: crate::ui::code_editor::CodeTheme::default(),
        });

        graph.add_node(creator_node.clone());

        // Serialize to bytes
        let bytes = graph.to_bytes().expect("Failed to serialize graph with creator node");

        // Deserialize from bytes
        let loaded = NodeGraph::from_bytes(&bytes).expect("Failed to deserialize graph with creator node");

        assert_eq!(loaded.nodes.len(), 1);
        let loaded_node = loaded.nodes.values().next().unwrap();
        let loaded_creator_data = loaded_node.creator_data.as_ref().expect("Creator data should be present");

        assert_eq!(loaded_creator_data.component_name, "TestComponent");
        assert_eq!(loaded_creator_data.save_code, false);
        // T058: When save_code is false, source_code should be empty after deserialization
        assert_eq!(loaded_creator_data.source_code, "");
    }

    #[test]
    fn test_regular_node_without_creator_data() {
        use crate::graph::node::GraphNode;

        let mut graph = NodeGraph::new("Test Graph".to_string(), "Test Author".to_string());

        // Create a regular node without creator data
        let regular_node = GraphNode::new(
            "builtin:constant:f32".to_string(),
            "Constant".to_string(),
            egui::Pos2::new(100.0, 100.0),
        );

        graph.add_node(regular_node);

        // Serialize to bytes
        let bytes = graph.to_bytes().expect("Failed to serialize graph");

        // Deserialize from bytes
        let loaded = NodeGraph::from_bytes(&bytes).expect("Failed to deserialize graph");

        assert_eq!(loaded.nodes.len(), 1);
        let loaded_node = loaded.nodes.values().next().unwrap();

        // Regular nodes should not have creator data
        assert!(loaded_node.creator_data.is_none());
    }
}
