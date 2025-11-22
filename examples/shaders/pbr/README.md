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
