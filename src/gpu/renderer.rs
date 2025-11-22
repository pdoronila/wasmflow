//! Core Rendering System
//!
//! Handles render pass encoding, command submission, and frame rendering.

use super::{CameraUniforms, GeometryBuffers, GpuTexture, LightUniforms, MaterialUniforms, RenderPipeline};
use thiserror::Error;

/// Renderer for executing draw commands
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

/// Scene description for rendering
pub struct SceneDescription {
    pub geometry: GeometryBuffers,
    pub camera_uniforms: CameraUniforms,
    pub material_uniforms: MaterialUniforms,
    pub light_uniforms: LightUniforms,
}

/// Render target configuration
pub struct RenderTarget {
    pub color_texture: GpuTexture,
    pub depth_texture: Option<GpuTexture>,
}

/// Rendering errors
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("No bind group created for pipeline")]
    NoBindGroup,

    #[error("Render target dimensions don't match: expected {expected_width}x{expected_height}, got {actual_width}x{actual_height}")]
    DimensionMismatch {
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },

    #[error("Render pass creation failed: {0}")]
    RenderPassError(String),

    #[error("Command encoding failed: {0}")]
    CommandEncodingError(String),
}

impl Renderer {
    /// Create a new renderer
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        log::info!("Renderer initialized");
        Self { device, queue }
    }

    /// Render a scene to a render target
    ///
    /// # Arguments
    /// * `pipeline` - Render pipeline with shaders and bind groups
    /// * `scene` - Scene description with geometry and uniforms
    /// * `target` - Render target (color + optional depth)
    ///
    /// # Returns
    /// The rendered color texture
    pub fn render_frame<'a>(
        &self,
        pipeline: &mut RenderPipeline,
        scene: &SceneDescription,
        target: &'a RenderTarget,
    ) -> Result<&'a GpuTexture, RenderError> {
        // Create uniform buffers if not cached
        let camera_buffer = scene.camera_uniforms.create_buffer(&self.device)
            .map_err(|e| RenderError::CommandEncodingError(format!("Camera buffer creation failed: {}", e)))?;

        let material_buffer = scene.material_uniforms.create_buffer(&self.device)
            .map_err(|e| RenderError::CommandEncodingError(format!("Material buffer creation failed: {}", e)))?;

        let light_buffer = scene.light_uniforms.create_buffer(&self.device)
            .map_err(|e| RenderError::CommandEncodingError(format!("Light buffer creation failed: {}", e)))?;

        // Create bind group if not already created
        if pipeline.bind_group.is_none() {
            pipeline.create_bind_group(
                &self.device,
                &camera_buffer.buffer,
                &material_buffer.buffer,
                &light_buffer.buffer,
            );
        }

        let bind_group = pipeline.bind_group.as_ref().ok_or(RenderError::NoBindGroup)?;

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Begin render pass
        {
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &target.color_texture.view,
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
            };

            let depth_stencil_attachment = target.depth_texture.as_ref().map(|depth| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Set pipeline and bind group
            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);

            // Set vertex and index buffers
            render_pass.set_vertex_buffer(0, scene.geometry.vertex_buffer.buffer.slice(..));
            render_pass.set_index_buffer(
                scene.geometry.index_buffer.buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            // Draw indexed
            render_pass.draw_indexed(0..scene.geometry.index_count, 0, 0..1);
        }

        // Submit commands
        self.queue.submit(Some(encoder.finish()));

        log::debug!(
            "Frame rendered: {} vertices, {} indices",
            scene.geometry.vertex_count,
            scene.geometry.index_count
        );

        Ok(&target.color_texture)
    }

    /// Clear a render target to a specific color
    pub fn clear_target(&self, target: &RenderTarget, color: wgpu::Color) -> Result<(), RenderError> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.color_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: target.depth_texture.as_ref().map(|depth| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &depth.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    /// Get device for buffer/texture creation
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get queue for buffer updates
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

/// Helper to create a complete render target
pub fn create_render_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    sample_count: u32,
    with_depth: bool,
) -> Result<RenderTarget, super::TextureError> {
    let color_texture = GpuTexture::create_render_target(
        device,
        width,
        height,
        format,
        sample_count,
        Some("Render Target Color"),
    )?;

    let depth_texture = if with_depth {
        Some(GpuTexture::create_depth_texture(
            device,
            width,
            height,
            sample_count,
            Some("Render Target Depth"),
        )?)
    } else {
        None
    };

    Ok(RenderTarget {
        color_texture,
        depth_texture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::{CompiledShader, GpuContext, PipelineBuilder, ShaderStage};

    // Helper to create test GPU context
    fn create_test_context() -> Option<GpuContext> {
        if std::env::var("CI").is_ok() {
            return None;
        }

        pollster::block_on(GpuContext::new()).ok()
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

        layout(set = 0, binding = 2) uniform Light {
            vec3 direction;
            vec3 color;
            float intensity;
        };

        void main() {
            vec3 normal = normalize(fragNormal);
            float lighting = max(dot(normal, normalize(direction)), 0.2);
            outColor = vec4(baseColor.rgb * lighting * color * intensity, baseColor.a);
        }
    "#;

    #[test]
    fn test_renderer_creation() {
        let Some(context) = create_test_context() else {
            println!("GPU not available, skipping test");
            return;
        };

        let renderer = Renderer::new(context.device, context.queue);
        assert!(renderer.device().limits().max_texture_dimension_2d > 0);
    }

    #[test]
    fn test_render_target_creation() {
        let Some(context) = create_test_context() else {
            return;
        };

        let result = create_render_target(
            &context.device,
            800,
            600,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            1,
            true,
        );

        assert!(result.is_ok());
        let target = result.unwrap();
        assert_eq!(target.color_texture.size.width, 800);
        assert_eq!(target.color_texture.size.height, 600);
        assert!(target.depth_texture.is_some());
    }

    #[test]
    fn test_clear_target() {
        let Some(context) = create_test_context() else {
            return;
        };

        let renderer = Renderer::new(context.device, context.queue);

        let target = create_render_target(
            renderer.device(),
            256,
            256,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            1,
            false,
        )
        .unwrap();

        let result = renderer.clear_target(
            &target,
            wgpu::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_render_frame() {
        let Some(context) = create_test_context() else {
            return;
        };

        // Create renderer
        let renderer = Renderer::new(context.device, context.queue);

        // Compile shaders
        let vertex = CompiledShader::from_glsl(
            renderer.device(),
            VERTEX_SHADER,
            ShaderStage::Vertex,
            None,
        )
        .expect("Vertex shader failed");

        let fragment = CompiledShader::from_glsl(
            renderer.device(),
            FRAGMENT_SHADER,
            ShaderStage::Fragment,
            None,
        )
        .expect("Fragment shader failed");

        // Create pipeline
        let mut pipeline = PipelineBuilder::new()
            .vertex_shader(vertex)
            .fragment_shader(fragment)
            .label("Test Pipeline")
            .build(renderer.device())
            .expect("Pipeline creation failed");

        // Create geometry (simple triangle)
        let positions = vec![
            -0.5, -0.5, 0.0, // Bottom left
            0.5, -0.5, 0.0, // Bottom right
            0.0, 0.5, 0.0, // Top
        ];

        let normals = vec![
            0.0, 0.0, 1.0, // Normal pointing forward
            0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        ];

        let uvs = vec![
            0.0, 0.0, // Bottom left UV
            1.0, 0.0, // Bottom right UV
            0.5, 1.0, // Top UV
        ];

        let tangents = vec![
            1.0, 0.0, 0.0, 1.0, // Tangent pointing right, handedness = 1.0
            1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
        ];

        let indices = vec![0, 1, 2];

        let geometry = GeometryBuffers::from_geometry_data(
            renderer.device(),
            &positions,
            &normals,
            &uvs,
            &tangents,
            &indices,
        )
        .expect("Geometry creation failed");

        // Create uniforms
        let camera_uniforms = CameraUniforms::new(
            [[1.0, 0.0, 0.0, 0.0]; 4], // Identity view
            [[1.0, 0.0, 0.0, 0.0]; 4], // Identity projection
            [0.0, 0.0, 5.0],            // Camera position
        );

        let material_uniforms = MaterialUniforms::new([1.0, 0.0, 0.0, 1.0], 0.0, 0.5);

        let light_uniforms = LightUniforms::new([0.0, 0.0, -1.0], [1.0, 1.0, 1.0], 1.0);

        let scene = SceneDescription {
            geometry,
            camera_uniforms,
            material_uniforms,
            light_uniforms,
        };

        // Create render target
        let target = create_render_target(
            renderer.device(),
            512,
            512,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            1,
            true,
        )
        .expect("Render target creation failed");

        // Render frame
        let result = renderer.render_frame(&mut pipeline, &scene, &target);

        assert!(result.is_ok(), "Rendering failed: {:?}", result.err());
    }
}
