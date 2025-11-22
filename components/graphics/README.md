# Graphics Components

WASM components for 3D graphics, rendering, and shader pipelines.

## Phase 1: Core Graphics Nodes (Complete)

Phase 1 provides foundational components for building graphics pipelines with placeholder shader preview.

### Vector Math Components

#### vec3-construct
Construct a 3D vector from x, y, z components.

**Inputs:**
- `x` (f32): X component
- `y` (f32): Y component
- `z` (f32): Z component

**Outputs:**
- `vec3` (list): 3-element list [x, y, z]

**Example:**
```
x: 1.0, y: 2.0, z: 3.0 → vec3: [1.0, 2.0, 3.0]
```

#### vec3-add
Add two 3D vectors component-wise.

**Inputs:**
- `a` (list): First vector [x, y, z]
- `b` (list): Second vector [x, y, z]

**Outputs:**
- `result` (list): Sum [a.x+b.x, a.y+b.y, a.z+b.z]

#### vec3-subtract
Subtract two 3D vectors.

**Inputs:**
- `a` (list): First vector
- `b` (list): Vector to subtract

**Outputs:**
- `result` (list): Difference [a.x-b.x, a.y-b.y, a.z-b.z]

#### vec3-scale
Scale a vector by a scalar value.

**Inputs:**
- `vec` (list): Input vector [x, y, z]
- `scalar` (f32): Scale factor

**Outputs:**
- `result` (list): Scaled vector [x*s, y*s, z*s]

#### vec3-normalize
Normalize a vector to unit length.

**Inputs:**
- `vec` (list): Input vector [x, y, z]

**Outputs:**
- `result` (list): Normalized vector (length = 1.0)
- `length` (f32): Original length before normalization

**Error:** Returns error if input is zero vector (cannot normalize)

#### vec3-dot
Calculate dot product of two vectors.

**Inputs:**
- `a` (list): First vector
- `b` (list): Second vector

**Outputs:**
- `result` (f32): Dot product (a.x*b.x + a.y*b.y + a.z*b.z)

#### vec3-cross
Calculate cross product of two vectors (perpendicular vector).

**Inputs:**
- `a` (list): First vector
- `b` (list): Second vector

**Outputs:**
- `result` (list): Cross product vector perpendicular to both inputs

### Matrix Components

#### mat4-construct
Construct a 4x4 matrix from components or column vectors.

**Inputs (Component Mode):**
- `m00` through `m33` (f32): All 16 matrix elements (column-major)

**Inputs (Column Mode):**
- `col0` (list): First column [x, y, z, w]
- `col1` (list): Second column
- `col2` (list): Third column
- `col3` (list): Fourth column

**Outputs:**
- `matrix` (list): 16-element list in column-major order

**Note:** Column mode takes precedence if any column is provided.

#### mat4-multiply
Multiply two 4x4 matrices (standard matrix product).

**Inputs:**
- `a` (list): First matrix (16 elements)
- `b` (list): Second matrix (16 elements)

**Outputs:**
- `result` (list): Product matrix A × B (16 elements)

**Use Cases:**
- Combining transformations
- Model-View-Projection (MVP) calculation
- Transform hierarchies

### Color Components

#### color-rgb
Create RGB color vector with clamping to [0.0, 1.0].

**Inputs:**
- `r` (f32): Red component
- `g` (f32): Green component
- `b` (f32): Blue component

**Outputs:**
- `color` (list): 3-element list [r, g, b] (clamped)

**Note:** Values outside [0.0, 1.0] are automatically clamped.

### Geometry Primitives

#### primitive-sphere
Generate UV sphere mesh using parametric latitude/longitude algorithm.

**Inputs:**
- `radius` (f32): Sphere radius (must be > 0)
- `segments` (u32): Horizontal divisions (minimum 3)
- `rings` (u32): Vertical divisions (minimum 2)

**Outputs:**
- `positions` (list): Vertex positions as vec3 [(segments+1) × (rings+1) vertices]
- `normals` (list): Vertex normals (normalized)
- `uvs` (list): UV coordinates as (u, v) tuples
- `indices` (list): Triangle indices (u32)

**Vertex Count:** (segments+1) × (rings+1)
**Triangle Count:** segments × rings × 2

**Example:**
```
radius: 1.0, segments: 16, rings: 8
→ 136 vertices (17×8), 256 triangles
```

#### primitive-cube
Generate cube with 24 vertices (4 per face for proper per-face normals).

**Inputs:**
- `size` (f32): Cube side length (must be > 0)

**Outputs:**
- `positions` (list): 24 vertex positions (6 faces × 4 corners)
- `normals` (list): 24 normals (one per vertex, shared per face)
- `uvs` (list): 24 UV coordinates
- `indices` (list): 36 triangle indices (12 triangles)

**Note:** Uses 24 vertices instead of 8 to enable proper per-face normals and UVs.

#### primitive-plane
Generate subdivided plane (XZ plane facing +Y).

**Inputs:**
- `width` (f32): Width along X axis (must be > 0)
- `depth` (f32): Depth along Z axis (must be > 0)
- `subdivisions` (u32): Grid subdivisions (tessellation detail)

**Outputs:**
- `positions` (list): Vertex positions in XZ plane
- `normals` (list): All normals point up (+Y)
- `uvs` (list): UV coordinates [0,0] to [1,1]
- `indices` (list): Triangle indices

**Vertex Count:** (subdivisions+1)²
**Triangle Count:** subdivisions² × 2

**Example:**
```
width: 10, depth: 10, subdivisions: 10
→ 121 vertices (11×11), 200 triangles
```

### Camera Components

#### perspective-camera
Calculate view and projection matrices for perspective camera.

**Inputs:**
- `position` (list): Camera position [x, y, z]
- `target` (list): Look-at target [x, y, z]
- `up` (list): Up direction [x, y, z] (usually [0, 1, 0])
- `fov` (f32): Field of view in degrees (must be > 0 and < 180)
- `aspect_ratio` (f32): Width/height ratio (must be > 0)
- `near` (f32): Near clipping plane (must be > 0)
- `far` (f32): Far clipping plane (must be > near)

**Outputs:**
- `view_matrix` (list): Look-at view matrix (16 elements)
- `projection_matrix` (list): Perspective projection matrix (16 elements)
- `camera_position` (list): Echo of input position [x, y, z]
- `view_direction` (list): Normalized direction from position to target

**Matrix Convention:** Right-handed coordinate system, column-major order

**Example:**
```
position: [0, 5, 10]
target: [0, 0, 0]
up: [0, 1, 0]
fov: 60°
aspect: 16:9 (1.777778)
near: 0.1
far: 100.0
→ Produces view and projection matrices for standard perspective camera
```

### Render Target Components

#### render-target
Configure render target parameters and output JSON configuration.

**Inputs:**
- `width` (u32): Width in pixels (must be > 0)
- `height` (u32): Height in pixels (must be > 0)
- `format` (string): Color format - "rgba8", "rgba16-float", "rgba32-float", "rgb8", "r8"
- `depth` (bool): Enable depth buffer
- `multisample` (u32): MSAA sample count - 1 (none), 2, 4, or 8

**Outputs:**
- `config` (string): JSON configuration for render system

**Example Output:**
```json
{
  "width": 1920,
  "height": 1080,
  "format": "rgba8",
  "depth": true,
  "multisample": 4
}
```

**Format Notes:**
- `rgba8`: Standard 8-bit color (most common)
- `rgba16-float`: 16-bit floating point (HDR)
- `rgba32-float`: 32-bit floating point (high precision HDR)
- `rgb8`: 8-bit without alpha
- `r8`: Single channel 8-bit

## Phase 2: Lighting Components (Step 8)

### light-directional

Create a directional light source (sun-like with parallel rays).

**Inputs:**
- `direction` (vec3): Light direction vector (automatically normalized)
- `color` (vec3): Light color RGB (automatically clamped to [0.0, 1.0])
- `intensity` (f32): Light intensity multiplier (must be non-negative)

**Outputs:**
- `light_data` (String): JSON-encoded light data compatible with GPU uniforms

**Example:**
```
direction: [0, -1, 0] (down)
color: [1, 1, 1] (white)
intensity: 1.2
→ light_data: {"light_type":"directional","direction":[0,-1,0],"color":[1,1,1],"intensity":1.2}
```

**Features:**
- Automatic direction vector normalization
- Color clamping to valid [0.0, 1.0] range
- Validates intensity is non-negative
- JSON output compatible with MultiLightUniforms GPU buffer

### light-point

Create a point light source with radial attenuation.

**Inputs:**
- `position` (vec3): Light position in world space
- `color` (vec3): Light color RGB (automatically clamped to [0.0, 1.0])
- `intensity` (f32): Light intensity multiplier (must be non-negative)
- `radius` (f32): Attenuation radius (must be positive)

**Outputs:**
- `light_data` (String): JSON-encoded light data compatible with GPU uniforms

**Example:**
```
position: [0, 5, 0]
color: [1, 0.8, 0.6] (warm white)
intensity: 1.5
radius: 10.0
→ light_data: {"light_type":"point","position":[0,5,0],"color":[1,0.8,0.6],"intensity":1.5,"radius":10}
```

**Features:**
- Position-based lighting with distance falloff
- Attenuation formula: `1 / (1 + (distance² / radius²))`
- Color clamping and validation
- JSON output compatible with MultiLightUniforms GPU buffer

### lighting-phong

Calculate Phong lighting (diffuse + specular) on CPU.

**Inputs:**
- `normal` (vec3): Surface normal vector (automatically normalized)
- `light_dir` (vec3): Direction to light (automatically normalized)
- `view_dir` (vec3): Direction to camera (automatically normalized)
- `surface_color` (vec3): Material/surface color RGB
- `light_color` (vec3): Light color RGB
- `shininess` (f32): Specular shininess factor (typically 1-128, must be non-negative)

**Outputs:**
- `lit_color` (vec3): Resulting lit color (diffuse + specular)

**Example:**
```
normal: [0, 1, 0] (up)
light_dir: [0, 1, 0] (from above)
view_dir: [0, 1, 0] (camera above)
surface_color: [0.5, 0.5, 0.5] (gray)
light_color: [1, 1, 1] (white)
shininess: 32.0
→ lit_color: [1.0, 1.0, 1.0] (full brightness due to alignment)
```

**Lighting Model:**
- **Diffuse**: Lambertian reflection `max(N · L, 0)`
- **Specular**: Phong reflection `(R · V)^shininess`
- **Shininess**: Lower values = broader highlights, higher = tighter highlights
- **Result**: Clamped to [0.0, 1.0] range

**Use Cases:**
- CPU-side lighting for validation
- Per-vertex lighting calculations
- Testing lighting formulas before GPU implementation

## Built-in Shader Nodes

### Shader Preview Node (Phase 1: Placeholder)

**Component ID:** `builtin:graphics:shader-preview`
**Display Name:** Shader Preview
**Category:** Graphics

Displays rendered shader output in the node footer.

**Phase 1 Status:** Placeholder mode - displays UI controls but no actual GPU rendering.

**Inputs:**
- `texture` (texture, optional): Rendered texture data to display
- `zoom` (f32, optional): Display zoom level (1.0 = 100%, range: 0.1 to 10.0)

**Outputs:** None (displays in footer)

**Footer UI Features (Phase 1):**
- Preview area placeholder (shows 🖼 icon and status message)
- Size presets: Small (400×300), Medium (600×450), Large (800×600)
- Zoom slider (0.1× to 10×)
- Auto-refresh toggle with refresh rate control (1-60 Hz)
- Stats: Last texture size, time since last update

**Future (Phase 2):**
- WebGPU integration for actual texture rendering
- Real-time shader preview
- GPU performance metrics
- Texture filtering and sampling controls

### Shader Program Linker Node (Phase 2: Step 9)

**Component ID:** `builtin:graphics:shader-program-linker`
**Display Name:** Shader Program Linker
**Category:** Graphics

Links vertex and fragment shaders into an executable GPU program with compilation validation.

**Phase 2 Status:** Fully implemented with GLSL compilation and error reporting.

**Inputs:**
- `vertex_shader` (string, required): Vertex shader GLSL source code
- `fragment_shader` (string, required): Fragment shader GLSL source code

**Outputs:**
- `program` (binary): Linked shader program ID (UUID) on successful compilation

**Footer UI Features:**
- **Status Indicator**: Color-coded compilation status
  - Gray: Not compiled (idle)
  - Yellow: Compiling
  - Green: ✓ Linked successfully
  - Red: ✗ Linking failed
- **Program ID Display**: Shows generated UUID for successfully linked programs
- **Error Details**: Scrollable error message panel with detailed compilation errors
- **Shader Source Info**: Line counts for vertex and fragment shaders
- **Link Button**: Manual compilation trigger (when idle or failed)

**Compilation Process:**
1. Validates both vertex and fragment shaders using naga GLSL parser
2. Compiles GLSL → WGSL via naga intermediate representation
3. Creates wgpu::ShaderModule instances for both stages
4. Validates shader interface compatibility (TODO: full interface matching)
5. Generates unique program ID on successful linking

**Example Usage:**
```
Vertex Shader (GLSL) → shader-program-linker ← Fragment Shader (GLSL)
                              ↓
                      Linked Program (UUID)
                              ↓
                    (Future: Render Pipeline)
```

**Error Handling:**
- **Parse Errors**: Detailed GLSL syntax errors with context
- **Validation Errors**: Shader semantic errors (types, uniforms, etc.)
- **Interface Mismatches**: Vertex outputs vs fragment inputs (TODO)

**Example Shaders:**
See `examples/shaders/lighting/` for reference implementations:
- `basic_diffuse.vert.glsl` / `.frag.glsl` - Simple Lambert diffuse
- `phong.vert.glsl` / `.frag.glsl` - Phong with specular
- `multi_light.vert.glsl` / `.frag.glsl` - Multi-light support (up to 8 lights)

**GPU Buffer Compatibility:**
The linker validates shaders that use the standard GPU buffer layouts:
- Vertex buffers: positions, normals, uvs
- Uniform buffers: MVP matrices, camera data
- Light buffers: MultiLightUniforms (up to 8 lights)

See `src/gpu/buffer.rs` and `examples/shaders/lighting/README.md` for buffer layout specifications.

## Testing

Integration test graphs are available in `tests/component_tests/`:

### graphics_geometry.json
Tests geometry primitive components:
- UV sphere generation (vertex count validation)
- Cube generation (24 vertices for proper normals)
- Plane generation (subdivided mesh)

### graphics_camera.json
Tests camera and matrix operations:
- Vector construction for camera position/target/up
- Perspective camera matrix generation
- Matrix multiplication for MVP calculation

### graphics_shader_pipeline.json
Complete end-to-end shader pipeline:
- Geometry generation (sphere)
- Camera setup (perspective with look-at)
- Render target configuration (1920×1080, MSAA 4×)
- Material color definition
- Shader preview (Phase 1 placeholder)

### graphics_lighting.json
Tests Phase 2 lighting components:
- Directional light creation with JSON output validation
- Point light creation with radius-based attenuation
- Phong lighting calculations (full brightness, perpendicular, colored surfaces)
- Multi-component lighting workflow (directional + point lights)

### graphics_complete_workflow.json
Comprehensive end-to-end graphics pipeline (Phase 2):
- Geometry generation (sphere primitive)
- Camera configuration (perspective with look-at)
- Lighting setup (directional + point lights)
- Shader authoring (vertex + fragment GLSL)
- Program linking with compilation validation
- Render target configuration (1920×1080, MSAA 4×)
- Shader preview integration
- 16 nodes demonstrating complete workflow from primitives to rendering

## Build Instructions

All graphics components are built using the shared build system:

```bash
# Build all graphics components
cd components/graphics
just build

# Build specific component
cd components/graphics/vec3-add
just build
just install  # Copy to components/bin/

# Run tests
just test
```

## Implementation Notes

### Coordinate System
- **Handedness:** Right-handed coordinate system
- **Up Direction:** +Y axis
- **Forward Direction:** -Z axis (camera looks toward -Z)
- **Matrix Order:** Column-major (consistent with GLSL/OpenGL)

### Data Types
- **vec3:** Represented as 3-element f32 list [x, y, z]
- **mat4:** Represented as 16-element f32 list (column-major)
- **color:** Represented as 3-element f32 list [r, g, b]
- **UV coordinates:** Represented as (f32, f32) tuple

### Performance Characteristics
- **Binary Sizes:** 100-150 KB per component (with glam dependency)
- **Execution Time:** <5ms for typical operations
- **Memory:** Stack-allocated, no heap allocations in hot paths

### Dependencies
- **glam 0.25:** Vector and matrix math (all components)
- **serde/serde_json:** Serialization (render-target)
- **wit-bindgen 0.30:** WASM interface generation

## Architecture

### Component Categories

**Vector Math:**
- Foundation for all 3D operations
- Pure mathematical operations (no side effects)
- All operations preserve precision

**Matrix Operations:**
- Transform composition and manipulation
- Column-major format (GLSL/OpenGL compatible)
- Right-handed coordinate system

**Geometry Primitives:**
- Parametric mesh generation
- Consistent vertex layout (positions, normals, UVs, indices)
- Optimized for GPU consumption

**Camera:**
- View and projection matrix generation
- Standard graphics pipeline conventions
- Right-handed look-at and perspective

**Render Target:**
- Configuration for GPU rendering
- Multiple format support (LDR/HDR)
- MSAA anti-aliasing configuration

**Built-in Nodes:**
- Integrated UI for complex interactions
- Footer views for visualization
- Stateful components with persistence

## Roadmap

### Phase 1: Core Components (Complete ✓)
- Vector math operations (7 components)
- Matrix operations (2 components)
- Color utilities (1 component)
- Geometry primitives (3 components)
- Camera system (1 component)
- Render target configuration (1 component)
- Shader preview placeholder (1 built-in node)

### Phase 2: GPU Integration & Lighting (Complete ✓)
- WebGPU integration (wgpu 22.0 + naga)
- Shader compilation (GLSL → WGSL via naga)
- GPU buffer management (vertex, index, uniform buffers)
- Light uniform buffers (directional & point lights, multi-light support)
- Basic lighting calculations (3 WASM components)
- Shader program linker (1 built-in node)
- Example GLSL shaders (diffuse, Phong, multi-light)

**New Components (Step 8):**
- `light-directional`: Directional light source (sun-like)
- `light-point`: Point light with radial attenuation
- `lighting-phong`: CPU-side Phong lighting calculation

**New Built-in Nodes (Step 9):**
- `shader-program-linker`: Links vertex + fragment shaders into executable GPU program

## Phase 3: PBR Materials and Texture System (In Progress)

Phase 3 extends the graphics system with physically-based rendering (PBR) materials, texture sampling, and advanced lighting.

### Phase 3 Step 1: Texture System Foundation (Complete ✓)

**Texture Loading and Sampling:**
- Built-in `texture-loader` node: Load PNG, JPG, BMP, GIF images with file picker UI
- GPU texture management (`src/gpu/texture.rs`): Upload textures to GPU, create samplers
- `texture-sampler` component: CPU-side bilinear texture sampling with UV wrapping modes

**Features:**
- Supported formats: PNG, JPG, BMP, GIF → RGBA8 (sRGB)
- UV wrapping modes: repeat, clamp, mirror
- Bilinear filtering for smooth sampling
- Thumbnail preview in texture loader UI
- Texture statistics display (dimensions, memory usage)

**Integration:**
- `image` crate dependency for file loading
- Registered as `builtin:graphics:texture-loader`
- Outputs: texture data (RGBA8), width, height

### Phase 3 Steps 3-7: PBR Materials (Complete ✓)

**PBR BRDF Components:**
- `pbr-fresnel`: Fresnel-Schlick approximation for reflectivity
- `pbr-ggx-distribution`: GGX/Trowbridge-Reitz normal distribution function
- `pbr-smith-geometry`: Smith geometry/visibility term (shadowing/masking)
- `pbr-material`: Material property management with F0 calculation
- `pbr-brdf`: Complete Cook-Torrance BRDF assembly

**Advanced Lighting:**
- `light-spot`: Spot light with cone-shaped emission and smooth falloff
- `normal-map`: Tangent-space to world-space normal transformation

**Example Shaders:**
- `pbr_single_light.vert/frag.glsl`: Single directional light PBR
- `pbr_multi_light.vert/frag.glsl`: Up to 8 mixed lights (directional/point/spot)
- `pbr_normal_mapped.vert/frag.glsl`: Full PBR with normal mapping

**Material Workflow:**
- Metallic/roughness workflow (industry standard)
- Cook-Torrance microfacet BRDF
- Energy conservation: diffuse + specular ≤ 1.0
- Physically accurate F0 calculation: `lerp(0.04, base_color, metallic)`

**Documentation:**
- Complete PBR implementation guide: `docs/PHASE3_PBR_COMPLETE.md`
- Graphics pipeline summary: `docs/GRAPHICS_PIPELINE_SUMMARY.md`
- PBR shader README: `examples/shaders/pbr/README.md`

## Phase 4: Advanced Rendering - Shadow Mapping (Complete)

Phase 4 implements advanced rendering features starting with a complete shadow mapping system.

### Shadow Components

#### shadow-directional
Calculate cascaded shadow map matrices for directional lights (sun/moon).

**Inputs:**
- `light_direction` (vec3): Light direction (will be normalized)
- `view_matrix` (mat4): Camera view matrix
- `projection_matrix` (mat4): Camera projection matrix
- `near` (f32): Camera near plane distance
- `far` (f32): Camera far plane distance
- `cascade_count` (u32): Number of cascades (1-4)

**Outputs:**
- `shadow_matrices` (list): Flattened shadow matrices (cascade_count × 16 floats)
- `cascade_splits` (list): Split distances for cascade selection

**Features:**
- Practical split scheme (λ=0.5) for balanced cascade distribution
- Frustum-fitted orthographic projection per cascade
- Tight AABB bounds for maximum shadow resolution
- 6 unit tests covering split schemes, matrix generation, error handling

**Example:**
```
light_direction: [0.0, -1.0, 0.0]  // Downward sun
view_matrix: <from perspective-camera>
projection_matrix: <from perspective-camera>
near: 0.1
far: 100.0
cascade_count: 4
→ shadow_matrices: [64 floats = 4 cascades × 16 elements]
→ cascade_splits: [split0, split1, split2, split3]
```

#### shadow-point
Calculate 6 cubemap shadow matrices for point lights (omnidirectional).

**Inputs:**
- `light_position` (vec3): Point light world position
- `near` (f32): Shadow near plane distance
- `far` (f32): Shadow far plane distance (light radius)

**Outputs:**
- `shadow_matrices` (list): 6 matrices flattened (96 floats)

**Features:**
- 90° FOV perspective projection for each cubemap face
- Face order: +X, -X, +Y, -Y, +Z, -Z
- Correct up vectors for each face orientation
- 7 unit tests covering count, validity, position variations

**Example:**
```
light_position: [0.0, 5.0, 0.0]
near: 0.1
far: 10.0  // Light radius
→ shadow_matrices: [96 floats = 6 faces × 16 elements]
```

#### shadow-spot
Calculate perspective shadow matrix for spot lights (cone-shaped).

**Inputs:**
- `light_position` (vec3): Spot light world position
- `light_direction` (vec3): Spot light direction (will be normalized)
- `cone_angle` (f32): Spot light cone angle in degrees (0-180)
- `near` (f32): Shadow near plane distance
- `far` (f32): Shadow far plane distance (light range)

**Outputs:**
- `shadow_matrix` (list): Shadow matrix (16 floats, column-major)

**Features:**
- FOV matches cone angle for exact shadow coverage
- Perspective projection matching light frustum
- Automatic up vector selection (avoids parallel-to-direction)
- 9 unit tests covering cone angles, directions, error handling

**Example:**
```
light_position: [0.0, 5.0, 0.0]
light_direction: [0.0, -1.0, 0.0]
cone_angle: 45.0  // Degrees
near: 0.1
far: 20.0
→ shadow_matrix: [16 floats]
```

### Shadow Sampling Shaders

**Location**: `examples/shaders/shadow/`

Complete GLSL shaders for shadow sampling with PCF (Percentage Closer Filtering):

- **shadow_common.glsl**: Shared PCF utilities (4, 9, 16 samples) + bias calculation
- **shadow_directional.frag.glsl**: CSM with automatic cascade selection
- **shadow_point.frag.glsl**: Cubemap shadows with distance attenuation
- **shadow_spot.frag.glsl**: Cone-matched shadows with smooth falloff

See `examples/shaders/shadow/README.md` for complete usage guide, buffer layouts, and performance tuning.

### Phase 4: Future Work
- Environment maps and skybox rendering
- Image-based lighting (IBL) with split-sum approximation
- Post-processing effects (bloom, tone mapping, SSAO, DOF, motion blur)
- Advanced PBR features (clear coat, subsurface scattering, anisotropic reflections)
- Performance optimizations (compute shaders, light culling, LOD systems)

## License

Part of WasmFlow Graphics Library.
