# Phase 2 Implementation Plan: WebGPU Rendering System

**Feature**: GLSL Physically Based Shader Authoring System
**Phase**: 2 - GPU Integration and Real-time Rendering
**Category**: Graphics
**Created**: 2025-11-20
**Status**: Planning
**Depends On**: Phase 1 (Complete ✓)

## Overview

Phase 2 integrates WebGPU rendering into WasmFlow, enabling actual shader compilation, execution, and real-time preview. This phase transforms the placeholder shader preview into a full GPU-powered rendering system capable of executing user-authored GLSL shaders.

**Key Deliverables**:
1. WebGPU integration with egui
2. GLSL → SPIR-V compilation pipeline
3. GPU buffer management system
4. Real-time shader preview with texture rendering
5. Texture loading nodes (file and data sources)
6. Basic lighting nodes (directional, point lights)
7. Shader program linker (vertex + fragment)
8. Performance monitoring and optimization

**Total New Nodes**: ~10 nodes (3 built-in + 7 WASM components)

---

## Architecture Decisions

### 1. WebGPU Integration Strategy

**Implementation Approach**:
- Use `wgpu` crate for WebGPU access
- Integrate with egui through custom texture callback
- Create dedicated GPU context manager
- Implement async shader compilation

**GPU Context Location**: `src/gpu/`
```
src/gpu/
├── mod.rs              # Public API and GPU context manager
├── context.rs          # WebGPU device/queue/surface management
├── shader.rs           # Shader compilation and module management
├── buffer.rs           # GPU buffer management (vertex, uniform, storage)
├── texture.rs          # Texture creation and management
├── pipeline.rs         # Render pipeline construction
└── renderer.rs         # High-level rendering orchestration
```

**Rationale**:
- Centralized GPU resource management prevents resource leaks
- Async compilation prevents UI blocking
- Separate modules for each GPU concept improve maintainability
- Shader hot-reload support built-in from start

### 2. Shader Compilation Pipeline

**GLSL → SPIR-V Flow**:
```
User GLSL Code
    ↓
naga::front::glsl::Parser
    ↓
naga::Module (validated IR)
    ↓
naga::back::spv::write_vec
    ↓
SPIR-V bytecode
    ↓
wgpu::ShaderModule
    ↓
GPU execution
```

**Compilation Stages**:
1. **Validation**: Check GLSL syntax and semantics (naga frontend)
2. **Translation**: Convert to naga IR (intermediate representation)
3. **Optimization**: Apply shader optimizations (naga transform)
4. **Code Generation**: Generate SPIR-V bytecode (naga backend)
5. **GPU Upload**: Create wgpu ShaderModule

**Error Handling**:
- Syntax errors: Display line/column info with context
- Semantic errors: Show type mismatches, undefined variables
- Linking errors: Report interface mismatches between stages
- Compilation errors: GPU-specific errors with recovery hints

### 3. Render Target Integration

**Egui Texture Rendering**:
```rust
// In shader_preview.rs
impl ShaderPreviewFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Get texture from GPU renderer
        let texture_id = self.renderer.get_output_texture()?;

        // Display in egui
        ui.image(egui::ImageSource::Texture(egui::load::SizedTexture {
            id: texture_id,
            size: egui::vec2(width, height),
        }));
    }
}
```

**Texture Updates**:
- Render to offscreen GPU texture
- Copy to egui-accessible texture handle
- Update every frame or on-demand (based on auto-refresh setting)
- Support multiple preview sizes (from Phase 1 UI)

### 4. GPU Resource Lifecycle

**Resource Management**:
```rust
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    shaders: HashMap<Uuid, CompiledShader>,
    buffers: HashMap<Uuid, GpuBuffer>,
    textures: HashMap<Uuid, GpuTexture>,
    pipelines: HashMap<Uuid, RenderPipeline>,
}
```

**Cleanup Strategy**:
- Reference counting for shared resources
- Automatic cleanup when nodes deleted
- Manual cleanup on graph clear
- Periodic orphan resource sweep

---

## Implementation Steps

### Step 1: WebGPU Foundation (Week 1)
**Goal**: Set up basic WebGPU integration with egui

**Tasks**:
1. Add dependencies to `Cargo.toml`:
   ```toml
   wgpu = "22.0"
   naga = { version = "22.0", features = ["glsl-in", "spv-out"] }
   pollster = "0.3"  # For blocking on async operations
   ```

2. Create `src/gpu/mod.rs` with GPU context:
   ```rust
   pub struct GpuContext {
       pub device: wgpu::Device,
       pub queue: wgpu::Queue,
       adapter_info: wgpu::AdapterInfo,
   }

   impl GpuContext {
       pub async fn new() -> Result<Self, GpuError> {
           // Initialize WebGPU
       }
   }
   ```

3. Add GPU context to `WasmFlowApp`:
   ```rust
   pub struct WasmFlowApp {
       // ... existing fields ...
       gpu_context: Option<Arc<Mutex<GpuContext>>>,
   }
   ```

4. Initialize GPU in app startup with error handling:
   - Try to initialize WebGPU
   - Fall back to CPU if unavailable
   - Display error message in UI if failed

**Deliverables**:
- [ ] GPU context initialized successfully
- [ ] Device capabilities logged
- [ ] Error handling for unsupported platforms
- [ ] Unit tests for GPU initialization

**Estimated Time**: 2-3 days

---

### Step 2: Shader Compilation System (Week 1-2)
**Goal**: Implement GLSL → SPIR-V compilation pipeline

**Tasks**:
1. Create `src/gpu/shader.rs`:
   ```rust
   pub struct CompiledShader {
       pub id: Uuid,
       pub source: String,
       pub module: wgpu::ShaderModule,
       pub stage: ShaderStage,
       pub entry_point: String,
   }

   pub enum ShaderStage {
       Vertex,
       Fragment,
   }

   impl CompiledShader {
       pub fn from_glsl(
           device: &wgpu::Device,
           source: &str,
           stage: ShaderStage,
       ) -> Result<Self, ShaderCompilationError> {
           // Parse GLSL with naga
           // Validate
           // Generate SPIR-V
           // Create wgpu::ShaderModule
       }
   }
   ```

2. Integrate with GLSL Shader Editor node:
   - Add "Compile" button to editor UI
   - Show compilation status (success/error)
   - Display error messages with line numbers
   - Cache compiled shaders for reuse

3. Add shader validation feedback:
   - Syntax highlighting for errors
   - Inline error annotations
   - Type inference information
   - Shader statistics (instructions, registers)

**Deliverables**:
- [ ] GLSL parsing and validation working
- [ ] SPIR-V generation functional
- [ ] Error messages with line/column info
- [ ] Integration with shader editor node
- [ ] Unit tests for various shader types

**Estimated Time**: 3-4 days

---

### Step 3: GPU Buffer Management (Week 2)
**Goal**: Implement vertex, index, and uniform buffer management

**Tasks**:
1. Create `src/gpu/buffer.rs`:
   ```rust
   pub struct GpuBuffer {
       pub buffer: wgpu::Buffer,
       pub size: u64,
       pub usage: wgpu::BufferUsages,
   }

   impl GpuBuffer {
       pub fn from_vertex_data(
           device: &wgpu::Device,
           vertices: &[f32],
       ) -> Self {
           // Create vertex buffer
       }

       pub fn from_index_data(
           device: &wgpu::Device,
           indices: &[u32],
       ) -> Self {
           // Create index buffer
       }

       pub fn from_uniform_data<T: bytemuck::Pod>(
           device: &wgpu::Device,
           data: &T,
       ) -> Self {
           // Create uniform buffer
       }
   }
   ```

2. Connect geometry primitives to GPU buffers:
   - Extract positions, normals, UVs from primitive nodes
   - Upload to GPU as vertex buffers
   - Create index buffers for triangle lists
   - Support dynamic updates

3. Implement uniform buffer management:
   - Camera matrices (view, projection)
   - Material properties (color, roughness, etc.)
   - Light data (position, color, intensity)
   - Time and frame counters

**Deliverables**:
- [ ] Vertex buffer creation from geometry data
- [ ] Index buffer support
- [ ] Uniform buffer management
- [ ] Buffer update mechanism
- [ ] Memory usage tracking

**Estimated Time**: 2-3 days

---

### Step 4: Texture System (Week 2-3)
**Goal**: Implement texture loading, management, and GPU upload

**Tasks**:
1. Create `src/gpu/texture.rs`:
   ```rust
   pub struct GpuTexture {
       pub texture: wgpu::Texture,
       pub view: wgpu::TextureView,
       pub sampler: wgpu::Sampler,
       pub size: wgpu::Extent3d,
       pub format: wgpu::TextureFormat,
   }

   impl GpuTexture {
       pub fn from_rgba8(
           device: &wgpu::Device,
           queue: &wgpu::Queue,
           width: u32,
           height: u32,
           data: &[u8],
       ) -> Self {
           // Create GPU texture from raw data
       }

       pub fn create_render_target(
           device: &wgpu::Device,
           width: u32,
           height: u32,
           format: wgpu::TextureFormat,
           sample_count: u32,
       ) -> Self {
           // Create texture for rendering
       }
   }
   ```

2. Add texture loading nodes (WASM components):
   - `texture-solid-color`: Generate solid color texture
   - `texture-checker`: Generate procedural checker pattern
   - `texture-gradient`: Generate gradient texture
   - (Future: `texture-from-file` in Phase 3)

3. Integrate with shader preview:
   - Render to GPU texture
   - Copy to egui texture handle
   - Support MSAA render targets
   - Handle texture format conversions

**Deliverables**:
- [ ] GPU texture creation and management
- [ ] Render target texture support
- [ ] Procedural texture generation nodes (3 components)
- [ ] Egui texture integration
- [ ] Texture format validation

**Estimated Time**: 3-4 days

---

### Step 5: Render Pipeline System (Week 3)
**Goal**: Build render pipeline management and shader linking

**Tasks**:
1. Create `src/gpu/pipeline.rs`:
   ```rust
   pub struct RenderPipeline {
       pub pipeline: wgpu::RenderPipeline,
       pub bind_group_layout: wgpu::BindGroupLayout,
       pub bind_group: wgpu::BindGroup,
   }

   impl RenderPipeline {
       pub fn new(
           device: &wgpu::Device,
           vertex_shader: &CompiledShader,
           fragment_shader: &CompiledShader,
           vertex_layout: &VertexLayout,
       ) -> Result<Self, PipelineError> {
           // Create render pipeline
           // Link vertex and fragment shaders
           // Set up bind groups
       }
   }
   ```

2. Add shader program linker (built-in node):
   - Takes vertex shader + fragment shader
   - Validates interface compatibility
   - Creates linked program
   - Reports linking errors

3. Implement vertex layout specification:
   - Extract from geometry primitive metadata
   - Support common layouts (position, normal, UV, color)
   - Custom layout specification
   - Layout validation

**Deliverables**:
- [ ] Render pipeline creation
- [ ] Shader linking and validation
- [ ] Vertex layout system
- [ ] Bind group management
- [ ] Shader program linker node

**Estimated Time**: 3-4 days

---

### Step 6: Rendering System (Week 3-4)
**Goal**: Implement core rendering loop and command encoding

**Tasks**:
1. Create `src/gpu/renderer.rs`:
   ```rust
   pub struct Renderer {
       context: Arc<GpuContext>,
       pipelines: HashMap<Uuid, RenderPipeline>,
       current_scene: Option<SceneDescription>,
   }

   impl Renderer {
       pub fn render_frame(
           &mut self,
           geometry: &GeometryData,
           camera: &CameraData,
           material: &MaterialData,
       ) -> Result<wgpu::Texture, RenderError> {
           // Encode render commands
           // Submit to GPU
           // Return output texture
       }
   }

   pub struct SceneDescription {
       pub geometry: GeometryData,
       pub camera: CameraData,
       pub lights: Vec<LightData>,
       pub materials: Vec<MaterialData>,
   }
   ```

2. Implement render command encoding:
   - Set pipeline
   - Bind vertex/index buffers
   - Bind uniform buffers
   - Set viewport and scissor
   - Draw indexed primitives

3. Add frame synchronization:
   - Wait for GPU completion
   - Texture readback for preview
   - Frame timing information
   - FPS counter

**Deliverables**:
- [ ] Render command encoding
- [ ] Frame submission to GPU
- [ ] Output texture generation
- [ ] Frame synchronization
- [ ] Performance metrics

**Estimated Time**: 3-4 days

---

### Step 7: Shader Preview Integration (Week 4)
**Goal**: Upgrade shader preview from placeholder to real-time rendering

**Tasks**:
1. Update `src/builtin/shader_preview.rs`:
   ```rust
   impl ShaderPreviewFooterView {
       fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
           // Check if rendering is available
           if let Some(renderer) = &self.renderer {
               // Collect inputs (geometry, camera, shader, etc.)
               let geometry = self.get_geometry_input(node)?;
               let camera = self.get_camera_input(node)?;
               let shader = self.get_shader_input(node)?;

               // Render frame
               let output = renderer.render_frame(geometry, camera, shader)?;

               // Display in egui
               let texture_id = egui_context.load_texture(output);
               ui.image(texture_id, preview_size);

               // Show stats (FPS, resolution, GPU time)
               self.render_stats(ui);
           } else {
               // Fallback to placeholder
               self.render_placeholder(ui);
           }
       }
   }
   ```

2. Add new inputs to shader preview node:
   - `geometry`: Geometry data (positions, normals, UVs, indices)
   - `vertex_shader`: Compiled vertex shader
   - `fragment_shader`: Compiled fragment shader
   - `camera`: Camera matrices
   - `textures`: Optional texture inputs

3. Implement preview controls:
   - Play/pause rendering
   - FPS limit control
   - Resolution control
   - Screenshot capture
   - GPU performance overlay

**Deliverables**:
- [ ] Real-time shader preview working
- [ ] Multiple input support
- [ ] Preview controls functional
- [ ] Performance metrics displayed
- [ ] Error handling and fallback

**Estimated Time**: 3-4 days

---

### Step 8: Basic Lighting Nodes (Week 4)
**Goal**: Add fundamental lighting calculations

**Tasks**:
1. Create lighting WASM components:
   - `light-directional`: Directional light (sun)
     - Inputs: direction (vec3), color (vec3), intensity (f32)
     - Outputs: light_data (JSON)

   - `light-point`: Point light source
     - Inputs: position (vec3), color (vec3), intensity (f32), radius (f32)
     - Outputs: light_data (JSON)

   - `lighting-phong`: Basic Phong lighting calculation
     - Inputs: normal (vec3), light_dir (vec3), view_dir (vec3), color (vec3)
     - Outputs: lit_color (vec3)

2. Add light uniform buffer support:
   - Pack light data for GPU
   - Support multiple lights (array)
   - Dynamic light count

3. Create example lighting shaders:
   - Basic diffuse lighting
   - Phong specular
   - Multiple light support

**Deliverables**:
- [ ] Directional light component
- [ ] Point light component
- [ ] Phong lighting component
- [ ] Light uniform buffer system
- [ ] Example lighting shaders
- [ ] Tests for lighting calculations

**Estimated Time**: 2-3 days

---

### Step 9: Shader Program Linker Node (Week 4-5)
**Goal**: Create built-in node for linking vertex and fragment shaders

**Tasks**:
1. Create `shader-program-linker` built-in node:
   ```rust
   pub fn spec() -> ComponentSpec {
       let mut spec = ComponentSpec::new_builtin(
           "builtin:graphics:shader-program-linker".to_string(),
           "Shader Program Linker".to_string(),
           "Link vertex and fragment shaders into executable program".to_string(),
           Some("Graphics".to_string()),
       );

       spec.input_spec = vec![
           PortSpec {
               name: "vertex_shader",
               data_type: DataType::String,  // GLSL source
               optional: false,
               description: "Vertex shader GLSL source code".to_string(),
           },
           PortSpec {
               name: "fragment_shader",
               data_type: DataType::String,  // GLSL source
               optional: false,
               description: "Fragment shader GLSL source code".to_string(),
           },
       ];

       spec.output_spec = vec![
           PortSpec {
               name: "program",
               data_type: DataType::Binary,  // Compiled program ID
               optional: false,
               description: "Linked shader program".to_string(),
           },
       ];

       spec
   }
   ```

2. Implement linking logic:
   - Compile both shaders
   - Validate interface matching (outputs → inputs)
   - Create render pipeline
   - Report linking errors

3. Add footer view for linker node:
   - Show compilation status for each shader
   - Display linking errors
   - Show program statistics
   - Interface compatibility table

**Deliverables**:
- [ ] Shader program linker node
- [ ] Interface validation
- [ ] Footer view with status display
- [ ] Error reporting
- [ ] Integration tests

**Estimated Time**: 2-3 days

---

### Step 10: Testing and Documentation (Week 5)
**Goal**: Comprehensive testing and documentation

**Tasks**:
1. Create integration test graphs:
   - `graphics_basic_rendering.json`: Simple colored triangle
   - `graphics_textured_quad.json`: Textured quad rendering
   - `graphics_lit_sphere.json`: Sphere with Phong lighting
   - `graphics_multiple_objects.json`: Multiple objects with different materials

2. Add unit tests:
   - GPU context initialization
   - Shader compilation pipeline
   - Buffer management
   - Texture creation
   - Pipeline construction
   - Rendering system

3. Update documentation:
   - Update `components/graphics/README.md` with Phase 2 nodes
   - Create `docs/GPU_INTEGRATION.md` guide
   - Add shader authoring tutorial
   - Document performance best practices
   - Update CLAUDE.md with Phase 2 guidelines

4. Create example shaders:
   - Basic vertex transformation
   - Simple diffuse lighting
   - Textured surface
   - Multi-light scene

**Deliverables**:
- [ ] 4+ integration test graphs
- [ ] 50+ unit tests
- [ ] Updated documentation
- [ ] Example shader library
- [ ] Performance profiling report

**Estimated Time**: 4-5 days

---

## New Nodes Summary

### Built-in Nodes (3)
1. **shader-program-linker**: Link vertex + fragment shaders
2. **shader-preview** (upgraded): Real-time GPU rendering
3. (Existing) **glsl-shader-editor** (enhanced): Compilation button and error display

### WASM Components (7)
1. **texture-solid-color**: Generate solid color texture
2. **texture-checker**: Generate checker pattern
3. **texture-gradient**: Generate gradient texture
4. **light-directional**: Directional light source
5. **light-point**: Point light source
6. **lighting-phong**: Phong lighting calculation
7. (Future potential): **texture-noise**, **light-spot**

---

## Dependencies

### New Crate Dependencies
```toml
[dependencies]
wgpu = "22.0"
naga = { version = "22.0", features = ["glsl-in", "spv-out", "validate"] }
pollster = "0.3"
bytemuck = { version = "1.14", features = ["derive"] }

[dev-dependencies]
image = "0.25"  # For texture loading in tests
```

### Platform Requirements
- **WebGPU Support**: Browser with WebGPU enabled, or native Vulkan/Metal/DX12
- **GPU**: Any GPU with compute capability (most GPUs from 2015+)
- **Drivers**: Up-to-date graphics drivers

---

## Risk Mitigation

### Risk 1: WebGPU Unavailable
**Impact**: High - Core feature won't work
**Mitigation**:
- Detect GPU availability at startup
- Graceful fallback to placeholder mode
- Clear error message with troubleshooting steps
- Document platform requirements

### Risk 2: Shader Compilation Failures
**Impact**: Medium - User experience degraded
**Mitigation**:
- Comprehensive error messages with line numbers
- Syntax highlighting for errors
- Example shaders that always compile
- GLSL reference documentation

### Risk 3: Performance Issues
**Impact**: Medium - Preview may be too slow
**Mitigation**:
- FPS limiting (default 30 FPS)
- Resolution control for preview
- Lazy rendering (only when inputs change)
- Performance profiling tools

### Risk 4: GPU Resource Leaks
**Impact**: High - App stability issues
**Mitigation**:
- Reference counting for all resources
- Automatic cleanup on node deletion
- Periodic resource audit
- Clear ownership model

---

## Success Criteria

### Functional Requirements
- [ ] WebGPU initializes successfully on supported platforms
- [ ] GLSL shaders compile to SPIR-V without errors
- [ ] Shader preview displays rendered output in real-time
- [ ] Geometry from Phase 1 primitives renders correctly
- [ ] Camera matrices correctly transform geometry
- [ ] Basic lighting produces expected results
- [ ] Texture nodes generate and apply textures
- [ ] Shader program linker validates interfaces

### Performance Requirements
- [ ] Preview maintains 30+ FPS for simple scenes
- [ ] Shader compilation completes in <500ms
- [ ] GPU memory usage stays below 512MB for typical scenes
- [ ] UI remains responsive during rendering

### Quality Requirements
- [ ] 50+ unit tests passing
- [ ] 4+ integration test graphs working
- [ ] Zero GPU resource leaks
- [ ] Comprehensive error handling

---

## Timeline Estimate

| Week | Focus Area | Deliverables |
|------|------------|-------------|
| Week 1 | WebGPU Foundation + Shader Compilation | GPU context, shader compiler |
| Week 2 | Buffers + Textures | Buffer management, texture system |
| Week 3 | Rendering Pipeline | Pipeline construction, rendering loop |
| Week 4 | Preview Integration + Lighting | Real-time preview, lighting nodes |
| Week 5 | Linker + Testing + Docs | Program linker, tests, documentation |

**Total Estimated Time**: 4-5 weeks

---

## Next Steps After Phase 2

**Phase 3: PBR Materials** (Future)
- Material property nodes (metallic, roughness, etc.)
- Multiple light types (spot lights, area lights)
- Advanced texture sampling
- BRDF calculations
- Material texture maps

**Phase 4: Advanced Features** (Future)
- Environment maps and IBL
- Cubemap processing
- BRDF lookup tables
- Compute shaders
- Post-processing effects

---

## References

- [WebGPU Specification](https://www.w3.org/TR/webgpu/)
- [wgpu Documentation](https://docs.rs/wgpu/)
- [naga Documentation](https://docs.rs/naga/)
- [GLSL Language Specification](https://www.khronos.org/registry/OpenGL/specs/gl/GLSLangSpec.4.60.pdf)
- [Learn wgpu Tutorial](https://sotrh.github.io/learn-wgpu/)
