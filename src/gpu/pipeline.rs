//! Render Pipeline Management
//!
//! Handles creation and management of GPU render pipelines, combining shaders,
//! vertex layouts, and bind groups for rendering.

use super::{CompiledShader, GeometryBuffers};
use thiserror::Error;
use uuid::Uuid;

/// Render pipeline wrapper with bind group management
pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: Option<wgpu::BindGroup>,
    pub id: Uuid,
    pub label: Option<String>,
}

/// Pipeline creation errors
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Shader stage mismatch: expected {expected:?}, got {actual:?}")]
    ShaderStageMismatch {
        expected: super::ShaderStage,
        actual: super::ShaderStage,
    },

    #[error("Missing vertex shader")]
    MissingVertexShader,

    #[error("Missing fragment shader")]
    MissingFragmentShader,

    #[error("Invalid vertex layout")]
    InvalidVertexLayout,

    #[error("Bind group creation failed: {0}")]
    BindGroupError(String),
}

/// Pipeline configuration
pub struct PipelineConfig {
    pub vertex_shader: CompiledShader,
    pub fragment_shader: CompiledShader,
    pub color_format: wgpu::TextureFormat,
    pub depth_format: Option<wgpu::TextureFormat>,
    pub sample_count: u32,
    pub label: Option<String>,
}

impl RenderPipeline {
    /// Create a new render pipeline from compiled shaders
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `config` - Pipeline configuration with shaders and formats
    ///
    /// # Returns
    /// Complete render pipeline ready for drawing
    pub fn new(device: &wgpu::Device, config: PipelineConfig) -> Result<Self, PipelineError> {
        // Validate shader stages
        if config.vertex_shader.stage != super::ShaderStage::Vertex {
            return Err(PipelineError::ShaderStageMismatch {
                expected: super::ShaderStage::Vertex,
                actual: config.vertex_shader.stage,
            });
        }

        if config.fragment_shader.stage != super::ShaderStage::Fragment {
            return Err(PipelineError::ShaderStageMismatch {
                expected: super::ShaderStage::Fragment,
                actual: config.fragment_shader.stage,
            });
        }

        log::info!(
            "Creating render pipeline: {}",
            config.label.as_deref().unwrap_or("unnamed")
        );

        // Create bind group layout (uniforms: camera, material, lights)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Pipeline Bind Group Layout"),
            entries: &[
                // Binding 0: Camera uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 1: Material uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 2: Light uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: config.label.as_deref(),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &config.vertex_shader.module,
                entry_point: &config.vertex_shader.entry_point,
                buffers: &[GeometryBuffers::vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &config.fragment_shader.module,
                entry_point: &config.fragment_shader.entry_point,
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.color_format,
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
            depth_stencil: config.depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: config.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        log::info!("Render pipeline created successfully");

        Ok(RenderPipeline {
            pipeline,
            bind_group_layout,
            bind_group: None,
            id: Uuid::new_v4(),
            label: config.label,
        })
    }

    /// Create bind group for uniforms
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `camera_buffer` - Camera uniform buffer
    /// * `material_buffer` - Material uniform buffer
    /// * `light_buffer` - Light uniform buffer
    pub fn create_bind_group(
        &mut self,
        device: &wgpu::Device,
        camera_buffer: &wgpu::Buffer,
        material_buffer: &wgpu::Buffer,
        light_buffer: &wgpu::Buffer,
    ) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Pipeline Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
        }));

        log::debug!("Bind group created for pipeline");
    }

    /// Get pipeline statistics for display
    pub fn stats(&self) -> String {
        format!(
            "Pipeline ID: {}\nLabel: {}\nBind Group: {}",
            self.id,
            self.label.as_deref().unwrap_or("unnamed"),
            if self.bind_group.is_some() {
                "Created"
            } else {
                "Not created"
            }
        )
    }
}

/// Pipeline builder for flexible pipeline configuration
pub struct PipelineBuilder {
    vertex_shader: Option<CompiledShader>,
    fragment_shader: Option<CompiledShader>,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    sample_count: u32,
    label: Option<String>,
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            depth_format: Some(wgpu::TextureFormat::Depth32Float),
            sample_count: 1,
            label: None,
        }
    }
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set vertex shader
    pub fn vertex_shader(mut self, shader: CompiledShader) -> Self {
        self.vertex_shader = Some(shader);
        self
    }

    /// Set fragment shader
    pub fn fragment_shader(mut self, shader: CompiledShader) -> Self {
        self.fragment_shader = Some(shader);
        self
    }

    /// Set color format
    pub fn color_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.color_format = format;
        self
    }

    /// Set depth format (None to disable depth testing)
    pub fn depth_format(mut self, format: Option<wgpu::TextureFormat>) -> Self {
        self.depth_format = format;
        self
    }

    /// Set MSAA sample count
    pub fn sample_count(mut self, count: u32) -> Self {
        self.sample_count = count;
        self
    }

    /// Set debug label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Build the render pipeline
    pub fn build(self, device: &wgpu::Device) -> Result<RenderPipeline, PipelineError> {
        let vertex_shader = self
            .vertex_shader
            .ok_or(PipelineError::MissingVertexShader)?;

        let fragment_shader = self
            .fragment_shader
            .ok_or(PipelineError::MissingFragmentShader)?;

        let config = PipelineConfig {
            vertex_shader,
            fragment_shader,
            color_format: self.color_format,
            depth_format: self.depth_format,
            sample_count: self.sample_count,
            label: self.label,
        };

        RenderPipeline::new(device, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::{CompiledShader, ShaderStage};

    // Helper to create test device (requires GPU)
    fn create_test_device() -> Option<wgpu::Device> {
        if std::env::var("CI").is_ok() {
            return None;
        }

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))?;

        pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Test Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .ok()
        .map(|(device, _queue)| device)
    }

    const VERTEX_SHADER: &str = r#"
        #version 450

        layout(location = 0) in vec3 position;
        layout(location = 1) in vec3 normal;
        layout(location = 2) in vec2 uv;

        layout(set = 0, binding = 0) uniform Camera {
            mat4 viewMatrix;
            mat4 projMatrix;
            vec3 cameraPos;
        };

        layout(location = 0) out vec3 fragNormal;

        void main() {
            gl_Position = projMatrix * viewMatrix * vec4(position, 1.0);
            fragNormal = normal;
        }
    "#;

    const FRAGMENT_SHADER: &str = r#"
        #version 450

        layout(location = 0) in vec3 fragNormal;
        layout(location = 0) out vec4 outColor;

        layout(set = 0, binding = 1) uniform Material {
            vec4 baseColor;
            float metallic;
            float roughness;
        };

        void main() {
            vec3 normal = normalize(fragNormal);
            float lighting = max(dot(normal, vec3(0.0, 0.0, 1.0)), 0.2);
            outColor = vec4(baseColor.rgb * lighting, baseColor.a);
        }
    "#;

    #[test]
    fn test_pipeline_builder() {
        let Some(device) = create_test_device() else {
            println!("GPU not available, skipping test");
            return;
        };

        let vertex = CompiledShader::from_glsl(&device, VERTEX_SHADER, ShaderStage::Vertex, None)
            .expect("Vertex shader compilation failed");

        let fragment =
            CompiledShader::from_glsl(&device, FRAGMENT_SHADER, ShaderStage::Fragment, None)
                .expect("Fragment shader compilation failed");

        let result = PipelineBuilder::new()
            .vertex_shader(vertex)
            .fragment_shader(fragment)
            .label("Test Pipeline")
            .build(&device);

        assert!(result.is_ok(), "Pipeline creation failed: {:?}", result.err());
    }

    #[test]
    fn test_pipeline_missing_shaders() {
        let Some(device) = create_test_device() else {
            return;
        };

        // Missing vertex shader
        let result = PipelineBuilder::new().build(&device);
        assert!(result.is_err());

        // Missing fragment shader
        let vertex = CompiledShader::from_glsl(&device, VERTEX_SHADER, ShaderStage::Vertex, None)
            .expect("Vertex shader compilation failed");

        let result = PipelineBuilder::new().vertex_shader(vertex).build(&device);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_shader_stage_mismatch() {
        let Some(device) = create_test_device() else {
            return;
        };

        // Use fragment shader as vertex shader (wrong stage)
        let wrong_vertex =
            CompiledShader::from_glsl(&device, FRAGMENT_SHADER, ShaderStage::Fragment, None)
                .expect("Shader compilation failed");

        let fragment =
            CompiledShader::from_glsl(&device, FRAGMENT_SHADER, ShaderStage::Fragment, None)
                .expect("Fragment shader compilation failed");

        let result = PipelineBuilder::new()
            .vertex_shader(wrong_vertex)
            .fragment_shader(fragment)
            .build(&device);

        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_with_msaa() {
        let Some(device) = create_test_device() else {
            return;
        };

        let vertex = CompiledShader::from_glsl(&device, VERTEX_SHADER, ShaderStage::Vertex, None)
            .expect("Vertex shader compilation failed");

        let fragment =
            CompiledShader::from_glsl(&device, FRAGMENT_SHADER, ShaderStage::Fragment, None)
                .expect("Fragment shader compilation failed");

        let result = PipelineBuilder::new()
            .vertex_shader(vertex)
            .fragment_shader(fragment)
            .sample_count(4) // 4x MSAA
            .build(&device);

        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_without_depth() {
        let Some(device) = create_test_device() else {
            return;
        };

        let vertex = CompiledShader::from_glsl(&device, VERTEX_SHADER, ShaderStage::Vertex, None)
            .expect("Vertex shader compilation failed");

        let fragment =
            CompiledShader::from_glsl(&device, FRAGMENT_SHADER, ShaderStage::Fragment, None)
                .expect("Fragment shader compilation failed");

        let result = PipelineBuilder::new()
            .vertex_shader(vertex)
            .fragment_shader(fragment)
            .depth_format(None) // Disable depth testing
            .build(&device);

        assert!(result.is_ok());
    }
}
