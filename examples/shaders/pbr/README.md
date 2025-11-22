# PBR (Physically Based Rendering) Shaders

This directory contains GLSL shaders implementing the Cook-Torrance BRDF model for physically-based rendering, matching the implementation in the WasmFlow PBR components.

## Shaders

### pbr_single_light.vert.glsl / pbr_single_light.frag.glsl

Basic PBR implementation with a single directional light.

**Features:**
- Cook-Torrance specular BRDF
- GGX (Trowbridge-Reitz) normal distribution
- Schlick-Fresnel approximation
- Smith geometry term (GGX variant)
- Lambertian diffuse with energy conservation
- Metallic/roughness workflow
- Tone mapping (Reinhard) and gamma correction

**Uniforms:**
- **CameraUniforms** (set=0, binding=0):
  - `mat4 view` - View matrix
  - `mat4 projection` - Projection matrix
  - `vec3 camera_position` - Camera position in world space

- **MaterialUniforms** (set=1, binding=0):
  - `vec4 base_color` - Base color (albedo) in linear RGB
  - `float metallic` - Metallic value [0=dielectric, 1=metal]
  - `float roughness` - Surface roughness [0=smooth, 1=rough]
  - `float ao` - Ambient occlusion [0=fully occluded, 1=no occlusion]

- **LightUniforms** (set=2, binding=0):
  - `vec3 light_direction` - Directional light direction (normalized)
  - `vec3 light_color` - Light color (linear RGB)
  - `float light_intensity` - Light intensity multiplier

**Vertex Inputs:**
- `location=0` - `vec3 position` - Vertex position
- `location=1` - `vec3 normal` - Vertex normal
- `location=2` - `vec2 uv` - Texture coordinates
- `location=3` - `vec3 tangent` - Tangent vector (for normal mapping)

### pbr_multi_light.vert.glsl / pbr_multi_light.frag.glsl

Advanced PBR implementation supporting up to 8 lights of mixed types (directional, point, spot).

**Features:**
- All features from single-light shader
- Support for directional, point, and spot lights
- Per-light attenuation (inverse square falloff for point/spot)
- Smooth cone falloff for spot lights
- Accumulates contributions from all active lights

**Additional Light Types:**

**Directional Light:**
- Parallel rays (sun-like)
- No attenuation
- Fields: `position_or_direction` (direction), `color`, `intensity`

**Point Light:**
- Omni-directional emission
- Inverse square attenuation with radius
- Fields: `position_or_direction` (position), `color`, `intensity`, `radius`

**Spot Light:**
- Cone-shaped emission
- Distance + angular attenuation
- Fields: `position_or_direction` (position), `spot_direction`, `color`, `intensity`, `radius`, `inner_cone_angle`, `outer_cone_angle`

**LightData Structure:**
```glsl
struct LightData {
    vec3 position_or_direction;  // Position for point/spot, direction for directional
    uint light_type;             // 0=directional, 1=point, 2=spot
    vec3 color;
    float intensity;
    vec3 spot_direction;         // Only for spot lights
    float radius;                // Attenuation radius (point/spot)
    float inner_cone_angle;      // Cosine of inner angle (spot)
    float outer_cone_angle;      // Cosine of outer angle (spot)
    vec2 _padding;               // Alignment padding
};
```

**MultiLightUniforms** (set=2, binding=0):
- `LightData lights[8]` - Array of up to 8 lights
- `uint light_count` - Number of active lights

## Cook-Torrance BRDF Formula

The specular term is calculated as:

```
f_specular = (D * F * G) / (4 * (N·V) * (N·L))
```

Where:
- **D** = GGX distribution function
  - `D = α² / (π * ((N·H)² * (α² - 1) + 1)²)`
  - `α = roughness²`

- **F** = Fresnel-Schlick approximation
  - `F = F0 + (1 - F0) * (1 - (H·V))^5`
  - `F0 = mix(0.04, base_color, metallic)`

- **G** = Smith geometry function (GGX variant)
  - `G = G1(V) * G1(L)`
  - `G1(v) = (2 * (N·v)) / ((N·v) + sqrt(α² + (1 - α²) * (N·v)²))`

The diffuse term uses energy conservation:

```
f_diffuse = (1 - F) * (1 - metallic) * base_color / π
```

## Material Properties

### Metallic Workflow

- **Dielectric (metallic=0):**
  - F0 = vec3(0.04) (4% reflection)
  - Full diffuse contribution
  - Examples: plastic, wood, fabric

- **Metal (metallic=1):**
  - F0 = base_color (colored reflection)
  - No diffuse contribution
  - Examples: gold, copper, iron

- **Mixed (0 < metallic < 1):**
  - F0 interpolated between 0.04 and base_color
  - Reduced diffuse contribution

### Roughness

- **Smooth (roughness=0):**
  - Sharp, mirror-like reflections
  - Narrow specular highlights
  - Examples: polished metal, glass

- **Rough (roughness=1):**
  - Diffuse, matte appearance
  - Wide specular highlights
  - Examples: concrete, rough wood

## Correspondence with WASM Components

These shaders implement the same PBR model as the WasmFlow components:

| WASM Component | GLSL Function |
|----------------|---------------|
| `pbr-ggx-distribution` | `distribution_ggx()` |
| `pbr-fresnel` | `fresnel_schlick()` |
| `pbr-smith-geometry` | `geometry_smith()` |
| `pbr-material` | F0 calculation in `main()` |
| `pbr-brdf` | `cook_torrance_brdf()` |

The WASM components allow CPU-side BRDF evaluation for node-based workflows, while these shaders provide GPU-accelerated rendering.

## Example Material Values

### Gold
```glsl
base_color = vec4(1.0, 0.71, 0.29, 1.0);
metallic = 1.0;
roughness = 0.2;
```

### Plastic
```glsl
base_color = vec4(0.8, 0.1, 0.1, 1.0);  // Red
metallic = 0.0;
roughness = 0.5;
```

### Brushed Metal
```glsl
base_color = vec4(0.9, 0.9, 0.9, 1.0);
metallic = 1.0;
roughness = 0.6;
```

### Wet Stone
```glsl
base_color = vec4(0.4, 0.4, 0.4, 1.0);
metallic = 0.0;
roughness = 0.3;
ao = 0.8;  // Slight occlusion
```

## Tone Mapping and Gamma Correction

Both shaders apply:
1. **Reinhard tone mapping:** `color / (color + 1.0)`
   - Maps HDR values to [0, 1] range
   - Preserves color ratios
   - Simple and fast

2. **Gamma correction:** `pow(color, 1.0/2.2)`
   - Converts from linear to sRGB color space
   - Standard gamma value: 2.2
   - Applied after tone mapping

## Performance Notes

- **Single light shader:** ~50-100 ALU operations per fragment
- **Multi-light shader:** ~50-100 ALU operations per light per fragment
- For scenes with many lights (>8), consider:
  - Deferred rendering
  - Light culling
  - Clustered shading

## Integration with WasmFlow

These shaders are designed to work with:
- Geometry primitives from `primitive-sphere`, `primitive-cube`, `primitive-plane`
- Camera matrices from `perspective-camera`
- Material properties from `pbr-material`
- Light data from `light-directional`, `light-point`, `light-spot`

The shader program linker (`builtin:graphics:shader-program-linker`) can compile these shaders and create render pipelines.

## Normal Mapping

### pbr_normal_mapped.vert.glsl / pbr_normal_mapped.frag.glsl

PBR implementation with tangent-space normal mapping for enhanced surface detail.

**Features:**
- All features from pbr_single_light shader
- Tangent-space normal map support
- TBN (Tangent-Bitangent-Normal) matrix transformation
- Normal strength parameter for blending
- Automatic bitangent calculation

**Additional Uniforms:**
- **MaterialUniforms** (set=1, binding=0):
  - `float normal_strength` - Normal map intensity [0=no effect, 1=full effect, >1=exaggerated]
  
- **Normal Map Texture** (set=1, binding=1):
  - `sampler2D normal_map` - Normal map texture in tangent space

**Vertex Outputs (additional):**
- `location=5` - `vec3 frag_bitangent` - Bitangent vector (calculated from normal × tangent)

**Normal Map Format:**
- Expected in standard [0, 1] range (RGB encoding)
- Automatically converted to [-1, 1] tangent-space normals
- Blue channel (Z) typically dominant (~1.0 for flat surfaces)
- Standard format: (R=X, G=Y, B=Z) in tangent space

**TBN Matrix:**
```glsl
mat3 TBN = mat3(
    normalize(frag_tangent),      // T (X-axis in tangent space)
    normalize(frag_bitangent),    // B (Y-axis in tangent space)
    normalize(frag_normal)        // N (Z-axis in tangent space)
);
```

**Normal Map Sampling:**
```glsl
vec3 tangent_normal = texture(normal_map, frag_uv).xyz;
tangent_normal = tangent_normal * 2.0 - 1.0;  // [0,1] → [-1,1]
tangent_normal.xy *= normal_strength;  // Apply strength
tangent_normal = normalize(tangent_normal);
vec3 world_normal = normalize(TBN * tangent_normal);
```

**Normal Strength Usage:**
- `0.0`: No normal mapping (flat surface)
- `0.5`: Subtle bumps (50% effect)
- `1.0`: Full normal map effect
- `>1.0`: Exaggerated bumps (can create interesting stylized effects)

**Use Cases:**
- Brick walls without modeling individual bricks
- Rough metal surfaces (scratches, dents)
- Fabric weave patterns
- Stone surfaces (cracks, pits)
- Wood grain detail
- Any surface requiring micro-geometry without vertex cost

**Performance:**
- Adds 1 texture lookup per fragment
- Adds ~10-15 ALU operations for TBN transformation
- Minimal performance impact on modern GPUs
- Much cheaper than actual geometry detail

**Creating Normal Maps:**
Normal maps can be created from:
- High-poly to low-poly baking (Blender, Substance Painter)
- Photo-based generation (CrazyBump, NormalMap-Online)
- Procedural generation (Substance Designer)
- Height map conversion (Photoshop, GIMP)

**Tangent Space vs World Space:**
- **Tangent space**: Normals relative to surface (portable across instances)
- **World space**: Normals in absolute coordinates (not portable)
- We use tangent space for flexibility and reusability

## Example Material Configurations with Normal Mapping

### Rough Brick Wall
```glsl
// Material uniforms
base_color = vec4(0.7, 0.4, 0.3, 1.0);
metallic = 0.0;
roughness = 0.9;
ao = 0.8;  // Mortar crevices
normal_strength = 1.0;

// Normal map: Brick pattern with deep grooves
```

### Brushed Metal with Scratches
```glsl
// Material uniforms
base_color = vec4(0.85, 0.85, 0.85, 1.0);
metallic = 1.0;
roughness = 0.4;
normal_strength = 0.7;  // Subtle scratches

// Normal map: Directional scratch pattern
```

### Polished Stone with Cracks
```glsl
// Material uniforms
base_color = vec4(0.3, 0.3, 0.35, 1.0);
metallic = 0.0;
roughness = 0.2;  // Polished
ao = 0.9;
normal_strength = 0.8;

// Normal map: Crack network on smooth surface
```

### Fabric with Weave Pattern
```glsl
// Material uniforms
base_color = vec4(0.6, 0.1, 0.1, 1.0);  // Red fabric
metallic = 0.0;
roughness = 0.7;
normal_strength = 0.5;  // Subtle weave

// Normal map: Cloth weave pattern
```

## Integration with WasmFlow Components

The normal-mapped shaders integrate with the following components:

**Geometry Requirements:**
- `primitive-sphere`, `primitive-cube`, `primitive-plane` (all include tangents)
- Custom geometry must provide tangent vectors (location=3)

**Normal Map Workflow:**
```
texture-sampler → tangent_normal (vec3)
       ↓
normal-map → world_normal
       ↓
pbr-brdf → lit_color
```

**Component Chain:**
1. **texture-sampler**: Sample normal map texture at UV coordinates
2. **normal-map**: Transform tangent-space normal to world-space using TBN
3. **pbr-brdf**: Calculate lighting with perturbed normal

**Shader Equivalent:**
The GPU shader combines steps 2-3 for efficiency, but the math is identical to the WASM components.

## Troubleshooting Normal Maps

**Problem: Normals pointing in wrong direction**
- Solution: Check tangent vector calculation (may need to flip)
- Verify normal map is in correct space (tangent, not world)

**Problem: Lighting artifacts at seams**
- Solution: Ensure tangents are consistent across shared vertices
- Use proper UV unwrapping with minimal distortion

**Problem: Normal map has no visible effect**
- Solution: Check normal_strength is not 0.0
- Verify normal map texture is bound correctly
- Ensure tangents are present in geometry

**Problem: Excessively bumpy surface**
- Solution: Reduce normal_strength parameter
- Check normal map isn't too "intense" (blue channel should be dominant)

**Problem: Inverted bumps (convex appears concave)**
- Solution: Invert green channel of normal map (Y-axis flip)
- Some software uses different Y-axis conventions

