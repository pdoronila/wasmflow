//! Node editor canvas with egui-snarl integration
//!
//! This module implements the visual node editor using egui-snarl for
//! node rendering, connection management, and user interactions.

mod node_data;
mod footer;
mod selection;
mod viewer;

use crate::graph::graph::NodeGraph;
use crate::graph::node::{ComponentRegistry, GraphNode};
use crate::ui::selection::{CanvasMode, SelectionState}; // T008: Rectangle selection state
use egui::Color32;
use egui_snarl::ui::{NodeLayout, PinPlacement, SnarlStyle};
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use std::collections::HashMap;
use uuid::Uuid;

pub use node_data::{SnarlNodeData, SnarlPort};
use selection::SelectionHelper;
use viewer::CanvasViewer;

/// Canvas state for the node editor
pub struct NodeCanvas {
    /// egui-snarl state
    snarl: Snarl<SnarlNodeData>,
    /// Mapping from snarl node ID to UUID
    snarl_to_uuid: HashMap<NodeId, Uuid>,
    /// Mapping from UUID to snarl node ID
    uuid_to_snarl: HashMap<Uuid, NodeId>,
    /// Nodes pending deletion (to be handled by app with undo/redo)
    pub pending_deletions: Vec<Uuid>,
    /// T078: Node pending permission view (to be handled by app)
    pub pending_permission_view: Option<Uuid>,
    /// Continuous nodes pending start
    pub pending_continuous_start: Vec<Uuid>,
    /// Continuous nodes pending stop
    pub pending_continuous_stop: Vec<Uuid>,
    /// T038: Composite node pending drill-down (to be handled by app)
    pub pending_drill_down: Option<Uuid>,
    /// T085: Dirty flag to track if graph needs re-sync
    needs_sync: bool,
    /// T085: Cached graph node count for detecting changes
    cached_node_count: usize,
    /// T085: Cached connection count for detecting changes
    cached_connection_count: usize,
    /// T008: Rectangle selection state
    pub selection: SelectionState,
    /// Current canvas interaction mode
    pub canvas_mode: CanvasMode,
    /// Cached viewport scale for coordinate transformation
    viewport_scale: f32,
    /// Cached viewport offset for coordinate transformation
    viewport_offset: egui::Vec2,
}

impl NodeCanvas {
    /// Create a new canvas
    pub fn new() -> Self {
        Self {
            snarl: Snarl::new(),
            snarl_to_uuid: HashMap::new(),
            uuid_to_snarl: HashMap::new(),
            pending_deletions: Vec::new(),
            pending_permission_view: None,
            pending_continuous_start: Vec::new(),
            pending_continuous_stop: Vec::new(),
            pending_drill_down: None, // T038: No pending drill-down initially
            needs_sync: true, // T085: Initially dirty
            cached_node_count: 0,
            cached_connection_count: 0,
            selection: SelectionState::new(), // T008: Initialize selection state
            canvas_mode: CanvasMode::Normal, // Start in normal mode
            viewport_scale: 1.0, // Default scale
            viewport_offset: egui::Vec2::ZERO, // Default offset
        }
    }

    /// T085: Mark canvas as needing sync (call when graph changes externally)
    pub fn mark_dirty(&mut self) {
        self.needs_sync = true;
    }

    /// Synchronize canvas with graph data
    pub fn sync_with_graph(&mut self, graph: &NodeGraph) {
        // T085: Only sync if needed or if structural changes detected
        let node_count_changed = graph.nodes.len() != self.cached_node_count;
        let connection_count_changed = graph.connections.len() != self.cached_connection_count;

        if !self.needs_sync && !node_count_changed && !connection_count_changed {
            // No changes detected - skip expensive sync
            return;
        }

        log::debug!(
            "Syncing canvas with graph (nodes: {}, connections: {}, forced: {})",
            graph.nodes.len(),
            graph.connections.len(),
            self.needs_sync
        );

        // Clear existing state
        self.snarl = Snarl::new();
        self.snarl_to_uuid.clear();
        self.uuid_to_snarl.clear();

        // Add all nodes
        for (uuid, node) in &graph.nodes {
            let snarl_node = Self::create_snarl_node(node);
            let snarl_id = self.snarl.insert_node(node.position, snarl_node);

            self.snarl_to_uuid.insert(snarl_id, *uuid);
            self.uuid_to_snarl.insert(*uuid, snarl_id);
        }

        // Add all connections
        for conn in &graph.connections {
            if let (Some(&from_id), Some(&to_id)) = (
                self.uuid_to_snarl.get(&conn.from_node),
                self.uuid_to_snarl.get(&conn.to_node),
            ) {
                // Find port indices
                if let (Some(from_node), Some(to_node)) = (
                    graph.nodes.get(&conn.from_node),
                    graph.nodes.get(&conn.to_node),
                ) {
                    let from_port_idx = from_node
                        .outputs
                        .iter()
                        .position(|p| p.id == conn.from_port);
                    let to_port_idx = to_node.inputs.iter().position(|p| p.id == conn.to_port);

                    if let (Some(from_port), Some(to_port)) = (from_port_idx, to_port_idx) {
                        self.snarl.connect(
                            OutPinId {
                                node: from_id,
                                output: from_port,
                            },
                            InPinId {
                                node: to_id,
                                input: to_port,
                            },
                        );
                    }
                }
            }
        }

        // T085: Update cache and mark as clean
        self.cached_node_count = graph.nodes.len();
        self.cached_connection_count = graph.connections.len();
        self.needs_sync = false;
    }

    /// Create snarl node data from graph node
    fn create_snarl_node(node: &GraphNode) -> SnarlNodeData {
        let inputs: Vec<SnarlPort> = node
            .inputs
            .iter()
            .map(|p| SnarlPort {
                uuid: p.id,
                name: p.name.clone(),
                data_type: p.data_type.clone(),
                current_value: None,
            })
            .collect();

        let outputs: Vec<SnarlPort> = node
            .outputs
            .iter()
            .map(|p| SnarlPort {
                uuid: p.id,
                name: p.name.clone(),
                data_type: p.data_type.clone(),
                current_value: p.current_value.as_ref().map(|v| v.format_display()),
            })
            .collect();

        // T048: Build port mappings for composite nodes
        let (input_mappings, output_mappings) = if let Some(comp_data) = &node.composition_data {
            let input_map = comp_data.exposed_inputs.iter()
                .filter_map(|(ext_name, mapping)| {
                    comp_data.internal_nodes.get(&mapping.internal_node_id)
                        .map(|internal_node| {
                            (ext_name.clone(), (internal_node.display_name.clone(), mapping.internal_port_name.clone()))
                        })
                })
                .collect();

            let output_map = comp_data.exposed_outputs.iter()
                .filter_map(|(ext_name, mapping)| {
                    comp_data.internal_nodes.get(&mapping.internal_node_id)
                        .map(|internal_node| {
                            (ext_name.clone(), (internal_node.display_name.clone(), mapping.internal_port_name.clone()))
                        })
                })
                .collect();

            (input_map, output_map)
        } else {
            (std::collections::BTreeMap::new(), std::collections::BTreeMap::new())
        };

        SnarlNodeData {
            uuid: node.id,
            display_name: node.display_name.clone(),
            component_id: node.component_id.clone(),
            inputs,
            outputs,
            execution_state: node.execution_state,
            needs_component_refresh: node.needs_component_refresh, // T059
            // WASM Creator nodes start with a default width of 650px for comfortable code editing
            custom_width: if node.component_id == "builtin:development:wasm-creator" {
                Some(650.0)
            } else {
                None
            },
            is_composite: node.composition_data.is_some(), // T040: Flag composite nodes
            input_mappings,
            output_mappings,
        }
    }

    /// Render the canvas with navigation controls
    pub fn show(&mut self, ui: &mut egui::Ui, graph: &mut NodeGraph, registry: &ComponentRegistry) {
        // T085: Sync before rendering (only if graph changed - uses caching for performance)
        self.sync_with_graph(graph);

        // T013: Handle ESC key to cancel selection
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selection.cancel_drag();
            self.selection.clear_selection();
            // Clear selected state on all nodes
            for node in graph.nodes.values_mut() {
                node.selected = false;
            }
        }

        // Mode toggle and navigation help
        ui.horizontal(|ui| {
            // Mode toggle button
            let mode_text = match self.canvas_mode {
                CanvasMode::Normal => "🖱 Normal Mode",
                CanvasMode::Selection => "⬚ Selection Mode",
            };

            let mode_button = ui.button(mode_text);
            if mode_button.clicked() {
                // Toggle mode
                self.canvas_mode = match self.canvas_mode {
                    CanvasMode::Normal => CanvasMode::Selection,
                    CanvasMode::Selection => CanvasMode::Normal,
                };
                // Clear selection when switching modes
                self.selection.clear_selection();
                self.selection.cancel_drag();
                // Clear selected state on all nodes
                for node in graph.nodes.values_mut() {
                    node.selected = false;
                }
            }

            mode_button.on_hover_ui(|ui| {
                match self.canvas_mode {
                    CanvasMode::Normal => {
                        ui.label("Switch to Selection Mode");
                        ui.label("• Rectangle selection enabled");
                        ui.label("• Node dragging disabled");
                    }
                    CanvasMode::Selection => {
                        ui.label("Switch to Normal Mode");
                        ui.label("• Node dragging enabled");
                        ui.label("• Rectangle selection disabled");
                    }
                }
            });

            ui.separator();
            ui.label("💡 Navigation:");
            ui.separator();
            ui.label("Pan: Middle-mouse drag");
            ui.separator();
            ui.label("Zoom: Scroll wheel");
            ui.separator();

            match self.canvas_mode {
                CanvasMode::Normal => {
                    ui.label("Mode: Drag nodes, Connect ports");
                }
                CanvasMode::Selection => {
                    ui.label("Mode: Drag to select multiple nodes");
                }
            }
        });

        ui.separator();

        // Allocate remaining space for the snarl canvas
        // This ensures the snarl gets all available space and can handle scroll events properly
        let available_height = ui.available_height();

        ui.allocate_ui(egui::vec2(ui.available_width(), available_height), |ui| {
            // Swap scroll wheel behavior: plain scroll = zoom, CMD+scroll = pan
            // egui-snarl by default uses: plain scroll = pan, CMD+scroll = zoom
            // We intercept the input and swap the modifier state
            let mut input = ui.input_mut(|i| i.clone());

            // Check for scroll events and swap modifier behavior
            let has_scroll = input.smooth_scroll_delta != egui::Vec2::ZERO || input.raw_scroll_delta != egui::Vec2::ZERO;

            if has_scroll {
                let is_cmd_pressed = input.modifiers.command;

                // Swap the command modifier state
                // If CMD is pressed (user wants to pan), remove CMD so snarl pans
                // If CMD is not pressed (user wants to zoom), add CMD so snarl zooms
                input.modifiers.command = !is_cmd_pressed;

                // Apply the modified input back
                ui.ctx().clone().input_mut(|i| {
                    i.modifiers = input.modifiers;
                });
            }

            // Create viewer
            let mut viewer = CanvasViewer {
                graph,
                registry,
                pending_deletions: &mut self.pending_deletions,
                pending_permission_view: &mut self.pending_permission_view,
                pending_continuous_start: &mut self.pending_continuous_start,
                pending_continuous_stop: &mut self.pending_continuous_stop,
                pending_drill_down: &mut self.pending_drill_down, // T040
                snarl_to_uuid: &self.snarl_to_uuid,
            };

            // Show snarl with navigation support
            // T090: egui-snarl provides built-in pan (middle-mouse) and zoom (scroll wheel)
            // Use custom style with edge-placed pins and explicit scale range
            let style = SnarlStyle {
                pin_placement: Some(PinPlacement::Edge),
                node_layout: Some(NodeLayout::coil()),
                min_scale: Some(0.1),  // Allow zooming out to 10%
                max_scale: Some(5.0),  // Allow zooming in to 500%
                ..Default::default()
            };

            // Disable snarl interactions when in Selection mode
            // This prevents nodes from being dragged during rectangle selection
            let is_interactive = self.canvas_mode == CanvasMode::Normal;

            ui.add_enabled_ui(is_interactive, |ui| {
                self.snarl.show(&mut viewer, &style, "canvas", ui);
            });

            //T016-T017: Selection highlighting
            // Note: We skip preview highlighting during drag to avoid coordinate transform complexity
            // Only show final selection highlighting after drag completes
            // This is simpler and clearer for users - the selection rectangle shows what will be selected
        });

        // T015: Rectangle selection rendering overlay
        // Add a layer on top for selection rectangle
        let selection_layer_id = ui.id().with("selection_layer");
        egui::Area::new(selection_layer_id)
            .fixed_pos(egui::pos2(0.0, 0.0))
            .interactable(false)
            .show(ui.ctx(), |ui| {
                // Render selection rectangle if dragging
                if let Some(rect) = self.selection.get_selection_rect() {
                    let painter = ui.ctx().debug_painter();

                    // Draw filled rectangle
                    painter.add(egui::Shape::rect_filled(
                        rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(100, 150, 200, 50)
                    ));

                    // Draw border
                    painter.add(egui::Shape::rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(1.5, Color32::from_rgb(100, 150, 200)),
                        egui::epaint::StrokeKind::Middle
                    ));
                }
            });

        // T009-T011: Handle mouse events for selection
        // Only process selection in Selection mode
        if self.canvas_mode == CanvasMode::Selection {
            ui.ctx().input(|i| {
                if let Some(pos) = i.pointer.interact_pos() {
                    // T009: Detect if primary mouse was pressed - start selection
                    if i.pointer.primary_pressed() {
                        self.selection.start_drag(pos);
                    }

                    // T010: Update selection during drag
                    if i.pointer.primary_down() && self.selection.is_dragging() {
                        self.selection.update_drag(pos);
                    }

                    // T011: End drag on release
                    if i.pointer.primary_released() {
                        if self.selection.is_dragging() {
                            if let Some(rect) = self.selection.get_selection_rect() {
                                let selected_nodes = self.find_nodes_in_rect(rect, graph);

                                // Clear all previous selections
                                for node in graph.nodes.values_mut() {
                                    node.selected = false;
                                }

                                // Mark selected nodes
                                for node_id in &selected_nodes {
                                    if let Some(node) = graph.nodes.get_mut(node_id) {
                                        node.selected = true;
                                    }
                                }

                                self.selection.end_drag(selected_nodes);
                            } else {
                                self.selection.end_drag(std::collections::HashSet::new());
                            }
                        }
                    }
                }
            });
        }

        // Sync changes back to graph (positions, connections)
        self.sync_to_graph(graph);
    }

    /// Sync changes from snarl back to graph (positions, connections)
    fn sync_to_graph(&mut self, graph: &mut NodeGraph) {
        // Update node positions
        for (snarl_id, uuid) in &self.snarl_to_uuid {
            if let Some(node_info) = self.snarl.get_node_info(*snarl_id) {
                if let Some(graph_node) = graph.nodes.get_mut(uuid) {
                    graph_node.position = node_info.pos;
                }
            }
        }

        // Sync connections from snarl to graph
        // Collect all connections that should exist according to snarl
        let mut snarl_connections = std::collections::HashSet::new();

        for (out_pin_id, in_pin_id) in self.snarl.wires() {
            // Map snarl IDs back to UUIDs
            if let (Some(&from_uuid), Some(&to_uuid)) = (
                self.snarl_to_uuid.get(&out_pin_id.node),
                self.snarl_to_uuid.get(&in_pin_id.node),
            ) {
                // Get the actual port UUIDs
                if let (Some(from_node), Some(to_node)) =
                    (graph.nodes.get(&from_uuid), graph.nodes.get(&to_uuid))
                {
                    if let (Some(from_port), Some(to_port)) = (
                        from_node.outputs.get(out_pin_id.output),
                        to_node.inputs.get(in_pin_id.input),
                    ) {
                        snarl_connections.insert((from_uuid, from_port.id, to_uuid, to_port.id));
                    }
                }
            }
        }

        // Build set of existing graph connections for comparison
        let mut graph_connections = std::collections::HashSet::new();
        for conn in &graph.connections {
            graph_connections.insert((conn.from_node, conn.from_port, conn.to_node, conn.to_port));
        }

        // Add connections that exist in snarl but not in graph
        for (from_node, from_port, to_node, to_port) in &snarl_connections {
            if !graph_connections.contains(&(*from_node, *from_port, *to_node, *to_port)) {
                // Try to add connection to graph
                let _ = graph.add_connection(*from_node, *from_port, *to_node, *to_port);
            }
        }

        // Remove connections that exist in graph but not in snarl
        let connections_to_remove: Vec<_> = graph
            .connections
            .iter()
            .filter(|conn| {
                !snarl_connections.contains(&(
                    conn.from_node,
                    conn.from_port,
                    conn.to_node,
                    conn.to_port,
                ))
            })
            .map(|conn| conn.id)
            .collect();

        for conn_id in connections_to_remove {
            let _ = graph.remove_connection(conn_id);
        }
    }

    /// Get snarl reference
    #[allow(dead_code)]
    pub fn snarl(&self) -> &Snarl<SnarlNodeData> {
        &self.snarl
    }

    /// Get mutable snarl reference
    #[allow(dead_code)]
    pub fn snarl_mut(&mut self) -> &mut Snarl<SnarlNodeData> {
        &mut self.snarl
    }

    /// T012: Find nodes whose centers fall within a given rectangle
    ///
    /// This is used for rectangle selection to determine which nodes
    /// should be selected when the user drags a selection box.
    ///
    /// Note: This assumes no viewport transform (scale=1, offset=0).
    /// Works best when canvas hasn't been panned/zoomed before entering Selection mode.
    pub fn find_nodes_in_rect(&self, rect: egui::Rect, graph: &NodeGraph) -> std::collections::HashSet<Uuid> {
        SelectionHelper::find_nodes_in_rect(
            rect,
            graph,
            &self.snarl,
            &self.uuid_to_snarl,
            self.viewport_scale,
            self.viewport_offset,
        )
    }
}

impl Default for NodeCanvas {
    fn default() -> Self {
        Self::new()
    }
}
