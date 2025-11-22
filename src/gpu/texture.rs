//! GPU Texture Management
//!
//! Handles texture creation, management, and GPU upload for rendering and display.

use thiserror::Error;

/// GPU texture with view and sampler
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    pub format: wgpu::TextureFormat,
    pub label: Option<String>,
}

/// Texture creation errors
#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Invalid texture dimensions: {0}x{1}")]
    InvalidDimensions(u32, u32),

    #[error("Invalid texture data size: expected {expected}, got {actual}")]
    InvalidDataSize { expected: usize, actual: usize },

    #[error("Unsupported texture format: {0:?}")]
    UnsupportedFormat(wgpu::TextureFormat),

    #[error("Invalid MSAA sample count: {0} (must be 1, 2, 4, or 8)")]
    InvalidSampleCount(u32),
}

impl GpuTexture {
    /// Create a texture from RGBA8 data
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `queue` - WebGPU queue for data transfer
    /// * `width` - Texture width in pixels
    /// * `height` - Texture height in pixels
    /// * `data` - RGBA pixel data (width * height * 4 bytes)
    /// * `label` - Debug label
    ///
    /// # Returns
    /// GPU texture ready for sampling in shaders
    pub fn from_rgba8(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidDimensions(width, height));
        }

        let expected_size = (width * height * 4) as usize;
        if data.len() != expected_size {
            return Err(TextureError::InvalidDataSize {
                expected: expected_size,
                actual: data.len(),
            });
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::create_default_sampler(device);

        log::debug!(
            "Created RGBA8 texture: {} ({}x{})",
            label.unwrap_or("unnamed"),
            width,
            height
        );

        Ok(GpuTexture {
            texture,
            view,
            sampler,
            size,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            label: label.map(String::from),
        })
    }

    /// Create a render target texture (for rendering into)
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `width` - Target width in pixels
    /// * `height` - Target height in pixels
    /// * `format` - Color format
    /// * `sample_count` - MSAA sample count (1, 2, 4, or 8)
    /// * `label` - Debug label
    ///
    /// # Returns
    /// Render target texture for offscreen rendering
    pub fn create_render_target(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        sample_count: u32,
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidDimensions(width, height));
        }

        if !matches!(sample_count, 1 | 2 | 4 | 8) {
            return Err(TextureError::InvalidSampleCount(sample_count));
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::create_default_sampler(device);

        log::info!(
            "Created render target: {} ({}x{}, {:?}, {}x MSAA)",
            label.unwrap_or("unnamed"),
            width,
            height,
            format,
            sample_count
        );

        Ok(GpuTexture {
            texture,
            view,
            sampler,
            size,
            format,
            label: label.map(String::from),
        })
    }

    /// Create a depth texture for depth testing
    pub fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidDimensions(width, height));
        }

        if !matches!(sample_count, 1 | 2 | 4 | 8) {
            return Err(TextureError::InvalidSampleCount(sample_count));
        }

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Depth textures need a comparison sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Depth Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        log::debug!(
            "Created depth texture: {} ({}x{}, {}x MSAA)",
            label.unwrap_or("unnamed"),
            width,
            height,
            sample_count
        );

        Ok(GpuTexture {
            texture,
            view,
            sampler,
            size,
            format: wgpu::TextureFormat::Depth32Float,
            label: label.map(String::from),
        })
    }

    /// Create a texture from RGB8 data (converts to RGBA8)
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `queue` - WebGPU queue for data transfer
    /// * `width` - Texture width in pixels
    /// * `height` - Texture height in pixels
    /// * `data` - RGB pixel data (width * height * 3 bytes)
    /// * `label` - Debug label
    ///
    /// # Returns
    /// GPU texture with alpha channel set to 255 (opaque)
    pub fn from_rgb8(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let expected_size = (width * height * 3) as usize;
        if data.len() != expected_size {
            return Err(TextureError::InvalidDataSize {
                expected: expected_size,
                actual: data.len(),
            });
        }

        // Convert RGB8 to RGBA8 (add alpha channel = 255)
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
        for chunk in data.chunks_exact(3) {
            rgba_data.push(chunk[0]); // R
            rgba_data.push(chunk[1]); // G
            rgba_data.push(chunk[2]); // B
            rgba_data.push(255); // A (opaque)
        }

        Self::from_rgba8(device, queue, width, height, &rgba_data, label)
    }

    /// Create a cubemap texture from 6 RGBA8 face images
    ///
    /// Face order: +X (right), -X (left), +Y (top), -Y (bottom), +Z (front), -Z (back)
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `queue` - WebGPU queue for data transfer
    /// * `size` - Cubemap face size (must be square, all faces same size)
    /// * `face_data` - Array of 6 RGBA8 face images (each size × size × 4 bytes)
    /// * `label` - Debug label
    ///
    /// # Returns
    /// GPU cubemap texture ready for sampling in shaders
    pub fn from_cubemap_rgba8(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: u32,
        face_data: &[&[u8]; 6],
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        if size == 0 {
            return Err(TextureError::InvalidDimensions(size, size));
        }

        let expected_face_size = (size * size * 4) as usize;
        for face in face_data.iter() {
            if face.len() != expected_face_size {
                return Err(TextureError::InvalidDataSize {
                    expected: expected_face_size,
                    actual: face.len(),
                });
            }
        }

        let texture_size = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6, // 6 faces for cubemap
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload each face to its corresponding layer
        for (face_index, face) in face_data.iter().enumerate() {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face_index as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                face,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * size),
                    rows_per_image: Some(size),
                },
                wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
            );
        }

        // Create cubemap view
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Cubemap View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        // Create sampler for cubemap (typically linear filtering with clamp to edge)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Cubemap Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        log::debug!(
            "Created cubemap texture: {} ({}x{} per face)",
            label.unwrap_or("unnamed"),
            size,
            size
        );

        Ok(GpuTexture {
            texture,
            view,
            sampler,
            size: texture_size,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            label: label.map(String::from),
        })
    }

    /// Create a custom sampler with specified filtering and addressing modes
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `mag_filter` - Magnification filter (Linear or Nearest)
    /// * `min_filter` - Minification filter (Linear or Nearest)
    /// * `address_mode` - UV wrapping mode (Repeat, ClampToEdge, MirrorRepeat)
    ///
    /// # Returns
    /// Custom sampler with specified parameters
    pub fn create_custom_sampler(
        device: &wgpu::Device,
        mag_filter: wgpu::FilterMode,
        min_filter: wgpu::FilterMode,
        address_mode: wgpu::AddressMode,
    ) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Custom Sampler"),
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter,
            min_filter,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })
    }

    /// Update the texture's sampler with custom settings
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `mag_filter` - Magnification filter
    /// * `min_filter` - Minification filter
    /// * `address_mode` - UV wrapping mode
    pub fn update_sampler(
        &mut self,
        device: &wgpu::Device,
        mag_filter: wgpu::FilterMode,
        min_filter: wgpu::FilterMode,
        address_mode: wgpu::AddressMode,
    ) {
        self.sampler = Self::create_custom_sampler(device, mag_filter, min_filter, address_mode);
    }

    /// Create default texture sampler
    fn create_default_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Default Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })
    }

    /// Get texture statistics for display
    pub fn stats(&self) -> String {
        format!(
            "Label: {}\nSize: {}x{}\nFormat: {:?}\nSample Count: {}",
            self.label.as_deref().unwrap_or("unnamed"),
            self.size.width,
            self.size.height,
            self.format,
            self.texture.sample_count()
        )
    }
}

/// Procedural texture generators

/// Generate a solid color texture
pub fn generate_solid_color(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    color: [u8; 4],
) -> Result<GpuTexture, TextureError> {
    let pixel_count = (width * height) as usize;
    let mut data = Vec::with_capacity(pixel_count * 4);

    for _ in 0..pixel_count {
        data.extend_from_slice(&color);
    }

    GpuTexture::from_rgba8(device, queue, width, height, &data, Some("Solid Color"))
}

/// Generate a checkerboard pattern texture
pub fn generate_checker(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    color1: [u8; 4],
    color2: [u8; 4],
    checker_size: u32,
) -> Result<GpuTexture, TextureError> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let checker_x = x / checker_size;
            let checker_y = y / checker_size;
            let is_even = (checker_x + checker_y) % 2 == 0;

            let color = if is_even { color1 } else { color2 };
            data.extend_from_slice(&color);
        }
    }

    GpuTexture::from_rgba8(device, queue, width, height, &data, Some("Checker Pattern"))
}

/// Generate a gradient texture
pub fn generate_gradient(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    color_start: [u8; 4],
    color_end: [u8; 4],
    horizontal: bool,
) -> Result<GpuTexture, TextureError> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let t = if horizontal {
                x as f32 / (width - 1).max(1) as f32
            } else {
                y as f32 / (height - 1).max(1) as f32
            };

            let color = [
                ((1.0 - t) * color_start[0] as f32 + t * color_end[0] as f32) as u8,
                ((1.0 - t) * color_start[1] as f32 + t * color_end[1] as f32) as u8,
                ((1.0 - t) * color_start[2] as f32 + t * color_end[2] as f32) as u8,
                ((1.0 - t) * color_start[3] as f32 + t * color_end[3] as f32) as u8,
            ];

            data.extend_from_slice(&color);
        }
    }

    GpuTexture::from_rgba8(device, queue, width, height, &data, Some("Gradient"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test device (requires GPU)
    fn create_test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
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
    }

    #[test]
    fn test_rgba8_texture_creation() {
        let Some((device, queue)) = create_test_device() else {
            println!("GPU not available, skipping test");
            return;
        };

        let width = 4;
        let height = 4;
        let data = vec![255u8; (width * height * 4) as usize]; // White texture

        let result =
            GpuTexture::from_rgba8(&device, &queue, width, height, &data, Some("Test Texture"));
        assert!(result.is_ok());

        let texture = result.unwrap();
        assert_eq!(texture.size.width, width);
        assert_eq!(texture.size.height, height);
        assert_eq!(texture.format, wgpu::TextureFormat::Rgba8UnormSrgb);
    }

    #[test]
    fn test_render_target_creation() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        let result = GpuTexture::create_render_target(
            &device,
            800,
            600,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            1,
            Some("Test Render Target"),
        );

        assert!(result.is_ok());
        let texture = result.unwrap();
        assert_eq!(texture.size.width, 800);
        assert_eq!(texture.size.height, 600);
    }

    #[test]
    fn test_depth_texture_creation() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        let result =
            GpuTexture::create_depth_texture(&device, 800, 600, 1, Some("Test Depth Texture"));

        assert!(result.is_ok());
        let texture = result.unwrap();
        assert_eq!(texture.format, wgpu::TextureFormat::Depth32Float);
    }

    #[test]
    fn test_invalid_dimensions() {
        let Some((device, queue)) = create_test_device() else {
            return;
        };

        let data = vec![0u8; 16];
        let result = GpuTexture::from_rgba8(&device, &queue, 0, 0, &data, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_data_size() {
        let Some((device, queue)) = create_test_device() else {
            return;
        };

        let data = vec![0u8; 10]; // Wrong size for 4x4 RGBA
        let result = GpuTexture::from_rgba8(&device, &queue, 4, 4, &data, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_msaa_sample_count() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        let result = GpuTexture::create_render_target(
            &device,
            800,
            600,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            3, // Invalid: not 1, 2, 4, or 8
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_solid_color_generation() {
        let Some((device, queue)) = create_test_device() else {
            return;
        };

        let result = generate_solid_color(&device, &queue, 8, 8, [255, 0, 0, 255]);
        assert!(result.is_ok());

        let texture = result.unwrap();
        assert_eq!(texture.size.width, 8);
        assert_eq!(texture.size.height, 8);
    }

    #[test]
    fn test_checker_generation() {
        let Some((device, queue)) = create_test_device() else {
            return;
        };

        let result = generate_checker(
            &device,
            &queue,
            16,
            16,
            [255, 255, 255, 255],
            [0, 0, 0, 255],
            4,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_gradient_generation() {
        let Some((device, queue)) = create_test_device() else {
            return;
        };

        let result = generate_gradient(
            &device,
            &queue,
            16,
            16,
            [255, 0, 0, 255],
            [0, 0, 255, 255],
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_rgb8_texture_creation() {
        let Some((device, queue)) = create_test_device() else {
            println!("GPU not available, skipping test");
            return;
        };

        let width = 4;
        let height = 4;
        let data = vec![255u8; (width * height * 3) as usize]; // White texture (RGB)

        let result =
            GpuTexture::from_rgb8(&device, &queue, width, height, &data, Some("Test RGB8 Texture"));
        assert!(result.is_ok());

        let texture = result.unwrap();
        assert_eq!(texture.size.width, width);
        assert_eq!(texture.size.height, height);
        assert_eq!(texture.format, wgpu::TextureFormat::Rgba8UnormSrgb);
    }

    #[test]
    fn test_rgb8_invalid_data_size() {
        let Some((device, queue)) = create_test_device() else {
            return;
        };

        let data = vec![0u8; 10]; // Wrong size for 4x4 RGB
        let result = GpuTexture::from_rgb8(&device, &queue, 4, 4, &data, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_sampler_creation() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        let sampler = GpuTexture::create_custom_sampler(
            &device,
            wgpu::FilterMode::Nearest,
            wgpu::FilterMode::Nearest,
            wgpu::AddressMode::ClampToEdge,
        );

        // Just verify it doesn't panic - we can't inspect sampler properties directly
        assert_eq!(std::mem::size_of_val(&sampler), std::mem::size_of::<wgpu::Sampler>());
    }

    #[test]
    fn test_sampler_update() {
        let Some((device, queue)) = create_test_device() else {
            return;
        };

        let width = 4;
        let height = 4;
        let data = vec![255u8; (width * height * 4) as usize];

        let mut texture =
            GpuTexture::from_rgba8(&device, &queue, width, height, &data, Some("Test Texture"))
                .unwrap();

        // Update sampler settings
        texture.update_sampler(
            &device,
            wgpu::FilterMode::Nearest,
            wgpu::FilterMode::Nearest,
            wgpu::AddressMode::ClampToEdge,
        );

        // Verify texture still works after sampler update
        assert_eq!(texture.size.width, width);
        assert_eq!(texture.size.height, height);
    }
}
