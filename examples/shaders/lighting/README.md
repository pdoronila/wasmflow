# Example Lighting Shaders

This directory contains example GLSL shaders demonstrating various lighting techniques for use with WasmFlow's graphics system.

## Shaders

### 1. Basic Diffuse Lighting (`basic_diffuse.*.glsl`)

Simple Lambertian diffuse lighting with a single directional light.

**Features:**
- Lambert's cosine law for diffuse reflection
- Single directional light support
- Basic ambient lighting

**Uniforms:**
- Set 0, Binding 0: `CameraUniforms` (view, projection, camera_position)
- Set 1, Binding 0: `MaterialUniforms` (base_color, metallic, roughness)
- Set 2, Binding 0: `LightUniforms` (direction, color, intensity)

**Use Case:** Simple scenes with a single sun/directional light source.

---

### 2. Phong Lighting (`phong.*.glsl`)

Classic Phong lighting model with diffuse and specular components.

**Features:**
- Lambertian diffuse reflection
- Phong specular highlights
- Roughness-based shininess calculation
- View direction dependent specular
- Ambient lighting

**Formula:**
- Diffuse: `I_d = k_d * (N · L) * I_light`
- Specular: `I_s = k_s * (R · V)^shininess * I_light`
- Shininess: `mix(128.0, 1.0, roughness)` (rough = low shininess)

**Uniforms:**
- Set 0, Binding 0: `CameraUniforms`
- Set 1, Binding 0: `MaterialUniforms`
- Set 2, Binding 0: `LightUniforms`

**Use Case:** Objects that need specular highlights (metal, plastic, shiny surfaces).

---

### 3. Multiple Lights (`multi_light.*.glsl`)

Advanced shader supporting up to 8 lights of mixed types (directional and point).

**Features:**
- Support for directional and point lights
- Distance-based attenuation for point lights
- Per-light diffuse and specular calculation
- Light accumulation (additive blending)
- Dynamic light count

**Light Types:**
- `LIGHT_TYPE_DIRECTIONAL` (0): Sun-like directional lights
- `LIGHT_TYPE_POINT` (1): Positional lights with radius-based falloff

**Attenuation Formula (Point Lights):**
```glsl
attenuation = 1.0 / (1.0 + (distance^2) / (radius^2))
```

**Uniforms:**
- Set 0, Binding 0: `CameraUniforms`
- Set 1, Binding 0: `MaterialUniforms`
- Set 2, Binding 0: `MultiLightUniforms` (lights array, light_count)

**Use Case:** Complex scenes with multiple light sources (e.g., street lights, interior lighting, mixed sun and artificial lights).

---

## Integration with WasmFlow

These shaders are designed to work with WasmFlow's lighting components:

1. **light-directional**: Outputs JSON data compatible with `LightUniforms` and `MultiLightUniforms`
2. **light-point**: Outputs JSON data compatible with `MultiLightUniforms`
3. **lighting-phong**: CPU-side lighting calculation for testing/validation

### Example Workflow

1. Create geometry using primitive nodes (e.g., `primitive-sphere`)
2. Set up camera with `perspective-camera`
3. Create lights using `light-directional` or `light-point`
4. Use `shader-program-linker` to combine vertex and fragment shaders
5. Connect to `shader-preview` for real-time rendering

---

## GPU Buffer Layout

### CameraUniforms (144 bytes)
```
mat4 view;                // 64 bytes
mat4 projection;          // 64 bytes
vec3 camera_position;     // 12 bytes
float _padding;           // 4 bytes
```

### MaterialUniforms (32 bytes)
```
vec4 base_color;          // 16 bytes
float metallic;           // 4 bytes
float roughness;          // 4 bytes
vec2 _padding;            // 8 bytes
```

### LightUniforms (32 bytes)
```
vec3 direction;           // 12 bytes
float _padding;           // 4 bytes
vec3 color;               // 12 bytes
float intensity;          // 4 bytes
```

### LightData (48 bytes)
```
vec3 position_or_direction; // 12 bytes
uint light_type;            // 4 bytes
vec3 color;                 // 12 bytes
float intensity;            // 4 bytes
float radius;               // 4 bytes
vec3 _padding;              // 12 bytes
```

### MultiLightUniforms (400 bytes)
```
LightData lights[8];        // 384 bytes
uint light_count;           // 4 bytes
vec3 _padding;              // 12 bytes
```

---

## Performance Notes

- **Basic Diffuse**: Fastest - single light, no specular
- **Phong**: Moderate - adds specular calculation per fragment
- **Multi-Light**: Slowest - iterates over all active lights per fragment

**Optimization Tips:**
- Keep `light_count` as low as possible
- Use directional lights when possible (cheaper than point lights)
- Consider using deferred rendering for scenes with many lights (future feature)

---

## Further Reading

- [OpenGL Shading Language Specification](https://www.khronos.org/registry/OpenGL/specs/gl/GLSLangSpec.4.60.pdf)
- [Learn OpenGL - Basic Lighting](https://learnopengl.com/Lighting/Basic-Lighting)
- [Learn OpenGL - Multiple Lights](https://learnopengl.com/Lighting/Multiple-lights)
- [Phong Reflection Model](https://en.wikipedia.org/wiki/Phong_reflection_model)
