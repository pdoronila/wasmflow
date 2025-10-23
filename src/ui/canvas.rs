//! Node editor canvas with egui-snarl integration
//!
//! This module implements the visual node editor using egui-snarl for
//! node rendering, connection management, and user interactions.

use crate::graph::graph::NodeGraph;
use crate::graph::node::{ComponentRegistry, DataType, ExecutionState, GraphNode};
use crate::ui::execution_status; // T033: Visual indicators for continuous execution states
use crate::ui::selection::{CanvasMode, SelectionState}; // T008: Rectangle selection state
use egui::{Color32, Pos2};
use egui_snarl::ui::{NodeLayout, PinInfo, PinPlacement, SnarlStyle, SnarlViewer};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use std::collections::HashMap;
use uuid::Uuid;

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

/// Data stored in snarl nodes
#[derive(Clone)]
pub struct SnarlNodeData {
    pub uuid: Uuid,
    pub display_name: String,
    pub component_id: String,
    pub inputs: Vec<SnarlPort>,
    pub outputs: Vec<SnarlPort>,
    #[allow(dead_code)]
    pub execution_state: ExecutionState,
    /// T059: Flag indicating component was updated
    pub needs_component_refresh: bool,
    /// Custom width for resizable nodes (e.g., WASM Creator)
    pub custom_width: Option<f32>,
    /// T040: Flag indicating this is a composite node (has internal structure)
    pub is_composite: bool,
    /// T048: Port mapping info for composite nodes (external_port_name -> (internal_node_name, internal_port_name))
    pub input_mappings: std::collections::BTreeMap<String, (String, String)>,
    pub output_mappings: std::collections::BTreeMap<String, (String, String)>,
}

/// Port data for snarl rendering
#[derive(Clone)]
pub struct SnarlPort {
    #[allow(dead_code)]
    pub uuid: Uuid,
    pub name: String,
    pub data_type: DataType,
    #[allow(dead_code)]
    pub current_value: Option<String>, // Formatted value for display
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
        let mut selected = std::collections::HashSet::new();

        // Use stored viewport transformation
        let scale = self.viewport_scale;
        let offset = self.viewport_offset;

        for (uuid, graph_node) in &graph.nodes {
            // Get node position from snarl if available (it may have been moved)
            let graph_pos = if let Some(&snarl_id) = self.uuid_to_snarl.get(uuid) {
                if let Some(node_info) = self.snarl.get_node_info(snarl_id) {
                    node_info.pos
                } else {
                    graph_node.position
                }
            } else {
                graph_node.position
            };

            // Transform to screen space using stored viewport
            let screen_pos = egui::pos2(
                graph_pos.x * scale + offset.x,
                graph_pos.y * scale + offset.y
            );

            // Calculate node center in screen space
            let screen_size = egui::vec2(200.0 * scale, 100.0 * scale);
            let node_center = screen_pos + screen_size / 2.0;

            if rect.contains(node_center) {
                selected.insert(*uuid);
            }
        }

        selected
    }
}

impl Default for NodeCanvas {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to render default footer view with FooterHead and FooterBody sections
struct DefaultFooterView;

impl DefaultFooterView {
    /// Render footer with two sections:
    /// - FooterHead: Input editors for unconnected inputs
    /// - FooterBody: Output values and execution status
    fn render_for_node(
        ui: &mut egui::Ui,
        node: &mut GraphNode,
        node_id: NodeId,
        snarl: &Snarl<SnarlNodeData>,
    ) -> Result<(), String> {
        // FOOTER HEAD: Input editors for unconnected inputs
        Self::render_footer_head(ui, node, node_id, snarl)?;

        // Visual separator between sections
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // FOOTER BODY: Output values and status
        Self::render_footer_body(ui, node)?;

        Ok(())
    }

    /// Render FooterHead: Input editors for unconnected inputs
    fn render_footer_head(
        ui: &mut egui::Ui,
        node: &mut GraphNode,
        node_id: NodeId,
        snarl: &Snarl<SnarlNodeData>,
    ) -> Result<(), String> {
        // Get total input count before borrowing mutably
        let total_inputs = node.inputs.len();

        // Check if there are any unconnected inputs
        let unconnected_inputs: Vec<(usize, &mut crate::graph::node::Port)> = node
            .inputs
            .iter_mut()
            .enumerate()
            .filter(|(idx, _)| {
                !snarl
                    .wires()
                    .any(|(_, in_pin)| in_pin.node == node_id && in_pin.input == *idx)
            })
            .collect();

        if unconnected_inputs.is_empty() {
            return Ok(());
        }

        // Use vertical layout for full-width input editors
        ui.vertical(|ui| {
            let max_fields = 20;
            for (field_count, (_idx, input_port)) in unconnected_inputs.into_iter().enumerate() {
                if field_count >= max_fields {
                    ui.label("...");
                    ui.label(format!("({} more)", total_inputs - max_fields));
                    break;
                }

                // Input label
                ui.label(
                    egui::RichText::new(format!("{}:", input_port.name))
                        .color(egui::Color32::from_rgb(180, 180, 180))
                );

                // Match on data type to emit appropriate widget
                match &input_port.data_type {
                        DataType::U32 => {
                            // Initialize if needed
                            if input_port.current_value.is_none() {
                                input_port.current_value = Some(crate::graph::node::NodeValue::U32(0));
                            }

                            if let Some(crate::graph::node::NodeValue::U32(ref mut v)) =
                                input_port.current_value
                            {
                                if ui.add(egui::DragValue::new(v).speed(1.0)).changed() {
                                    node.dirty = true;
                                }
                            }
                        }
                        DataType::I32 => {
                            // Initialize if needed
                            if input_port.current_value.is_none() {
                                input_port.current_value = Some(crate::graph::node::NodeValue::I32(0));
                            }

                            if let Some(crate::graph::node::NodeValue::I32(ref mut v)) =
                                input_port.current_value
                            {
                                if ui.add(egui::DragValue::new(v).speed(1.0)).changed() {
                                    node.dirty = true;
                                }
                            }
                        }
                        DataType::F32 => {
                            // Initialize if needed
                            if input_port.current_value.is_none() {
                                input_port.current_value =
                                    Some(crate::graph::node::NodeValue::F32(0.0));
                            }

                            if let Some(crate::graph::node::NodeValue::F32(ref mut v)) =
                                input_port.current_value
                            {
                                if ui.add(egui::DragValue::new(v).speed(0.1)).changed() {
                                    node.dirty = true;
                                }
                            }
                        }
                        DataType::String => {
                            // Initialize if needed
                            if input_port.current_value.is_none() {
                                input_port.current_value =
                                    Some(crate::graph::node::NodeValue::String(String::new()));
                            }

                            if let Some(crate::graph::node::NodeValue::String(ref mut s)) =
                                input_port.current_value
                            {
                                if ui
                                    .add(egui::TextEdit::singleline(s))
                                    .changed()
                                {
                                    node.dirty = true;
                                }
                            }
                        }
                        DataType::List(_) | DataType::Record(_) | DataType::Binary => {
                            ui.label("(complex - needs custom view)");
                        }
                        DataType::Any => {
                            ui.label("(any - no editor)");
                        }
                    }

                ui.add_space(4.0);
            }
        });

        Ok(())
    }

    /// Render FooterBody: Output values and execution status
    fn render_footer_body(ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
        // Handle execution state awareness
        match node.execution_state {
            ExecutionState::Idle => {
                ui.label("⏸ Awaiting execution");
                return Ok(());
            }
            ExecutionState::Running => {
                // Show spinner if execution is taking longer than 500ms
                if let Some(started_at) = node.execution_started_at {
                    let elapsed = started_at.elapsed();
                    if elapsed.as_millis() > 500 {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(format!("Computing... ({:.1}s)", elapsed.as_secs_f32()));
                        });
                        return Ok(());
                    }
                }
                ui.label("⏳ Computing...");
                return Ok(());
            }
            ExecutionState::Failed => {
                ui.colored_label(egui::Color32::RED, "❌ Execution failed");
                return Ok(());
            }
            ExecutionState::Completed => {
                // Continue to show output values below
            }
        }

        // Check if any outputs have values
        let has_values = node.outputs.iter().any(|p| p.current_value.is_some());
        if !has_values {
            ui.label("(no output values yet)");
            return Ok(());
        }

        // Use vertical layout for full-width output display
        ui.vertical(|ui| {
            let max_outputs = 10;
            for (_idx, output_port) in node.outputs.iter().take(max_outputs).enumerate() {
                if let Some(value) = &output_port.current_value {
                    // Output name
                    ui.label(
                        egui::RichText::new(format!("{}:", output_port.name))
                            .color(egui::Color32::from_rgb(180, 180, 180))
                    );

                    // Output value with wrapping (use full available width)
                    let value_text = value.format_display();
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 0.0),
                        egui::Label::new(
                            egui::RichText::new(value_text)
                                .color(egui::Color32::from_rgb(100, 200, 255))
                        ).wrap()
                    );

                    ui.add_space(4.0);
                }
            }

            if node.outputs.len() > max_outputs {
                ui.label("...");
                ui.label(format!("({} more)", node.outputs.len() - max_outputs));
            }
        });

        Ok(())
    }
}

/// Viewer implementation for egui-snarl
struct CanvasViewer<'a> {
    graph: &'a mut NodeGraph,
    registry: &'a ComponentRegistry,
    pending_deletions: &'a mut Vec<Uuid>,
    pending_permission_view: &'a mut Option<Uuid>,
    pending_continuous_start: &'a mut Vec<Uuid>,
    pending_continuous_stop: &'a mut Vec<Uuid>,
    pending_drill_down: &'a mut Option<Uuid>, // T040: Drill-down requests
    snarl_to_uuid: &'a HashMap<NodeId, Uuid>,
}

impl<'a> SnarlViewer<SnarlNodeData> for CanvasViewer<'a> {
    fn title(&mut self, node: &SnarlNodeData) -> String {
        node.display_name.clone()
    }

    fn show_header(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<SnarlNodeData>,
    ) {
        if let Some(node_data) = snarl.get_node(node) {
            // Get the actual node from the graph to check completion timestamp, continuous state, and selection
            let node_uuid = node_data.uuid;
            let (recently_completed, continuous_state, is_continuous_enabled, is_selected) =
                if let Some(graph_node) = self.graph.nodes.get(&node_uuid) {
                    let recently_completed = if let Some(completed_at) = graph_node.execution_completed_at {
                        completed_at.elapsed().as_millis() < 500
                    } else {
                        false
                    };

                    let (continuous_state, is_enabled) = if let Some(config) = &graph_node.continuous_config {
                        (Some(config.runtime_state.execution_state), config.enabled)
                    } else {
                        (None, false)
                    };

                    (recently_completed, continuous_state, is_enabled, graph_node.selected)
                } else {
                    (false, None, false, false)
                };

            let is_running = matches!(node_data.execution_state, ExecutionState::Running);

            // Check continuous execution state
            let is_continuous_running = matches!(
                continuous_state,
                Some(crate::graph::node::ContinuousExecutionState::Running)
            );
            let is_continuous_starting = matches!(
                continuous_state,
                Some(crate::graph::node::ContinuousExecutionState::Starting)
            );
            let is_continuous_stopping = matches!(
                continuous_state,
                Some(crate::graph::node::ContinuousExecutionState::Stopping)
            );
            let can_start_continuous = is_continuous_enabled && matches!(
                continuous_state,
                Some(crate::graph::node::ContinuousExecutionState::Idle) |
                Some(crate::graph::node::ContinuousExecutionState::Stopped) |
                Some(crate::graph::node::ContinuousExecutionState::Error)
            );
            let can_stop_continuous = is_continuous_enabled && (is_continuous_running || is_continuous_starting);

            // T033-T037: Enhanced state visualization for continuous nodes
            if let Some(state) = continuous_state {
                // Get state color and display info
                let color = execution_status::state_color(&state);
                let state_name = execution_status::state_display_name(&state);

                // Get icon based on state
                let icon = match state {
                    crate::graph::node::ContinuousExecutionState::Idle => "",
                    crate::graph::node::ContinuousExecutionState::Starting => "▶",
                    crate::graph::node::ContinuousExecutionState::Running => "⏵",
                    crate::graph::node::ContinuousExecutionState::Stopping => "⏸",
                    crate::graph::node::ContinuousExecutionState::Stopped => "⏹",
                    crate::graph::node::ContinuousExecutionState::Error => "❌",
                };

                ui.horizontal(|ui| {
                    // T019: Show selection indicator for selected nodes
                    if is_selected {
                        ui.label(egui::RichText::new("✓").color(egui::Color32::from_rgb(100, 200, 255)).strong());
                    }

                    // State icon with color
                    // T034: Apply pulsing animation for running state
                    if !icon.is_empty() {
                        let display_color = if matches!(state, crate::graph::node::ContinuousExecutionState::Running) {
                            // Get pulsing alpha based on when node started
                            if let Some(graph_node) = self.graph.nodes.get(&node_uuid) {
                                if let Some(config) = &graph_node.continuous_config {
                                    if let Some(started_at) = config.runtime_state.started_at {
                                        let pulse_speed = 2.0;
                                        let alpha = execution_status::pulsing_alpha(Some(started_at), pulse_speed);
                                        let [r, g, b, _] = color.to_array();
                                        egui::Color32::from_rgba_premultiplied(
                                            (r as f32 * alpha) as u8,
                                            (g as f32 * alpha) as u8,
                                            (b as f32 * alpha) as u8,
                                            255,
                                        )
                                    } else {
                                        color
                                    }
                                } else {
                                    color
                                }
                            } else {
                                color
                            }
                        } else {
                            color
                        };
                        ui.label(egui::RichText::new(icon).color(display_color));
                    }

                    ui.label(&node_data.display_name);

                    // T035: Show iteration counter for running nodes
                    if matches!(state, crate::graph::node::ContinuousExecutionState::Running) {
                        if let Some(graph_node) = self.graph.nodes.get(&node_uuid) {
                            if let Some(config) = &graph_node.continuous_config {
                                let iterations = config.runtime_state.iterations;
                                ui.label(
                                    egui::RichText::new(format!("#{}", iterations))
                                        .color(egui::Color32::from_rgb(150, 150, 150))
                                        .size(10.0)
                                );
                            }
                        }
                    }

                    // T059: Show warning icon if component was updated
                    if node_data.needs_component_refresh {
                        ui.label(egui::RichText::new("⚠").color(egui::Color32::from_rgb(255, 180, 0)))
                            .on_hover_text("Component updated - node may need refresh");
                    }

                    // Right-aligned buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button (X)
                        if ui.small_button("✖").clicked() {
                            if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                self.pending_deletions.push(uuid);
                            }
                        }

                        // Stop button for continuous nodes
                        if can_stop_continuous {
                            if ui.small_button("⏹").clicked() {
                                if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                    self.pending_continuous_stop.push(uuid);
                                }
                            }
                        }

                        // Play button for continuous nodes (if in stoppable states)
                        if can_start_continuous {
                            if ui.small_button("▶").clicked() {
                                if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                    self.pending_continuous_start.push(uuid);
                                }
                            }
                        }
                    });
                })
                // T037: Add tooltip with execution state details
                .response.on_hover_ui(|ui| {
                    ui.label(format!("State: {}", state_name));
                    if let Some(graph_node) = self.graph.nodes.get(&node_uuid) {
                        if let Some(config) = &graph_node.continuous_config {
                            if config.runtime_state.iterations > 0 {
                                ui.label(format!("Iterations: {}", config.runtime_state.iterations));
                            }
                            if let Some(started_at) = config.runtime_state.started_at {
                                let duration = execution_status::format_duration(Some(started_at));
                                ui.label(format!("Running for: {}", duration));
                            }
                            // T036: Show error details in tooltip
                            if let Some(error) = &config.runtime_state.last_error {
                                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                            }
                        }
                    }
                });
            } else if is_running || is_continuous_running || is_continuous_starting {
                // Fallback for non-continuous nodes or nodes without continuous_config
                let color = egui::Color32::from_rgb(255, 200, 0);

                ui.horizontal(|ui| {
                    // T019: Show selection indicator for selected nodes
                    if is_selected {
                        ui.label(egui::RichText::new("✓").color(egui::Color32::from_rgb(100, 200, 255)).strong());
                    }

                    ui.label(egui::RichText::new("⏳").color(color));
                    ui.label(&node_data.display_name);

                    // T059: Show warning icon if component was updated
                    if node_data.needs_component_refresh {
                        ui.label(egui::RichText::new("⚠").color(egui::Color32::from_rgb(255, 180, 0)))
                            .on_hover_text("Component updated - node may need refresh");
                    }

                    // Right-aligned buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button (X)
                        if ui.small_button("✖").clicked() {
                            if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                self.pending_deletions.push(uuid);
                            }
                        }

                        // Stop button for continuous nodes
                        if can_stop_continuous {
                            if ui.small_button("⏹").clicked() {
                                if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                    self.pending_continuous_stop.push(uuid);
                                }
                            }
                        }
                    });
                });
            } else if recently_completed {
                // Show brief green flash for recently completed nodes
                let color = egui::Color32::from_rgb(50, 200, 100);

                ui.horizontal(|ui| {
                    // T019: Show selection indicator for selected nodes
                    if is_selected {
                        ui.label(egui::RichText::new("◆").color(egui::Color32::from_rgb(100, 200, 255)).strong());
                    }

                    ui.label(egui::RichText::new("✓").color(color));
                    ui.label(&node_data.display_name);

                    // T059: Show warning icon if component was updated
                    if node_data.needs_component_refresh {
                        ui.label(egui::RichText::new("⚠").color(egui::Color32::from_rgb(255, 180, 0)))
                            .on_hover_text("Component updated - node may need refresh");
                    }

                    // Right-aligned buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button (X)
                        if ui.small_button("✖").clicked() {
                            if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                self.pending_deletions.push(uuid);
                            }
                        }

                        // Play button for continuous nodes
                        if can_start_continuous {
                            if ui.small_button("▶").clicked() {
                                if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                    self.pending_continuous_start.push(uuid);
                                }
                            }
                        }
                    });
                });
            } else {
                // No indicator for Idle or old Completed/Failed states
                ui.horizontal(|ui| {
                    // T047: Show composition badge for composite nodes
                    if node_data.is_composite {
                        ui.label(egui::RichText::new("⚙").color(egui::Color32::from_rgb(255, 200, 80)).strong())
                            .on_hover_text("Composite Node - Right-click to drill down");
                    }

                    // T019: Show selection indicator for selected nodes
                    if is_selected {
                        ui.label(egui::RichText::new("◆").color(egui::Color32::from_rgb(100, 200, 255)).strong());
                    }

                    ui.label(&node_data.display_name);

                    // T059: Show warning icon if component was updated
                    if node_data.needs_component_refresh {
                        ui.label(egui::RichText::new("⚠").color(egui::Color32::from_rgb(255, 180, 0)))
                            .on_hover_text("Component updated - node may need refresh");
                    }

                    // Right-aligned buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button (X)
                        if ui.small_button("✖").clicked() {
                            if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                self.pending_deletions.push(uuid);
                            }
                        }

                        // Play button for continuous nodes
                        if can_start_continuous {
                            if ui.small_button("▶").clicked() {
                                if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                                    self.pending_continuous_start.push(uuid);
                                }
                            }
                        }

                        // Stop button for continuous nodes (for stopping state)
                        if is_continuous_stopping {
                            ui.add_enabled(false, egui::Button::new("⏹").small());
                        }
                    });
                });
            }
        }
    }

    fn inputs(&mut self, node: &SnarlNodeData) -> usize {
        node.inputs.len()
    }

    fn outputs(&mut self, node: &SnarlNodeData) -> usize {
        node.outputs.len()
    }

    fn has_body(&mut self, node: &SnarlNodeData) -> bool {
        // T045: Enable body for composite nodes to show footer with component names
        node.is_composite
    }

    fn show_body(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<SnarlNodeData>,
    ) {
        // T045: Show footer for composite nodes with component names
        if let Some(node_data) = snarl.get_node(node) {
            if node_data.is_composite {
                // Get composition data from the actual graph node
                if let Some(graph_node) = self.graph.nodes.get(&node_data.uuid) {
                    if let Some(composition_data) = &graph_node.composition_data {
                        // T045: Render footer with component names
                        ui.add_space(4.0);

                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("Composed from {} components",
                                composition_data.metadata.component_count))
                                .small()
                                .color(egui::Color32::from_rgb(150, 150, 160))
                        ).wrap());

                        // List component names (limited to avoid tall nodes)
                        let display_count = composition_data.metadata.component_names.len().min(3);
                        for name in composition_data.metadata.component_names.iter().take(display_count) {
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("  • {}", name))
                                    .small()
                                    .color(egui::Color32::from_rgb(180, 180, 190))
                            ).wrap());
                        }

                        if composition_data.metadata.component_names.len() > 3 {
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("  ... and {} more",
                                    composition_data.metadata.component_names.len() - 3))
                                    .small()
                                    .italics()
                                    .color(egui::Color32::from_rgb(120, 120, 130))
                            ).wrap());
                        }

                        ui.add_space(4.0);
                    }
                }
            }
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<SnarlNodeData>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let color = if let Some(node) = snarl.get_node(pin.id.node) {
            if let Some(port) = node.inputs.get(pin.id.input) {
                // Show port name and type with wrapping
                let label = ui.add(egui::Label::new(format!("{}: {}", port.name, port.data_type.name())).wrap());

                // T048: Add tooltip for composite nodes showing internal mapping
                if node.is_composite {
                    if let Some((internal_node, internal_port)) = node.input_mappings.get(&port.name) {
                        label.on_hover_text(format!("from {}.{}", internal_node, internal_port));
                    }
                }

                Self::type_color(&port.data_type)
            } else {
                Color32::GRAY
            }
        } else {
            Color32::GRAY
        };

        // Return circular pin info with color based on type
        PinInfo::circle().with_fill(color)
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<SnarlNodeData>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let color = if let Some(node) = snarl.get_node(pin.id.node) {
            if let Some(port) = node.outputs.get(pin.id.output) {
                // Show port name and type only (values moved to default footer)
                // This is the connections section - keep it clean and focused
                let label = ui.add(egui::Label::new(format!("{}: {}", port.name, port.data_type.name())).wrap());

                // T048: Add tooltip for composite nodes showing internal mapping
                if node.is_composite {
                    if let Some((internal_node, internal_port)) = node.output_mappings.get(&port.name) {
                        label.on_hover_text(format!("to {}.{}", internal_node, internal_port));
                    }
                }

                Self::type_color(&port.data_type)
            } else {
                Color32::GRAY
            }
        } else {
            Color32::GRAY
        };

        // Return circular pin info with color based on type
        PinInfo::circle().with_fill(color)
    }

    fn has_graph_menu(&mut self, _pos: Pos2, _snarl: &mut Snarl<SnarlNodeData>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        _pos: Pos2,
        ui: &mut egui::Ui,
        _snarl: &mut Snarl<SnarlNodeData>,
    ) {
        ui.label("Right-click on nodes for options");
        ui.label("Drag from output → input to connect");
        ui.label("Use palette on left to add nodes");
    }

    fn has_node_menu(&mut self, _node: &SnarlNodeData) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<SnarlNodeData>,
    ) {
        if let Some(node_data) = snarl.get_node(node) {
            ui.label(format!("Node: {}", node_data.display_name));
            ui.label(format!("Component: {}", node_data.component_id));
            ui.separator();

            // T040: Show "Drill Down" option for composite nodes
            if node_data.is_composite {
                if ui.button("🔍 Drill Down").clicked() {
                    if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                        *self.pending_drill_down = Some(uuid);
                    }
                    ui.close();
                }
                ui.separator();
            }

            // T078: Show "View Permissions" for user-defined components
            if node_data.component_id.starts_with("user:")
                && ui.button("🔐 View Permissions").clicked()
            {
                if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                    *self.pending_permission_view = Some(uuid);
                }
                ui.close();
            }

            if ui.button("Delete Node").clicked() {
                // Queue for deletion instead of deleting directly
                // This allows the app to handle deletion through command history
                if let Some(&uuid) = self.snarl_to_uuid.get(&node) {
                    self.pending_deletions.push(uuid);
                }
                ui.close();
            }

            if ui.button("Properties...").clicked() {
                // TODO: Show properties dialog
                ui.close();
            }
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<SnarlNodeData>) {
        // TODO: Validate type compatibility before connecting
        snarl.connect(from.id, to.id);
    }

    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<SnarlNodeData>) {
        snarl.disconnect(from.id, to.id);
    }

    fn node_layout(
        &mut self,
        _default: NodeLayout,
        _node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        _snarl: &Snarl<SnarlNodeData>,
    ) -> NodeLayout {
        // Use Coil layout to place pins on the edges (horizontally)
        NodeLayout::coil()
    }

    fn has_footer(&mut self, _node: &SnarlNodeData) -> bool {
        // Always show footer section for the four-section layout
        // Footer will contain either default output status or custom footer view
        true
    }

    fn show_footer(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<SnarlNodeData>,
    ) {
        // Add spacing before footer
        ui.add_space(6.0);

        // Get node data to check if it has custom width (e.g., WASM Creator)
        let is_wasm_creator = snarl.get_node(node)
            .map(|n| n.component_id == "builtin:development:wasm-creator")
            .unwrap_or(false);

        let custom_width = snarl.get_node(node)
            .and_then(|n| n.custom_width);

        // Wrap all footer content in a scope with width constraint
        if is_wasm_creator {
            // WASM Creator nodes are resizable - use Resize container
            let resize_id = ui.id().with(node);
            let min_width = 600.0;  // 400.0 * 1.5
            let max_width = 1800.0; // 1200.0 * 1.5
            let current_width = custom_width.unwrap_or(975.0); // 650.0 * 1.5

            let resize = egui::Resize::default()
                .id_salt(resize_id)
                .default_width(current_width)
                .min_width(min_width)
                .max_width(max_width)
                .resizable([true, false]); // Only horizontal resizing

            resize.show(ui, |ui| {
                ui.style_mut().spacing.item_spacing.x = 4.0;

                // Store the new width back to snarl node
                let new_width = ui.available_width();
                if let Some(node_data) = snarl.get_node_mut(node) {
                    node_data.custom_width = Some(new_width);
                    let node_uuid = node_data.uuid;
                    let component_id = node_data.component_id.clone();

                    if let Some(graph_node) = self.graph.nodes.get_mut(&node_uuid) {
                        // Check if component has custom footer view
                        let has_custom_footer = self
                            .registry
                            .get_by_id(&component_id)
                            .map(|spec| spec.has_footer_view())
                            .unwrap_or(false);

                        // Performance timing
                        let start_time = std::time::Instant::now();

                        let result = if has_custom_footer {
                            // Use custom footer view (read-only)
                            if let Some(spec) = self.registry.get_by_id(&component_id) {
                                if let Some(view) = spec.get_footer_view() {
                                    view.render_footer(ui, graph_node)
                                } else {
                                    Ok(())
                                }
                            } else {
                                Ok(())
                            }
                        } else {
                            // Use default footer view (with mutable access for input editing)
                            DefaultFooterView::render_for_node(ui, graph_node, node, snarl)
                        };

                        // Performance logging
                        let elapsed = start_time.elapsed();
                        if elapsed.as_millis() > 50 {
                            log::warn!(
                                "Slow footer view rendering for component '{}': {}ms (target: <50ms)",
                                component_id,
                                elapsed.as_millis()
                            );
                        } else {
                            log::trace!(
                                "Footer view rendered for '{}' in {}ms",
                                component_id,
                                elapsed.as_millis()
                            );
                        }

                        // Handle errors
                        if let Err(err) = result {
                            ui.colored_label(egui::Color32::RED, "⚠️ View render failed");
                            ui.label(&err);
                        }
                    }

                    // Add spacing at the bottom of footer
                    ui.add_space(6.0);
                }
            });
        } else {
            // Non-resizable nodes - constrain both width and height
            ui.scope(|ui| {
                ui.set_max_width(300.0);  // Prevent horizontal growth (200.0 * 1.5)
                ui.set_max_height(200.0); // Prevent vertical growth
                ui.style_mut().spacing.item_spacing.x = 4.0;
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

                // Wrap content in scroll area
                // auto_shrink([false, true]) = don't shrink horizontally, allow vertical shrinking
                // This ensures ui.available_width() inside returns the full 300px
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {

                    if let Some(node_data) = snarl.get_node_mut(node) {
                        let node_uuid = node_data.uuid;
                        let component_id = node_data.component_id.clone();

                        if let Some(graph_node) = self.graph.nodes.get_mut(&node_uuid) {
                            // Check if component has custom footer view
                            let has_custom_footer = self
                                .registry
                                .get_by_id(&component_id)
                                .map(|spec| spec.has_footer_view())
                                .unwrap_or(false);

                            // Performance timing
                            let start_time = std::time::Instant::now();

                            let result = if has_custom_footer {
                                // Use custom footer view (read-only)
                                if let Some(spec) = self.registry.get_by_id(&component_id) {
                                    if let Some(view) = spec.get_footer_view() {
                                        view.render_footer(ui, graph_node)
                                    } else {
                                        Ok(())
                                    }
                                } else {
                                    Ok(())
                                }
                            } else {
                                // Use default footer view (with mutable access for input editing)
                                DefaultFooterView::render_for_node(ui, graph_node, node, snarl)
                            };

                        // Performance logging
                        let elapsed = start_time.elapsed();
                        if elapsed.as_millis() > 50 {
                            log::warn!(
                                "Slow footer view rendering for component '{}': {}ms (target: <50ms)",
                                component_id,
                                elapsed.as_millis()
                            );
                        } else {
                            log::trace!(
                                "Footer view rendered for '{}' in {}ms",
                                component_id,
                                elapsed.as_millis()
                            );
                        }

                        // Handle errors
                        if let Err(err) = result {
                            ui.colored_label(egui::Color32::RED, "⚠️ View render failed");
                            ui.label(&err);
                        }
                        }
                    }

                    // Add spacing at the bottom of footer
                    ui.add_space(6.0);
                }); // Close ScrollArea show
            }); // Close scope
        }
    }
}

impl CanvasViewer<'_> {
    /// Get color for data type
    fn type_color(data_type: &DataType) -> Color32 {
        match data_type {
            DataType::F32 => Color32::from_rgb(100, 150, 255), // Blue
            DataType::I32 => Color32::from_rgb(150, 100, 255), // Purple
            DataType::U32 => Color32::from_rgb(255, 150, 100), // Orange
            DataType::String => Color32::from_rgb(100, 255, 150), // Green
            DataType::Binary => Color32::from_rgb(200, 200, 200), // Gray
            DataType::List(_) => Color32::from_rgb(255, 200, 100), // Yellow
            DataType::Record(_) => Color32::from_rgb(255, 100, 150), // Pink
            DataType::Any => Color32::WHITE,
        }
    }
}
