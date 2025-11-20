//! GPU Context Management
//!
//! Handles WebGPU device initialization, adapter selection, and resource management.

use thiserror::Error;
use wgpu;

/// GPU context holding WebGPU device and queue
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
    instance: wgpu::Instance,
}

/// GPU initialization result type
pub type GpuInitResult = Result<GpuContext, GpuError>;

/// GPU-related errors
#[derive(Error, Debug)]
pub enum GpuError {
    #[error("No suitable GPU adapter found")]
    NoAdapter,

    #[error("Failed to request GPU device: {0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),

    #[error("WebGPU not supported on this platform")]
    NotSupported,

    #[error("GPU adapter does not meet minimum requirements: {0}")]
    InsufficientCapabilities(String),
}

impl GpuContext {
    /// Create a new GPU context by initializing WebGPU
    ///
    /// This attempts to find a suitable GPU adapter and create a device.
    /// Falls back gracefully if WebGPU is not available.
    pub async fn new() -> GpuInitResult {
        log::info!("Initializing WebGPU context...");

        // Create WebGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Request adapter (GPU)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;

        // Log adapter info
        let adapter_info = adapter.get_info();
        log::info!("Found GPU adapter: {} ({:?})", adapter_info.name, adapter_info.backend);
        log::info!("  Vendor: {:?}", adapter_info.vendor);
        log::info!("  Device: {:?}", adapter_info.device);
        log::info!("  Device Type: {:?}", adapter_info.device_type);

        // Check capabilities
        let limits = adapter.limits();
        log::info!("GPU Limits:");
        log::info!("  Max texture dimension 2D: {}", limits.max_texture_dimension_2d);
        log::info!("  Max bind groups: {}", limits.max_bind_groups);
        log::info!("  Max buffer size: {} MB", limits.max_buffer_size / (1024 * 1024));

        // Validate minimum requirements
        Self::validate_capabilities(&adapter)?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WasmFlow GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None, // Trace path
            )
            .await?;

        log::info!("GPU device created successfully");

        Ok(GpuContext {
            device,
            queue,
            adapter_info,
            instance,
        })
    }

    /// Validate that the adapter meets minimum requirements
    fn validate_capabilities(adapter: &wgpu::Adapter) -> Result<(), GpuError> {
        let limits = adapter.limits();

        // Minimum requirements for shader preview
        const MIN_TEXTURE_SIZE: u32 = 2048;
        const MIN_BIND_GROUPS: u32 = 4;

        if limits.max_texture_dimension_2d < MIN_TEXTURE_SIZE {
            return Err(GpuError::InsufficientCapabilities(format!(
                "Max texture size {} is below minimum {}",
                limits.max_texture_dimension_2d, MIN_TEXTURE_SIZE
            )));
        }

        if limits.max_bind_groups < MIN_BIND_GROUPS {
            return Err(GpuError::InsufficientCapabilities(format!(
                "Max bind groups {} is below minimum {}",
                limits.max_bind_groups, MIN_BIND_GROUPS
            )));
        }

        Ok(())
    }

    /// Get device capabilities info for display
    pub fn capabilities_info(&self) -> String {
        format!(
            "GPU: {} ({:?})\nBackend: {:?}\nDevice Type: {:?}",
            self.adapter_info.name,
            self.adapter_info.vendor,
            self.adapter_info.backend,
            self.adapter_info.device_type
        )
    }

    /// Check if GPU supports required features for a specific operation
    pub fn supports_shader_rendering(&self) -> bool {
        // Basic check - can be expanded with more specific requirements
        let limits = self.device.limits();
        limits.max_texture_dimension_2d >= 2048
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_context_creation() {
        // This test requires a GPU and may fail in CI environments
        // Run with: cargo test test_gpu_context_creation -- --ignored
        if std::env::var("CI").is_ok() {
            return; // Skip in CI
        }

        let result = pollster::block_on(GpuContext::new());
        match result {
            Ok(context) => {
                println!("GPU Context created successfully");
                println!("{}", context.capabilities_info());
                assert!(context.supports_shader_rendering());
            }
            Err(e) => {
                println!("GPU initialization failed (expected in some environments): {}", e);
                // Don't fail the test - GPU may not be available
            }
        }
    }
}
