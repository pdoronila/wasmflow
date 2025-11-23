# WasmFlow Example Graphs

This directory contains demonstration `.wasmflow` files showcasing various features of the visual programming system.

## Graphics PBR Examples

Created with `cargo run --bin create_pbr_demos`

### basic_pbr.wasmflow
**Simple PBR scene starter template**

Empty template with detailed instructions for creating a basic PBR scene:
- Single gold sphere
- Directional sun light
- Perspective camera
- PBR material setup

**Recommended for**: Learning the basics of the graphics pipeline

### multi_light_pbr.wasmflow
**Multi-light PBR scene with shadows**

Template for creating dramatic three-point lighting:
- Directional sun light with cascaded shadow maps (CSM)
- Blue point light for accent
- Orange spot light for rim lighting with shadows
- Gold sphere and ground plane

**Demonstrates**: Multiple light types, shadow mapping, three-point lighting

### material_showcase.wasmflow
**PBR material comparison**

Template for comparing different material types side by side:
- Gold metallic material (roughness 0.2)
- Copper metallic material (roughness 0.3)
- Red plastic dielectric material (roughness 0.5)

**Demonstrates**: Metallic vs dielectric materials, roughness effects, F0 calculation, energy conservation

## Scheduler Examples

### scheduler_demo.wasmflow
Priority-based task scheduling demonstration with 3 math tasks at different priorities.

### scheduler_periodic_demo.wasmflow
Time-partitioned scheduling with periodic task execution.

## How to Use

1. **Load a demo file** in WasmFlow
2. **Read the description** - Each file contains detailed setup instructions in its metadata
3. **Add nodes** from the palette according to the instructions
4. **Connect nodes** as described
5. **Execute** the graph

## Creating Your Own Demos

Use the demo creator tools:

```bash
# Create PBR graphics demos
cargo run --bin create_pbr_demos

# Create custom demos programmatically
# See src/bin/create_pbr_demos.rs for an example
```

## Available Graphics Components

### Primitives (Graphics/Primitives)
- `primitive-sphere` - UV sphere with configurable segments/rings
- `primitive-cube` - Box with proper normals (24 vertices)
- `primitive-plane` - Subdivided XZ plane

### Vector Math (Graphics/Vector)
- `vec3-construct` - Build 3D vector from x, y, z
- `vec3-add`, `vec3-subtract` - Vector arithmetic
- `vec3-scale`, `vec3-normalize`, `vec3-dot`, `vec3-cross`

### Matrix Operations (Graphics/Matrix)
- `mat4-construct` - Build 4×4 matrix
- `mat4-multiply` - Matrix multiplication for transforms

### Camera (Graphics/Camera)
- `perspective-camera` - Look-at view + perspective projection

### Lighting (Graphics/Lighting)
- `light-directional` - Sun-like parallel light
- `light-point` - Omni-directional point light with radius
- `light-spot` - Cone-shaped spot light
- `lighting-phong` - CPU-side Phong calculation

### PBR Materials (Graphics/PBR)
- `pbr-material` - Complete PBR material (base color, metallic, roughness, ao)
- `pbr-fresnel` - Fresnel-Schlick approximation
- `pbr-ggx-distribution` - GGX normal distribution function
- `pbr-smith-geometry` - Smith geometry function
- `pbr-brdf` - Complete Cook-Torrance BRDF

### Shadows (Graphics/Shadows)
- `shadow-directional` - Cascaded shadow maps (CSM) for directional lights
- `shadow-point` - Cubemap shadows for point lights
- `shadow-spot` - Perspective shadows for spot lights

### Utilities (Graphics/Color)
- `color-rgb` - Create RGB color with clamping
- `render-target` - Configure render target parameters

### Advanced (Graphics/Advanced)
- `normal-map` - Tangent-space to world-space normal transformation
- `texture-sampler` - CPU-side bilinear texture sampling

## Example GLSL Shaders

Located in `examples/shaders/`:

### Lighting (`lighting/`)
- `basic_diffuse.*` - Simple Lambert diffuse
- `phong.*` - Phong lighting model
- `multi_light.*` - Up to 8 mixed lights

### PBR (`pbr/`)
- `pbr_single_light.*` - Cook-Torrance BRDF with one light
- `pbr_multi_light.*` - Full PBR with multiple lights
- `pbr_normal_mapped.*` - PBR with normal mapping

### Shadows (`shadow/`)
- `shadow_common.glsl` - PCF utilities (shared)
- `shadow_directional.frag` - CSM with cascade selection
- `shadow_point.frag` - Cubemap shadow sampling
- `shadow_spot.frag` - Cone attenuation + shadows

### Skybox (`skybox/`)
- `skybox.*` - Environment rendering with HDR tone mapping

### IBL (`ibl/`)
- `irradiance_convolution.frag` - Diffuse irradiance map
- `prefilter_specular.frag` - Specular pre-filter (GGX)
- `brdf_integration.frag` - BRDF integration LUT
- `pbr_ibl.frag` - Complete PBR with IBL
- `equirect_to_cubemap.frag` - Format conversion utility

## Material Presets

### Metals
- **Gold**: `base_color: [1.0, 0.71, 0.29], metallic: 1.0, roughness: 0.2`
- **Copper**: `base_color: [0.95, 0.64, 0.54], metallic: 1.0, roughness: 0.3`
- **Aluminum**: `base_color: [0.9, 0.9, 0.9], metallic: 1.0, roughness: 0.6`

### Dielectrics
- **Red Plastic**: `base_color: [0.8, 0.1, 0.1], metallic: 0.0, roughness: 0.5`
- **Polished Stone**: `base_color: [0.3, 0.3, 0.35], metallic: 0.0, roughness: 0.2`
- **Rough Fabric**: `base_color: [0.6, 0.1, 0.1], metallic: 0.0, roughness: 0.7`

## Documentation

- **Graphics Pipeline**: `docs/GRAPHICS_PIPELINE_SUMMARY.md` (650+ lines)
- **PBR Implementation**: `docs/PHASE3_PBR_COMPLETE.md` (472 lines)
- **GPU Integration**: `docs/GPU_INTEGRATION.md`
- **Component Library**: `components/graphics/README.md`
- **CLAUDE.md**: Complete development guidelines

## Tips

- Start with `basic_pbr.wasmflow` to learn the fundamentals
- Use `material_showcase.wasmflow` to understand PBR material properties
- Try `multi_light_pbr.wasmflow` for advanced lighting setups
- Refer to GLSL shader examples for GPU-side implementations
- Check component README files for detailed API documentation
