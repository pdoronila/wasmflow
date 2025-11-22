//! Texture Loader Node
//!
//! A built-in node that loads image files (PNG, JPG) and outputs texture data for GPU rendering.
//! Phase 3: Texture system foundation for PBR materials

use crate::graph::node::{ComponentRegistry, ComponentSpec, DataType, GraphNode, NodeValue, PortSpec, TextureData, TextureLoaderNodeData};
use crate::ui::component_view::ComponentFooterView;
use crate::ComponentError;
use egui::{Color32, ColorImage, RichText};
use std::collections::HashMap;
use std::path::Path;

impl TextureLoaderNodeData {
    /// Load an image file and cache pixel data
    pub fn load_image(&mut self, path: &Path) -> Result<(), String> {
        // Load image using the image crate
        let img = image::open(path).map_err(|e| format!("Failed to load image: {}", e))?;

        // Convert to RGBA8
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Cache pixel data
        self.cached_pixels = Some(rgba.into_raw());
        self.dimensions = Some((width, height));
        self.file_path = Some(path.to_path_buf());
        self.error_message = None;

        log::info!("Loaded texture: {} ({}x{})", path.display(), width, height);

        Ok(())
    }

    /// Get texture data for output
    pub fn get_texture_data(&self) -> Option<TextureData> {
        if let (Some(pixels), Some((width, height))) = (&self.cached_pixels, self.dimensions) {
            Some(TextureData {
                width,
                height,
                data: pixels.clone(),
                format: crate::graph::node::TextureFormat::Rgba8,
            })
        } else {
            None
        }
    }
}

/// Create component specification
pub fn spec() -> ComponentSpec {
    let mut spec = ComponentSpec::new_builtin(
        "builtin:graphics:texture-loader".to_string(),
        "Texture Loader".to_string(),
        "Load image files (PNG, JPG) as textures for GPU rendering".to_string(),
        Some("Graphics".to_string()),
    );

    // No inputs - file is selected via UI

    // Outputs
    spec.output_spec = vec![
        PortSpec {
            name: "texture".to_string(),
            data_type: DataType::Texture,
            optional: false,
            description: "Loaded texture data (RGBA8)".to_string(),
        },
        PortSpec {
            name: "width".to_string(),
            data_type: DataType::U32,
            optional: false,
            description: "Texture width in pixels".to_string(),
        },
        PortSpec {
            name: "height".to_string(),
            data_type: DataType::U32,
            optional: false,
            description: "Texture height in pixels".to_string(),
        },
    ];

    spec
}

/// Execute the texture loader node
pub fn execute(
    node_data: &mut TextureLoaderNodeData,
    _inputs: &HashMap<String, NodeValue>,
) -> Result<HashMap<String, NodeValue>, ComponentError> {
    let mut outputs = HashMap::new();

    // Output loaded texture data
    if let Some(texture_data) = node_data.get_texture_data() {
        outputs.insert("width".to_string(), NodeValue::U32(texture_data.width));
        outputs.insert("height".to_string(), NodeValue::U32(texture_data.height));
        outputs.insert("texture".to_string(), NodeValue::Texture(texture_data));
    }

    Ok(outputs)
}

/// Register the texture loader node with the component registry
pub fn register_texture_loader_node(
    registry: &mut ComponentRegistry,
) -> Result<(), ComponentError> {
    let spec = spec().with_footer_view(std::sync::Arc::new(TextureLoaderFooterView::new()));
    registry.register_component(spec)?;
    log::info!("Registered texture-loader builtin node");
    Ok(())
}

/// Footer view for Texture Loader Node
pub struct TextureLoaderFooterView;

impl TextureLoaderFooterView {
    pub fn new() -> Self {
        Self
    }
}

impl ComponentFooterView for TextureLoaderFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Extract texture loader data from node
        let loader_data = match &mut node.texture_loader_data {
            Some(data) => data,
            None => {
                ui.label("Texture Loader Node data not initialized");
                return Ok(());
            }
        };

        ui.vertical(|ui| {
            ui.set_min_width(300.0);
            ui.set_max_width(600.0);

            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Texture Loader").color(Color32::WHITE));
                if loader_data.file_path.is_some() {
                    ui.label(RichText::new("● Loaded").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("○ No File").color(Color32::GRAY));
                }
            });

            ui.add_space(8.0);

            // File selection button
            if ui.button("📁 Select Image File...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
                    .pick_file()
                {
                    if let Err(e) = loader_data.load_image(&path) {
                        loader_data.error_message = Some(e);
                    }
                }
            }

            ui.add_space(8.0);

            // Show loaded file path
            if let Some(path) = &loader_data.file_path {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("File:").strong());
                    ui.label(path.display().to_string());
                });
            }

            // Show dimensions
            if let Some((width, height)) = loader_data.dimensions {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Size:").strong());
                    ui.label(format!("{}×{} pixels", width, height));
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Memory:").strong());
                    let bytes = (width * height * 4) as usize;
                    ui.label(format!("{} bytes ({:.2} MB)", bytes, bytes as f64 / 1024.0 / 1024.0));
                });
            }

            ui.add_space(8.0);

            // Show thumbnail
            if let Some(pixels) = &loader_data.cached_pixels {
                if let Some((width, height)) = loader_data.dimensions {
                    // Create or use cached thumbnail
                    let thumbnail_handle = loader_data.thumbnail.get_or_insert_with(|| {
                        let color_image =
                            ColorImage::from_rgba_unmultiplied([width as usize, height as usize], pixels);
                        ui.ctx().load_texture("texture_thumbnail", color_image, Default::default())
                    });

                    // Calculate thumbnail display size (max 256×256, preserve aspect ratio)
                    let max_size = 256.0;
                    let aspect = width as f32 / height as f32;
                    let (display_width, display_height) = if width > height {
                        (max_size, max_size / aspect)
                    } else {
                        (max_size * aspect, max_size)
                    };

                    // Display thumbnail
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Preview:").strong());
                        ui.image((thumbnail_handle.id(), egui::vec2(display_width, display_height)));
                    });
                }
            }

            // Show error message if any
            if let Some(error) = &loader_data.error_message {
                ui.add_space(8.0);
                ui.colored_label(Color32::RED, format!("❌ Error: {}", error));
            }

            ui.add_space(8.0);

            // Help text
            ui.colored_label(
                Color32::GRAY,
                "Supported formats: PNG, JPG, BMP, GIF\nOutput format: RGBA8 (sRGB)",
            );
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_loader_data_default() {
        let data = TextureLoaderNodeData::new();
        assert!(data.file_path.is_none());
        assert!(data.dimensions.is_none());
        assert!(data.cached_pixels.is_none());
    }

    #[test]
    fn test_texture_loader_spec() {
        let spec = spec();
        assert_eq!(spec.name, "Texture Loader");
        assert_eq!(spec.component_id, "builtin:graphics:texture-loader");
        assert_eq!(spec.output_spec.len(), 3);
        assert_eq!(spec.output_spec[0].name, "texture");
        assert_eq!(spec.output_spec[1].name, "width");
        assert_eq!(spec.output_spec[2].name, "height");
    }

    #[test]
    fn test_execute_no_texture() {
        let mut data = TextureLoaderNodeData::new();
        let inputs = HashMap::new();
        let outputs = execute(&mut data, &inputs).unwrap();
        assert!(outputs.is_empty());
    }

    // Note: Image loading tests require actual image files and are tested via integration tests
}
