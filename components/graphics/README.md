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

### Phase 2: GPU Integration (Future)
- WebGPU integration
- Real-time shader rendering
- Texture display and manipulation
- GPU buffer management
- Shader compilation and hot-reload

### Phase 3: Advanced Features (Future)
- Lighting calculations
- Material system
- Post-processing effects
- Compute shaders
- Ray tracing utilities

## License

Part of WasmFlow Graphics Library.
