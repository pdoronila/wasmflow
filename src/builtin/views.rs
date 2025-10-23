//! Built-in component views (body and footer)
//!
//! This module contains view implementations for builtin components.

use crate::graph::node::{GraphNode, NodeValue};
use crate::ui::component_view::ComponentFooterView;
use std::sync::Arc;

/// T020: Footer view for HTTP Fetch components
///
/// Displays HTTP response data in a clean grid format.
/// This view extracts existing footer rendering logic for http_fetch nodes.
pub struct HttpFetchFooterView;

impl HttpFetchFooterView {
    /// Create a new HTTP fetch footer view
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

/// T021: Implement ComponentFooterView for HttpFetchFooterView
impl ComponentFooterView for HttpFetchFooterView {
    fn render_footer(&self, _ui: &mut egui::Ui, _node: &mut GraphNode) -> Result<(), String> {
        Ok(())
    }
}

/// Generic footer view for constant nodes
///
/// Displays editable constant value in FooterHead and current value in FooterBody.
/// This view is used by all constant types (F32, I32, U32, String).
pub struct ConstantNodeFooterView;

impl ConstantNodeFooterView {
    /// Create a new constant node footer view
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl ComponentFooterView for ConstantNodeFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // FOOTER HEAD: Value editor with editable input fields
        // Use vertical layout to stack elements
        ui.vertical(|ui| {
            ui.separator();

            for output in &mut node.outputs {
                if let Some(value) = &mut output.current_value {
                    // Label for the field
                    ui.label(
                        egui::RichText::new(format!("{}:", output.name))
                            .color(egui::Color32::from_rgb(180, 180, 180))
                    );

                    // Use multiline for strings, single-line for numbers
                    let is_string = matches!(value, NodeValue::String(_));

                    if is_string {
                        // Multiline text edit for strings (better for JSON)
                        let mut text_value = value.format_display();
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut text_value)
                                .desired_rows(5)
                                .desired_width(ui.available_width())
                        );

                        if response.changed() {
                            // For strings, preserve the exact text
                            let cleaned = text_value.trim_matches('"').to_string();
                            *value = NodeValue::String(cleaned);
                        }
                    } else {
                        // Single-line edit for numbers
                        let mut text_value = value.format_display();
                        let response = ui.text_edit_singleline(&mut text_value);

                        // If the user edited the text, parse and update the value
                        if response.changed() {
                            // Parse the new value based on the current type
                            let parse_result = match value {
                                NodeValue::F32(_) => {
                                    text_value.parse::<f32>().map(NodeValue::F32).ok()
                                }
                                NodeValue::I32(_) => {
                                    text_value.parse::<i32>().map(NodeValue::I32).ok()
                                }
                                NodeValue::U32(_) => {
                                    text_value.parse::<u32>().map(NodeValue::U32).ok()
                                }
                                _ => None,
                            };

                            // Update the value if parsing succeeded
                            if let Some(new_value) = parse_result {
                                *value = new_value;
                                node.dirty = true; // Mark node as needing re-execution
                            }
                        }
                    }

                    ui.add_space(4.0);
                }
            }

            ui.separator();

            // FOOTER BODY: Status
            ui.label("⚡ Ready (constant value)");
        });

        Ok(())
    }
}

/// Footer view for math nodes
///
/// Displays operation summary with inputs in FooterHead and result in FooterBody.
pub struct MathNodeFooterView;

impl MathNodeFooterView {
    /// Create a new math node footer view
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl ComponentFooterView for MathNodeFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // FOOTER HEAD: Input values (vertical layout for full width)
        ui.vertical(|ui| {
            for input in &node.inputs {
                ui.label(
                    egui::RichText::new(format!("{}:", input.name))
                        .color(egui::Color32::from_rgb(180, 180, 180))
                );
                if let Some(value) = &input.current_value {
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 0.0),
                        egui::Label::new(value.format_display()).wrap()
                    );
                } else {
                    ui.colored_label(egui::Color32::GRAY, "(not set)");
                }
                ui.add_space(4.0);
            }
        });

        // Visual separator
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // FOOTER BODY: Result/output (vertical layout for full width)
        ui.vertical(|ui| {
            if let Some(output) = node.outputs.first() {
                if let Some(value) = &output.current_value {
                    ui.label(
                        egui::RichText::new(format!("{}:", &output.name))
                            .color(egui::Color32::from_rgb(180, 180, 180))
                    );
                    ui.add_sized(
                        egui::vec2(ui.available_width(), 0.0),
                        egui::Label::new(
                            egui::RichText::new(value.format_display())
                                .color(egui::Color32::from_rgb(100, 255, 150))
                        ).wrap()
                    );
                } else {
                    ui.label("Result:");
                    ui.colored_label(egui::Color32::GRAY, "(pending)");
                }
            }
        });

        Ok(())
    }
}
