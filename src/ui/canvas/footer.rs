//! Footer rendering for canvas nodes
//!
//! This module handles the rendering of node footer sections, including
//! input editors for unconnected inputs and output value displays.

use crate::graph::node::{DataType, ExecutionState, GraphNode, NodeValue};
use egui_snarl::{NodeId, Snarl};
use super::node_data::SnarlNodeData;

/// Helper to render default footer view with FooterHead and FooterBody sections
pub(super) struct DefaultFooterView;

impl DefaultFooterView {
    /// Render footer with two sections:
    /// - FooterHead: Input editors for unconnected inputs
    /// - FooterBody: Output values and execution status
    pub(super) fn render_for_node(
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
                                input_port.current_value = Some(NodeValue::U32(0));
                            }

                            if let Some(NodeValue::U32(ref mut v)) =
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
                                input_port.current_value = Some(NodeValue::I32(0));
                            }

                            if let Some(NodeValue::I32(ref mut v)) =
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
                                    Some(NodeValue::F32(0.0));
                            }

                            if let Some(NodeValue::F32(ref mut v)) =
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
                                    Some(NodeValue::String(String::new()));
                            }

                            if let Some(NodeValue::String(ref mut s)) =
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
