# Skybox Shaders

GLSL shaders for rendering environment cubemaps as skybox backgrounds.

## Shaders

### skybox.vert.glsl
Vertex shader for skybox rendering.

**Key Features:**
- Removes translation from view matrix (skybox appears infinitely far)
- Sets depth to 1.0 (renders behind all geometry)
- Uses cube vertex positions as texture coordinates

**Vertex Inputs:**
- `position` (vec3): Cube vertices in range [-1, 1]

**Outputs:**
- `texCoord` (vec3): Cubemap sampling direction

**Uniforms:**
```glsl
layout(set = 0, binding = 0) uniform Camera {
    mat4 viewMatrix;
    mat4 projectionMatrix;
    vec3 cameraPosition;
};
```

### skybox.frag.glsl
Fragment shader for skybox rendering with HDR support.

**Inputs:**
- `texCoord` (vec3): Cubemap sampling direction

**Outputs:**
- `outColor` (vec4): Final skybox color

**Uniforms:**
```glsl
layout(set = 0, binding = 1) uniform samplerCube u_skybox;

layout(set = 0, binding = 2) uniform SkyboxParams {
    float exposure;       // HDR exposure (default: 1.0)
    float gamma;          // Gamma correction (default: 2.2)
    float brightness;     // Brightness multiplier (default: 1.0)
    uint enableToneMap;   // 0 = off, 1 = on
};
```

**Features:**
- Cubemap sampling
- HDR exposure control
- Tone mapping (Reinhard or ACES filmic)
- Gamma correction
- Brightness adjustment

## Usage

### Basic Skybox

1. **Create cubemap texture** (6 face images):
   ```rust
   let cubemap = GpuTexture::from_cubemap_rgba8(
       &device,
       &queue,
       512,  // Face size
       &[right, left, top, bottom, front, back],  // RGBA8 data
       Some("Skybox")
   )?;
   ```

2. **Create skybox cube geometry**:
   ```rust
   // Simple cube vertices (positions only, -1 to 1)
   let vertices = [
       // ... 36 vertices for cube (6 faces × 2 triangles)
   ];
   ```

3. **Set up camera uniforms**:
   ```rust
   struct Camera {
       view_matrix: [[f32; 16]],
       projection_matrix: [[f32; 16]],
       camera_position: [f32; 3],
   }
   ```

4. **Set up skybox parameters**:
   ```rust
   struct SkyboxParams {
       exposure: f32,        // 1.0 for default
       gamma: f32,           // 2.2 for sRGB
       brightness: f32,      // 1.0 for default
       enable_tone_map: u32, // 0 or 1
   }
   ```

5. **Render skybox** (after clearing depth, before scene):
   ```rust
   // Disable depth write (but keep depth test)
   // Use <= depth function to pass at far plane (depth = 1.0)
   // Bind cubemap sampler
   // Draw cube
   ```

### HDR Environment Maps

For HDR environment maps (e.g., .hdr, .exr files):

```rust
let params = SkyboxParams {
    exposure: 1.5,         // Adjust for scene brightness
    gamma: 2.2,
    brightness: 1.0,
    enable_tone_map: 1,    // Enable ACES tone mapping
};
```

**Tone Mapping Options:**
- **Reinhard**: Simple, fast, good for moderate HDR ranges
- **ACES**: Filmic look, better for wide HDR ranges (default)

### LDR Environment Maps

For standard LDR cubemaps (e.g., PNG, JPG):

```rust
let params = SkyboxParams {
    exposure: 1.0,         // No exposure adjustment
    gamma: 2.2,
    brightness: 1.0,
    enable_tone_map: 0,    // Disable tone mapping
};
```

## Rendering Pipeline

**Correct render order:**

1. **Clear color and depth buffers**
2. **Render skybox** (with depth test, writes depth = 1.0)
   - Disable depth write or use reverse-Z
   - Depth function: `LessEqual` (standard) or `GreaterEqual` (reverse-Z)
3. **Render scene geometry** (depth test enabled, writes depth < 1.0)

**Why skybox first?**
- Skybox pixels at depth 1.0 are overwritten by scene geometry
- Avoids overdraw (no fragments wasted on hidden skybox pixels)
- Works correctly with depth testing

## Cubemap Face Order

**WebGPU/OpenGL convention:**
- Face 0: +X (right)
- Face 1: -X (left)
- Face 2: +Y (top)
- Face 3: -Y (bottom)
- Face 4: +Z (front)
- Face 5: -Z (back)

**Loading cubemaps:**
```rust
// Face order matches array index
let faces = [
    load_image("right.png"),   // +X
    load_image("left.png"),    // -X
    load_image("top.png"),     // +Y
    load_image("bottom.png"),  // -Y
    load_image("front.png"),   // +Z
    load_image("back.png"),    // -Z
];
```

## Performance

**Vertex Shader:**
- ~10-20 ALU instructions
- Minimal overhead (8 vertices, 12 triangles for cube)

**Fragment Shader:**
- ~20-40 ALU instructions (without tone mapping)
- ~40-60 ALU instructions (with ACES tone mapping)
- 1 cubemap sample per fragment

**Optimization:**
- Render skybox early to avoid overdraw
- Use low-poly cube (8 vertices sufficient)
- Consider mipmap filtering for distant skybox

## Common Issues

### Issue: Skybox appears to move with camera

**Cause**: Translation not removed from view matrix
**Fix**: Ensure view matrix translation is zeroed:
```glsl
mat4 viewRotation = mat4(mat3(u_camera.viewMatrix));
```

### Issue: Skybox renders in front of scene

**Cause**: Depth test incorrect or skybox depth not 1.0
**Fix**: Use `gl_Position = clipPos.xyww` in vertex shader

### Issue: Skybox seams visible

**Cause**: Incorrect face order or texture filtering
**Fix**:
1. Verify face order matches +X, -X, +Y, -Y, +Z, -Z
2. Use `ClampToEdge` address mode on cubemap sampler
3. Ensure all faces are same size and aligned

### Issue: HDR skybox too bright/dark

**Cause**: Incorrect exposure setting
**Fix**: Adjust `exposure` parameter (typical range: 0.5-2.0)

## Integration with PBR

Skybox can serve as IBL (Image-Based Lighting) source:

1. **Diffuse IBL**: Convolve cubemap for diffuse irradiance
2. **Specular IBL**: Pre-filter cubemap for specular reflections
3. **Reflection Probes**: Use skybox as environment reflection

See Phase 4 Step 5 (IBL implementation) for details.

## Examples

### Example 1: Basic Skybox
```rust
// Load 6 faces
let faces = load_cubemap_faces("skybox/");

// Create cubemap
let cubemap = GpuTexture::from_cubemap_rgba8(&device, &queue, 512, &faces, Some("Skybox"))?;

// Render with default params
let params = SkyboxParams {
    exposure: 1.0,
    gamma: 2.2,
    brightness: 1.0,
    enable_tone_map: 0,
};
```

### Example 2: HDR Environment
```rust
// Load HDR .hdr file and split into 6 faces
let hdr_faces = load_hdr_cubemap("environment.hdr");

// Create HDR cubemap
let cubemap = GpuTexture::from_cubemap_rgba8(&device, &queue, 1024, &hdr_faces, Some("HDR Sky"))?;

// Render with tone mapping
let params = SkyboxParams {
    exposure: 1.5,
    gamma: 2.2,
    brightness: 1.2,
    enable_tone_map: 1,  // ACES tone mapping
};
```

## Future Enhancements

- Mipmap generation for cubemaps
- Automatic irradiance/pre-filter generation (IBL)
- Equirectangular to cubemap conversion
- Procedural sky generation (atmospheric scattering)
- Dynamic skybox (time-of-day)

## References

- **Cubemaps**: [learnopengl.com/Advanced-OpenGL/Cubemaps](https://learnopengl.com/Advanced-OpenGL/Cubemaps)
- **IBL**: [learnopengl.com/PBR/IBL](https://learnopengl.com/PBR/IBL)
- **ACES Tone Mapping**: [knarkowicz.wordpress.com](https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/)
