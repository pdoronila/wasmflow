# Image-Based Lighting (IBL) Shaders

GLSL shaders for physically-based image-based lighting using the split-sum approximation.

## Overview

IBL provides realistic environment lighting for PBR materials by pre-computing two components:
1. **Diffuse Irradiance**: Convolves environment map for diffuse lighting
2. **Specular Pre-filter**: Pre-filters environment for different roughness levels
3. **BRDF Integration LUT**: 2D lookup table for split-sum approximation

## Shaders

### Pre-computation Shaders

These shaders are run **once** during asset loading to generate IBL maps.

#### cubemap_convolution.vert.glsl
Shared vertex shader for cubemap rendering (irradiance and pre-filter).

**Inputs:**
- `position` (vec3): Cube vertex positions

**Outputs:**
- `localPos` (vec3): Position for fragment shader sampling

**Uniforms:**
```glsl
layout(set = 0, binding = 2) uniform CubemapMatrices {
    mat4 viewMatrices[6];     // View matrices for each face
    mat4 projectionMatrix;    // Perspective projection
    uint faceIndex;           // Current face (0-5)
};
```

#### irradiance_convolution.frag.glsl
Generates diffuse irradiance cubemap (typically 32×32 or 64×64).

**Purpose**: Convolves environment map over hemisphere for Lambertian diffuse

**Algorithm**:
- Samples hemisphere around each normal direction
- Weights samples by `cos(theta) * sin(theta)`
- Integrates using Monte Carlo sampling (≈10,000 samples per pixel)

**Input:**
- `u_envMap` (samplerCube): Environment cubemap to convolve

**Output:**
- RGB irradiance color

**Quality vs Performance**:
- `sampleDelta = 0.025`: High quality (default), ~2500 samples/pixel
- `sampleDelta = 0.05`: Medium quality, ~625 samples/pixel
- `sampleDelta = 0.1`: Low quality, ~156 samples/pixel

#### prefilter_specular.frag.glsl
Generates specular pre-filtered cubemap with mip levels (e.g., 512×512 base).

**Purpose**: Pre-filters environment for various roughness levels

**Algorithm**:
- Importance samples GGX distribution
- Uses Hammersley low-discrepancy sequence
- Samples 1024 directions per pixel
- Adjusts mip level based on sample footprint

**Inputs:**
```glsl
layout(set = 0, binding = 0) uniform samplerCube u_envMap;

layout(set = 0, binding = 1) uniform PrefilterParams {
    float roughness;   // Roughness level (0.0 - 1.0)
    float resolution;  // Base cubemap resolution
};
```

**Output:**
- RGB pre-filtered color

**Mip Levels**:
- Mip 0 (roughness 0.0): Sharp reflections
- Mip 1 (roughness 0.2): Slightly blurred
- Mip 2 (roughness 0.4): Medium blur
- Mip 3 (roughness 0.6): High blur
- Mip 4 (roughness 1.0): Fully diffuse

#### brdf_integration.frag.glsl
Generates BRDF integration LUT (typically 512×512 RG texture).

**Purpose**: Pre-computes Fresnel scale and bias for split-sum

**Algorithm**:
- Integrates GGX BRDF over hemisphere
- Uses Hammersley sequence (1024 samples)
- Outputs (scale, bias) for Fresnel approximation

**Input:**
- `fragUV` (vec2): Texture coordinates
  - `u` = `NdotV` (0-1)
  - `v` = `roughness` (0-1)

**Output:**
- `RG` = (scale, bias) for: `F * scale + bias`

**Usage**: Generate once per application, reuse for all materials

### Runtime Shader

#### pbr_ibl.frag.glsl
Complete PBR shader with IBL and optional direct lighting.

**Inputs:**
- `fragPosition`, `fragNormal`, `fragUV` from vertex shader

**Uniforms:**
```glsl
// Camera
layout(set = 0, binding = 0) uniform Camera {
    mat4 viewMatrix;
    mat4 projectionMatrix;
    vec3 cameraPosition;
};

// Material
layout(set = 0, binding = 1) uniform Material {
    vec3 baseColor;
    float metallic;
    float roughness;
    float ao;
};

// IBL textures (pre-computed)
layout(set = 0, binding = 2) uniform samplerCube u_irradianceMap;
layout(set = 0, binding = 3) uniform samplerCube u_prefilterMap;
layout(set = 0, binding = 4) uniform sampler2D u_brdfLUT;

// Optional direct light
layout(set = 0, binding = 5) uniform DirectLight {
    vec3 direction;
    vec3 color;
    float intensity;
    uint enabled;  // 0 = off, 1 = on
};
```

**Features:**
- Full Cook-Torrance BRDF
- Diffuse + Specular IBL
- Optional direct lighting
- ACES tone mapping
- Gamma correction (sRGB)

**Output:**
- Tone-mapped, gamma-corrected color

## Usage Workflow

### 1. Pre-computation (Asset Pipeline)

Run these steps **once** per environment map:

**Step 1: Generate Irradiance Map**
```rust
// Create small cubemap (32×32 or 64×64)
let irradiance_map = create_cubemap(32, wgpu::TextureFormat::Rgba16Float);

// Render 6 faces
for face_index in 0..6 {
    // Set viewport, bind framebuffer
    // Render cube with irradiance_convolution.frag.glsl
    // u_envMap = original environment cubemap
    // u_matrices.faceIndex = face_index
}

// Result: Diffuse irradiance cubemap
```

**Step 2: Generate Specular Pre-filter Map**
```rust
// Create cubemap with mip levels (512×512 base, 5 mip levels)
let prefilter_map = create_cubemap_with_mips(512, 5, wgpu::TextureFormat::Rgba16Float);

for mip in 0..5 {
    let roughness = mip as f32 / 4.0;  // 0.0, 0.25, 0.5, 0.75, 1.0
    let mip_size = 512 >> mip;

    for face_index in 0..6 {
        // Set viewport to mip_size × mip_size
        // Render cube with prefilter_specular.frag.glsl
        // u_envMap = original environment
        // u_params.roughness = roughness
        // u_params.resolution = 512.0
        // u_matrices.faceIndex = face_index
    }
}

// Result: Pre-filtered specular cubemap
```

**Step 3: Generate BRDF LUT** (once per application)
```rust
// Create 2D texture (512×512, RG16Float)
let brdf_lut = create_texture_2d(512, 512, wgpu::TextureFormat::Rg16Float);

// Render fullscreen quad with brdf_integration.frag.glsl
// No inputs needed, uses fragment UV

// Result: BRDF integration lookup table
```

### 2. Runtime Rendering

Use pre-computed maps in PBR shader:

```rust
// Bind IBL textures
render_pass.set_bind_group(0, &ibl_bind_group, &[]);

// ibl_bind_group contains:
// - binding 2: irradiance_map (samplerCube)
// - binding 3: prefilter_map (samplerCube with mips)
// - binding 4: brdf_lut (sampler2D)

// Render scene geometry with pbr_ibl.frag.glsl
// Shader automatically samples IBL for each fragment
```

## Performance Characteristics

### Pre-computation

**Irradiance Map** (32×32):
- Samples: ~2500 per pixel × 6 faces × 32² ≈ 15M samples
- Time: ~50-200ms (GPU-dependent)
- Memory: 49 KB (Rgba16Float)

**Prefilter Map** (512×512, 5 mips):
- Samples: 1024 per pixel × 6 faces × all mips ≈ 1.3B samples
- Time: ~500ms-2s (GPU-dependent)
- Memory: 6 MB (Rgba16Float with mips)

**BRDF LUT** (512×512):
- Samples: 1024 per pixel × 512² ≈ 268M samples
- Time: ~100-400ms (GPU-dependent)
- Memory: 1 MB (Rg16Float)

### Runtime

**pbr_ibl.frag.glsl**:
- ALU: ~120-150 instructions per fragment
- Texture samples: 3 per fragment (irradiance, prefilter, BRDF LUT)
- Throughput: Millions of fragments per second

## Quality Settings

### Irradiance Map Resolution

| Resolution | Quality | Memory | Use Case |
|------------|---------|--------|----------|
| 16×16      | Low     | 12 KB  | Mobile   |
| 32×32      | Good    | 49 KB  | Default  |
| 64×64      | High    | 196 KB | Desktop  |

Diffuse irradiance is low-frequency, so 32×32 is usually sufficient.

### Prefilter Map Resolution

| Base Resolution | Quality | Memory | Use Case |
|-----------------|---------|--------|----------|
| 256×256         | Low     | 1.5 MB | Mobile   |
| 512×512         | Good    | 6 MB   | Default  |
| 1024×1024       | High    | 24 MB  | Desktop  |

Includes 5 mip levels (roughness 0.0, 0.25, 0.5, 0.75, 1.0).

### BRDF LUT Resolution

| Resolution | Quality | Memory | Use Case |
|------------|---------|--------|----------|
| 256×256    | Good    | 256 KB | Mobile   |
| 512×512    | High    | 1 MB   | Default  |

BRDF LUT is smooth, so 256×256 is often sufficient.

## Common Issues

### Issue: Seams on specular reflections

**Cause**: Incorrect mip level calculation or filtering
**Fix**:
1. Ensure prefilter map has correct mip levels
2. Use linear mipmap filtering on prefilter sampler
3. Verify `MAX_REFLECTION_LOD` matches actual mip count

### Issue: Dark or missing diffuse lighting

**Cause**: Incorrect irradiance convolution or sampling
**Fix**:
1. Verify irradiance map generation completed
2. Check normal direction is normalized
3. Ensure cubemap sampler uses linear filtering

### Issue: Specular reflections too sharp/blurry

**Cause**: Incorrect roughness to mip level mapping
**Fix**: Adjust `MAX_REFLECTION_LOD` constant in `pbr_ibl.frag.glsl`:
```glsl
// If prefilter map has 5 mip levels (0-4):
const float MAX_REFLECTION_LOD = 4.0;

// If prefilter map has 6 mip levels (0-5):
const float MAX_REFLECTION_LOD = 5.0;
```

### Issue: Fireflies or noise in reflections

**Cause**: Insufficient sampling or NaN values
**Fix**:
1. Increase `SAMPLE_COUNT` in pre-filter shader
2. Add epsilon to denominators (already done)
3. Clamp extreme values in environment map

### Issue: Incorrect Fresnel behavior

**Cause**: Wrong F0 calculation for metallic/dielectric
**Fix**: Verify F0 calculation:
```glsl
vec3 F0 = vec3(0.04);  // Dielectric base (4% reflectivity)
F0 = mix(F0, baseColor, metallic);  // Metals use base color
```

## Integration with Existing PBR

To add IBL to an existing PBR shader:

1. **Add IBL textures** (irradiance, prefilter, BRDF LUT)
2. **Calculate reflection vector**: `R = reflect(-V, N)`
3. **Sample diffuse**: `irradiance = texture(u_irradianceMap, N)`
4. **Sample specular**: `prefilteredColor = textureLod(u_prefilterMap, R, roughness * MAX_LOD)`
5. **Sample BRDF**: `brdf = texture(u_brdfLUT, vec2(NdotV, roughness))`
6. **Combine**: `ambient = kD * irradiance * baseColor + specular * (F * brdf.x + brdf.y)`

See `pbr_ibl.frag.glsl` for complete implementation.

## Optimization Tips

**Pre-computation**:
- Generate IBL maps offline during asset import
- Cache results per environment map
- Use lower resolutions for mobile

**Runtime**:
- Share BRDF LUT across all materials (generated once)
- Use Rgba16Float for quality, Rgba8 for memory savings
- Consider RGB9E5 format for HDR with less memory

**Sampling**:
- Use trilinear filtering on prefilter map
- Use linear filtering on irradiance and BRDF LUT
- Enable mip streaming for large prefilter maps

## Example: Complete IBL Setup

```rust
// Pre-computation (once per environment)
let irradiance = generate_irradiance_map(&env_cubemap, 32);
let prefilter = generate_prefilter_map(&env_cubemap, 512, 5);

// Pre-computation (once per application)
let brdf_lut = generate_brdf_lut(512);

// Runtime setup
let ibl_bind_group = device.create_bind_group(&BindGroupDescriptor {
    entries: &[
        // ... camera and material bindings ...
        BindGroupEntry {
            binding: 2,
            resource: BindingResource::TextureView(&irradiance.view),
        },
        BindGroupEntry {
            binding: 3,
            resource: BindingResource::TextureView(&prefilter.view),
        },
        BindGroupEntry {
            binding: 4,
            resource: BindingResource::TextureView(&brdf_lut.view),
        },
    ],
});

// Rendering
render_pass.set_pipeline(&pbr_ibl_pipeline);
render_pass.set_bind_group(0, &ibl_bind_group, &[]);
render_pass.draw_indexed(0..indices.len(), 0, 0..1);
```

## References

- **Split-sum Approximation**: [Brian Karis, Epic Games (SIGGRAPH 2013)](https://cdn2.unrealengine.com/Resources/files/2013SiggraphPresentationsNotes-26915738.pdf)
- **IBL Theory**: [learnopengl.com/PBR/IBL](https://learnopengl.com/PBR/IBL)
- **Hammersley Sequence**: [Holger Dammertz](http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html)
- **GGX Distribution**: [Walter et al., EGSR 2007](https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf)
