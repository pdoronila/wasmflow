///! Environment Map Loader Built-in Node
///!
///! Loads cubemap environment maps from 6 separate image files (one per face).
///! Supports PNG, JPG, BMP, GIF formats.

use crate::graph::node::{ComponentRegistry, ComponentSpec, DataType, GraphNode, NodeValue, PortSpec};
use crate::ui::component_view::ComponentFooterView;
use crate::ComponentError;
use egui::{Color32, RichText};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Environment map loader node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvMapLoaderData {
    pub face_paths: [Option<PathBuf>; 6], // +X, -X, +Y, -Y, +Z, -Z
    pub face_size: Option<u32>,            // Size once loaded (all faces must match)
    #[serde(skip)]
    pub face_data: [Option<Vec<u8>>; 6], // RGBA8 data for each face
    #[serde(skip)]
    pub error_message: Option<String>,
}

impl Default for EnvMapLoaderData {
    fn default() -> Self {
        Self {
            face_paths: [None, None, None, None, None, None],
            face_size: None,
            face_data: [None, None, None, None, None, None],
            error_message: None,
        }
    }
}

impl EnvMapLoaderData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Face names for UI display
    pub const FACE_NAMES: [&'static str; 6] = [
        "+X (Right)",
        "-X (Left)",
        "+Y (Top)",
        "-Y (Bottom)",
        "+Z (Front)",
        "-Z (Back)",
    ];

    /// Load all faces from disk
    pub fn load_faces(&mut self) {
        self.error_message = None;
        self.face_data = [None, None, None, None, None, None];
        self.face_size = None;

        let mut loaded_size: Option<u32> = None;

        for (i, path_opt) in self.face_paths.iter().enumerate() {
            let Some(path) = path_opt else {
                continue;
            };

            match Self::load_face(path) {
                Ok((data, width, height)) => {
                    // Verify square dimensions
                    if width != height {
                        self.error_message = Some(format!(
                            "Face {} is not square: {}x{} (must be square for cubemap)",
                            Self::FACE_NAMES[i],
                            width,
                            height
                        ));
                        return;
                    }

                    // Verify all faces same size
                    if let Some(size) = loaded_size {
                        if width != size {
                            self.error_message = Some(format!(
                                "Face {} size mismatch: {}x{} (expected {}x{})",
                                Self::FACE_NAMES[i],
                                width,
                                height,
                                size,
                                size
                            ));
                            return;
                        }
                    } else {
                        loaded_size = Some(width);
                    }

                    self.face_data[i] = Some(data);
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load face {}: {}", Self::FACE_NAMES[i], e));
                    return;
                }
            }
        }

        // Check if at least one face loaded
        if self.face_data.iter().all(|f| f.is_none()) {
            self.error_message = Some("No faces loaded (select at least one image)".to_string());
            return;
        }

        // Check if all loaded faces are present (no gaps)
        let mut found_none = false;
        for (i, face) in self.face_data.iter().enumerate() {
            if face.is_none() {
                found_none = true;
            } else if found_none {
                self.error_message = Some(format!(
                    "Missing face {} (all 6 faces must be loaded)",
                    Self::FACE_NAMES[i]
                ));
                return;
            }
        }

        self.face_size = loaded_size;
    }

    /// Load a single face image from disk
    fn load_face(path: &PathBuf) -> Result<(Vec<u8>, u32, u32), String> {
        let img = image::open(path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        Ok((rgba.into_raw(), width, height))
    }

    /// Check if all 6 faces are loaded
    pub fn all_faces_loaded(&self) -> bool {
        self.face_data.iter().all(|f| f.is_some()) && self.face_size.is_some()
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.face_data
            .iter()
            .map(|f| f.as_ref().map(|d| d.len()).unwrap_or(0))
            .sum()
    }
}

/// Create component specification
pub fn spec() -> ComponentSpec {
    let mut spec = ComponentSpec::new_builtin(
        "builtin:graphics:envmap-loader".to_string(),
        "Environment Map Loader".to_string(),
        "Load cubemap environment maps from 6 image files (one per face)".to_string(),
        Some("Graphics".to_string()),
    );

    // No inputs - files are selected via UI

    // Outputs
    spec.output_spec = vec![
        PortSpec {
            name: "cubemap".to_string(),
            data_type: DataType::Binary, // TODO: Add Cubemap data type
            optional: false,
            description: "Loaded cubemap data (6 faces, RGBA8)".to_string(),
        },
        PortSpec {
            name: "size".to_string(),
            data_type: DataType::U32,
            optional: false,
            description: "Cubemap face size in pixels (square)".to_string(),
        },
    ];

    spec
}

/// Execute environment map loader
pub fn execute(
    data: &mut EnvMapLoaderData,
    _inputs: &HashMap<String, NodeValue>,
) -> Result<HashMap<String, NodeValue>, ComponentError> {
    let mut outputs = HashMap::new();

    if let Some(size) = data.face_size {
        outputs.insert("size".to_string(), NodeValue::U32(size));

        // Output cubemap as binary data (packed 6 faces)
        if data.all_faces_loaded() {
            let mut cubemap_data = Vec::new();
            for face in &data.face_data {
                if let Some(face_pixels) = face {
                    cubemap_data.extend_from_slice(face_pixels);
                }
            }
            outputs.insert("cubemap".to_string(), NodeValue::Binary(cubemap_data));
        }
    }

    Ok(outputs)
}

/// Register environment map loader node
pub fn register_envmap_loader_node(
    registry: &mut ComponentRegistry,
) -> Result<(), ComponentError> {
    let spec = spec().with_footer_view(std::sync::Arc::new(EnvMapLoaderFooterView::new()));
    registry.register_component(spec)?;
    log::info!("Registered envmap-loader builtin node");
    Ok(())
}

/// Footer view for Environment Map Loader Node
pub struct EnvMapLoaderFooterView;

impl EnvMapLoaderFooterView {
    pub fn new() -> Self {
        Self
    }
}

impl ComponentFooterView for EnvMapLoaderFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        let Some(loader_data) = &node.envmap_loader_data else {
            return Err("No environment map loader data found".to_string());
        };

        ui.vertical(|ui| {
            ui.label(RichText::new("Environment Map Loader").strong());
            ui.add_space(4.0);

            // File selection status
            let loaded_count = loader_data.face_paths.iter().filter(|p| p.is_some()).count();
            ui.label(format!("Loaded faces: {}/6", loaded_count));

            if loader_data.all_faces_loaded() {
                if let Some(size) = loader_data.face_size {
                    ui.label(format!("Face size: {}×{} pixels", size, size));
                    ui.label(format!("Memory: {:.2} MB", loader_data.memory_usage() as f32 / (1024.0 * 1024.0)));
                    ui.colored_label(Color32::GREEN, "✓ All faces loaded");
                }
            } else {
                ui.colored_label(Color32::YELLOW, "⚠ Load all 6 faces to output cubemap");
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
                "Face order: +X, -X, +Y, -Y, +Z, -Z\nSupported formats: PNG, JPG, BMP, GIF\nAll faces must be square and same size",
            );
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_data() {
        let data = EnvMapLoaderData::default();
        assert!(data.face_paths.iter().all(|p| p.is_none()));
        assert!(data.face_size.is_none());
        assert!(!data.all_faces_loaded());
    }

    #[test]
    fn test_face_names() {
        assert_eq!(EnvMapLoaderData::FACE_NAMES.len(), 6);
        assert_eq!(EnvMapLoaderData::FACE_NAMES[0], "+X (Right)");
        assert_eq!(EnvMapLoaderData::FACE_NAMES[5], "-Z (Back)");
    }

    #[test]
    fn test_memory_usage_empty() {
        let data = EnvMapLoaderData::default();
        assert_eq!(data.memory_usage(), 0);
    }

    #[test]
    fn test_envmap_spec() {
        let spec = spec();
        assert_eq!(spec.name, "Environment Map Loader");
        assert_eq!(spec.component_id, "builtin:graphics:envmap-loader");
        assert_eq!(spec.output_spec.len(), 2);
        assert_eq!(spec.output_spec[0].name, "cubemap");
        assert_eq!(spec.output_spec[1].name, "size");
    }

    #[test]
    fn test_execute_no_faces() {
        let mut data = EnvMapLoaderData::new();
        let inputs = HashMap::new();
        let outputs = execute(&mut data, &inputs).unwrap();
        assert!(outputs.is_empty());
    }
}
