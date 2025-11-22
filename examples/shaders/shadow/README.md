# Shadow Mapping Shaders

This directory contains GLSL fragment shaders for shadow mapping with PCF (Percentage Closer Filtering).

## Components

### Shadow Matrix Calculation (WASM Components)

**shadow-directional** (`components/graphics/shadow-directional/`)
- Calculates cascaded shadow map matrices for directional lights (sun, moon)
- Inputs: light_direction, view_matrix, projection_matrix, near, far, cascade_count (1-4)
- Outputs: shadow_matrices (flattened), cascade_splits
- Uses practical split scheme (λ=0.5) for optimal cascade distribution
- Binary: `components/bin/shadow_directional.wasm` (105 KB)

**shadow-point** (`components/graphics/shadow-point/`)
- Calculates 6 cubemap shadow matrices for point lights (omni-directional)
- Inputs: light_position, near, far
- Output: shadow_matrices (96 floats = 6 matrices)
- 90° FOV perspective projection for each cubemap face
- Face order: +X, -X, +Y, -Y, +Z, -Z
- Binary: `components/bin/shadow_point.wasm` (97 KB)

**shadow-spot** (`components/graphics/shadow-spot/`)
- Calculates perspective shadow matrix for spot lights (cone-shaped)
- Inputs: light_position, light_direction, cone_angle, near, far
- Output: shadow_matrix (16 floats)
- FOV matches cone angle for exact shadow coverage
- Binary: `components/bin/shadow_spot.wasm` (105 KB)

### Shadow Sampling Shaders (GLSL)

All shaders use PCF filtering for soft shadows and include slope-scale bias to prevent shadow acne.

**shadow_common.glsl**
- Shared PCF sampling functions
- `pcf4()` - 2×2 sample pattern (4 samples)
- `pcf9()` - 3×3 sample pattern (9 samples)
- `pcf16()` - 4×4 sample pattern (16 samples)
- `shadowTest()` - Single sample (hard shadows)
- `calculateShadowBias()` - Slope-scale bias calculation

**shadow_directional.frag.glsl**
- Fragment shader for directional light shadows with CSM
- Automatic cascade selection based on view-space depth
- PCF filtering (9 samples per cascade)
- Smooth transitions between cascades
- Outside shadow map bounds = fully lit

**Buffer Layout:**
```glsl
uniform DirectionalShadow {
    mat4 shadowMatrices[4];  // Cascade matrices
    vec4 cascadeSplits;      // Split distances
    uint cascadeCount;       // Number of cascades
    float shadowBias;        // Base bias (0.005)
    float maxBias;           // Max bias (0.05)
};
```

**shadow_point.frag.glsl**
- Fragment shader for point light shadows with cubemap
- PCF filtering (6 samples with offset pattern)
- Distance attenuation (inverse square law)
- Cubemap sampling in light-space
- Outside light radius = fully shadowed

**Buffer Layout:**
```glsl
uniform PointLight {
    vec3 position;
    vec3 color;
    float intensity;
    float radius;       // Attenuation radius
    float farPlane;     // Shadow far plane
    float shadowBias;   // Base bias (0.05)
};
```

**shadow_spot.frag.glsl**
- Fragment shader for spot light shadows
- PCF filtering (9 samples, 3×3 pattern)
- Cone attenuation (smooth falloff between inner/outer angles)
- Distance attenuation
- Outside shadow map bounds = fully lit

**Buffer Layout:**
```glsl
uniform SpotLight {
    vec3 position;
    vec3 direction;
    vec3 color;
    float intensity;
    float innerAngle;   // Inner cone (radians)
    float outerAngle;   // Outer cone (radians)
    float radius;
    mat4 shadowMatrix;
    float shadowBias;   // Base bias (0.005)
    float maxBias;      // Max bias (0.05)
};
```

## Usage Examples

### Directional Shadow Workflow

1. **Calculate shadow matrices** (WASM component):
   ```
   shadow-directional:
     light_direction: [0.0, -1.0, 0.0]  # Downward sun
     view_matrix: <from camera>
     projection_matrix: <from camera>
     near: 0.1
     far: 100.0
     cascade_count: 4
   → shadow_matrices: [64 floats]
   → cascade_splits: [split0, split1, split2, split3]
   ```

2. **Upload to GPU** (uniform buffer):
   ```rust
   struct DirectionalShadow {
       shadow_matrices: [[f32; 16]; 4],
       cascade_splits: [f32; 4],
       cascade_count: u32,
       shadow_bias: f32,
       max_bias: f32,
   }
   ```

3. **Create shadow maps** (render pass):
   - Render scene 4 times (once per cascade)
   - Use depth-only framebuffer (e.g., 2048×2048)
   - Each cascade uses corresponding shadow matrix

4. **Sample shadows** (fragment shader):
   - Use `shadow_directional.frag.glsl`
   - Bind shadow map array as `sampler2DShadow`
   - Shader selects cascade based on fragment depth

### Point Shadow Workflow

1. **Calculate cubemap matrices** (WASM component):
   ```
   shadow-point:
     light_position: [0.0, 5.0, 0.0]
     near: 0.1
     far: 10.0  # Light radius
   → shadow_matrices: [96 floats = 6 faces]
   ```

2. **Create cubemap shadow map**:
   - 6 render passes (one per face)
   - Depth cubemap (e.g., 1024×1024 per face)
   - Each face uses corresponding matrix from output

3. **Sample shadows** (fragment shader):
   - Use `shadow_point.frag.glsl`
   - Bind cubemap as `samplerCubeShadow`
   - Direction from fragment to light selects face

### Spot Shadow Workflow

1. **Calculate shadow matrix** (WASM component):
   ```
   shadow-spot:
     light_position: [0.0, 5.0, 0.0]
     light_direction: [0.0, -1.0, 0.0]
     cone_angle: 45.0  # Degrees
     near: 0.1
     far: 20.0
   → shadow_matrix: [16 floats]
   ```

2. **Create shadow map**:
   - Single render pass
   - Depth texture (e.g., 1024×1024)
   - Use shadow matrix for rendering

3. **Sample shadows** (fragment shader):
   - Use `shadow_spot.frag.glsl`
   - Bind shadow map as `sampler2DShadow`
   - Shader applies cone attenuation

## Performance Characteristics

### WASM Components

| Component           | Binary Size | Execution Time | Memory    |
|---------------------|-------------|----------------|-----------|
| shadow-directional  | 105 KB      | <1ms           | Stack only|
| shadow-point        | 97 KB       | <1ms           | Stack only|
| shadow-spot         | 105 KB      | <1ms           | Stack only|

### GPU Shaders

| Shader Type        | PCF Samples | ALU per Fragment | Texture Samples |
|--------------------|-------------|------------------|-----------------|
| Directional (CSM)  | 9           | ~50-60           | 9 per cascade   |
| Point (Cubemap)    | 6           | ~40-50           | 6               |
| Spot               | 9           | ~50-60           | 9               |

### Shadow Map Resolutions

**Recommended resolutions:**
- **Directional CSM**: 2048×2048 per cascade (4 cascades = 16 MB total)
- **Point cubemap**: 1024×1024 per face (6 faces = 6 MB total)
- **Spot**: 1024×1024 (1 MB)

**Memory usage** (depth24_stencil8 format):
- 1024×1024 = 1 MB
- 2048×2048 = 4 MB
- 4096×4096 = 16 MB

## Shadow Quality Tuning

### Cascade Split Tuning

The `lambda` parameter in cascade split calculation controls distribution:
- `λ = 0.0`: Uniform splits (equal distance intervals)
- `λ = 0.5`: Practical split scheme (default, balanced)
- `λ = 1.0`: Logarithmic splits (more detail near camera)

**Default** (λ=0.5) provides excellent balance for most scenes.

### Bias Tuning

**Shadow acne** (incorrect self-shadowing):
- **Symptom**: Surface shadowing itself in a striped pattern
- **Fix**: Increase `shadowBias` or `maxBias`
- **Typical values**:
  - Directional: base=0.005, max=0.05
  - Point: base=0.05 (larger due to cubemap sampling)
  - Spot: base=0.005, max=0.05

**Peter panning** (shadows detached from objects):
- **Symptom**: Shadows appear to float above surfaces
- **Fix**: Decrease `shadowBias`
- **Balance**: Trade-off between acne and panning

### PCF Sample Count

Trade-off between quality and performance:

| Samples | Quality       | Performance | Use Case              |
|---------|---------------|-------------|-----------------------|
| 1       | Hard edges    | Fastest     | Stylized, retro games |
| 4       | Soft edges    | Fast        | Mobile, low-end       |
| 9       | Smooth edges  | Medium      | Default, balanced     |
| 16      | Very smooth   | Slower      | High quality, desktop |

## Common Issues

### Issue: Shadows too dark

**Cause**: Shadow factor not applied correctly or ambient term missing
**Fix**: Ensure shader includes ambient term:
```glsl
vec3 ambient = baseColor * 0.1;
outColor = vec4(lighting + ambient, 1.0);
```

### Issue: Shadows appear in wrong location

**Cause**: Shadow matrix mismatch or incorrect coordinate transforms
**Fix**:
1. Verify shadow matrix from component is uploaded correctly
2. Check fragment shader coordinate transforms (world → shadow space)
3. Ensure `shadowCoord * 0.5 + 0.5` transform is applied

### Issue: Cubemap shadows incorrect

**Cause**: Incorrect face ordering or up vectors
**Fix**: Use exact face order from shadow-point component:
- Face 0: +X (right)
- Face 1: -X (left)
- Face 2: +Y (top)
- Face 3: -Y (bottom)
- Face 4: +Z (front)
- Face 5: -Z (back)

### Issue: Cascade transitions visible

**Cause**: No blending between cascades
**Fix**: Implement cascade blending in fragment shader:
```glsl
// Blend between cascades in overlap region
float blendFactor = smoothstep(splitDist - blendRange, splitDist, viewDepth);
float shadow = mix(cascade0Shadow, cascade1Shadow, blendFactor);
```

## Phase 4 Context

This shadow mapping implementation is **Step 1** of Phase 4: Advanced Rendering.

**Completed:**
- ✅ Shadow matrix calculation (3 WASM components)
- ✅ Shadow sampling shaders (GLSL with PCF)
- ✅ Documentation and usage examples

**Next Steps:**
- Environment mapping and cubemap utilities
- Image-based lighting (IBL)
- Post-processing effects (bloom, tone mapping)
- Advanced PBR features (clear coat, subsurface)
- Performance optimizations (compute shaders, light culling)

## References

- **Cascaded Shadow Maps**: [NVIDIA GPU Gems 3, Chapter 10](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-10-parallel-split-shadow-maps-programmable-gpus)
- **PCF Filtering**: [NVIDIA Shadow Mapping](https://developer.nvidia.com/gpugems/gpugems/part-ii-lighting-and-shadows/chapter-11-shadow-map-antialiasing)
- **Practical CSM**: [Practical Split Scheme](https://developer.download.nvidia.com/SDK/10.5/opengl/src/cascaded_shadow_maps/doc/cascaded_shadow_maps.pdf)
