//! Shader Preview Node
//!
//! A built-in node that displays rendered shader output in the node footer.
//! Phase 2: Integrated with WebGPU for real-time texture display

use crate::graph::node::{ComponentSpec, DataType, GraphNode, NodeValue, PortSpec, ShaderPreviewNodeData, TextureData};
use crate::ui::component_view::ComponentFooterView;
use crate::ComponentError;
use egui::{Color32, ColorImage, RichText};
use std::collections::HashMap;

/// Create component specification
pub fn spec() -> ComponentSpec {
    let mut spec = ComponentSpec::new_builtin(
        "builtin:graphics:shader-preview".to_string(),
        "Shader Preview".to_string(),
        "Display rendered shader output with real-time GPU rendering".to_string(),
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

        // Cache texture data for GPU upload in footer view
        node_data.cached_texture_data = Some(texture.clone());

        // Clear GPU texture ID to force re-upload
        node_data.gpu_texture_id = None;

        log::debug!("Shader preview received texture: {}x{}", texture.width, texture.height);
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
            ui.set_max_width(800.0);

            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Shader Preview").color(Color32::WHITE));
                if preview_node.cached_texture_data.is_some() {
                    ui.label(RichText::new("● Active").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("○ Idle").color(Color32::GRAY));
                }
            });

            ui.add_space(8.0);

            // Preview area
            ui.group(|ui| {
                let preview_width = preview_node.preview_size.0 as f32 * preview_node.zoom;
                let preview_height = preview_node.preview_size.1 as f32 * preview_node.zoom;

                ui.set_min_height(preview_height.min(600.0));
                ui.set_min_width(preview_width.min(800.0));

                // Upload texture to GPU if needed
                if let Some(texture_data) = &preview_node.cached_texture_data {
                    if preview_node.gpu_texture_id.is_none() {
                        // Convert texture data to egui ColorImage
                        let color_image = convert_texture_to_color_image(texture_data);

                        // Upload to egui texture manager
                        let texture_id = ui.ctx().load_texture(
                            "shader_preview",
                            color_image,
                            egui::TextureOptions::default(),
                        );

                        preview_node.gpu_texture_id = Some(texture_id.id());
                    }

                    // Display the texture
                    if let Some(texture_id) = preview_node.gpu_texture_id {
                        ui.vertical_centered(|ui| {
                            ui.image(egui::load::SizedTexture::new(
                                texture_id,
                                egui::vec2(preview_width, preview_height),
                            ));
                        });
                    }
                } else {
                    // No texture available - show placeholder
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);

                        ui.label(
                            RichText::new("🖼")
                                .size(64.0)
                                .color(Color32::from_gray(100)),
                        );

                        ui.add_space(16.0);

                        ui.label(
                            RichText::new("No Texture Input")
                                .size(16.0)
                                .color(Color32::GRAY),
                        );

                        ui.add_space(8.0);

                        ui.label(
                            RichText::new("Connect a texture to see preview")
                                .size(12.0)
                                .color(Color32::DARK_GRAY),
                        );
                    });
                }
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

/// Convert TextureData to egui ColorImage
fn convert_texture_to_color_image(texture: &TextureData) -> ColorImage {
    use crate::graph::node::TextureFormat;

    let width = texture.width as usize;
    let height = texture.height as usize;

    match texture.format {
        TextureFormat::Rgba8 => {
            // RGBA8 format - directly convert to Color32
            let pixels: Vec<Color32> = texture
                .data
                .chunks_exact(4)
                .map(|chunk| Color32::from_rgba_premultiplied(chunk[0], chunk[1], chunk[2], chunk[3]))
                .collect();

            ColorImage {
                size: [width, height],
                source_size: egui::vec2(width as f32, height as f32),
                pixels,
            }
        }
        TextureFormat::Rgb8 => {
            // RGB8 format - add alpha channel
            let pixels: Vec<Color32> = texture
                .data
                .chunks_exact(3)
                .map(|chunk| Color32::from_rgb(chunk[0], chunk[1], chunk[2]))
                .collect();

            ColorImage {
                size: [width, height],
                source_size: egui::vec2(width as f32, height as f32),
                pixels,
            }
        }
        TextureFormat::R8 => {
            // Grayscale - replicate to RGB
            let pixels: Vec<Color32> = texture
                .data
                .iter()
                .map(|&gray| Color32::from_gray(gray))
                .collect();

            ColorImage {
                size: [width, height],
                source_size: egui::vec2(width as f32, height as f32),
                pixels,
            }
        }
        TextureFormat::Rgba32Float => {
            // 32-bit float RGBA - convert to 8-bit
            let pixels: Vec<Color32> = texture
                .data
                .chunks_exact(16) // 4 floats * 4 bytes
                .map(|chunk| {
                    let r = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let g = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                    let b = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
                    let a = f32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]);

                    Color32::from_rgba_premultiplied(
                        (r.clamp(0.0, 1.0) * 255.0) as u8,
                        (g.clamp(0.0, 1.0) * 255.0) as u8,
                        (b.clamp(0.0, 1.0) * 255.0) as u8,
                        (a.clamp(0.0, 1.0) * 255.0) as u8,
                    )
                })
                .collect();

            ColorImage {
                size: [width, height],
                source_size: egui::vec2(width as f32, height as f32),
                pixels,
            }
        }
        TextureFormat::Depth24Stencil8 => {
            // Depth/stencil - visualize depth as grayscale
            let pixels: Vec<Color32> = texture
                .data
                .chunks_exact(4)
                .map(|chunk| {
                    // Depth is in first 3 bytes (24-bit), visualize as grayscale
                    let depth = ((chunk[0] as u32) | ((chunk[1] as u32) << 8) | ((chunk[2] as u32) << 16)) as f32
                        / 16777215.0; // Max 24-bit value
                    let gray = (depth * 255.0) as u8;
                    Color32::from_gray(gray)
                })
                .collect();

            ColorImage {
                size: [width, height],
                source_size: egui::vec2(width as f32, height as f32),
                pixels,
            }
        }
    }
}

/// Register the shader preview node in the component registry
pub fn register_shader_preview_node(registry: &mut crate::graph::node::ComponentRegistry) {
    let spec = spec().with_footer_view(std::sync::Arc::new(ShaderPreviewFooterView::new()));
    registry.register_builtin(spec);
    log::info!("Registered Shader Preview Node with GPU rendering support");
}
