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

                    // Use multiline for strings, checkboxes for bools, single-line for numbers, multiline for lists
                    match value {
                        NodeValue::String(_) => {
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
                        }
                        NodeValue::Bool(b) => {
                            // Checkbox for boolean values
                            let mut bool_value = *b;
                            if ui.checkbox(&mut bool_value, "").changed() {
                                *value = NodeValue::Bool(bool_value);
                                node.dirty = true;
                            }
                        }
                        NodeValue::List(items) => {
                            // Multiline text edit for lists (one item per line)
                            ui.label(
                                egui::RichText::new("List values (one per line):")
                                    .color(egui::Color32::from_rgb(150, 150, 150))
                                    .size(11.0)
                            );

                            // Create a unique ID for this text editor based on node ID and output name
                            let editor_id = ui.id().with(&node.id).with(&output.name);

                            // Get or initialize the editing text from persistent state
                            let mut text_value = ui.data_mut(|data| {
                                data.get_temp::<String>(editor_id).unwrap_or_else(|| {
                                    // First time: initialize from the list
                                    items
                                        .iter()
                                        .map(|item| match item {
                                            NodeValue::String(s) => s.clone(),
                                            NodeValue::U32(n) => n.to_string(),
                                            NodeValue::F32(n) => n.to_string(),
                                            NodeValue::I32(n) => n.to_string(),
                                            _ => item.format_display(),
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                })
                            });

                            let response = ui.add(
                                egui::TextEdit::multiline(&mut text_value)
                                    .desired_rows(8)
                                    .desired_width(ui.available_width())
                            );

                            // Store the current editing state
                            ui.data_mut(|data| {
                                data.insert_temp(editor_id, text_value.clone());
                            });

                            // Parse and update when user finishes editing (loses focus)
                            if response.lost_focus() {
                                // Parse lines into list items
                                let non_empty_lines: Vec<&str> = text_value
                                    .lines()
                                    .map(|line| line.trim())
                                    .filter(|line| !line.is_empty())
                                    .collect();

                                if non_empty_lines.is_empty() {
                                    // Empty list
                                    *value = NodeValue::List(vec![]);
                                    node.dirty = true;
                                } else {
                                    // Detect list type from first element
                                    let list_type = if !items.is_empty() {
                                        match &items[0] {
                                            NodeValue::U32(_) => "u32",
                                            NodeValue::F32(_) => "f32",
                                            NodeValue::I32(_) => "i32",
                                            _ => "string",
                                        }
                                    } else {
                                        "string"
                                    };

                                    // Parse based on detected type
                                    let parsed_items: Vec<NodeValue> = match list_type {
                                        "u32" => non_empty_lines
                                            .iter()
                                            .filter_map(|line| line.parse::<u32>().ok().map(NodeValue::U32))
                                            .collect(),
                                        "f32" => non_empty_lines
                                            .iter()
                                            .filter_map(|line| line.parse::<f32>().ok().map(NodeValue::F32))
                                            .collect(),
                                        "i32" => non_empty_lines
                                            .iter()
                                            .filter_map(|line| line.parse::<i32>().ok().map(NodeValue::I32))
                                            .collect(),
                                        _ => non_empty_lines
                                            .iter()
                                            .map(|line| NodeValue::String(line.to_string()))
                                            .collect(),
                                    };

                                    if !parsed_items.is_empty() {
                                        *value = NodeValue::List(parsed_items);
                                        node.dirty = true;
                                    }
                                }
                            }
                        }
                        _ => {
                            // Single-line edit for numbers and other types
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
