//! SnarlViewer implementation for canvas
//!
//! This module implements the egui-snarl SnarlViewer trait which handles
//! node rendering, pin rendering, and user interactions with nodes.

use crate::graph::graph::NodeGraph;
use crate::graph::node::{ComponentRegistry, DataType};
use crate::ui::execution_status;
use egui::{Color32, Pos2};
use egui_snarl::ui::{NodeLayout, PinInfo, SnarlViewer};
use egui_snarl::{InPin, NodeId, OutPin, Snarl};
use std::collections::HashMap;
use uuid::Uuid;
use super::node_data::SnarlNodeData;
use super::footer::DefaultFooterView;

/// Viewer implementation for egui-snarl
pub(super) struct CanvasViewer<'a> {
    pub graph: &'a mut NodeGraph,
    pub registry: &'a ComponentRegistry,
    pub pending_deletions: &'a mut Vec<Uuid>,
    pub pending_permission_view: &'a mut Option<Uuid>,
    pub pending_continuous_start: &'a mut Vec<Uuid>,
    pub pending_continuous_stop: &'a mut Vec<Uuid>,
    pub pending_drill_down: &'a mut Option<Uuid>, // T040: Drill-down requests
    pub snarl_to_uuid: &'a HashMap<NodeId, Uuid>,
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

            let is_running = matches!(node_data.execution_state, crate::graph::node::ExecutionState::Running);

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
