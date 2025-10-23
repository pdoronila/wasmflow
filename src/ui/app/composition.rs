//! Composition workflow and drill-down navigation
//!
//! This module handles WebAssembly component composition, boundary port aggregation,
//! and drill-down navigation into composite nodes.

use super::WasmFlowApp;
use uuid::Uuid;

impl WasmFlowApp {
    /// T030: Handle composition action - compose selected nodes into a single composite node
    ///
    /// This is the core composition workflow:
    /// 1. Validate selection (≥2 nodes, all connected)
    /// 2. Extract selected subgraph
    /// 3. Collect component paths
    /// 4. Use ComponentComposer to generate composed binary
    /// 5. Create composite node with internal structure
    /// 6. Replace selected nodes with composite node
    pub(super) fn handle_compose_action(&mut self) {
        log::info!("Starting composition workflow");

        // T033: Get selected nodes
        let selected_nodes: Vec<Uuid> = self
            .graph
            .nodes
            .iter()
            .filter(|(_, node)| node.selected)
            .map(|(id, _)| *id)
            .collect();

        // T033: Validate selection (need at least 2 nodes)
        if selected_nodes.len() < 2 {
            self.composition_error = Some("Please select at least 2 nodes to compose".to_string());
            log::warn!(
                "Composition failed: {} nodes selected (need ≥2)",
                selected_nodes.len()
            );
            return;
        }

        log::debug!("Selected {} nodes for composition", selected_nodes.len());

        // T033: Validate connectivity using validation module
        match crate::graph::validation::is_connected_subgraph(&self.graph, &selected_nodes) {
            Ok(true) => {
                log::debug!("Selected nodes form a connected subgraph");
            }
            Ok(false) => {
                self.composition_error = Some(
                    "Selected nodes must form a connected graph. Ensure all nodes are connected to each other.".to_string()
                );
                log::warn!("Composition failed: selected nodes are not connected");
                return;
            }
            Err(e) => {
                self.composition_error = Some(format!("Validation error: {}", e));
                log::error!("Composition validation failed: {}", e);
                return;
            }
        }

        // T029: Collect component paths for all selected nodes
        let mut component_paths: Vec<std::path::PathBuf> = Vec::new();
        let mut component_names: Vec<String> = Vec::new();

        for node_id in &selected_nodes {
            if let Some(node) = self.graph.nodes.get(node_id) {
                if let Some(path) = self.get_component_path(node) {
                    component_paths.push(path);
                    component_names.push(node.display_name.clone());
                } else {
                    // T033: Cannot compose builtin nodes (no WASM path)
                    self.composition_error = Some(format!(
                        "Cannot compose builtin node '{}'. Only user-defined WASM components can be composed.",
                        node.display_name
                    ));
                    log::warn!(
                        "Composition failed: builtin node '{}' in selection",
                        node.display_name
                    );
                    return;
                }
            }
        }

        // T033: Need at least 2 component paths
        if component_paths.len() < 2 {
            self.composition_error = Some("Need at least 2 WASM components to compose".to_string());
            return;
        }

        log::debug!("Collected {} component paths", component_paths.len());

        // T022-T025: Perform composition using ComponentComposer
        // Socket is the first component, plugs are the rest
        let socket = &component_paths[0];
        let plugs: Vec<&std::path::Path> =
            component_paths[1..].iter().map(|p| p.as_path()).collect();

        log::info!(
            "Composing: socket={}, plugs={}",
            socket.display(),
            plugs.len()
        );

        let composed_binary = match self.composer.compose(socket, &plugs) {
            Ok(bytes) => {
                log::info!("Composition successful: {} bytes", bytes.len());
                bytes
            }
            Err(e) => {
                // T032: Show composition error dialog
                self.composition_error = Some(format!("Composition failed: {}", e));
                log::error!("Composition failed: {}", e);
                return;
            }
        };

        // T027: Create CompositionData with internal structure
        let internal_nodes: std::collections::BTreeMap<Uuid, crate::graph::node::GraphNode> =
            selected_nodes
                .iter()
                .filter_map(|id| self.graph.nodes.get(id).map(|n| (*id, n.clone())))
                .collect();

        let internal_edges: Vec<crate::graph::connection::Connection> = self
            .graph
            .connections
            .iter()
            .filter(|conn| {
                selected_nodes.contains(&conn.from_node) && selected_nodes.contains(&conn.to_node)
            })
            .cloned()
            .collect();

        // T035: Aggregate boundary ports for the composite node
        let (composite_inputs, composite_outputs, input_mappings, output_mappings) =
            self.aggregate_boundary_ports(&selected_nodes);

        let mut composition_data = crate::graph::node::CompositionData::new(
            "CompositeNode".to_string(),
            socket.clone(),
            component_paths[1..].to_vec(),
            internal_nodes,
            internal_edges,
            component_names,
            composed_binary.clone(),
        );

        // Update composition data with port mappings
        composition_data.exposed_inputs = input_mappings;
        composition_data.exposed_outputs = output_mappings;

        // Calculate center position of selected nodes
        let center_pos = if !selected_nodes.is_empty() {
            let sum_x: f32 = selected_nodes
                .iter()
                .filter_map(|id| self.graph.nodes.get(id))
                .map(|n| n.position.x)
                .sum();
            let sum_y: f32 = selected_nodes
                .iter()
                .filter_map(|id| self.graph.nodes.get(id))
                .map(|n| n.position.y)
                .sum();
            let count = selected_nodes.len() as f32;
            egui::pos2(sum_x / count, sum_y / count)
        } else {
            egui::pos2(200.0, 200.0)
        };

        // T034: Create new composite node
        let mut composite_node = crate::graph::node::GraphNode::new(
            "composite:generated".to_string(),
            "Composite Node".to_string(),
            center_pos,
        );

        // Assign boundary ports to the composite node
        composite_node.inputs = composite_inputs;
        composite_node.outputs = composite_outputs;
        composite_node.composition_data = Some(composition_data);

        // Add the composite node to the graph
        let composite_id = composite_node.id;
        let _ = self.graph.add_node(composite_node);

        // Remove selected nodes
        for node_id in &selected_nodes {
            let _ = self.graph.remove_node(*node_id);
        }

        // Clear selection
        self.canvas.selection.clear_selection();

        // Mark dirty and update status
        self.dirty = true;
        self.status_message = format!(
            "Composed {} nodes into composite node",
            selected_nodes.len()
        );
        log::info!(
            "Composition complete: created composite node {}",
            composite_id
        );

        // Sync canvas
        self.canvas.mark_dirty();
    }

    /// Aggregate boundary ports from selected nodes for composite node
    ///
    /// Returns: (input_ports, output_ports, input_mappings, output_mappings)
    fn aggregate_boundary_ports(
        &self,
        selected_nodes: &[Uuid],
    ) -> (
        Vec<crate::graph::node::Port>,
        Vec<crate::graph::node::Port>,
        std::collections::BTreeMap<String, crate::graph::node::PortMapping>,
        std::collections::BTreeMap<String, crate::graph::node::PortMapping>,
    ) {
        use crate::graph::node::PortMapping;
        use std::collections::{BTreeMap, HashSet};

        let selected_set: HashSet<Uuid> = selected_nodes.iter().copied().collect();
        let mut composite_inputs = Vec::new();
        let mut composite_outputs = Vec::new();
        let mut input_mappings = BTreeMap::new();
        let mut output_mappings = BTreeMap::new();

        // For each selected node, check its ports for boundary conditions
        for node_id in selected_nodes {
            let Some(node) = self.graph.nodes.get(node_id) else {
                continue;
            };

            // Check input ports - boundary if they have connections from outside selection
            for input_port in &node.inputs {
                // Find connections TO this input port
                let has_external_input = self.graph.connections.iter().any(|conn| {
                    conn.to_node == *node_id
                        && conn.to_port == input_port.id
                        && !selected_set.contains(&conn.from_node)
                });

                if has_external_input {
                    // This is a boundary input - expose it on composite node
                    let external_name = format!("{}.{}", node.display_name, input_port.name);
                    log::debug!("Boundary input found: {}", external_name);

                    // Create port for composite node
                    let mut composite_port = input_port.clone();
                    composite_port.id = Uuid::new_v4(); // New ID for composite port
                    composite_port.name = external_name.clone();
                    composite_inputs.push(composite_port);

                    // Create port mapping
                    let mapping = PortMapping {
                        external_name: external_name.clone(),
                        internal_node_id: *node_id,
                        internal_port_name: input_port.name.clone(),
                        port_type: input_port.data_type.clone(),
                    };
                    input_mappings.insert(external_name, mapping);
                }
            }

            // Check output ports - boundary if they have NO connections or connections outside selection
            for output_port in &node.outputs {
                // Find connections FROM this output port
                let connections_from_port: Vec<_> = self
                    .graph
                    .connections
                    .iter()
                    .filter(|conn| conn.from_node == *node_id && conn.from_port == output_port.id)
                    .collect();

                // Boundary if: no connections OR any connection goes outside selection
                let is_boundary = connections_from_port.is_empty()
                    || connections_from_port
                        .iter()
                        .any(|conn| !selected_set.contains(&conn.to_node));

                if is_boundary {
                    // This is a boundary output - expose it on composite node
                    let external_name = format!("{}.{}", node.display_name, output_port.name);
                    log::debug!("Boundary output found: {}", external_name);

                    // Create port for composite node
                    let mut composite_port = output_port.clone();
                    composite_port.id = Uuid::new_v4(); // New ID for composite port
                    composite_port.name = external_name.clone();
                    composite_outputs.push(composite_port);

                    // Create port mapping
                    let mapping = PortMapping {
                        external_name: external_name.clone(),
                        internal_node_id: *node_id,
                        internal_port_name: output_port.name.clone(),
                        port_type: output_port.data_type.clone(),
                    };
                    output_mappings.insert(external_name, mapping);
                }
            }
        }

        log::info!(
            "Aggregated {} boundary inputs, {} boundary outputs",
            composite_inputs.len(),
            composite_outputs.len()
        );

        (
            composite_inputs,
            composite_outputs,
            input_mappings,
            output_mappings,
        )
    }

    /// T038: Handle drill-down into a composite node
    ///
    /// Enters drill-down mode to view the internal structure of a composite node
    pub(super) fn handle_drill_down(&mut self, composite_node_id: Uuid) {
        log::info!("Attempting to drill down into node {}", composite_node_id);

        // Get the composite node
        let node = match self.graph.nodes.get(&composite_node_id) {
            Some(n) => n,
            None => {
                log::warn!("Node {} not found", composite_node_id);
                return;
            }
        };

        // Check if node has composition data
        let composition_data = match &node.composition_data {
            Some(data) => data,
            None => {
                log::warn!("Node {} is not a composite node", composite_node_id);
                return;
            }
        };

        // Drill down using the view stack
        if let Err(e) = self.view_stack.drill_down(
            composite_node_id,
            node.display_name.clone(),
            composition_data,
        ) {
            log::error!("Failed to drill down: {}", e);
            self.error_message = Some(format!("Cannot drill down: {}", e));
            return;
        }

        // Clear any error messages
        self.error_message = None;
        self.status_message = format!("Viewing internal structure of '{}'", node.display_name);

        // Mark canvas as needing sync
        self.canvas.mark_dirty();
    }

    /// T041: Navigate back from drill-down view
    pub(super) fn handle_back_navigation(&mut self) {
        if self.view_stack.go_back() {
            self.status_message = "Returned to parent view".to_string();
            self.canvas.mark_dirty();
        }
    }

    /// T029: Get the component path for a given node
    ///
    /// For user-defined components, returns the path to the .wasm file.
    /// For composite components, returns the socket component path.
    /// For builtin components, returns None.
    fn get_component_path(
        &self,
        node: &crate::graph::node::GraphNode,
    ) -> Option<std::path::PathBuf> {
        // Look up the component spec
        let spec = self.registry.get_by_id(&node.component_id)?;

        match &spec.component_type {
            crate::graph::node::ComponentType::Builtin => None,
            crate::graph::node::ComponentType::UserDefined(path) => Some(path.clone()),
            crate::graph::node::ComponentType::Composed { socket_path, .. } => {
                Some(socket_path.clone())
            }
        }
    }

    /// T032: Show composition error dialog
    pub(super) fn show_composition_error_dialog(&mut self, ctx: &egui::Context) {
        // Clone error message to avoid borrow checker issues
        let error_msg = self.composition_error.clone();

        if let Some(error_text) = error_msg {
            let mut should_close = false;

            egui::Window::new("Composition Error")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(8.0);

                        // Error icon and title
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("❌").size(24.0));
                            ui.label(
                                egui::RichText::new("Failed to compose nodes")
                                    .strong()
                                    .size(16.0),
                            );
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Error message in scrollable area
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 100, 100),
                                    &error_text,
                                );
                            });

                        ui.add_space(12.0);

                        // Help text
                        ui.label(egui::RichText::new("Requirements for composition:").strong());
                        ui.label("• Select at least 2 nodes");
                        ui.label("• All nodes must be connected");
                        ui.label("• Only user-defined WASM components can be composed");

                        ui.add_space(12.0);

                        // Close button
                        if ui.button("Close").clicked() {
                            should_close = true;
                        }
                    });
                });

            if should_close {
                self.composition_error = None;
            }
        }
    }
}
