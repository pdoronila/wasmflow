//! GPU Integration Module
//!
//! This module provides WebGPU integration for real-time shader rendering.
//! Phase 2: Implements shader compilation, buffer management, and GPU rendering.

pub mod buffer;
pub mod context;
pub mod pipeline;
pub mod shader;
pub mod texture;

pub use buffer::{
    BufferError, CameraUniforms, GeometryBuffers, GpuBuffer, LightUniforms, MaterialUniforms,
};
pub use context::{GpuContext, GpuError, GpuInitResult};
pub use pipeline::{PipelineBuilder, PipelineConfig, PipelineError, RenderPipeline};
pub use shader::{CompiledShader, CompilationResult, ShaderCompilationError, ShaderStage};
pub use texture::{
    generate_checker, generate_gradient, generate_solid_color, GpuTexture, TextureError,
};

/// Initialize GPU context asynchronously
pub fn init_gpu_async() -> GpuInitResult {
    pollster::block_on(GpuContext::new())
}
