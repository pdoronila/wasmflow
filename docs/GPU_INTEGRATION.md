# GPU Integration Guide

**Phase 2: GLSL Shader Nodes and WebGPU Integration**

This document describes the GPU integration architecture in WasmFlow, including shader compilation, buffer management, and the lighting system.

## Table of Contents

- [Overview](#overview)
- [Shader Compilation Pipeline](#shader-compilation-pipeline)
- [GPU Buffer System](#gpu-buffer-system)
- [Lighting System](#lighting-system)
- [Built-in Shader Nodes](#built-in-shader-nodes)
- [Usage Examples](#usage-examples)
- [Troubleshooting](#troubleshooting)

## Overview

WasmFlow's GPU integration provides:

- **GLSL Shader Compilation**: GLSL → WGSL via naga → wgpu::ShaderModule
- **GPU Buffer Management**: Vertex, index, and uniform buffer abstractions
- **Multi-Light Support**: Up to 8 lights (directional/point) in a single shader
- **Shader Program Linking**: Validate and link vertex + fragment shaders
- **Real-time Compilation**: Error reporting with detailed diagnostics

**Key Technologies:**
- **wgpu 22.0**: WebGPU implementation for Rust
- **naga 22.0**: Shader translation (GLSL → WGSL)
- **egui**: UI for shader preview and error display

## Shader Compilation Pipeline

### Architecture

```
GLSL Source Code
       ↓
   naga Parser (GLSL Frontend)
       ↓
   naga IR (Intermediate Representation)
       ↓
   naga Validator
       ↓
   WGSL Generator (naga Backend)
       ↓
   wgpu::ShaderModule
```

### Implementation

**Location**: `src/gpu/shader.rs`

**Key Struct**: `CompiledShader`

```rust
pub struct CompiledShader {
    pub id: Uuid,
    pub source: String,
    pub module: wgpu::ShaderModule,
    pub stage: ShaderStage,
    pub entry_point: String,
}
```

**Compilation API**:

```rust
let shader = CompiledShader::from_glsl(
    &device,           // wgpu::Device
    glsl_source,       // &str - GLSL code
    ShaderStage::Vertex, // or ShaderStage::Fragment
    Some("main"),      // Entry point (default: "main")
)?;
```

**Error Types**:

```rust
pub enum ShaderCompilationError {
    ParseError(String),         // GLSL syntax errors
    ValidationError(String),    // Semantic errors
    SpirVGenerationError(String), // WGSL generation errors
    InvalidStage,
    EntryPointNotFound(String),
}
```

### Supported GLSL Features

**Version**: GLSL 450 (Vulkan-compatible)

**Supported**:
- Vertex and fragment shaders
- Uniform blocks (`layout(set = X, binding = Y)`)
- Vertex attributes (`layout(location = X)`)
- Fragment outputs (`layout(location = X)`)
- Standard GLSL functions (dot, cross, normalize, etc.)
- Textures and samplers
- Arrays and structs

**Not Supported**:
- Geometry shaders
- Tessellation shaders
- Compute shaders (future)

### Example Vertex Shader

```glsl
#version 450

// Vertex attributes
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

// Uniforms
layout(set = 0, binding = 0) uniform Uniforms {
    mat4 modelViewProj;
    mat4 model;
    vec3 cameraPosition;
};

// Outputs to fragment shader
layout(location = 0) out vec3 fragPosition;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragUV;

void main() {
    vec4 worldPos = model * vec4(position, 1.0);
    fragPosition = worldPos.xyz;
    fragNormal = mat3(model) * normal;
    fragUV = uv;
    gl_Position = modelViewProj * vec4(position, 1.0);
}
```

### Example Fragment Shader

```glsl
#version 450

// Inputs from vertex shader
layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUV;

// Output color
layout(location = 0) out vec4 outColor;

// Uniforms
layout(set = 0, binding = 1) uniform Material {
    vec3 albedo;
    float shininess;
};

void main() {
    vec3 normal = normalize(fragNormal);
    float lighting = max(dot(normal, vec3(0.0, 1.0, 0.0)), 0.2);
    outColor = vec4(albedo * lighting, 1.0);
}
```

## GPU Buffer System

### Architecture

**Location**: `src/gpu/buffer.rs`

The buffer system provides abstractions for:
- **Vertex Buffers**: Geometry data (positions, normals, UVs)
- **Index Buffers**: Triangle indices
- **Uniform Buffers**: Shader constants (matrices, material properties, lights)

### Vertex Buffer Layout

**Standard Layout** (used by all geometry primitives):

```rust
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],  // 12 bytes
    pub normal: [f32; 3],    // 12 bytes
    pub uv: [f32; 2],        // 8 bytes
}
// Total: 32 bytes per vertex
```

**GLSL Attributes**:
```glsl
layout(location = 0) in vec3 position;  // offset 0
layout(location = 1) in vec3 normal;    // offset 12
layout(location = 2) in vec2 uv;        // offset 24
```

### Uniform Buffer Layouts

**Camera Uniforms**:

```rust
#[repr(C)]
pub struct CameraUniforms {
    pub view_matrix: [f32; 16],       // 64 bytes
    pub projection_matrix: [f32; 16], // 64 bytes
    pub camera_position: [f32; 3],    // 12 bytes
    pub _padding: f32,                // 4 bytes (alignment)
}
// Total: 144 bytes
```

**GLSL Binding**:
```glsl
layout(set = 0, binding = 0) uniform CameraUniforms {
    mat4 viewMatrix;
    mat4 projectionMatrix;
    vec3 cameraPosition;
};
```

**Material Uniforms**:

```rust
#[repr(C)]
pub struct MaterialUniforms {
    pub albedo: [f32; 3],     // 12 bytes
    pub shininess: f32,       // 4 bytes
    pub metallic: f32,        // 4 bytes
    pub roughness: f32,       // 4 bytes
    pub _padding: [f32; 2],   // 8 bytes (alignment)
}
// Total: 32 bytes
```

### Creating Buffers

**Vertex Buffer**:
```rust
let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Vertex Buffer"),
    contents: bytemuck::cast_slice(&vertices),
    usage: wgpu::BufferUsages::VERTEX,
});
```

**Uniform Buffer**:
```rust
let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Uniform Buffer"),
    contents: bytemuck::cast_slice(&[uniforms]),
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
});
```

## Lighting System

### Architecture

**Location**: `src/gpu/buffer.rs` (LightData, MultiLightUniforms)

**Components**: `components/graphics/light-directional/`, `components/graphics/light-point/`, `components/graphics/lighting-phong/`

### Multi-Light Uniform Buffer

**Rust Definition**:

```rust
pub const MAX_LIGHTS: usize = 8;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightData {
    pub position_or_direction: [f32; 3],  // 12 bytes
    pub light_type: u32,                  // 4 bytes (0=directional, 1=point)
    pub color: [f32; 3],                  // 12 bytes
    pub intensity: f32,                   // 4 bytes
    pub radius: f32,                      // 4 bytes (point lights only)
    pub _padding: [f32; 3],               // 12 bytes (alignment)
}
// Total: 48 bytes per light

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MultiLightUniforms {
    pub lights: [LightData; MAX_LIGHTS],  // 384 bytes (48 * 8)
    pub light_count: u32,                 // 4 bytes
    pub _padding: [f32; 3],               // 12 bytes (alignment)
}
// Total: 400 bytes
```

**GLSL Definition**:

```glsl
const uint MAX_LIGHTS = 8u;
const uint LIGHT_TYPE_DIRECTIONAL = 0u;
const uint LIGHT_TYPE_POINT = 1u;

struct LightData {
    vec3 positionOrDirection;  // For directional: direction, for point: position
    uint lightType;            // 0 = directional, 1 = point
    vec3 color;                // RGB color
    float intensity;           // Brightness multiplier
    float radius;              // Attenuation radius (point lights only)
    vec3 _padding;
};

layout(set = 0, binding = 1) uniform Lights {
    LightData lights[MAX_LIGHTS];
    uint lightCount;
} u_lights;
```

### Light Types

**Directional Light** (Sun-like, parallel rays):

```rust
// Component: light-directional
DirectionalLightData {
    light_type: "directional",
    direction: [0.0, -1.0, 0.0],  // Pointing down
    color: [1.0, 1.0, 1.0],       // White
    intensity: 1.0,
}
```

**Shader Usage**:
```glsl
vec3 lightDir = normalize(light.positionOrDirection);
float diffuse = max(dot(normal, lightDir), 0.0);
vec3 color = light.color * light.intensity * diffuse;
```

**Point Light** (Omni-directional with falloff):

```rust
// Component: light-point
PointLightData {
    light_type: "point",
    position: [0.0, 5.0, 0.0],    // World position
    color: [1.0, 0.8, 0.6],       // Warm white
    intensity: 2.5,
    radius: 10.0,                 // Attenuation range
}
```

**Shader Usage**:
```glsl
vec3 lightDir = normalize(light.positionOrDirection - fragPosition);
float distance = length(light.positionOrDirection - fragPosition);
float attenuation = 1.0 / (1.0 + (distance * distance) / (light.radius * light.radius));
float diffuse = max(dot(normal, lightDir), 0.0);
vec3 color = light.color * light.intensity * diffuse * attenuation;
```

### Phong Lighting Model

**Component**: `lighting-phong` (CPU-side calculation)

**Formula**:
```
Diffuse = max(N · L, 0) * surfaceColor * lightColor
Specular = (R · V)^shininess * lightColor
Final = Diffuse + Specular (clamped to [0, 1])
```

**Where**:
- N = surface normal (normalized)
- L = light direction (normalized)
- R = reflection vector = 2(N · L)N - L
- V = view direction (normalized)
- shininess = specular exponent (1-128, higher = tighter highlights)

**Component Usage**:
```
normal: [0, 1, 0]
light_dir: [0, 1, 0]
view_dir: [0, 1, 0]
surface_color: [0.5, 0.5, 0.5]
light_color: [1, 1, 1]
shininess: 32.0
→ lit_color: [0.9, 0.9, 0.9] (diffuse + specular)
```

### Multi-Light Shader Example

**Location**: `examples/shaders/lighting/multi_light.frag.glsl`

```glsl
void main() {
    vec3 normal = normalize(fragNormal);
    vec3 viewDir = normalize(u_camera.cameraPosition - fragPosition);
    vec3 finalColor = vec3(0.0);

    // Ambient lighting
    finalColor += u_material.albedo * 0.1;

    // Process all active lights
    for (uint i = 0u; i < u_lights.lightCount && i < MAX_LIGHTS; i++) {
        LightData light = u_lights.lights[i];

        vec3 lightColor = vec3(0.0);

        if (light.lightType == LIGHT_TYPE_DIRECTIONAL) {
            // Directional light
            vec3 lightDir = normalize(light.positionOrDirection);
            float diffuse = max(dot(normal, lightDir), 0.0);

            // Specular (Phong)
            vec3 reflectDir = reflect(-lightDir, normal);
            float spec = pow(max(dot(viewDir, reflectDir), 0.0), u_material.shininess);

            lightColor = (diffuse + spec) * light.color * light.intensity;

        } else if (light.lightType == LIGHT_TYPE_POINT) {
            // Point light
            vec3 lightVec = light.positionOrDirection - fragPosition;
            vec3 lightDir = normalize(lightVec);
            float distance = length(lightVec);

            // Attenuation
            float attenuation = 1.0 / (1.0 + (distance * distance) / (light.radius * light.radius));

            // Diffuse
            float diffuse = max(dot(normal, lightDir), 0.0);

            // Specular
            vec3 reflectDir = reflect(-lightDir, normal);
            float spec = pow(max(dot(viewDir, reflectDir), 0.0), u_material.shininess);

            lightColor = (diffuse + spec) * light.color * light.intensity * attenuation;
        }

        finalColor += u_material.albedo * lightColor;
    }

    outColor = vec4(finalColor, 1.0);
}
```

## Built-in Shader Nodes

### Shader Program Linker

**Component ID**: `builtin:graphics:shader-program-linker`

**Purpose**: Links vertex and fragment shaders into a GPU program with validation.

**Workflow**:
1. User provides GLSL source for vertex and fragment shaders
2. Node compiles both shaders via naga
3. Validates compilation success
4. Creates wgpu::ShaderModule for each stage
5. Generates unique program ID (UUID)
6. Displays status in footer (success/error with details)

**Implementation**: `src/builtin/shader_program_linker.rs`

**Data Structure**:
```rust
pub struct LinkedProgram {
    pub id: Uuid,
    pub vertex_shader_source: String,
    pub fragment_shader_source: String,
    pub compilation_status: ProgramStatus,
    pub error_message: Option<String>,
}

pub enum ProgramStatus {
    Idle,       // Not compiled
    Compiling,  // In progress
    Success,    // ✓ Linked successfully
    Failed,     // ✗ Error
}
```

**Linking API**:
```rust
let mut program = LinkedProgram::new();
program.link(
    vertex_source,
    fragment_source,
    Some(&gpu_context),
)?;
// Returns Ok(()) on success, Err(String) with detailed error on failure
```

**Error Reporting**:
- Parse errors with line numbers (when available)
- Validation errors with semantic context
- Shader stage identification (vertex/fragment)
- Recovery hints in error messages

### Shader Preview (Future)

**Component ID**: `builtin:graphics:shader-preview`

**Current Status**: Phase 1 placeholder (UI only, no rendering)

**Future**: Phase 2 will integrate actual WebGPU rendering with texture display.

## Usage Examples

### Example 1: Simple Diffuse Lighting

**Graph Setup**:
```
1. primitive-sphere → geometry (positions, normals, uvs, indices)
2. vec3-construct(0, 5, 10) → camera position
3. vec3-construct(0, 0, 0) → camera target
4. perspective-camera → view_matrix, projection_matrix
5. vec3-construct(0, -1, 0) → light direction (down)
6. light-directional → light_data (JSON)
7. Load basic_diffuse.vert.glsl → vertex_shader
8. Load basic_diffuse.frag.glsl → fragment_shader
9. shader-program-linker → linked program
10. render-target(1920, 1080) → config
11. shader-preview (future: renders output)
```

**Shader Files**: `examples/shaders/lighting/basic_diffuse.vert.glsl`, `.frag.glsl`

### Example 2: Phong Lighting with Specular

**Graph Setup**:
```
Similar to Example 1, but using:
- phong.vert.glsl / phong.frag.glsl
- Additional uniform: material shininess (32.0)
- Camera position passed to fragment shader for view direction
```

**Shader Files**: `examples/shaders/lighting/phong.vert.glsl`, `.frag.glsl`

### Example 3: Multi-Light Scene

**Graph Setup**:
```
1. Geometry setup (sphere/cube/plane)
2. Camera setup (perspective)
3. light-directional → sun light
4. light-point(position: [5, 5, 0], color: [1, 0.8, 0.6]) → warm fill
5. light-point(position: [-5, 5, 0], color: [0.6, 0.8, 1]) → cool accent
6. Load multi_light.vert.glsl / multi_light.frag.glsl
7. shader-program-linker → linked program
8. shader-preview (future: displays lit scene)
```

**Shader Files**: `examples/shaders/lighting/multi_light.vert.glsl`, `.frag.glsl`

**Result**: Scene lit by 1 directional + 2 point lights with mixed color temperatures.

### Example 4: CPU-side Phong Calculation

**Graph Setup** (validation/debugging):
```
1. vec3-construct(0, 1, 0) → normal
2. vec3-construct(0, 1, 0) → light_dir
3. vec3-construct(0, 1, 0) → view_dir
4. vec3-construct(0.5, 0.5, 0.5) → surface_color
5. vec3-construct(1, 1, 1) → light_color
6. lighting-phong(shininess: 32) → lit_color
```

**Purpose**: Validate lighting calculations before GPU implementation.

## Troubleshooting

### Shader Compilation Errors

**Error: "GLSL parsing failed"**

**Cause**: Syntax error in GLSL code

**Solution**:
- Check GLSL version (`#version 450`)
- Validate syntax against GLSL 4.5 specification
- Look for typos in variable/function names
- Ensure all statements end with semicolons

**Error: "Shader validation failed"**

**Cause**: Semantic error (type mismatch, undefined variable, etc.)

**Solution**:
- Check variable types match between declarations and usage
- Ensure all uniforms/attributes are declared
- Verify function signatures match usage
- Check array bounds and indexing

**Error: "Entry point 'main' not found"**

**Cause**: Missing or misspelled main() function

**Solution**:
- Ensure shader has `void main() { ... }`
- Check entry point name matches expected value
- Verify shader stage matches code (vertex vs fragment)

### Linking Errors

**Error: "Vertex shader compilation failed"**

**Solution**: Check vertex shader separately, fix compilation errors

**Error: "Fragment shader compilation failed"**

**Solution**: Check fragment shader separately, fix compilation errors

**Error: "Interface mismatch" (TODO: not yet implemented)**

**Cause**: Vertex outputs don't match fragment inputs

**Solution**:
- Ensure `out` variables in vertex shader match `in` variables in fragment shader
- Check location indices match: `layout(location = N)`
- Verify types match exactly (vec3 → vec3, not vec3 → vec4)

### Buffer Errors

**Error: Buffer alignment issues**

**Cause**: Uniform buffer data not aligned to GPU requirements

**Solution**:
- Use `#[repr(C)]` on all buffer structs
- Add padding fields to satisfy 16-byte alignment
- Use `bytemuck::Pod` and `bytemuck::Zeroable` derives
- Check buffer sizes are multiples of alignment

**Error: "Uniform not found"**

**Cause**: Shader expects uniform that isn't bound

**Solution**:
- Verify bind group layout matches shader declarations
- Check `set` and `binding` indices match
- Ensure uniform buffer is created and bound
- Validate buffer size matches struct size

### Lighting Issues

**Problem**: Scene is too dark

**Solutions**:
- Increase light intensity (1.5-3.0)
- Add ambient term (0.1-0.2 of albedo)
- Check normal vectors are normalized
- Verify light direction points toward surface (not away)

**Problem**: Point lights don't attenuate

**Solutions**:
- Check radius value is reasonable (5.0-20.0 for typical scenes)
- Verify attenuation formula in shader
- Ensure distance calculation uses world-space positions

**Problem**: Specular highlights missing

**Solutions**:
- Check shininess value (32-128 typical)
- Verify view direction calculation
- Ensure reflection vector uses normalized normal
- Check camera position is passed to shader

### Performance Issues

**Problem**: Shader compilation is slow

**Solutions**:
- Cache compiled shaders by source hash
- Compile shaders at load time, not runtime
- Use async compilation for background loading

**Problem**: Too many lights

**Solutions**:
- Current limit: 8 lights per scene
- Use light culling (only lights affecting object)
- Consider deferred rendering for many lights (future)

## Reference Documentation

**Shader Examples**: `examples/shaders/lighting/README.md`

**Component Documentation**: `components/graphics/README.md`

**Buffer Layouts**: `src/gpu/buffer.rs`

**Shader Compilation**: `src/gpu/shader.rs`

**Test Graphs**: `tests/component_tests/graphics_*.json`

## Future Enhancements (Phase 3+)

- **PBR Materials**: Physically-based rendering with metallic/roughness workflow
- **Shadow Mapping**: Real-time shadows for directional and point lights
- **Deferred Rendering**: Support for many lights (>8) via G-buffer
- **Compute Shaders**: GPU-accelerated particle systems, post-processing
- **Ray Tracing**: Hardware-accelerated ray tracing for reflections/GI
- **Texture Support**: Diffuse maps, normal maps, PBR texture sets
- **Post-Processing**: Bloom, tone mapping, SSAO, etc.

## License

Part of WasmFlow Graphics Library.
