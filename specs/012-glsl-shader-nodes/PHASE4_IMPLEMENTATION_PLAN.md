# Phase 4 Implementation Plan: Advanced Rendering Features

**Feature**: GLSL Physically Based Shader Authoring System
**Phase**: 4 - Advanced Rendering and Optimization
**Category**: Graphics
**Created**: 2025-11-22
**Status**: Planning
**Depends On**: Phase 3 (Complete ✓)

## Overview

Phase 4 extends the graphics system with advanced rendering features including shadow mapping, image-based lighting (IBL), post-processing effects, and performance optimizations. This phase transforms the system from a capable PBR renderer into a production-quality rendering pipeline.

**Key Deliverables**:
1. Shadow mapping (directional, point, spot lights)
2. Image-based lighting with environment maps
3. Post-processing effects (bloom, advanced tone mapping, SSAO)
4. Advanced PBR features (clear coat, subsurface scattering)
5. Performance optimizations (deferred rendering, compute shaders)

**Total New Components**: ~20-25 components + 3-5 built-in nodes

---

## Current State Assessment (Post-Phase 3)

### Already Implemented ✓
- Complete PBR material system (Cook-Torrance BRDF)
- Multi-light support (directional, point, spot - up to 8)
- Texture loading and sampling
- Normal mapping
- GPU texture management with depth texture support
- Shader compilation pipeline (GLSL → WGSL)

### Missing for Advanced Rendering
- Shadow mapping and shadow texture sampling
- Cubemap support and environment maps
- IBL (image-based lighting) with diffuse/specular
- BRDF lookup tables
- Post-processing pipeline
- HDR texture formats
- Advanced PBR layers (clear coat, SSS)
- Deferred rendering support
- Compute shader integration

---

## Architecture Decisions

### 1. Shadow Mapping Strategy

**Decision**: Cascaded Shadow Maps (CSM) for directional lights, standard shadow maps for point/spot

**Rationale**:
- CSM provides better shadow quality at varying distances
- Standard cubemap shadows for point lights (6 faces)
- Spot lights use simple perspective shadow maps
- PCF (Percentage Closer Filtering) for soft shadows

**Shadow Map Formats**:
- Directional: 2048×2048 depth texture (4 cascades)
- Point: 1024×1024 cubemap depth texture
- Spot: 1024×1024 depth texture

### 2. IBL Architecture

**Decision**: Pre-computed split-sum approximation with runtime sampling

**Rationale**:
- Split-sum approximation is industry standard (Epic, Unity)
- Pre-filtered environment maps for different roughness levels
- Pre-integrated BRDF lookup table (128×128 or 256×256)
- Runtime cubemap sampling in fragment shader

**IBL Components**:
- Environment map loader (HDR: .hdr, .exr files)
- Cubemap pre-filter (compute shader or CPU)
- BRDF LUT generator (compute shader)
- IBL sampler (WASM component for CPU validation)

### 3. Post-Processing Pipeline

**Decision**: Multi-pass render pipeline with intermediate textures

**Rationale**:
- Each effect as separate render pass
- Ping-pong between render targets
- Composable effects (can enable/disable individually)

**Effect Order**:
1. Scene render → HDR buffer
2. Bloom (downsampling + blur + upsample)
3. SSAO (screen-space occlusion)
4. Tone mapping (HDR → LDR)
5. Color grading (optional)
6. Final composite

### 4. Advanced PBR Layers

**Decision**: Multi-layer BRDF with blending

**Clear Coat**:
- Second specular lobe on top of base layer
- Separate roughness for clear coat
- Common in car paint, plastics

**Subsurface Scattering**:
- Burley diffusion profile
- Subsurface color and distance parameters
- Used for skin, wax, marble

---

## Implementation Steps

### Step 1: Shadow Mapping Foundation (Week 1-2)

**1.1 Shadow Texture System** (`src/gpu/shadow.rs`)
- [ ] Create shadow texture type (depth-only render targets)
- [ ] Shadow map atlas for multiple lights
- [ ] Shadow sampler with PCF support
- [ ] Cascade frustum calculation for CSM

**1.2 Shadow Map Components**

**`shadow-directional`** (`components/graphics/shadow-directional/`)
- [ ] Inputs: `light_direction` (vec3), `view_matrix` (mat4), `projection_matrix` (mat4)
- [ ] Calculate light view/projection matrices
- [ ] Output: `shadow_matrix` (mat4), `cascade_splits` (list)
- [ ] Unit tests (6+ tests)

**`shadow-point`** (`components/graphics/shadow-point/`)
- [ ] Inputs: `light_position` (vec3), `near`, `far`
- [ ] Generate 6 view matrices (cubemap faces)
- [ ] Output: `shadow_matrices` (list of 6 mat4)
- [ ] Unit tests (5+ tests)

**`shadow-spot`** (`components/graphics/shadow-spot/`)
- [ ] Inputs: `light_position`, `light_direction`, `cone_angle`, `near`, `far`
- [ ] Calculate spot light view/projection
- [ ] Output: `shadow_matrix` (mat4)
- [ ] Unit tests (5+ tests)

**1.3 Shadow Sampling Shaders**
- [ ] `shadow_pcf.glsl` - PCF filtering functions
- [ ] `shadow_directional.frag.glsl` - Directional shadow sampling
- [ ] `shadow_point.frag.glsl` - Point light shadow sampling (cubemap)
- [ ] `shadow_spot.frag.glsl` - Spot light shadow sampling

**Files Created**:
- `src/gpu/shadow.rs` (new)
- `components/graphics/shadow-directional/` (new)
- `components/graphics/shadow-point/` (new)
- `components/graphics/shadow-spot/` (new)
- `examples/shaders/shadows/` (new directory with shaders)

**Tests**:
- Shadow matrix calculation tests
- Cascade split calculation tests
- PCF filtering validation
- Integration test: `graphics_shadow_mapping.json`

---

### Step 2: Environment Maps and Cubemaps (Week 2-3)

**2.1 Cubemap Texture Support** (`src/gpu/texture.rs` extensions)
- [ ] Add `TextureFormat::Cubemap` variant
- [ ] Cubemap texture creation from 6 faces
- [ ] Cubemap sampler configuration
- [ ] HDR texture support (Rgba16Float, Rgba32Float)

**2.2 Environment Map Loader** (Built-in node: `src/builtin/environment_map_loader.rs`)
- [ ] Load HDR environment maps (.hdr, .exr formats)
- [ ] Load cubemap faces (6 separate images)
- [ ] Convert equirectangular → cubemap
- [ ] Display preview in footer
- [ ] Output: `environment_map` (cubemap texture data)

**2.3 Environment Map Components**

**`cubemap-construct`** (`components/graphics/cubemap-construct/`)
- [ ] Inputs: 6 texture faces (pos_x, neg_x, pos_y, neg_y, pos_z, neg_z)
- [ ] Validate all faces same size
- [ ] Output: `cubemap` (cubemap texture data)
- [ ] Unit tests (7+ tests)

**`cubemap-sample`** (`components/graphics/cubemap-sample/`)
- [ ] Inputs: `direction` (vec3), `cubemap` (texture), `lod` (f32, optional)
- [ ] Sample cubemap in given direction
- [ ] Output: `color` (vec3 or vec4)
- [ ] Unit tests (8+ tests)

**`equirect-to-cubemap`** (`components/graphics/equirect-to-cubemap/`)
- [ ] Input: `equirectangular` (texture)
- [ ] Convert to 6 cubemap faces
- [ ] Output: `cubemap` (cubemap texture data)
- [ ] Unit tests (5+ tests)

**Files Created**:
- `src/builtin/environment_map_loader.rs` (new)
- `components/graphics/cubemap-construct/` (new)
- `components/graphics/cubemap-sample/` (new)
- `components/graphics/equirect-to-cubemap/` (new)

**Dependencies**:
```toml
# For HDR/EXR loading
image = { version = "0.25", features = ["hdr", "exr"] }
```

---

### Step 3: Image-Based Lighting (IBL) (Week 3-4)

**3.1 BRDF LUT Generator** (Built-in node: `src/builtin/brdf_lut_generator.rs`)
- [ ] Generate pre-integrated BRDF lookup table
- [ ] Size: 128×128 or 256×256 (RG16Float)
- [ ] Cook-Torrance integration over hemisphere
- [ ] Cache generated LUT
- [ ] Output: `brdf_lut` (texture)

**3.2 Environment Prefiltering** (Compute shader or CPU)
- [ ] Convolve environment map for diffuse irradiance
- [ ] Pre-filter specular for different roughness levels (mipmap chain)
- [ ] Generate up to 5-7 roughness levels
- [ ] Store in mipmap chain of cubemap

**3.3 IBL Components**

**`ibl-diffuse`** (`components/graphics/ibl-diffuse/`)
- [ ] Inputs: `normal` (vec3), `irradiance_map` (cubemap)
- [ ] Sample irradiance map for diffuse lighting
- [ ] Output: `diffuse` (vec3)
- [ ] Unit tests (5+ tests)

**`ibl-specular`** (`components/graphics/ibl-specular/`)
- [ ] Inputs: `normal`, `view_dir`, `roughness`, `prefiltered_map` (cubemap), `brdf_lut` (texture)
- [ ] Sample pre-filtered environment at roughness level
- [ ] Apply BRDF LUT for Fresnel term
- [ ] Output: `specular` (vec3)
- [ ] Unit tests (7+ tests)

**`ibl-combine`** (`components/graphics/ibl-combine/`)
- [ ] Inputs: `diffuse`, `specular`, `ao`, `base_color`, `metallic`
- [ ] Combine IBL diffuse + specular with material properties
- [ ] Output: `ibl_color` (vec3)
- [ ] Unit tests (6+ tests)

**3.4 IBL Shaders**
- [ ] `ibl_pbr.frag.glsl` - PBR with IBL support
- [ ] Functions: `sampleDiffuseIrradiance()`, `sampleSpecularIBL()`

**Files Created**:
- `src/builtin/brdf_lut_generator.rs` (new)
- `components/graphics/ibl-diffuse/` (new)
- `components/graphics/ibl-specular/` (new)
- `components/graphics/ibl-combine/` (new)
- `examples/shaders/ibl/` (new directory)

**Tests**:
- BRDF LUT generation validation
- IBL sampling correctness
- Integration test: `graphics_ibl_workflow.json`

---

### Step 4: Post-Processing Effects (Week 4-5)

**4.1 Bloom Effect**

**`bloom-downsample`** (`components/graphics/bloom-downsample/`)
- [ ] Input: `hdr_texture` (texture), `threshold` (f32)
- [ ] Extract bright pixels (> threshold)
- [ ] Downsample to multiple levels (4-5 levels)
- [ ] Output: `bloom_chain` (list of textures)
- [ ] Unit tests (5+ tests)

**`bloom-blur`** (`components/graphics/bloom-blur/`)
- [ ] Input: `texture` (texture), `blur_radius` (f32)
- [ ] Gaussian blur (two-pass: horizontal + vertical)
- [ ] Output: `blurred` (texture)
- [ ] Unit tests (4+ tests)

**`bloom-composite`** (`components/graphics/bloom-composite/`)
- [ ] Inputs: `scene` (texture), `bloom` (texture), `intensity` (f32)
- [ ] Upsample and add bloom to scene
- [ ] Output: `composited` (texture)
- [ ] Unit tests (5+ tests)

**4.2 Screen-Space Ambient Occlusion (SSAO)**

**`ssao-generate`** (`components/graphics/ssao-generate/`)
- [ ] Inputs: `depth` (texture), `normals` (texture), `noise` (texture), `samples` (u32)
- [ ] Generate occlusion factor using depth sampling
- [ ] Output: `ao` (texture - R8 or R16)
- [ ] Unit tests (6+ tests)

**`ssao-blur`** (`components/graphics/ssao-blur/`)
- [ ] Input: `ao` (texture)
- [ ] Bilateral blur to reduce noise
- [ ] Output: `blurred_ao` (texture)
- [ ] Unit tests (3+ tests)

**4.3 Advanced Tone Mapping**

**`tone-map-aces`** (`components/graphics/tone-map-aces/`)
- [ ] Input: `hdr_color` (vec3)
- [ ] ACES filmic tone mapping
- [ ] Output: `ldr_color` (vec3)
- [ ] Unit tests (5+ tests)

**`tone-map-uncharted2`** (`components/graphics/tone-map-uncharted2/`)
- [ ] Input: `hdr_color` (vec3), `exposure` (f32)
- [ ] Uncharted 2 tone mapping (John Hable)
- [ ] Output: `ldr_color` (vec3)
- [ ] Unit tests (5+ tests)

**4.4 Post-Processing Shaders**
- [ ] `bloom.frag.glsl` - Bloom extraction and blur
- [ ] `ssao.frag.glsl` - SSAO generation
- [ ] `tone_mapping.frag.glsl` - Advanced tone mapping operators

**Files Created**:
- `components/graphics/bloom-*` (3 components)
- `components/graphics/ssao-*` (2 components)
- `components/graphics/tone-map-*` (2 components)
- `examples/shaders/postprocessing/` (new directory)

**Tests**:
- Bloom extraction validation
- SSAO occlusion factor tests
- Tone mapping curve tests
- Integration test: `graphics_postprocessing.json`

---

### Step 5: Advanced PBR Layers (Week 5-6)

**5.1 Clear Coat Layer**

**`pbr-clearcoat`** (`components/graphics/pbr-clearcoat/`)
- [ ] Inputs: `base_brdf`, `normal`, `clearcoat_normal`, `clearcoat_roughness`, `clearcoat_strength`
- [ ] Calculate second specular lobe
- [ ] Combine with base layer
- [ ] Output: `brdf_with_clearcoat` (vec3)
- [ ] Unit tests (8+ tests)

**5.2 Subsurface Scattering**

**`pbr-subsurface`** (`components/graphics/pbr-subsurface/`)
- [ ] Inputs: `base_color`, `subsurface_color`, `subsurface_radius`, `thickness`
- [ ] Burley diffusion approximation
- [ ] Output: `sss_diffuse` (vec3)
- [ ] Unit tests (7+ tests)

**5.3 Advanced PBR Shaders**
- [ ] `pbr_clearcoat.frag.glsl` - PBR with clear coat
- [ ] `pbr_subsurface.frag.glsl` - PBR with SSS
- [ ] `pbr_full.frag.glsl` - All PBR features combined

**Files Created**:
- `components/graphics/pbr-clearcoat/` (new)
- `components/graphics/pbr-subsurface/` (new)
- `examples/shaders/pbr_advanced/` (new directory)

**Material Presets with Advanced Features**:
- Car Paint (clear coat)
- Skin (subsurface scattering)
- Wax (SSS + translucency)

---

### Step 6: Performance Optimizations (Week 6-7)

**6.1 Deferred Rendering Support**

**`gbuffer-pack`** (`components/graphics/gbuffer-pack/`)
- [ ] Inputs: `position`, `normal`, `base_color`, `metallic`, `roughness`, `ao`
- [ ] Pack into G-Buffer format (multiple render targets)
- [ ] Output: `gbuffer_data` (JSON configuration)
- [ ] Unit tests (5+ tests)

**`gbuffer-unpack`** (`components/graphics/gbuffer-unpack/`)
- [ ] Input: `gbuffer_data` (textures)
- [ ] Unpack into individual components
- [ ] Outputs: All material properties
- [ ] Unit tests (5+ tests)

**6.2 Compute Shader Support** (`src/gpu/compute.rs`)
- [ ] Compute pipeline creation
- [ ] Compute shader compilation
- [ ] Dispatch and synchronization
- [ ] Storage buffer bindings

**6.3 Light Culling** (`components/graphics/light-culling/`)
- [ ] Tiled or clustered light culling
- [ ] Frustum culling for lights
- [ ] Output: Visible light indices per tile/cluster

**Files Created**:
- `src/gpu/compute.rs` (new)
- `components/graphics/gbuffer-pack/` (new)
- `components/graphics/gbuffer-unpack/` (new)
- `components/graphics/light-culling/` (new)
- `examples/shaders/deferred/` (new directory)

---

### Step 7: Integration and Documentation (Week 7)

**7.1 Complete Workflow Integration Tests**
- [ ] `graphics_shadow_mapping.json` - All shadow types
- [ ] `graphics_ibl_workflow.json` - Complete IBL setup
- [ ] `graphics_postprocessing.json` - Multi-pass post-processing
- [ ] `graphics_advanced_pbr.json` - Clear coat + SSS materials
- [ ] `graphics_deferred_rendering.json` - G-Buffer workflow
- [ ] `graphics_complete_scene.json` - Everything combined

**7.2 Documentation**

**Create `docs/PHASE4_ADVANCED_RENDERING.md`**:
- [ ] Shadow mapping techniques and best practices
- [ ] IBL theory and implementation
- [ ] Post-processing pipeline architecture
- [ ] Advanced PBR layer documentation
- [ ] Performance optimization guidelines
- [ ] Complete feature reference

**Update Existing Docs**:
- [ ] `components/graphics/README.md` - Add Phase 4 sections
- [ ] `CLAUDE.md` - Add Phase 4 implementation notes
- [ ] `examples/shaders/README.md` - Document new shader examples

**7.3 Example Scenes**
- [ ] Create example scene: "Car Showroom" (clear coat, IBL, shadows)
- [ ] Create example scene: "Character Portrait" (SSS, IBL, soft shadows)
- [ ] Create example scene: "Outdoor Environment" (CSM shadows, IBL skybox)

---

## Component Summary

### New WASM Components (20-22)

**Shadow Mapping** (3):
1. `shadow-directional` - CSM shadow matrix calculation
2. `shadow-point` - Cubemap shadow matrices
3. `shadow-spot` - Spot light shadow matrix

**Environment Maps** (3):
4. `cubemap-construct` - Build cubemap from 6 faces
5. `cubemap-sample` - Sample cubemap by direction
6. `equirect-to-cubemap` - Convert equirectangular to cubemap

**Image-Based Lighting** (3):
7. `ibl-diffuse` - Diffuse irradiance sampling
8. `ibl-specular` - Specular IBL with BRDF LUT
9. `ibl-combine` - Combine IBL components

**Post-Processing** (7):
10. `bloom-downsample` - Bright pixel extraction
11. `bloom-blur` - Gaussian blur
12. `bloom-composite` - Add bloom to scene
13. `ssao-generate` - Screen-space AO generation
14. `ssao-blur` - Bilateral blur for SSAO
15. `tone-map-aces` - ACES filmic tone mapping
16. `tone-map-uncharted2` - Uncharted 2 tone mapping

**Advanced PBR** (2):
17. `pbr-clearcoat` - Clear coat layer
18. `pbr-subsurface` - Subsurface scattering

**Performance** (2-4):
19. `gbuffer-pack` - Pack G-Buffer data
20. `gbuffer-unpack` - Unpack G-Buffer
21. `light-culling` (optional) - Tiled/clustered culling
22. `frustum-cull` (optional) - Frustum culling

### New Built-in Nodes (2-3)

1. `environment-map-loader` - Load HDR environment maps
2. `brdf-lut-generator` - Generate BRDF lookup table
3. `compute-shader-runner` (optional) - Run compute shaders

---

## GPU Features

### Extended Texture Formats

```rust
// In src/gpu/texture.rs
pub enum TextureFormat {
    // Existing
    Rgba8Srgb,
    Rgba8Linear,

    // New for Phase 4
    Rgba16Float,      // HDR textures
    Rgba32Float,      // HDR textures
    Rg16Float,        // BRDF LUT
    R8Unorm,          // AO textures
    Depth32Float,     // Shadow maps (already exists)
    DepthCubemap,     // Point light shadows
}
```

### Shadow Map Uniform Buffers

```rust
#[repr(C)]
pub struct ShadowUniforms {
    pub shadow_matrices: [[f32; 16]; 4],  // Up to 4 cascades
    pub cascade_splits: [f32; 4],         // Cascade split distances
    pub shadow_map_size: f32,             // Shadow map resolution
    pub pcf_samples: u32,                 // PCF sample count
    pub bias: f32,                        // Shadow bias
    pub _padding: f32,
}
// Total: 4×64 + 4×4 + 4 + 4 + 4 + 4 = 288 bytes
```

### IBL Uniform Buffers

```rust
#[repr(C)]
pub struct IBLUniforms {
    pub irradiance_intensity: f32,     // Diffuse IBL strength
    pub specular_intensity: f32,       // Specular IBL strength
    pub max_reflection_lod: f32,       // Max mip level for reflections
    pub _padding: f32,
}
// Total: 16 bytes
```

---

## Dependencies

### New Crate Dependencies

```toml
[dependencies]
# Existing
image = { version = "0.25", features = ["png", "jpeg", "bmp", "gif", "hdr", "exr"] }

# HDR/EXR support (may need separate crates)
exr = "1.7"  # For OpenEXR files
```

---

## Testing Strategy

### Unit Tests (~100+ tests)
- Shadow matrix calculations: 16 tests
- Cubemap operations: 20 tests
- IBL sampling: 18 tests
- Post-processing: 18 tests
- Advanced PBR: 15 tests
- Performance features: 10 tests

### Integration Tests (6 graphs)
- Shadow mapping workflow
- IBL complete pipeline
- Post-processing chain
- Advanced PBR materials
- Deferred rendering
- Complete scene with all features

### Performance Targets
- Shadow map generation: <5ms per light (1024×1024)
- BRDF LUT generation: <50ms (256×256)
- Bloom: <3ms (1920×1080 → 240×135 → blur → composite)
- SSAO: <5ms (1920×1080)
- Deferred lighting: 100+ lights at 60 FPS

---

## Success Metrics

### Functional Requirements
- [ ] All 20-22 components implemented and tested
- [ ] Shadow mapping produces correct shadows
- [ ] IBL provides realistic environment lighting
- [ ] Post-processing effects work correctly
- [ ] Advanced PBR materials look physically accurate
- [ ] All unit tests passing (100+)
- [ ] Integration tests demonstrate complete workflows

### Quality Requirements
- Shadows have minimal artifacts (peter-panning, acne)
- IBL matches offline renders (validate against Blender/Maya)
- Post-processing doesn't introduce visual artifacts
- Clear coat and SSS look believable
- Performance meets targets on mid-range GPUs

### Performance Requirements
- Maintain 60 FPS with shadows + IBL + post-processing
- Deferred rendering supports 100+ lights
- Memory usage stays under 512MB for typical scenes

---

## Risk Assessment

### Technical Risks

**Risk**: HDR texture loading adds significant dependencies
- **Mitigation**: Use feature flags, provide fallback formats
- **Fallback**: Support only .hdr format initially, add .exr later

**Risk**: BRDF LUT generation is complex
- **Mitigation**: Pre-generate common LUTs, provide as assets
- **Fallback**: Use approximation formulas instead of LUT

**Risk**: Compute shaders not supported on all platforms
- **Mitigation**: Provide CPU fallback for critical features
- **Fallback**: Use fragment shader alternatives

**Risk**: Deferred rendering requires multiple render targets
- **Mitigation**: Check MRT support at runtime
- **Fallback**: Keep forward rendering as option

### Timeline Risks

**Risk**: IBL implementation takes longer than estimated (Week 3-4)
- **Mitigation**: Start with simplified IBL (single mip level)
- **Buffer**: Can defer advanced IBL features to polish phase

**Risk**: Compute shader integration delayed (Week 6)
- **Mitigation**: Use fragment shader alternatives initially
- **Buffer**: Compute shaders are optimization, not requirement

---

## Phase 4 Timeline

### Week 1-2: Shadow Mapping
- **Days 1-4**: Shadow texture system and component implementation
- **Days 5-7**: Shadow sampling shaders and integration
- **Days 8-10**: Testing and bug fixes
- **Deliverable**: Complete shadow mapping system

### Week 2-3: Environment Maps and Cubemaps
- **Days 1-3**: Cubemap texture support
- **Days 4-6**: Environment map loader built-in node
- **Days 7-10**: Cubemap components and conversion utilities
- **Deliverable**: Environment map loading and sampling

### Week 3-4: Image-Based Lighting
- **Days 1-3**: BRDF LUT generator
- **Days 4-7**: IBL components (diffuse, specular, combine)
- **Days 8-10**: IBL shaders and integration tests
- **Deliverable**: Complete IBL pipeline

### Week 4-5: Post-Processing Effects
- **Days 1-4**: Bloom effect (downsample, blur, composite)
- **Days 5-7**: SSAO implementation
- **Days 8-10**: Advanced tone mapping
- **Deliverable**: Post-processing pipeline

### Week 5-6: Advanced PBR Layers
- **Days 1-4**: Clear coat implementation
- **Days 5-8**: Subsurface scattering
- **Days 9-10**: Advanced PBR shaders
- **Deliverable**: Advanced material system

### Week 6-7: Performance Optimizations
- **Days 1-4**: Deferred rendering (G-Buffer)
- **Days 5-7**: Compute shader support
- **Days 8-10**: Light culling and optimizations
- **Deliverable**: Performance features

### Week 7: Integration and Documentation
- **Days 1-3**: Complete integration tests
- **Days 4-6**: Documentation (all docs)
- **Day 7**: Example scenes and polish
- **Deliverable**: Phase 4 complete and documented

**Total**: 7 weeks (49 working days)

---

## Future Work (Beyond Phase 4)

### Deferred to Future Phases
- Volumetric effects (fog, god rays)
- Temporal anti-aliasing (TAA)
- Ray tracing integration
- Global illumination (light probes, voxel GI)
- Advanced cloth shading
- Hair/fur rendering
- Water rendering
- Terrain rendering systems

---

## References

### Shadow Mapping
- "Cascaded Shadow Maps" (Microsoft DirectX documentation)
- "Percentage-Closer Soft Shadows" (Randima Fernando, NVIDIA)
- "Variance Shadow Maps" (Andrew Lauritzen, 2006)

### Image-Based Lighting
- "Real Shading in Unreal Engine 4" (Brian Karis, SIGGRAPH 2013)
- "Moving Frostbite to PBR" (Sébastien Lagarde, SIGGRAPH 2014)
- "Physically Based Shading at Disney" (Brent Burley, 2012)

### Post-Processing
- "Next Generation Post Processing in Call of Duty: Advanced Warfare" (Jorge Jimenez, SIGGRAPH 2014)
- "Practical Real-Time Strategies for Accurate Indirect Occlusion" (Louis Bavoil, NVIDIA)
- "High Quality Bloom" (Kawase filtering)

### Advanced PBR
- "Extending the Disney BRDF to a BSDF with Integrated Subsurface Scattering" (Brent Burley, 2015)
- "Approximate Reflectance Profiles for Efficient Subsurface Scattering" (Per Christensen, Pixar)

---

## Next Steps

1. **Review and Approve Plan**: Confirm Phase 4 scope and priorities
2. **Prioritize Features**: Choose which features to implement first
3. **Begin Step 1**: Start with shadow mapping foundation
4. **Iterate**: Build incrementally, test frequently
5. **Document**: Keep docs updated as implementation progresses

---

**Plan Status**: Ready for Implementation
**Estimated Start**: 2025-11-22
**Estimated Completion**: +7 weeks from start (2026-01-10)
