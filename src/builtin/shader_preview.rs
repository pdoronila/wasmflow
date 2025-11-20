//! Shader Preview Node
//!
//! A built-in node that displays rendered shader output in the node footer.
//! Phase 1: Accepts texture-data but displays placeholder (no actual rendering yet)
//! Phase 2: Will integrate WebGPU for actual texture display

use crate::graph::node::{ComponentSpec, DataType, GraphNode, NodeValue, PortSpec, ShaderPreviewNodeData};
use crate::ui::component_view::ComponentFooterView;
use crate::ComponentError;
use egui::{Color32, RichText};
use std::collections::HashMap;

/// Create component specification
pub fn spec() -> ComponentSpec {
    let mut spec = ComponentSpec::new_builtin(
        "builtin:graphics:shader-preview".to_string(),
        "Shader Preview".to_string(),
        "Display rendered shader output (Phase 1: placeholder mode)".to_string(),
        Some("Graphics".to_string()),
    );

    // Define inputs
    spec.input_spec = vec![
        PortSpec {
            name: "texture".to_string(),
            data_type: DataType::Texture,
            optional: true,
            description: "Rendered texture data to display".to_string(),
        },
        PortSpec {
            name: "zoom".to_string(),
            data_type: DataType::F32,
            optional: true,
            description: "Display zoom level (1.0 = 100%)".to_string(),
        },
    ];

    // No outputs - displays in footer
    spec
}

/// Execute the shader preview node (processes inputs, updates state)
pub fn execute(
    node_data: &mut ShaderPreviewNodeData,
    inputs: &HashMap<String, NodeValue>,
) -> Result<HashMap<String, NodeValue>, ComponentError> {
    // Update zoom if provided
    if let Some(NodeValue::F32(zoom_val)) = inputs.get("zoom") {
        node_data.zoom = zoom_val.max(0.1).min(10.0); // Clamp to reasonable range
    }

    // Process texture data if provided
    if let Some(NodeValue::Texture(texture)) = inputs.get("texture") {
        node_data.last_texture_size = Some((texture.width, texture.height));
        node_data.last_update = Some(std::time::Instant::now());
    }

    // No outputs
    Ok(HashMap::new())
}

/// Footer view for Shader Preview Node
pub struct ShaderPreviewFooterView;

impl ShaderPreviewFooterView {
    pub fn new() -> Self {
        Self
    }
}

impl ComponentFooterView for ShaderPreviewFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Extract shader preview data from node
        let preview_node = match &mut node.shader_preview_data {
            Some(data) => data,
            None => {
                ui.label("Shader Preview Node data not initialized");
                return Ok(());
            }
        };

        ui.vertical(|ui| {
            ui.set_min_width(400.0);
            ui.set_max_width(600.0);

            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Shader Preview").color(Color32::WHITE));
                ui.label(RichText::new("(Phase 1: Placeholder)").color(Color32::GRAY));
            });

            ui.add_space(8.0);

            // Preview area (placeholder for Phase 1)
            ui.group(|ui| {
                ui.set_min_height(preview_node.preview_size.1 as f32);
                ui.set_min_width(preview_node.preview_size.0 as f32);

                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);

                    // Placeholder icon/text
                    ui.label(
                        RichText::new("🖼")
                            .size(64.0)
                            .color(Color32::from_gray(100)),
                    );

                    ui.add_space(16.0);

                    ui.label(
                        RichText::new("Shader Preview Placeholder")
                            .size(16.0)
                            .color(Color32::GRAY),
                    );

                    ui.add_space(8.0);

                    ui.label(
                        RichText::new("GPU rendering will be available in Phase 2")
                            .size(12.0)
                            .color(Color32::DARK_GRAY),
                    );

                    ui.add_space(16.0);

                    // Show texture info if available
                    if let Some((width, height)) = preview_node.last_texture_size {
                        ui.label(
                            RichText::new(format!("Last texture: {}x{}", width, height))
                                .size(12.0)
                                .color(Color32::from_gray(150)),
                        );
                    } else {
                        ui.label(
                            RichText::new("No texture data received")
                                .size(12.0)
                                .color(Color32::from_gray(120)),
                        );
                    }
                });
            });

            ui.add_space(8.0);

            // Controls
            ui.horizontal(|ui| {
                ui.label("Size:");
                if ui
                    .selectable_label(preview_node.preview_size == (400, 300), "Small")
                    .clicked()
                {
                    preview_node.preview_size = (400, 300);
                }
                if ui
                    .selectable_label(preview_node.preview_size == (600, 450), "Medium")
                    .clicked()
                {
                    preview_node.preview_size = (600, 450);
                }
                if ui
                    .selectable_label(preview_node.preview_size == (800, 600), "Large")
                    .clicked()
                {
                    preview_node.preview_size = (800, 600);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut preview_node.zoom, 0.1..=10.0).suffix("x"));
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut preview_node.auto_refresh, "Auto-refresh");
                if preview_node.auto_refresh {
                    ui.add(
                        egui::Slider::new(&mut preview_node.refresh_rate, 1.0..=60.0)
                            .suffix(" Hz")
                            .text("Rate"),
                    );
                }
            });

            ui.add_space(4.0);

            // Stats
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Stats:").strong());

                if let Some(update_time) = preview_node.last_update {
                    let elapsed = update_time.elapsed().as_secs_f32();
                    ui.label(format!("Last update: {:.2}s ago", elapsed));
                } else {
                    ui.label("No updates yet");
                }
            });
        });

        Ok(())
    }
}

/// Register the shader preview node in the component registry
pub fn register_shader_preview_node(registry: &mut crate::graph::node::ComponentRegistry) {
    let spec = spec().with_footer_view(std::sync::Arc::new(ShaderPreviewFooterView::new()));
    registry.register_builtin(spec);
    log::info!("Registered Shader Preview Node (Phase 1: placeholder mode)");
}
