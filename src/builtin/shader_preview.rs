//! Shader Preview Node
//!
//! A built-in node that renders 3D scenes using GPU PBR shaders.
//! Accepts geometry, camera, material, and lighting data and produces rendered output.

use crate::graph::node::{ComponentSpec, DataType, GraphNode, NodeValue, PortSpec, ShaderPreviewNodeData};
use crate::runtime::engine::NodeExecutor;
use crate::ui::component_view::ComponentFooterView;
use crate::ComponentError;
use egui::{Color32, ColorImage, RichText};
use std::collections::HashMap;
use wgpu;

/// Create component specification
pub fn spec() -> ComponentSpec {
    let mut spec = ComponentSpec::new_builtin(
        "builtin:graphics:shader-preview".to_string(),
        "Shader Preview".to_string(),
        "GPU shader preview and rendering with PBR support".to_string(),
        Some("Graphics".to_string()),
    );

    // Define inputs - matching what the demo expects
    spec.input_spec = vec![
        // Geometry inputs
        PortSpec {
            name: "positions".to_string(),
            data_type: DataType::List(Box::new(DataType::F32)),
            optional: true,
            description: "Vertex positions (flat list of x,y,z)".to_string(),
        },
        PortSpec {
            name: "normals".to_string(),
            data_type: DataType::List(Box::new(DataType::F32)),
            optional: true,
            description: "Vertex normals (flat list of x,y,z)".to_string(),
        },
        PortSpec {
            name: "uvs".to_string(),
            data_type: DataType::List(Box::new(DataType::F32)),
            optional: true,
            description: "UV coordinates (flat list of u,v pairs)".to_string(),
        },
        PortSpec {
            name: "tangents".to_string(),
            data_type: DataType::List(Box::new(DataType::F32)),
            optional: true,
            description: "Tangent vectors (flat list of x,y,z,w where w is handedness)".to_string(),
        },
        PortSpec {
            name: "indices".to_string(),
            data_type: DataType::List(Box::new(DataType::U32)),
            optional: true,
            description: "Triangle indices".to_string(),
        },
        // Camera inputs
        PortSpec {
            name: "view_matrix".to_string(),
            data_type: DataType::Mat4,
            optional: true,
            description: "Camera view matrix".to_string(),
        },
        PortSpec {
            name: "projection_matrix".to_string(),
            data_type: DataType::Mat4,
            optional: true,
            description: "Projection matrix".to_string(),
        },
        // Material inputs
        PortSpec {
            name: "base_color".to_string(),
            data_type: DataType::Vec3,
            optional: true,
            description: "Material base color".to_string(),
        },
        PortSpec {
            name: "metallic".to_string(),
            data_type: DataType::F32,
            optional: true,
            description: "Material metallic (0.0-1.0)".to_string(),
        },
        PortSpec {
            name: "roughness".to_string(),
            data_type: DataType::F32,
            optional: true,
            description: "Material roughness (0.0-1.0)".to_string(),
        },
        // Lighting inputs
        PortSpec {
            name: "light_data".to_string(),
            data_type: DataType::String,
            optional: true,
            description: "Light data JSON string".to_string(),
        },
        // Display controls
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

/// Execute the shader preview node (processes inputs, caches data)
pub fn execute(
    node_data: &mut ShaderPreviewNodeData,
    inputs: &HashMap<String, NodeValue>,
) -> Result<HashMap<String, NodeValue>, ComponentError> {
    // Log what we received
    log::info!("Shader preview execute() called with {} inputs", inputs.len());
    for (key, value) in inputs {
        let type_name = match value {
            NodeValue::String(_) => "String",
            NodeValue::F32(_) => "F32",
            NodeValue::U32(_) => "U32",
            NodeValue::I32(_) => "I32",
            NodeValue::Bool(_) => "Bool",
            NodeValue::List(_) => "List",
            NodeValue::Record(_) => "Record",
            NodeValue::Vec2(_) => "Vec2",
            NodeValue::Vec3(_) => "Vec3",
            NodeValue::Vec4(_) => "Vec4",
            NodeValue::Mat4(_) => "Mat4",
            NodeValue::Binary(_) => "Binary",
            NodeValue::Texture(_) => "Texture",
        };
        log::info!("  Input '{}': {}", key, type_name);
    }

    // Update zoom if provided
    if let Some(NodeValue::F32(zoom_val)) = inputs.get("zoom") {
        node_data.zoom = zoom_val.max(0.1).min(10.0);
    }

    // Cache all rendering inputs for the footer view
    node_data.cached_positions = extract_f32_list(inputs, "positions");
    node_data.cached_normals = extract_f32_list(inputs, "normals");
    node_data.cached_uvs = extract_f32_list(inputs, "uvs");
    node_data.cached_tangents = extract_f32_list(inputs, "tangents");
    node_data.cached_indices = extract_u32_list(inputs, "indices");
    node_data.cached_view_matrix = extract_f32_list(inputs, "view_matrix");
    node_data.cached_projection_matrix = extract_f32_list(inputs, "projection_matrix");
    node_data.cached_base_color = extract_f32_list(inputs, "base_color");
    node_data.cached_metallic = extract_f32(inputs, "metallic");
    node_data.cached_roughness = extract_f32(inputs, "roughness");
    node_data.cached_light_data = extract_string(inputs, "light_data");

    // Mark that we have new data
    if node_data.has_complete_scene_data() {
        node_data.needs_rerender = true;
        log::info!("✓ Shader preview received complete scene data, flagging for rerender");
        log::debug!("  - positions: {}", node_data.cached_positions.as_ref().map(|v| v.len()).unwrap_or(0));
        log::debug!("  - normals: {}", node_data.cached_normals.as_ref().map(|v| v.len()).unwrap_or(0));
        log::debug!("  - indices: {}", node_data.cached_indices.as_ref().map(|v| v.len()).unwrap_or(0));
    } else {
        log::warn!("Shader preview missing required data:");
        log::warn!("  - positions: {}", node_data.cached_positions.is_some());
        log::warn!("  - normals: {}", node_data.cached_normals.is_some());
        log::warn!("  - uvs: {}", node_data.cached_uvs.is_some());
        log::warn!("  - indices: {}", node_data.cached_indices.is_some());
        log::warn!("  - view_matrix: {}", node_data.cached_view_matrix.is_some());
        log::warn!("  - projection_matrix: {}", node_data.cached_projection_matrix.is_some());
        log::warn!("  - base_color: {}", node_data.cached_base_color.is_some());
    }

    Ok(HashMap::new())
}

/// NodeExecutor implementation for graph execution
pub struct ShaderPreviewExecutor;

impl NodeExecutor for ShaderPreviewExecutor {
    fn execute(
        &self,
        inputs: &HashMap<String, NodeValue>,
    ) -> Result<HashMap<String, NodeValue>, ComponentError> {
        // For the NodeExecutor, we just validate that we have inputs
        // The actual caching happens in a post-execution step by the engine
        // which calls the execute() function above with access to the node data

        log::debug!("ShaderPreviewExecutor::execute called with {} inputs", inputs.len());

        // Return inputs as outputs so they're available to the engine
        // This allows the engine to update the node's shader_preview_data
        Ok(inputs.clone())
    }
}

// Helper functions to extract data from inputs
fn extract_f32_list(inputs: &HashMap<String, NodeValue>, key: &str) -> Option<Vec<f32>> {
    match inputs.get(key) {
        Some(NodeValue::List(list)) => {
            // Log what's in the list
            if list.len() > 0 {
                let first_type = match &list[0] {
                    NodeValue::String(_) => "String",
                    NodeValue::F32(_) => "F32",
                    NodeValue::U32(_) => "U32",
                    NodeValue::I32(_) => "I32",
                    NodeValue::Bool(_) => "Bool",
                    NodeValue::List(_) => "List",
                    NodeValue::Record(_) => "Record",
                    NodeValue::Vec2(_) => "Vec2",
                    NodeValue::Vec3(_) => "Vec3",
                    NodeValue::Vec4(_) => "Vec4",
                    NodeValue::Mat4(_) => "Mat4",
                    NodeValue::Binary(_) => "Binary",
                    NodeValue::Texture(_) => "Texture",
                };
                log::info!("  List '{}' has {} items, first item type: {}", key, list.len(), first_type);

                // If it's a string, log the first few characters
                if let NodeValue::String(s) = &list[0] {
                    log::info!("    First item value: {}", &s[..s.len().min(100)]);
                }
            }

            // Extract f32 values from list
            let mut result = Vec::with_capacity(list.len());
            for value in list {
                match value {
                    NodeValue::F32(f) => result.push(*f),
                    _ => return None, // Not all values are f32
                }
            }
            Some(result)
        }
        Some(NodeValue::Vec3(vec3)) => {
            // Also handle Vec3 inputs (e.g., from pbr-material base_color)
            Some(vec![vec3.x, vec3.y, vec3.z])
        }
        Some(NodeValue::Mat4(mat)) => {
            // Also handle Mat4 inputs (e.g., from perspective-camera)
            // Column-major order for GLSL compatibility
            // m00-m03 = column 0, m10-m13 = column 1, m20-m23 = column 2, m30-m33 = column 3
            Some(vec![
                mat.m00, mat.m01, mat.m02, mat.m03,  // Column 0
                mat.m10, mat.m11, mat.m12, mat.m13,  // Column 1
                mat.m20, mat.m21, mat.m22, mat.m23,  // Column 2
                mat.m30, mat.m31, mat.m32, mat.m33,  // Column 3
            ])
        }
        _ => None,
    }
}

fn extract_u32_list(inputs: &HashMap<String, NodeValue>, key: &str) -> Option<Vec<u32>> {
    match inputs.get(key) {
        Some(NodeValue::List(list)) => {
            // Log what's in the list
            if list.len() > 0 {
                let first_type = match &list[0] {
                    NodeValue::String(_) => "String",
                    NodeValue::F32(_) => "F32",
                    NodeValue::U32(_) => "U32",
                    NodeValue::I32(_) => "I32",
                    NodeValue::Bool(_) => "Bool",
                    NodeValue::List(_) => "List",
                    NodeValue::Record(_) => "Record",
                    NodeValue::Vec2(_) => "Vec2",
                    NodeValue::Vec3(_) => "Vec3",
                    NodeValue::Vec4(_) => "Vec4",
                    NodeValue::Mat4(_) => "Mat4",
                    NodeValue::Binary(_) => "Binary",
                    NodeValue::Texture(_) => "Texture",
                };
                log::info!("  List '{}' has {} items, first item type: {}", key, list.len(), first_type);
            }

            // Extract u32 values from list
            let mut result = Vec::with_capacity(list.len());
            for value in list {
                match value {
                    NodeValue::U32(u) => result.push(*u),
                    _ => return None, // Not all values are u32
                }
            }
            Some(result)
        }
        _ => None,
    }
}

fn extract_f32(inputs: &HashMap<String, NodeValue>, key: &str) -> Option<f32> {
    match inputs.get(key) {
        Some(NodeValue::F32(val)) => Some(*val),
        _ => None,
    }
}

fn extract_string(inputs: &HashMap<String, NodeValue>, key: &str) -> Option<String> {
    match inputs.get(key) {
        Some(NodeValue::String(s)) => Some(s.clone()),
        _ => None,
    }
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
        let preview_data = match &mut node.shader_preview_data {
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
                ui.heading(RichText::new("🎨 Shader Preview").color(Color32::WHITE));
                if preview_data.has_complete_scene_data() {
                    ui.label(RichText::new("● Ready").color(Color32::GREEN));
                } else {
                    ui.label(RichText::new("○ Waiting for inputs").color(Color32::GRAY));
                }
            });

            ui.add_space(8.0);

            // Preview area
            ui.group(|ui| {
                let preview_width = preview_data.preview_size.0 as f32 * preview_data.zoom;
                let preview_height = preview_data.preview_size.1 as f32 * preview_data.zoom;

                ui.set_min_height(preview_height.min(600.0));
                ui.set_min_width(preview_width.min(800.0));

                // Render if we have complete data
                if preview_data.has_complete_scene_data() && preview_data.needs_rerender {
                    log::info!("🎨 Attempting to render scene to texture...");
                    // Attempt GPU rendering
                    match render_scene_to_texture(ui, preview_data) {
                        Ok(()) => {
                            preview_data.needs_rerender = false;
                            preview_data.last_update = Some(std::time::Instant::now());
                            log::info!("✓ Successfully rendered scene to texture");
                        }
                        Err(e) => {
                            preview_data.render_error = Some(e.clone());
                            log::error!("✗ GPU rendering failed: {}", e);
                        }
                    }
                } else if !preview_data.has_complete_scene_data() {
                    log::debug!("Footer view called but scene data incomplete");
                } else if !preview_data.needs_rerender {
                    log::trace!("Footer view called but rerender not needed");
                }

                // Display the rendered texture or placeholder
                if let Some(texture_id) = preview_data.gpu_texture_id {
                    ui.vertical_centered(|ui| {
                        ui.image(egui::load::SizedTexture::new(
                            texture_id,
                            egui::vec2(preview_width, preview_height),
                        ));
                    });
                } else {
                    // No texture available - show placeholder or error
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);

                        if let Some(error) = &preview_data.render_error {
                            ui.label(
                                RichText::new("⚠")
                                    .size(64.0)
                                    .color(Color32::from_rgb(255, 100, 100)),
                            );
                            ui.add_space(16.0);
                            ui.label(RichText::new("Rendering Error").size(16.0).color(Color32::RED));
                            ui.add_space(8.0);
                            ui.label(RichText::new(error).size(12.0).color(Color32::GRAY));
                        } else {
                            ui.label(
                                RichText::new("🖼")
                                    .size(64.0)
                                    .color(Color32::from_gray(100)),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new("Waiting for Scene Data")
                                    .size(16.0)
                                    .color(Color32::GRAY),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("Connect geometry, camera, material, and lights")
                                    .size(12.0)
                                    .color(Color32::DARK_GRAY),
                            );
                        }
                    });
                }
            });

            ui.add_space(8.0);

            // Controls
            ui.horizontal(|ui| {
                ui.label("Size:");
                if ui
                    .selectable_label(preview_data.preview_size == (400, 300), "Small")
                    .clicked()
                {
                    preview_data.preview_size = (400, 300);
                    preview_data.needs_rerender = true;
                }
                if ui
                    .selectable_label(preview_data.preview_size == (600, 450), "Medium")
                    .clicked()
                {
                    preview_data.preview_size = (600, 450);
                    preview_data.needs_rerender = true;
                }
                if ui
                    .selectable_label(preview_data.preview_size == (800, 600), "Large")
                    .clicked()
                {
                    preview_data.preview_size = (800, 600);
                    preview_data.needs_rerender = true;
                }
                if ui
                    .selectable_label(preview_data.preview_size == (1200, 900), "XL")
                    .clicked()
                {
                    preview_data.preview_size = (1200, 900);
                    preview_data.needs_rerender = true;
                }
                if ui
                    .selectable_label(preview_data.preview_size == (1600, 1200), "XXL")
                    .clicked()
                {
                    preview_data.preview_size = (1600, 1200);
                    preview_data.needs_rerender = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Zoom:");
                if ui.add(egui::Slider::new(&mut preview_data.zoom, 0.1..=10.0).suffix("x")).changed() {
                    // Zoom doesn't require rerender, just display scaling
                }
            });

            if ui.button("🔄 Force Render").clicked() {
                preview_data.needs_rerender = true;
            }

            ui.add_space(4.0);

            // Stats
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Stats:").strong());

                if let Some(update_time) = preview_data.last_update {
                    let elapsed = update_time.elapsed().as_secs_f32();
                    ui.label(format!("Last render: {:.2}s ago", elapsed));
                } else {
                    ui.label("No renders yet");
                }

                // Vertex count
                if let Some(positions) = &preview_data.cached_positions {
                    let vertex_count = positions.len() / 3;
                    ui.label(format!("Vertices: {}", vertex_count));
                }

                // Triangle count
                if let Some(indices) = &preview_data.cached_indices {
                    let triangle_count = indices.len() / 3;
                    ui.label(format!("Triangles: {}", triangle_count));
                }
            });
        });

        Ok(())
    }
}

/// Render the scene to a texture using GPU
fn render_scene_to_texture(
    ui: &mut egui::Ui,
    preview_data: &mut ShaderPreviewNodeData,
) -> Result<(), String> {
    log::info!("→ render_scene_to_texture() called");

    // Initialize GPU context if needed
    if preview_data.gpu_context.is_none() {
        log::info!("  Initializing GPU context for shader preview...");
        let context = pollster::block_on(crate::gpu::context::GpuContext::new())
            .map_err(|e| format!("Failed to initialize GPU: {}", e))?;
        preview_data.gpu_context = Some(context);
        log::info!("  ✓ GPU context initialized successfully");
    } else {
        log::debug!("  GPU context already initialized");
    }

    // Compile shaders if needed (before we borrow gpu_context)
    if preview_data.render_pipeline.is_none() {
        log::info!("Compiling PBR shaders and creating render pipeline...");

        let gpu_context = preview_data
            .gpu_context
            .as_ref()
            .ok_or("GPU context not initialized")?;

        // Read embedded GLSL shaders
        let vertex_glsl = include_str!("../../examples/shaders/pbr/pbr_single_light.vert.glsl");
        let fragment_glsl = include_str!("../../examples/shaders/pbr/pbr_single_light.frag.glsl");

        // Compile shaders to WGSL
        let vertex_shader = compile_glsl_to_wgsl(vertex_glsl, naga::ShaderStage::Vertex)
            .map_err(|e| format!("Vertex shader compilation failed: {}", e))?;
        let fragment_shader = compile_glsl_to_wgsl(fragment_glsl, naga::ShaderStage::Fragment)
            .map_err(|e| format!("Fragment shader compilation failed: {}", e))?;

        // Create shader modules
        let vs_module = gpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PBR Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(vertex_shader.into()),
        });

        let fs_module = gpu_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PBR Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(fragment_shader.into()),
        });

        // Create bind group layouts
        let camera_bind_group_layout =
            gpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let material_bind_group_layout =
            gpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Material Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let light_bind_group_layout =
            gpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create pipeline layout
        let pipeline_layout = gpu_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PBR Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &material_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = gpu_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PBR Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[
                    // Vertex buffer layout
                    wgpu::VertexBufferLayout {
                        array_stride: 44, // 3 floats (pos) + 3 floats (normal) + 2 floats (uv) + 3 floats (tangent) = 11 floats * 4 bytes
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0, // position
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 12,
                                shader_location: 1, // normal
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 24,
                                shader_location: 2, // uv
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 32,
                                shader_location: 3, // tangent
                            },
                        ],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        preview_data.render_pipeline = Some(pipeline);
        preview_data.camera_bind_group_layout = Some(camera_bind_group_layout);
        preview_data.material_bind_group_layout = Some(material_bind_group_layout);
        preview_data.light_bind_group_layout = Some(light_bind_group_layout);

        log::info!("Render pipeline created successfully");
    }

    // Create/update vertex buffer if geometry changed
    if preview_data.vertex_buffer.is_none() || preview_data.needs_buffer_update {
        let gpu_context = preview_data
            .gpu_context
            .as_ref()
            .ok_or("GPU context not initialized")?;

        let positions = preview_data.cached_positions.as_ref().ok_or("Missing positions")?;
        let normals = preview_data.cached_normals.as_ref().ok_or("Missing normals")?;
        let uvs = preview_data.cached_uvs.as_ref().ok_or("Missing UVs")?;
        let tangents = preview_data.cached_tangents.as_ref().ok_or("Missing tangents")?;
        let indices = preview_data.cached_indices.as_ref().ok_or("Missing indices")?;

        log::info!("  Creating vertex and index buffers...");
        create_vertex_and_index_buffers(
            gpu_context,
            positions,
            normals,
            uvs,
            tangents,
            indices,
            &mut preview_data.vertex_buffer,
            &mut preview_data.index_buffer,
            &mut preview_data.index_count,
        )?;
        preview_data.needs_buffer_update = false;
        log::info!("  ✓ Buffers created: {} vertices, {} indices", positions.len() / 3, indices.len());
    }

    // Create/update uniform buffers
    {
        let gpu_context = preview_data
            .gpu_context
            .as_ref()
            .ok_or("GPU context not initialized")?;

        let view_matrix = preview_data.cached_view_matrix.as_ref().ok_or("Missing view matrix")?;
        let projection_matrix = preview_data.cached_projection_matrix.as_ref().ok_or("Missing projection matrix")?;
        let base_color = preview_data.cached_base_color.as_ref().ok_or("Missing base color")?;
        let metallic = preview_data.cached_metallic.unwrap_or(0.0);
        let roughness = preview_data.cached_roughness.unwrap_or(0.5);

        let camera_layout = preview_data.camera_bind_group_layout.as_ref().ok_or("Missing camera layout")?;
        let material_layout = preview_data.material_bind_group_layout.as_ref().ok_or("Missing material layout")?;
        let light_layout = preview_data.light_bind_group_layout.as_ref().ok_or("Missing light layout")?;

        log::info!("  Creating uniform buffers (camera, material, light)...");
        create_uniform_buffers(
            gpu_context,
            view_matrix,
            projection_matrix,
            base_color,
            metallic,
            roughness,
            camera_layout,
            material_layout,
            light_layout,
            &mut preview_data.camera_bind_group,
            &mut preview_data.material_bind_group,
            &mut preview_data.light_bind_group,
        )?;
        log::info!("  ✓ Uniform buffers created");
    }

    // Render to texture
    let (width, height) = preview_data.preview_size;
    log::info!("  Rendering to {}x{} texture...", width, height);
    {
        let gpu_context = preview_data
            .gpu_context
            .as_ref()
            .ok_or("GPU context not initialized")?;

        let render_pipeline = preview_data.render_pipeline.as_ref().ok_or("Missing render pipeline")?;
        let camera_bind_group = preview_data.camera_bind_group.as_ref().ok_or("Missing camera bind group")?;
        let material_bind_group = preview_data.material_bind_group.as_ref().ok_or("Missing material bind group")?;
        let light_bind_group = preview_data.light_bind_group.as_ref().ok_or("Missing light bind group")?;
        let vertex_buffer = preview_data.vertex_buffer.as_ref().ok_or("Missing vertex buffer")?;
        let index_buffer = preview_data.index_buffer.as_ref().ok_or("Missing index buffer")?;
        let index_count = preview_data.index_count;

        render_to_texture(
            gpu_context,
            render_pipeline,
            camera_bind_group,
            material_bind_group,
            light_bind_group,
            vertex_buffer,
            index_buffer,
            index_count,
            width,
            height,
            &mut preview_data.gpu_texture_id,
            &mut preview_data.gpu_texture_handle,
            ui,
        )?;
        log::info!("  ✓ Texture rendered successfully, uploaded to egui");
    }

    log::info!("← render_scene_to_texture() completed successfully");
    Ok(())
}

/// Compile GLSL to WGSL using naga
fn compile_glsl_to_wgsl(glsl_source: &str, stage: naga::ShaderStage) -> Result<String, String> {
    use naga::front::glsl::{Frontend, Options};
    use naga::back::wgsl;

    let mut frontend = Frontend::default();
    let options = Options::from(stage);

    let module = frontend
        .parse(&options, glsl_source)
        .map_err(|errors| format!("GLSL parse error: {:?}", errors))?;

    // Validate module
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|e| format!("Shader validation error: {}", e))?;

    // Generate WGSL
    let wgsl_source = wgsl::write_string(&module, &info, wgsl::WriterFlags::empty())
        .map_err(|e| format!("WGSL generation error: {}", e))?;

    Ok(wgsl_source)
}

/// Create vertex and index buffers from cached geometry data
fn create_vertex_and_index_buffers(
    gpu_context: &crate::gpu::context::GpuContext,
    cached_positions: &[f32],
    cached_normals: &[f32],
    cached_uvs: &[f32],
    cached_tangents: &[f32],
    cached_indices: &[u32],
    vertex_buffer: &mut Option<wgpu::Buffer>,
    index_buffer: &mut Option<wgpu::Buffer>,
    index_count: &mut u32,
) -> Result<(), String> {
    use wgpu::util::DeviceExt;

    let positions = cached_positions;
    let normals = cached_normals;
    let uvs = cached_uvs;
    let tangents = cached_tangents;
    let indices = cached_indices;

    // Interleave vertex data: pos(3) + normal(3) + uv(2) + tangent(3) = 11 floats per vertex
    let vertex_count = positions.len() / 3;
    let mut vertex_data = Vec::with_capacity(vertex_count * 11);

    for i in 0..vertex_count {
        // Position
        vertex_data.push(positions[i * 3]);
        vertex_data.push(positions[i * 3 + 1]);
        vertex_data.push(positions[i * 3 + 2]);
        // Normal
        vertex_data.push(normals[i * 3]);
        vertex_data.push(normals[i * 3 + 1]);
        vertex_data.push(normals[i * 3 + 2]);
        // UV
        vertex_data.push(uvs[i * 2]);
        vertex_data.push(uvs[i * 2 + 1]);
        // Tangent (vec3 - we use only x,y,z from the vec4 tangent data)
        vertex_data.push(tangents[i * 4]);     // x
        vertex_data.push(tangents[i * 4 + 1]); // y
        vertex_data.push(tangents[i * 4 + 2]); // z
        // Note: Ignoring w (handedness) since shader expects vec3
    }

    // Convert to bytes
    let vertex_bytes: Vec<u8> = vertex_data
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();

    let index_bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();

    // Create buffers
    let vb = gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: &vertex_bytes,
        usage: wgpu::BufferUsages::VERTEX,
    });

    let ib = gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: &index_bytes,
        usage: wgpu::BufferUsages::INDEX,
    });

    *vertex_buffer = Some(vb);
    *index_buffer = Some(ib);
    *index_count = indices.len() as u32;

    log::debug!(
        "Created vertex buffer: {} vertices, {} indices",
        vertex_count,
        indices.len()
    );

    Ok(())
}

/// Create uniform buffers for camera, material, and lights
fn create_uniform_buffers(
    gpu_context: &crate::gpu::context::GpuContext,
    view_matrix: &[f32],
    projection_matrix: &[f32],
    base_color: &[f32],
    metallic: f32,
    roughness: f32,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    light_bind_group_layout: &wgpu::BindGroupLayout,
    camera_bind_group_out: &mut Option<wgpu::BindGroup>,
    material_bind_group_out: &mut Option<wgpu::BindGroup>,
    light_bind_group_out: &mut Option<wgpu::BindGroup>,
) -> Result<(), String> {
    use wgpu::util::DeviceExt;

    // Camera uniforms: view(16) + projection(16) + camera_pos(3) + padding(1) = 36 floats

    let mut camera_data = Vec::with_capacity(36);
    camera_data.extend_from_slice(view_matrix);
    camera_data.extend_from_slice(projection_matrix);
    camera_data.extend_from_slice(&[0.0, 0.0, 10.0]); // Camera position (TODO: extract from view matrix)
    camera_data.push(0.0); // Padding

    // Debug: Log the matrices
    log::debug!("View matrix: {:?}", &view_matrix[..]);
    log::debug!("Projection matrix: {:?}", &projection_matrix[..]);

    let camera_bytes: Vec<u8> = camera_data.iter().flat_map(|f| f.to_le_bytes()).collect();

    let camera_buffer = gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Uniform Buffer"),
        contents: &camera_bytes,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Material uniforms: base_color(4) + metallic(1) + roughness(1) + ao(1) + padding(1) = 8 floats
    let material_data = vec![
        base_color.get(0).copied().unwrap_or(1.0),
        base_color.get(1).copied().unwrap_or(1.0),
        base_color.get(2).copied().unwrap_or(1.0),
        1.0, // alpha
        metallic,
        roughness,
        1.0, // ao
        0.0, // padding
    ];

    let material_bytes: Vec<u8> = material_data.iter().flat_map(|f| f.to_le_bytes()).collect();

    let material_buffer = gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: &material_bytes,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Light uniforms: direction(3) + padding(1) + color(3) + intensity(1) = 8 floats
    let light_data: Vec<f32> = vec![
        0.0,
        -1.0,
        0.0,
        0.0, // direction (pointing down) + padding
        1.0,
        1.0,
        1.0,
        1.0, // white light with intensity 1.0
    ];

    let light_bytes: Vec<u8> = light_data.iter().flat_map(|f| f.to_le_bytes()).collect();

    let light_buffer = gpu_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light Uniform Buffer"),
        contents: &light_bytes,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Create bind groups
    let camera_bind_group = gpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Camera Bind Group"),
        layout: camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
    });

    let material_bind_group = gpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Material Bind Group"),
        layout: material_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: material_buffer.as_entire_binding(),
        }],
    });

    let light_bind_group = gpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Light Bind Group"),
        layout: light_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: light_buffer.as_entire_binding(),
        }],
    });

    *camera_bind_group_out = Some(camera_bind_group);
    *material_bind_group_out = Some(material_bind_group);
    *light_bind_group_out = Some(light_bind_group);

    Ok(())
}

/// Render the scene to a texture
fn render_to_texture(
    gpu_context: &crate::gpu::context::GpuContext,
    render_pipeline: &wgpu::RenderPipeline,
    camera_bind_group: &wgpu::BindGroup,
    material_bind_group: &wgpu::BindGroup,
    light_bind_group: &wgpu::BindGroup,
    vertex_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    index_count: u32,
    width: u32,
    height: u32,
    gpu_texture_id_out: &mut Option<egui::TextureId>,
    gpu_texture_handle_out: &mut Option<egui::TextureHandle>,
    ui: &mut egui::Ui,
) -> Result<(), String> {
    // Create render texture
    let texture_descriptor = wgpu::TextureDescriptor {
        label: Some("Render Target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    };

    let render_texture = gpu_context.device.create_texture(&texture_descriptor);
    let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create depth texture
    let depth_texture = gpu_context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create command encoder
    let mut encoder = gpu_context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    // Begin render pass
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.1,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, material_bind_group, &[]);
        render_pass.set_bind_group(2, light_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..index_count, 0, 0..1);
    }

    // Copy texture to buffer for readback
    let bytes_per_pixel = 4;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;

    let buffer_size = (padded_bytes_per_row * height) as u64;
    let staging_buffer = gpu_context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &render_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &staging_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    // Submit commands
    gpu_context.queue.submit(std::iter::once(encoder.finish()));

    // Read back texture data
    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });

    gpu_context.device.poll(wgpu::Maintain::Wait);

    receiver
        .recv()
        .map_err(|e| format!("Failed to receive buffer mapping result: {}", e))?
        .map_err(|e| format!("Failed to map buffer: {:?}", e))?;

    // Extract pixel data
    let data = buffer_slice.get_mapped_range();
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);

    for row in 0..height {
        let offset = (row * padded_bytes_per_row) as usize;
        let row_data = &data[offset..offset + (unpadded_bytes_per_row as usize)];
        pixels.extend_from_slice(row_data);
    }

    drop(data);
    staging_buffer.unmap();

    // Convert to egui ColorImage
    let color_pixels: Vec<egui::Color32> = pixels
        .chunks_exact(4)
        .map(|chunk| egui::Color32::from_rgba_premultiplied(chunk[0], chunk[1], chunk[2], chunk[3]))
        .collect();

    let color_image = ColorImage {
        size: [width as usize, height as usize],
        source_size: egui::vec2(width as f32, height as f32),
        pixels: color_pixels,
    };

    // Upload to egui
    let texture_handle = ui.ctx().load_texture("shader_preview_render", color_image, egui::TextureOptions::default());
    *gpu_texture_id_out = Some(texture_handle.id());
    *gpu_texture_handle_out = Some(texture_handle); // Keep handle alive!

    log::debug!("Rendered scene to texture: {}x{}", width, height);

    Ok(())
}

/// Register the shader preview node in the component registry
pub fn register_shader_preview_node(registry: &mut crate::graph::node::ComponentRegistry) {
    let spec = spec().with_footer_view(std::sync::Arc::new(ShaderPreviewFooterView::new()));
    registry.register_builtin(spec);
    log::info!("Registered Shader Preview Node with full GPU rendering support");
}
