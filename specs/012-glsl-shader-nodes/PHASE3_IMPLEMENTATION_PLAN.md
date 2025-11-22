# Phase 3 Implementation Plan: PBR Material System

**Feature**: GLSL Physically Based Shader Authoring System
**Phase**: 3 - PBR Materials and Texture Sampling
**Category**: Graphics
**Created**: 2025-11-22
**Status**: Planning
**Depends On**: Phase 2 (Complete ✓)

## Overview

Phase 3 extends the graphics system with physically-based rendering (PBR) materials, texture sampling, and advanced lighting. This phase builds on the GPU integration from Phase 2 to enable production-quality material authoring with the metallic/roughness workflow.

**Key Deliverables**:
1. PBR material property nodes (metallic, roughness, normal maps)
2. Texture sampling and loading system
3. Material texture map nodes (albedo, normal, metallic-roughness)
4. Cook-Torrance BRDF implementation
5. Spot light support (completing light types)
6. Fresnel effects and specular reflections
7. Normal mapping and tangent-space calculations

**Total New Nodes**: ~12-15 nodes (1-2 built-in + 10-13 WASM components)

---

## Current State Assessment (Post-Phase 2)

### Already Implemented ✓
- Basic lighting: Directional and point lights
- Multi-light support (up to 8 lights)
- Phong lighting model (diffuse + specular)
- GPU buffer system with uniform layouts
- Shader compilation pipeline (GLSL → WGSL)
- Vertex format: positions, normals, UVs

### Missing for PBR
- Texture loading and sampling
- PBR material properties (metallic, roughness, base color)
- Cook-Torrance BRDF (vs simple Phong)
- Image-based lighting (IBL) - deferred to Phase 4
- Normal mapping
- Spot lights (directional cone)
- Tangent-space calculations

---

## Architecture Decisions

### 1. PBR Workflow Choice

**Decision**: Metallic/Roughness Workflow (vs Specular/Glossiness)

**Rationale**:
- Industry standard (glTF 2.0, Unreal, Unity)
- More physically accurate energy conservation
- Simpler parameter space for artists
- Better texture compression (metallic+roughness in one texture)

**Material Inputs**:
- `base_color` (vec3 or texture): Albedo/diffuse color
- `metallic` (f32 or texture): 0.0 = dielectric, 1.0 = metal
- `roughness` (f32 or texture): 0.0 = smooth, 1.0 = rough
- `normal` (texture, optional): Normal map (tangent space)
- `ao` (f32 or texture, optional): Ambient occlusion
- `emissive` (vec3, optional): Self-illumination

### 2. Texture System Architecture

**Texture Loading Strategy**:
- Built-in node: `texture-loader` (file path → texture data)
- WASM component: `texture-sampler` (UV + texture → color)
- GPU integration: Texture upload and binding

**Texture Format Support** (Phase 3):
- RGBA8 (sRGB) - Base color, emissive
- RG8 or RGB8 - Metallic-roughness combined
- RGB8 (linear) - Normal maps
- R8 - Ambient occlusion, height maps

**Location**:
- `src/gpu/texture.rs` - GPU texture management
- `src/builtin/texture_loader.rs` - File loading built-in node
- `components/graphics/texture-sampler/` - UV sampling component

### 3. BRDF Implementation

**Cook-Torrance Microfacet BRDF**:

```
f(l, v) = diffuse + specular
diffuse = (1 - metallic) * base_color / π
specular = D(h) * G(l, v, h) * F(v, h) / (4 * (n·l) * (n·v))

Where:
  D = GGX Normal Distribution Function
  G = Smith Geometry Function (visibility)
  F = Fresnel-Schlick approximation
  h = normalize(l + v)
```

**Components**:
- `pbr-brdf` - CPU-side BRDF calculation (validation)
- Example shaders: `pbr_metallic_roughness.frag.glsl`

### 4. Normal Mapping

**Tangent-Space Normal Mapping**:
- Vertex format extension: Add tangent vectors (vec4)
- Geometry primitives updated to generate tangents
- Tangent-space → World-space transformation in shader

**New Vertex Layout**:
```rust
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],   // 12 bytes
    pub normal: [f32; 3],     // 12 bytes
    pub uv: [f32; 2],         // 8 bytes
    pub tangent: [f32; 4],    // 16 bytes (w = handedness)
}
// Total: 48 bytes per vertex
```

---

## Implementation Steps

### Step 1: Texture System Foundation (Week 1)

**1.1 GPU Texture Management** (`src/gpu/texture.rs`)
- [ ] Create `Texture` struct with wgpu::Texture and wgpu::TextureView
- [ ] Implement texture creation from raw RGBA data
- [ ] Add texture upload to GPU
- [ ] Create sampler management (linear, nearest, mipmaps)
- [ ] Add bind group support for textures

**1.2 Texture Loader Built-in Node** (`src/builtin/texture_loader.rs`)
- [ ] File picker UI for texture selection
- [ ] Load image files (PNG, JPG via `image` crate)
- [ ] Convert to RGBA8 format
- [ ] Output texture data (binary blob + dimensions)
- [ ] Display thumbnail in footer
- [ ] Cloning support

**1.3 Texture Sampler Component** (`components/graphics/texture-sampler/`)
- [ ] Inputs: `uv` (vec2), `texture_data` (binary), `dimensions` (u32, u32)
- [ ] Bilinear interpolation sampling
- [ ] UV wrapping modes (repeat, clamp, mirror)
- [ ] Output: `color` (vec3 or vec4)
- [ ] Unit tests (9+ tests)

**Files Modified/Created**:
- `src/gpu/texture.rs` (new)
- `src/builtin/texture_loader.rs` (new)
- `src/builtin/mod.rs` (register texture_loader)
- `src/ui/app.rs` (add texture_loader to palette)
- `src/graph/node.rs` (add texture_loader_data field)
- `components/graphics/texture-sampler/` (new WASM component)

**Dependencies**:
```toml
# Cargo.toml
image = "0.24"  # Image loading
```

**Tests**:
- Texture loading from PNG/JPG files
- Bilinear sampling correctness
- UV wrapping modes
- Texture upload to GPU

---

### Step 2: Tangent-Space System (Week 1)

**2.1 Update Vertex Format** (`src/gpu/buffer.rs`)
- [ ] Add `tangent` field to `Vertex` struct
- [ ] Update GLSL vertex attributes (location 3)
- [ ] Add tangent calculation helper functions

**2.2 Update Geometry Primitives** (Week 1)
- [ ] `primitive-sphere`: Generate tangents from UV gradients
- [ ] `primitive-cube`: Per-face tangent generation
- [ ] `primitive-plane`: Simple tangent calculation (aligned with X axis)
- [ ] Update outputs to include `tangents` port
- [ ] Update unit tests

**2.3 Tangent-Space Utility Component** (`components/graphics/tangent-space-normal/`)
- [ ] Inputs: `normal_map` (vec3), `tangent` (vec4), `normal` (vec3)
- [ ] Calculate TBN matrix (tangent, bitangent, normal)
- [ ] Transform normal from tangent space to world space
- [ ] Output: `world_normal` (vec3)
- [ ] Unit tests

**Files Modified**:
- `src/gpu/buffer.rs` (Vertex struct)
- `components/graphics/primitive-sphere/src/lib.rs`
- `components/graphics/primitive-cube/src/lib.rs`
- `components/graphics/primitive-plane/src/lib.rs`
- `components/graphics/tangent-space-normal/` (new)

---

### Step 3: PBR Material Components (Week 2)

**3.1 PBR Material Properties Component** (`components/graphics/pbr-material/`)
- [ ] Inputs: `base_color` (vec3), `metallic` (f32), `roughness` (f32), `ao` (f32, optional)
- [ ] Validation (clamp metallic/roughness to [0, 1])
- [ ] Output: `material_data` (JSON for GPU uniforms)
- [ ] Unit tests (6+ tests)

**3.2 Fresnel Component** (`components/graphics/pbr-fresnel/`)
- [ ] Fresnel-Schlick approximation
- [ ] Inputs: `f0` (vec3 - base reflectivity), `view_dir` (vec3), `half_vector` (vec3)
- [ ] Output: `fresnel` (vec3)
- [ ] Unit tests (5+ tests)

**3.3 GGX Distribution Component** (`components/graphics/pbr-ggx-distribution/`)
- [ ] GGX/Trowbridge-Reitz normal distribution function
- [ ] Inputs: `normal` (vec3), `half_vector` (vec3), `roughness` (f32)
- [ ] Output: `distribution` (f32)
- [ ] Unit tests (6+ tests)

**3.4 Smith Geometry Function Component** (`components/graphics/pbr-smith-geometry/`)
- [ ] Smith geometry/visibility term (GGX variant)
- [ ] Inputs: `normal` (vec3), `view_dir` (vec3), `light_dir` (vec3), `roughness` (f32)
- [ ] Output: `geometry` (f32)
- [ ] Unit tests (6+ tests)

**3.5 Cook-Torrance BRDF Component** (`components/graphics/pbr-brdf/`)
- [ ] Complete Cook-Torrance microfacet BRDF
- [ ] Inputs: All PBR parameters + lighting vectors
- [ ] Combines Fresnel, GGX, Smith into full BRDF
- [ ] Output: `lit_color` (vec3)
- [ ] Unit tests (9+ tests - various material types)

**Files Created**:
- `components/graphics/pbr-material/` (new)
- `components/graphics/pbr-fresnel/` (new)
- `components/graphics/pbr-ggx-distribution/` (new)
- `components/graphics/pbr-smith-geometry/` (new)
- `components/graphics/pbr-brdf/` (new)

---

### Step 4: Spot Light Support (Week 2)

**4.1 Spot Light Component** (`components/graphics/light-spot/`)
- [ ] Inputs: `position` (vec3), `direction` (vec3), `color` (vec3), `intensity` (f32), `inner_cone` (f32), `outer_cone` (f32), `radius` (f32)
- [ ] Calculate cone attenuation (smooth falloff between inner/outer)
- [ ] Distance attenuation (like point lights)
- [ ] Output: `light_data` (JSON)
- [ ] Unit tests (8+ tests)

**4.2 Update GPU Buffer System** (`src/gpu/buffer.rs`)
- [ ] Extend `LightData` struct with spot light parameters:
  - `direction` field (for spot lights)
  - `inner_cone_cos` (f32)
  - `outer_cone_cos` (f32)
  - `light_type = 2` (LIGHT_TYPE_SPOT)
- [ ] Update `MultiLightUniforms` size calculation
- [ ] Add JSON parsing for spot light data

**4.3 Example Spot Light Shader**
- [ ] Create `spot_light.vert.glsl` and `spot_light.frag.glsl`
- [ ] Cone angle attenuation calculation
- [ ] Combined with distance attenuation
- [ ] Documentation in `examples/shaders/lighting/README.md`

**Files Modified/Created**:
- `components/graphics/light-spot/` (new)
- `src/gpu/buffer.rs` (extend LightData)
- `examples/shaders/lighting/spot_light.vert.glsl` (new)
- `examples/shaders/lighting/spot_light.frag.glsl` (new)

---

### Step 5: PBR Example Shaders (Week 3)

**5.1 Complete PBR Shader** (`examples/shaders/pbr/`)
- [ ] `pbr_metallic_roughness.vert.glsl` - Vertex shader with tangent space
- [ ] `pbr_metallic_roughness.frag.glsl` - Full PBR fragment shader
  - Cook-Torrance BRDF implementation
  - Multi-light support
  - Normal mapping
  - Texture sampling (base color, metallic-roughness, normal, AO)
- [ ] `pbr_simple.frag.glsl` - Simplified PBR (constants only, no textures)
- [ ] README.md with buffer layouts and usage examples

**5.2 Material Presets** (`examples/shaders/pbr/materials/`)
- [ ] Gold material parameters
- [ ] Copper material parameters
- [ ] Plastic material parameters
- [ ] Rough metal parameters
- [ ] Glass/dielectric parameters

**Files Created**:
- `examples/shaders/pbr/pbr_metallic_roughness.vert.glsl`
- `examples/shaders/pbr/pbr_metallic_roughness.frag.glsl`
- `examples/shaders/pbr/pbr_simple.frag.glsl`
- `examples/shaders/pbr/README.md`
- `examples/shaders/pbr/materials/README.md`

---

### Step 6: Integration Testing (Week 3)

**6.1 PBR Material Test Graph** (`tests/component_tests/pbr_materials.json`)
- [ ] Test metallic materials (gold, copper)
- [ ] Test dielectric materials (plastic, rubber)
- [ ] Test roughness variations (0.0 to 1.0)
- [ ] Test Fresnel effect at glancing angles
- [ ] Test BRDF component calculations

**6.2 Texture Sampling Test Graph** (`tests/component_tests/texture_sampling.json`)
- [ ] Load test textures (checkerboard, UV test pattern)
- [ ] Sample at various UV coordinates
- [ ] Test wrapping modes (repeat, clamp, mirror)
- [ ] Test bilinear interpolation

**6.3 Complete PBR Workflow** (`tests/component_tests/pbr_complete_workflow.json`)
- [ ] Geometry with tangents
- [ ] Texture loading (base color, normal, metallic-roughness)
- [ ] PBR material setup
- [ ] Multi-light scene (directional + point + spot)
- [ ] Shader compilation with PBR shader
- [ ] Preview rendering

**Files Created**:
- `tests/component_tests/pbr_materials.json`
- `tests/component_tests/texture_sampling.json`
- `tests/component_tests/pbr_complete_workflow.json`

---

### Step 7: Documentation and Polish (Week 3)

**7.1 Update Component README** (`components/graphics/README.md`)
- [ ] Add Phase 3 PBR components section
- [ ] Document texture system (loader + sampler)
- [ ] Document PBR BRDF components (Fresnel, GGX, Smith)
- [ ] Document spot light component
- [ ] Update testing section
- [ ] Mark Phase 3 complete in roadmap

**7.2 Create PBR Integration Guide** (`docs/PBR_MATERIALS.md`)
- [ ] PBR theory overview (metallic/roughness workflow)
- [ ] Cook-Torrance BRDF explanation
- [ ] Material parameter guidelines
- [ ] Texture authoring best practices
- [ ] Example material setups (metals, dielectrics, mixed)
- [ ] Troubleshooting guide

**7.3 Update CLAUDE.md**
- [ ] Add Phase 3 section with PBR guidelines
- [ ] Texture system architecture
- [ ] PBR material component patterns
- [ ] Tangent-space normal mapping
- [ ] Spot light implementation details

**Files Modified/Created**:
- `components/graphics/README.md`
- `docs/PBR_MATERIALS.md` (new)
- `CLAUDE.md`

---

## Component Summary

### New WASM Components (10-11)

**Texture System** (2):
1. `texture-sampler` - UV sampling with interpolation

**PBR Material** (5):
2. `pbr-material` - Material property packaging
3. `pbr-fresnel` - Fresnel-Schlick approximation
4. `pbr-ggx-distribution` - GGX normal distribution
5. `pbr-smith-geometry` - Smith geometry/visibility
6. `pbr-brdf` - Complete Cook-Torrance BRDF

**Lighting** (1):
7. `light-spot` - Spot light with cone attenuation

**Tangent Space** (1):
8. `tangent-space-normal` - Normal map transformation

**Utilities** (2-3):
9. `texture-combine` - Combine separate R, G, B channels (optional)
10. `ao-apply` - Apply ambient occlusion (optional)
11. `emissive-add` - Add emissive contribution (optional)

### Updated WASM Components (3)
- `primitive-sphere` - Add tangent generation
- `primitive-cube` - Add tangent generation
- `primitive-plane` - Add tangent generation

### New Built-in Nodes (1-2)
1. `texture-loader` - File loading with UI picker
2. `material-editor` (optional) - Visual material property editor

---

## GPU Buffer Updates

### New Uniform Buffers

**Material Uniforms**:
```rust
#[repr(C)]
pub struct PbrMaterialUniforms {
    pub base_color: [f32; 3],       // 12 bytes
    pub metallic: f32,              // 4 bytes
    pub roughness: f32,             // 4 bytes
    pub ao: f32,                    // 4 bytes
    pub emissive: [f32; 3],         // 12 bytes
    pub has_base_color_tex: u32,    // 4 bytes (bool)
    pub has_normal_tex: u32,        // 4 bytes
    pub has_metallic_roughness_tex: u32, // 4 bytes
    pub has_ao_tex: u32,            // 4 bytes
    pub _padding: [f32; 2],         // 8 bytes
}
// Total: 64 bytes
```

**GLSL Binding**:
```glsl
layout(set = 0, binding = 2) uniform Material {
    vec3 baseColor;
    float metallic;
    float roughness;
    float ao;
    vec3 emissive;
    bool hasBaseColorTex;
    bool hasNormalTex;
    bool hasMetallicRoughnessTex;
    bool hasAoTex;
} u_material;

// Textures
layout(set = 1, binding = 0) uniform sampler2D u_baseColorTexture;
layout(set = 1, binding = 1) uniform sampler2D u_normalTexture;
layout(set = 1, binding = 2) uniform sampler2D u_metallicRoughnessTexture;
layout(set = 1, binding = 3) uniform sampler2D u_aoTexture;
```

### Updated Light Uniforms

**Extended LightData** (for spot lights):
```rust
#[repr(C)]
pub struct LightData {
    pub position_or_direction: [f32; 3],  // 12 bytes (position for point/spot, direction for directional)
    pub light_type: u32,                  // 4 bytes (0=directional, 1=point, 2=spot)
    pub color: [f32; 3],                  // 12 bytes
    pub intensity: f32,                   // 4 bytes
    pub direction: [f32; 3],              // 12 bytes (spot light cone direction)
    pub radius: f32,                      // 4 bytes (point/spot attenuation radius)
    pub inner_cone_cos: f32,              // 4 bytes (spot light inner cone)
    pub outer_cone_cos: f32,              // 4 bytes (spot light outer cone)
    pub _padding: [f32; 2],               // 8 bytes
}
// Total: 64 bytes per light (increased from 48)
```

---

## Testing Strategy

### Unit Tests (60+ total)
- Texture sampler: 9 tests (bilinear, wrapping, edge cases)
- PBR components: 32 tests (Fresnel 5, GGX 6, Smith 6, BRDF 9, material 6)
- Spot light: 8 tests (cone angles, attenuation, validation)
- Tangent-space: 5 tests (TBN matrix, transformation)

### Integration Tests (3 graphs)
- PBR material test graph (metal vs dielectric)
- Texture sampling test graph (wrapping, interpolation)
- Complete PBR workflow (geometry → textures → materials → lighting → rendering)

### Manual Testing
- Load various texture formats (PNG, JPG)
- Test PBR materials with different parameters
- Verify normal mapping correctness
- Test spot light cone visualization
- Performance testing (texture upload, sampling)

---

## Dependencies

### New Crate Dependencies
```toml
[dependencies]
image = "0.24"    # Texture loading (PNG, JPG, etc.)
# Existing: wgpu, naga, glam, bytemuck
```

### Build Requirements
- All existing Phase 2 dependencies
- Image file test assets (textures/test_checkerboard.png, etc.)

---

## Success Metrics

### Functional Requirements ✓
- [ ] All 10-11 new components implemented and tested
- [ ] Texture loading from PNG/JPG files works
- [ ] Bilinear sampling produces correct colors
- [ ] Cook-Torrance BRDF matches reference implementation
- [ ] Normal mapping transforms correctly to world space
- [ ] Spot lights produce correct cone attenuation
- [ ] All unit tests passing (60+)
- [ ] Integration tests demonstrate complete workflows

### Performance Requirements
- Texture upload: <100ms for 1024×1024 RGBA8
- Texture sampling (CPU): <1ms for 1000 samples
- PBR BRDF calculation (CPU): <5ms for 1000 calculations
- GPU PBR shader: Maintain 30+ FPS for typical scenes

### Quality Requirements
- PBR materials look physically plausible
- Metals show correct Fresnel response
- Rough surfaces scatter light appropriately
- Normal maps produce believable surface detail
- Spot lights have smooth cone falloff

---

## Risk Assessment

### Technical Risks

**Risk**: Image crate adds significant binary size
- **Mitigation**: Use feature flags to enable only needed formats
- **Fallback**: Implement minimal PNG loader if needed

**Risk**: Tangent generation for complex geometry is non-trivial
- **Mitigation**: Start with simple primitives (sphere, cube, plane)
- **Fallback**: Provide pre-generated tangents in test assets

**Risk**: Cook-Torrance BRDF is computationally expensive
- **Mitigation**: Optimize hot paths, use GPU shader for real-time
- **Fallback**: Offer simplified BRDF option

### Timeline Risks

**Risk**: Texture system integration takes longer than estimated (Week 1)
- **Mitigation**: Implement minimal texture support first, expand later
- **Buffer**: Week 1 has some slack for texture system complexity

**Risk**: PBR component implementation delayed (Week 2)
- **Mitigation**: Implement in order of dependency (Fresnel → GGX → Smith → BRDF)
- **Buffer**: Can defer optional utility components to Week 3

---

## Phase 3 Timeline

### Week 1: Texture and Tangent Systems
- **Days 1-3**: Texture system (GPU texture, loader, sampler)
- **Days 4-5**: Tangent-space system (vertex format, geometry updates)
- **Deliverable**: Texture loading and sampling working

### Week 2: PBR Materials and Spot Lights
- **Days 1-4**: PBR components (Fresnel, GGX, Smith, BRDF, material)
- **Days 5**: Spot light implementation
- **Deliverable**: Complete PBR material system

### Week 3: Integration and Documentation
- **Days 1-2**: Example PBR shaders and material presets
- **Days 3-4**: Integration testing (3 test graphs)
- **Day 5**: Documentation (README updates, PBR guide, CLAUDE.md)
- **Deliverable**: Phase 3 complete and documented

**Total**: 3 weeks (15 working days)

---

## Future Work (Phase 4)

### Deferred Features
- Image-based lighting (IBL) and environment maps
- Cubemap processing and reflections
- BRDF lookup tables for performance
- Advanced texture features (mipmaps, anisotropic filtering)
- Texture compression formats
- Compute shaders for post-processing
- Subsurface scattering
- Clearcoat/sheen layers

### Phase 4 Preview
Phase 4 will focus on advanced rendering features:
- Environment maps and skyboxes
- IBL with split-sum approximation
- BRDF LUT generation
- Post-processing effects (bloom, tone mapping, SSAO)
- Compute shader integration
- Performance optimizations

---

## Questions & Decisions

### Open Questions
- **Q**: Support HDR textures (EXR, HDR formats) in Phase 3?
  - **A**: Defer to Phase 4 (IBL requires HDR)

- **Q**: Implement texture mipmaps in Phase 3?
  - **A**: Basic support yes, auto-generation defer to Phase 4

- **Q**: Include material editor built-in node?
  - **A**: Optional - can create materials via component nodes

### Key Decisions
- ✓ **Texture Format**: RGBA8 sRGB for base color, linear RGB for normal/metallic-roughness
- ✓ **BRDF Choice**: Cook-Torrance with GGX, Smith, Fresnel-Schlick (industry standard)
- ✓ **Tangent Format**: vec4 with handedness in W component
- ✓ **Material Workflow**: Metallic/Roughness (vs Specular/Glossiness)
- ✓ **Spot Light Model**: Inner/outer cone angles + distance attenuation

---

## References

### PBR Theory
- "Physically Based Rendering: From Theory to Implementation" (PBRT)
- Unreal Engine PBR documentation
- Unity Standard Shader documentation
- glTF 2.0 specification (metallic-roughness)

### Technical References
- Real Shading in Unreal Engine 4 (Brian Karis, SIGGRAPH 2013)
- Moving Frostbite to PBR (Sébastien Lagarde, SIGGRAPH 2014)
- LearnOpenGL PBR tutorial
- WebGPU specification for texture formats

---

## Next Steps

1. **Review and Approve Plan**: Discuss architecture decisions and timeline
2. **Set Up Test Assets**: Create test textures (checkerboard, normal maps, etc.)
3. **Begin Step 1**: Start with texture system implementation
4. **Iterate**: Build incrementally, test frequently
5. **Document**: Keep docs updated as implementation progresses

---

**Plan Status**: Ready for Implementation
**Estimated Start**: TBD
**Estimated Completion**: +3 weeks from start
