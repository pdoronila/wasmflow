//! GPU Integration Module
//!
//! This module provides WebGPU integration for real-time shader rendering.
//! Phase 2: Implements shader compilation, buffer management, and GPU rendering.

pub mod context;

pub use context::{GpuContext, GpuError, GpuInitResult};

/// Initialize GPU context asynchronously
pub fn init_gpu_async() -> GpuInitResult {
    pollster::block_on(GpuContext::new())
}
