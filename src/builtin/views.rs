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

/// Unified Constant Footer View with type dropdown and value editor
///
/// Allows users to select the type (U32, I32, F32, String, Binary) and edit the value
pub struct UnifiedConstantFooterView;

impl UnifiedConstantFooterView {
    /// Create a new unified constant footer view
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    /// Get the type name for display
    fn get_type_name(value: &NodeValue) -> &'static str {
        match value {
            NodeValue::U32(_) => "U32",
            NodeValue::I32(_) => "I32",
            NodeValue::F32(_) => "F32",
            NodeValue::String(_) => "String",
            NodeValue::Binary(_) => "Binary",
            _ => "Unknown",
        }
    }

    /// Convert value to a different type, preserving data when possible
    fn convert_value(from: &NodeValue, to_type: &str) -> NodeValue {
        match to_type {
            "U32" => match from {
                NodeValue::U32(v) => NodeValue::U32(*v),
                NodeValue::I32(v) => NodeValue::U32((*v).max(0) as u32),
                NodeValue::F32(v) => NodeValue::U32((*v).max(0.0) as u32),
                NodeValue::String(s) => NodeValue::U32(s.parse().unwrap_or(0)),
                _ => NodeValue::U32(0),
            },
            "I32" => match from {
                NodeValue::U32(v) => NodeValue::I32(*v as i32),
                NodeValue::I32(v) => NodeValue::I32(*v),
                NodeValue::F32(v) => NodeValue::I32(*v as i32),
                NodeValue::String(s) => NodeValue::I32(s.parse().unwrap_or(0)),
                _ => NodeValue::I32(0),
            },
            "F32" => match from {
                NodeValue::U32(v) => NodeValue::F32(*v as f32),
                NodeValue::I32(v) => NodeValue::F32(*v as f32),
                NodeValue::F32(v) => NodeValue::F32(*v),
                NodeValue::String(s) => NodeValue::F32(s.parse().unwrap_or(0.0)),
                _ => NodeValue::F32(0.0),
            },
            "String" => match from {
                NodeValue::String(s) => NodeValue::String(s.clone()),
                _ => NodeValue::String(from.format_display()),
            },
            "Binary" => match from {
                NodeValue::Binary(b) => NodeValue::Binary(b.clone()),
                NodeValue::String(s) => NodeValue::Binary(s.as_bytes().to_vec()),
                _ => NodeValue::Binary(vec![]),
            },
            _ => from.clone(),
        }
    }
}

impl ComponentFooterView for UnifiedConstantFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        ui.vertical(|ui| {
            ui.heading("Constant Value");
            ui.separator();

            if let Some(output) = node.outputs.first_mut() {
                if let Some(current_value) = &mut output.current_value {
                    // Type selector dropdown
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Type:").color(egui::Color32::from_rgb(180, 180, 180)));

                        let current_type = Self::get_type_name(current_value);

                        egui::ComboBox::from_id_salt("constant_type_selector")
                            .selected_text(current_type)
                            .show_ui(ui, |ui| {
                                let mut type_changed = false;
                                let mut new_type = current_type;

                                if ui.selectable_value(&mut new_type, "U32", "U32 (Unsigned Integer)").clicked() {
                                    type_changed = current_type != "U32";
                                }
                                if ui.selectable_value(&mut new_type, "I32", "I32 (Signed Integer)").clicked() {
                                    type_changed = current_type != "I32";
                                }
                                if ui.selectable_value(&mut new_type, "F32", "F32 (Float)").clicked() {
                                    type_changed = current_type != "F32";
                                }
                                if ui.selectable_value(&mut new_type, "String", "String (Text)").clicked() {
                                    type_changed = current_type != "String";
                                }
                                if ui.selectable_value(&mut new_type, "Binary", "Binary (Bytes)").clicked() {
                                    type_changed = current_type != "Binary";
                                }

                                // If type changed, convert the value
                                if type_changed {
                                    *current_value = Self::convert_value(current_value, new_type);
                                    node.dirty = true;
                                }
                            });
                    });

                    ui.add_space(8.0);

                    // Value editor
                    ui.label(egui::RichText::new("Value:").color(egui::Color32::from_rgb(180, 180, 180)));

                    let is_string = matches!(current_value, NodeValue::String(_));
                    let is_binary = matches!(current_value, NodeValue::Binary(_));

                    if is_binary {
                        // Binary: hex display and byte count
                        if let NodeValue::Binary(bytes) = current_value {
                            ui.label(format!("{} bytes", bytes.len()));

                            // Show first few bytes in hex
                            let preview = bytes.iter()
                                .take(16)
                                .map(|b| format!("{:02x}", b))
                                .collect::<Vec<_>>()
                                .join(" ");

                            let preview_text = if bytes.len() > 16 {
                                format!("{}...", preview)
                            } else {
                                preview
                            };

                            ui.add(
                                egui::TextEdit::singleline(&mut preview_text.to_string())
                                    .desired_width(ui.available_width())
                            ).on_hover_text("Binary values are displayed in hexadecimal");
                        }
                    } else if is_string {
                        // Multiline text edit for strings
                        let mut text_value = current_value.format_display();
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut text_value)
                                .desired_rows(5)
                                .desired_width(ui.available_width())
                        );

                        if response.changed() {
                            let cleaned = text_value.trim_matches('"').to_string();
                            *current_value = NodeValue::String(cleaned);
                            node.dirty = true;
                        }
                    } else {
                        // Single-line edit for numbers
                        let mut text_value = current_value.format_display();
                        let response = ui.text_edit_singleline(&mut text_value);

                        if response.changed() {
                            let parse_result = match current_value {
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

                            if let Some(new_value) = parse_result {
                                *current_value = new_value;
                                node.dirty = true;
                            }
                        }
                    }
                } else {
                    // Initialize with a default value if none exists
                    output.current_value = Some(NodeValue::F32(0.0));
                    node.dirty = true;
                }
            }

            ui.add_space(4.0);
            ui.separator();
            ui.label("⚡ Ready (constant value)");
        });

        Ok(())
    }
}
