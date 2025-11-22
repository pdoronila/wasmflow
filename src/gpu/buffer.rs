//! GPU Buffer Management
//!
//! Handles creation and management of GPU buffers for vertex, index, and uniform data.

use thiserror::Error;
use wgpu::util::DeviceExt;

/// GPU buffer wrapper with metadata
pub struct GpuBuffer {
    pub buffer: wgpu::Buffer,
    pub size: u64,
    pub usage: wgpu::BufferUsages,
    pub label: Option<String>,
}

/// Buffer creation errors
#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Buffer size must be greater than 0")]
    ZeroSize,

    #[error("Invalid buffer data: {0}")]
    InvalidData(String),

    #[error("Buffer alignment error: size {size} is not aligned to {alignment}")]
    AlignmentError { size: u64, alignment: u64 },
}

impl GpuBuffer {
    /// Create a vertex buffer from raw f32 data
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `vertices` - Vertex data as f32 slice (e.g., positions, normals, UVs interleaved)
    /// * `label` - Debug label for the buffer
    ///
    /// # Returns
    /// Vertex buffer ready for rendering
    pub fn from_vertex_data(
        device: &wgpu::Device,
        vertices: &[f32],
        label: Option<&str>,
    ) -> Result<Self, BufferError> {
        if vertices.is_empty() {
            return Err(BufferError::ZeroSize);
        }

        let size = (vertices.len() * std::mem::size_of::<f32>()) as u64;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        log::debug!(
            "Created vertex buffer: {} ({} vertices, {} bytes)",
            label.unwrap_or("unnamed"),
            vertices.len(),
            size
        );

        Ok(GpuBuffer {
            buffer,
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            label: label.map(String::from),
        })
    }

    /// Create an index buffer from u32 indices
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `indices` - Index data (triangle list)
    /// * `label` - Debug label for the buffer
    ///
    /// # Returns
    /// Index buffer ready for indexed drawing
    pub fn from_index_data(
        device: &wgpu::Device,
        indices: &[u32],
        label: Option<&str>,
    ) -> Result<Self, BufferError> {
        if indices.is_empty() {
            return Err(BufferError::ZeroSize);
        }

        let size = (indices.len() * std::mem::size_of::<u32>()) as u64;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        log::debug!(
            "Created index buffer: {} ({} indices, {} bytes)",
            label.unwrap_or("unnamed"),
            indices.len(),
            size
        );

        Ok(GpuBuffer {
            buffer,
            size,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            label: label.map(String::from),
        })
    }

    /// Create a uniform buffer from any Pod type
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `data` - Uniform data (must implement bytemuck::Pod)
    /// * `label` - Debug label for the buffer
    ///
    /// # Returns
    /// Uniform buffer ready for binding
    pub fn from_uniform_data<T: bytemuck::Pod>(
        device: &wgpu::Device,
        data: &T,
        label: Option<&str>,
    ) -> Result<Self, BufferError> {
        let bytes = bytemuck::bytes_of(data);
        let size = bytes.len() as u64;

        // Check alignment (uniforms must be 256-byte aligned on some platforms)
        const UNIFORM_ALIGNMENT: u64 = 256;
        if size > UNIFORM_ALIGNMENT && size % UNIFORM_ALIGNMENT != 0 {
            log::warn!(
                "Uniform buffer size {} is not aligned to {} bytes",
                size,
                UNIFORM_ALIGNMENT
            );
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        log::debug!(
            "Created uniform buffer: {} ({} bytes)",
            label.unwrap_or("unnamed"),
            size
        );

        Ok(GpuBuffer {
            buffer,
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label: label.map(String::from),
        })
    }

    /// Update buffer contents
    ///
    /// # Arguments
    /// * `queue` - WebGPU queue for data transfer
    /// * `data` - New data to write to buffer
    /// * `offset` - Byte offset in buffer
    pub fn update<T: bytemuck::Pod>(&self, queue: &wgpu::Queue, data: &T, offset: u64) {
        queue.write_buffer(&self.buffer, offset, bytemuck::bytes_of(data));
    }

    /// Update buffer from slice
    pub fn update_slice<T: bytemuck::Pod>(&self, queue: &wgpu::Queue, data: &[T], offset: u64) {
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(data));
    }

    /// Get buffer statistics for display
    pub fn stats(&self) -> String {
        format!(
            "Label: {}\nSize: {} bytes\nUsage: {:?}",
            self.label.as_deref().unwrap_or("unnamed"),
            self.size,
            self.usage
        )
    }
}

/// Geometry buffer set containing all buffers for a mesh
pub struct GeometryBuffers {
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl GeometryBuffers {
    /// Create geometry buffers from primitive output
    ///
    /// # Arguments
    /// * `device` - WebGPU device
    /// * `positions` - Vertex positions (flattened vec3 array)
    /// * `normals` - Vertex normals (flattened vec3 array)
    /// * `uvs` - Texture coordinates (flattened vec2 array)
    /// * `indices` - Triangle indices
    ///
    /// # Returns
    /// Complete geometry buffer set ready for rendering
    pub fn from_geometry_data(
        device: &wgpu::Device,
        positions: &[f32],
        normals: &[f32],
        uvs: &[f32],
        indices: &[u32],
    ) -> Result<Self, BufferError> {
        // Validate input sizes
        if positions.len() % 3 != 0 {
            return Err(BufferError::InvalidData(
                "Positions must be multiple of 3 (vec3)".to_string(),
            ));
        }
        if normals.len() % 3 != 0 {
            return Err(BufferError::InvalidData(
                "Normals must be multiple of 3 (vec3)".to_string(),
            ));
        }
        if uvs.len() % 2 != 0 {
            return Err(BufferError::InvalidData(
                "UVs must be multiple of 2 (vec2)".to_string(),
            ));
        }

        let vertex_count = positions.len() / 3;
        if normals.len() / 3 != vertex_count {
            return Err(BufferError::InvalidData(format!(
                "Position and normal counts don't match: {} vs {}",
                vertex_count,
                normals.len() / 3
            )));
        }
        if uvs.len() / 2 != vertex_count {
            return Err(BufferError::InvalidData(format!(
                "Position and UV counts don't match: {} vs {}",
                vertex_count,
                uvs.len() / 2
            )));
        }

        // Interleave vertex data: [pos.xyz, normal.xyz, uv.xy] per vertex
        let mut vertex_data = Vec::with_capacity(vertex_count * 8);
        for i in 0..vertex_count {
            // Position (3 floats)
            vertex_data.push(positions[i * 3]);
            vertex_data.push(positions[i * 3 + 1]);
            vertex_data.push(positions[i * 3 + 2]);

            // Normal (3 floats)
            vertex_data.push(normals[i * 3]);
            vertex_data.push(normals[i * 3 + 1]);
            vertex_data.push(normals[i * 3 + 2]);

            // UV (2 floats)
            vertex_data.push(uvs[i * 2]);
            vertex_data.push(uvs[i * 2 + 1]);
        }

        let vertex_buffer =
            GpuBuffer::from_vertex_data(device, &vertex_data, Some("Geometry Vertices"))?;

        let index_buffer =
            GpuBuffer::from_index_data(device, indices, Some("Geometry Indices"))?;

        log::info!(
            "Created geometry buffers: {} vertices, {} indices",
            vertex_count,
            indices.len()
        );

        Ok(GeometryBuffers {
            vertex_buffer,
            index_buffer,
            vertex_count: vertex_count as u32,
            index_count: indices.len() as u32,
        })
    }

    /// Get vertex buffer layout descriptor for pipeline creation
    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            // Position (location 0)
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            // Normal (location 1)
            wgpu::VertexAttribute {
                offset: 12, // 3 * 4 bytes
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            // UV (location 2)
            wgpu::VertexAttribute {
                offset: 24, // 6 * 4 bytes
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: 32, // 8 floats * 4 bytes
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: VERTEX_ATTRIBUTES,
        }
    }
}

/// Uniform data structures for common shader parameters

/// Camera uniform buffer (view + projection matrices)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniforms {
    pub view_matrix: [[f32; 4]; 4],       // 64 bytes
    pub projection_matrix: [[f32; 4]; 4], // 64 bytes
    pub camera_position: [f32; 3],        // 12 bytes
    pub _padding1: f32,                   // 4 bytes (alignment)
}

impl CameraUniforms {
    /// Create camera uniforms from matrices
    pub fn new(
        view_matrix: [[f32; 4]; 4],
        projection_matrix: [[f32; 4]; 4],
        camera_position: [f32; 3],
    ) -> Self {
        Self {
            view_matrix,
            projection_matrix,
            camera_position,
            _padding1: 0.0,
        }
    }

    /// Create buffer from camera uniforms
    pub fn create_buffer(&self, device: &wgpu::Device) -> Result<GpuBuffer, BufferError> {
        GpuBuffer::from_uniform_data(device, self, Some("Camera Uniforms"))
    }
}

/// Material uniform buffer (basic PBR properties)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniforms {
    pub base_color: [f32; 4],    // 16 bytes (RGBA)
    pub metallic: f32,            // 4 bytes
    pub roughness: f32,           // 4 bytes
    pub _padding1: [f32; 2],      // 8 bytes (alignment)
}

impl MaterialUniforms {
    /// Create material uniforms
    pub fn new(base_color: [f32; 4], metallic: f32, roughness: f32) -> Self {
        Self {
            base_color,
            metallic,
            roughness,
            _padding1: [0.0; 2],
        }
    }

    /// Create buffer from material uniforms
    pub fn create_buffer(&self, device: &wgpu::Device) -> Result<GpuBuffer, BufferError> {
        GpuBuffer::from_uniform_data(device, self, Some("Material Uniforms"))
    }
}

/// Light uniform buffer (single directional light)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniforms {
    pub direction: [f32; 3],  // 12 bytes
    pub _padding1: f32,       // 4 bytes (alignment)
    pub color: [f32; 3],      // 12 bytes
    pub intensity: f32,       // 4 bytes
}

impl LightUniforms {
    /// Create light uniforms
    pub fn new(direction: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self {
            direction,
            _padding1: 0.0,
            color,
            intensity,
        }
    }

    /// Create buffer from light uniforms
    pub fn create_buffer(&self, device: &wgpu::Device) -> Result<GpuBuffer, BufferError> {
        GpuBuffer::from_uniform_data(device, self, Some("Light Uniforms"))
    }
}

/// Single light in array (supports both directional and point lights)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightData {
    pub position_or_direction: [f32; 3], // 12 bytes - position for point, direction for directional
    pub light_type: u32,                  // 4 bytes - 0=directional, 1=point
    pub color: [f32; 3],                  // 12 bytes
    pub intensity: f32,                   // 4 bytes
    pub radius: f32,                      // 4 bytes - only for point lights
    pub _padding: [f32; 3],               // 12 bytes (alignment to 16 bytes)
}

/// Light types for GPU shaders
pub const LIGHT_TYPE_DIRECTIONAL: u32 = 0;
pub const LIGHT_TYPE_POINT: u32 = 1;

impl LightData {
    /// Create directional light data
    pub fn directional(direction: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self {
            position_or_direction: direction,
            light_type: LIGHT_TYPE_DIRECTIONAL,
            color,
            intensity,
            radius: 0.0,
            _padding: [0.0; 3],
        }
    }

    /// Create point light data
    pub fn point(position: [f32; 3], color: [f32; 3], intensity: f32, radius: f32) -> Self {
        Self {
            position_or_direction: position,
            light_type: LIGHT_TYPE_POINT,
            color,
            intensity,
            radius,
            _padding: [0.0; 3],
        }
    }

    /// Parse from JSON string (from light WASM components)
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        use serde_json::Value;

        let data: Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse light JSON: {}", e))?;

        let light_type_str = data["light_type"]
            .as_str()
            .ok_or("Missing light_type field")?;

        match light_type_str {
            "directional" => {
                let direction = data["direction"]
                    .as_array()
                    .ok_or("Missing direction field")?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect::<Vec<_>>();
                let color = data["color"]
                    .as_array()
                    .ok_or("Missing color field")?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect::<Vec<_>>();
                let intensity = data["intensity"]
                    .as_f64()
                    .ok_or("Missing intensity field")? as f32;

                if direction.len() != 3 || color.len() != 3 {
                    return Err("Invalid direction or color array length".to_string());
                }

                Ok(Self::directional(
                    [direction[0], direction[1], direction[2]],
                    [color[0], color[1], color[2]],
                    intensity,
                ))
            }
            "point" => {
                let position = data["position"]
                    .as_array()
                    .ok_or("Missing position field")?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect::<Vec<_>>();
                let color = data["color"]
                    .as_array()
                    .ok_or("Missing color field")?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect::<Vec<_>>();
                let intensity = data["intensity"]
                    .as_f64()
                    .ok_or("Missing intensity field")? as f32;
                let radius = data["radius"]
                    .as_f64()
                    .ok_or("Missing radius field")? as f32;

                if position.len() != 3 || color.len() != 3 {
                    return Err("Invalid position or color array length".to_string());
                }

                Ok(Self::point(
                    [position[0], position[1], position[2]],
                    [color[0], color[1], color[2]],
                    intensity,
                    radius,
                ))
            }
            _ => Err(format!("Unknown light type: {}", light_type_str)),
        }
    }
}

/// Multi-light uniform buffer (supports up to MAX_LIGHTS lights)
pub const MAX_LIGHTS: usize = 8;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MultiLightUniforms {
    pub lights: [LightData; MAX_LIGHTS], // Array of lights
    pub light_count: u32,                 // Active light count
    pub _padding: [f32; 3],               // Alignment
}

impl MultiLightUniforms {
    /// Create empty multi-light uniforms
    pub fn new() -> Self {
        Self {
            lights: [LightData {
                position_or_direction: [0.0; 3],
                light_type: LIGHT_TYPE_DIRECTIONAL,
                color: [0.0; 3],
                intensity: 0.0,
                radius: 0.0,
                _padding: [0.0; 3],
            }; MAX_LIGHTS],
            light_count: 0,
            _padding: [0.0; 3],
        }
    }

    /// Add a light (returns false if max lights reached)
    pub fn add_light(&mut self, light: LightData) -> bool {
        if self.light_count as usize >= MAX_LIGHTS {
            return false;
        }

        self.lights[self.light_count as usize] = light;
        self.light_count += 1;
        true
    }

    /// Create buffer from multi-light uniforms
    pub fn create_buffer(&self, device: &wgpu::Device) -> Result<GpuBuffer, BufferError> {
        GpuBuffer::from_uniform_data(device, self, Some("Multi-Light Uniforms"))
    }

    /// Parse from array of JSON strings
    pub fn from_json_array(json_strings: &[&str]) -> Result<Self, String> {
        let mut uniforms = Self::new();

        for (i, json_str) in json_strings.iter().enumerate() {
            if i >= MAX_LIGHTS {
                log::warn!("Exceeded maximum number of lights ({}), ignoring extras", MAX_LIGHTS);
                break;
            }

            let light_data = LightData::from_json(json_str)?;
            if !uniforms.add_light(light_data) {
                break;
            }
        }

        Ok(uniforms)
    }
}

impl Default for MultiLightUniforms {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test device (requires GPU, may fail in CI)
    fn create_test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        if std::env::var("CI").is_ok() {
            return None; // Skip GPU tests in CI
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
    fn test_vertex_buffer_creation() {
        let Some((device, _queue)) = create_test_device() else {
            println!("GPU not available, skipping test");
            return;
        };

        let vertices = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let result = GpuBuffer::from_vertex_data(&device, &vertices, Some("Test Vertices"));

        assert!(result.is_ok());
        let buffer = result.unwrap();
        assert_eq!(buffer.size, (vertices.len() * 4) as u64);
        assert!(buffer.usage.contains(wgpu::BufferUsages::VERTEX));
    }

    #[test]
    fn test_index_buffer_creation() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        let indices = vec![0, 1, 2];
        let result = GpuBuffer::from_index_data(&device, &indices, Some("Test Indices"));

        assert!(result.is_ok());
        let buffer = result.unwrap();
        assert_eq!(buffer.size, (indices.len() * 4) as u64);
        assert!(buffer.usage.contains(wgpu::BufferUsages::INDEX));
    }

    #[test]
    fn test_uniform_buffer_creation() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        let uniforms = CameraUniforms {
            view_matrix: [[0.0; 4]; 4],
            projection_matrix: [[0.0; 4]; 4],
            camera_position: [0.0, 0.0, 0.0],
            _padding1: 0.0,
        };

        let result = uniforms.create_buffer(&device);
        assert!(result.is_ok());

        let buffer = result.unwrap();
        assert_eq!(buffer.size, std::mem::size_of::<CameraUniforms>() as u64);
        assert!(buffer.usage.contains(wgpu::BufferUsages::UNIFORM));
    }

    #[test]
    fn test_geometry_buffers_validation() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        // Valid geometry: triangle
        let positions = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let uvs = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];
        let indices = vec![0, 1, 2];

        let result =
            GeometryBuffers::from_geometry_data(&device, &positions, &normals, &uvs, &indices);
        assert!(result.is_ok());

        let buffers = result.unwrap();
        assert_eq!(buffers.vertex_count, 3);
        assert_eq!(buffers.index_count, 3);
    }

    #[test]
    fn test_geometry_buffers_invalid_sizes() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        // Invalid: positions not multiple of 3
        let positions = vec![0.0, 0.0];
        let normals = vec![0.0, 0.0, 1.0];
        let uvs = vec![0.0, 0.0];
        let indices = vec![0];

        let result =
            GeometryBuffers::from_geometry_data(&device, &positions, &normals, &uvs, &indices);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_buffer_error() {
        let Some((device, _queue)) = create_test_device() else {
            return;
        };

        let empty: Vec<f32> = vec![];
        let result = GpuBuffer::from_vertex_data(&device, &empty, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_camera_uniforms_size() {
        // Ensure proper alignment
        assert_eq!(
            std::mem::size_of::<CameraUniforms>(),
            144 // 64 + 64 + 12 + 4
        );
    }

    #[test]
    fn test_material_uniforms_size() {
        assert_eq!(
            std::mem::size_of::<MaterialUniforms>(),
            32 // 16 + 4 + 4 + 8
        );
    }
}
