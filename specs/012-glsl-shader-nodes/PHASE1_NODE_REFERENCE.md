# Phase 1 Node Quick Reference

**Total Nodes**: 21 (2 built-in + 19 WASM components)
**Category**: Graphics

---

## Built-in Nodes (2)

### glsl-shader-editor
**Type**: Built-in (code editor)
**Purpose**: Write and validate GLSL shaders

| Input | Type | Description |
|-------|------|-------------|
| _(none)_ | - | Code edited in footer |

| Output | Type | Description |
|--------|------|-------------|
| shader_source | string | GLSL source code |
| shader_type | string | "vertex", "fragment", or "compute" |
| entry_point | string | Main function name |

**Features**:
- GLSL syntax highlighting
- Real-time validation (naga)
- Templates (basic, PBR, textured)
- Compilation error display
- Save code in graph option

---

### shader-preview
**Type**: Built-in (display)
**Purpose**: Display rendered shader output

| Input | Type | Description |
|-------|------|-------------|
| texture | texture-data | Rendered image |
| zoom | f32 (optional) | Display zoom level |

| Output | Type | Description |
|--------|------|-------------|
| _(none)_ | - | Displays in footer |

**Features**:
- Image display with zoom/pan
- Size controls (fit/actual)
- Auto-refresh option
- Stats display (resolution, format)

---

## Vector Math (9 WASM components)

### vec2-construct, vec3-construct, vec4-construct
**Purpose**: Create vectors from scalar components

**vec2-construct**:
- Inputs: `x` (f32), `y` (f32)
- Outputs: `vector` (vec2)

**vec3-construct**:
- Inputs: `x` (f32), `y` (f32), `z` (f32)
- Outputs: `vector` (vec3)

**vec4-construct**:
- Inputs: `x` (f32), `y` (f32), `z` (f32), `w` (f32)
- Outputs: `vector` (vec4)

**Uses**: Position, color, UV coordinates, direction

---

### vec2-add, vec3-add, vec4-add
**Purpose**: Add two vectors component-wise

**vec3-add** (example):
- Inputs: `a` (vec3), `b` (vec3)
- Outputs: `result` (vec3)

**Implementation**: `result = a + b`

**Uses**: Translate positions, combine directions

---

### vec-normalize
**Purpose**: Normalize vector to unit length

| Input | Type | Description |
|-------|------|-------------|
| vector | vec3 | Input vector |

| Output | Type | Description |
|--------|------|-------------|
| normalized | vec3 | Unit vector |
| length | f32 | Original length |

**Implementation**: `normalized = vector / length(vector)`

**Uses**: Direction vectors, normals, lighting

**Error**: Returns error if vector has zero length

---

### vec-dot
**Purpose**: Calculate dot product

| Input | Type | Description |
|-------|------|-------------|
| a | vec3 | First vector |
| b | vec3 | Second vector |

| Output | Type | Description |
|--------|------|-------------|
| dot | f32 | Dot product |

**Implementation**: `dot = a.x*b.x + a.y*b.y + a.z*b.z`

**Uses**:
- N·L lighting calculations
- Angle between vectors (cos θ)
- Projection length
- Surface check (front/back facing)

---

### vec-cross
**Purpose**: Calculate cross product (vec3 only)

| Input | Type | Description |
|-------|------|-------------|
| a | vec3 | First vector |
| b | vec3 | Second vector |

| Output | Type | Description |
|--------|------|-------------|
| cross | vec3 | Perpendicular vector |

**Implementation**: Standard cross product formula

**Uses**:
- Calculate normals from edges
- Build coordinate systems
- Find perpendicular direction

**Note**: Result is perpendicular to both inputs

---

## Matrix Math (2 WASM components)

### mat4-construct
**Purpose**: Create 4x4 matrix

**Option A - From components**:
- Inputs: `m00` through `m33` (16 x f32)
- Outputs: `matrix` (mat4)

**Option B - From column vectors**:
- Inputs: `col0`, `col1`, `col2`, `col3` (4 x vec4)
- Outputs: `matrix` (mat4)

**Format**: Column-major (OpenGL/WebGPU standard)

**Uses**: Custom transformations, projection matrices

---

### mat4-multiply
**Purpose**: Multiply two matrices

| Input | Type | Description |
|-------|------|-------------|
| a | mat4 | First matrix |
| b | mat4 | Second matrix |

| Output | Type | Description |
|--------|------|-------------|
| result | mat4 | Product matrix |

**Implementation**: Standard matrix multiplication

**Uses**:
- Combine transformations
- Model-View-Projection calculation
- Transform stacking

**Note**: Order matters! `A * B ≠ B * A`

---

## Color (1 WASM component)

### color-rgb
**Purpose**: Create color from RGB components

| Input | Type | Description |
|-------|------|-------------|
| r | f32 | Red (0-1) |
| g | f32 | Green (0-1) |
| b | f32 | Blue (0-1) |

| Output | Type | Description |
|--------|------|-------------|
| color | vec3 | RGB color vector |

**Validation**: Values clamped to [0, 1] range

**Uses**: Material colors, light colors, tints

---

## Geometry Primitives (3 WASM components)

### primitive-sphere
**Purpose**: Generate UV sphere mesh

| Input | Type | Description |
|-------|------|-------------|
| radius | f32 | Sphere radius |
| segments | u32 | Horizontal segments |
| rings | u32 | Vertical rings |

| Output | Type | Description |
|--------|------|-------------|
| vertices | StringListVal | JSON array of vec3 positions |
| normals | StringListVal | JSON array of vec3 normals |
| uvs | StringListVal | JSON array of vec2 UVs |
| indices | U32ListVal | Triangle indices |

**Vertex Count**: `(segments + 1) * (rings + 1)`
**Triangle Count**: `segments * rings * 2`

**Example**:
- `segments=32, rings=16` → 561 vertices, 1024 triangles
- Good for: Preview quality spheres

**Uses**: Planets, balls, round objects

---

### primitive-cube
**Purpose**: Generate box mesh

| Input | Type | Description |
|-------|------|-------------|
| size | vec3 | Dimensions (x, y, z) |

| Output | Type | Description |
|--------|------|-------------|
| vertices | StringListVal | JSON array of vec3 positions |
| normals | StringListVal | JSON array of vec3 normals |
| uvs | StringListVal | JSON array of vec2 UVs |
| indices | U32ListVal | Triangle indices |

**Vertex Count**: 24 (4 per face, 6 faces)
**Triangle Count**: 12 (2 per face)

**Uses**: Boxes, buildings, rooms

---

### primitive-plane
**Purpose**: Generate subdivided plane mesh

| Input | Type | Description |
|-------|------|-------------|
| width | f32 | Width (X axis) |
| height | f32 | Height (Z axis) |
| subdivisions | u32 | Grid subdivisions |

| Output | Type | Description |
|--------|------|-------------|
| vertices | StringListVal | JSON array of vec3 positions |
| normals | StringListVal | JSON array of vec3 normals |
| uvs | StringListVal | JSON array of vec2 UVs |
| indices | U32ListVal | Triangle indices |

**Vertex Count**: `(subdivisions + 1)^2`
**Triangle Count**: `subdivisions^2 * 2`

**Example**:
- `subdivisions=10` → 121 vertices, 200 triangles
- `subdivisions=100` → 10,201 vertices, 20,000 triangles

**Uses**: Ground, water, terrain, flat surfaces

---

## Camera (1 WASM component)

### perspective-camera
**Purpose**: Calculate view and projection matrices

| Input | Type | Description |
|-------|------|-------------|
| position | vec3 | Camera position |
| target | vec3 | Look-at target point |
| up | vec3 | Up vector (default: 0,1,0) |
| fov | f32 | Field of view (degrees) |
| aspect_ratio | f32 | Width / height |
| near | f32 | Near clipping plane |
| far | f32 | Far clipping plane |

| Output | Type | Description |
|--------|------|-------------|
| view_matrix | mat4 | World → Camera transform |
| projection_matrix | mat4 | Camera → Clip space |
| camera_position | vec3 | Camera position (passthrough) |
| view_direction | vec3 | Normalized look direction |

**Implementation**:
- View matrix: Look-at matrix from position/target/up
- Projection: Perspective projection with FOV

**Typical Values**:
- `fov`: 45-90 degrees (60 is common)
- `aspect_ratio`: 16/9 = 1.777, 4/3 = 1.333
- `near`: 0.1
- `far`: 1000.0

**Uses**: 3D scene camera setup

---

## Render Configuration (1 WASM component)

### render-target
**Purpose**: Configure render target parameters

| Input | Type | Description |
|-------|------|-------------|
| width | u32 | Framebuffer width |
| height | u32 | Framebuffer height |
| format | string | "rgba8", "rgba32-float", etc. |
| depth | bool | Enable depth buffer |
| multisample | u32 | MSAA samples (1, 2, 4, 8) |

| Output | Type | Description |
|--------|------|-------------|
| config | string | JSON config for renderer |

**Formats**:
- `"rgba8"` - Standard 8-bit color (most common)
- `"rgba32-float"` - HDR rendering
- `"r8"` - Single channel (masks, depth)

**MSAA Samples**:
- `1` - No antialiasing
- `2` - 2x MSAA
- `4` - 4x MSAA (good quality/performance)
- `8` - 8x MSAA (best quality)

**Typical Settings**:
- HD: 1920x1080, rgba8, depth=true, multisample=4
- Preview: 800x600, rgba8, depth=true, multisample=1

**Uses**: Define rendering resolution and quality

---

## Node Count Summary

| Category | Count | Notes |
|----------|-------|-------|
| **Built-in** | **2** | Editor + Preview |
| **Vector Math** | **9** | Construct (3) + Add (3) + Ops (3) |
| **Matrix Math** | **2** | Construct + Multiply |
| **Color** | **1** | RGB constructor |
| **Geometry** | **3** | Sphere, Cube, Plane |
| **Camera** | **1** | Perspective camera |
| **Config** | **1** | Render target |
| **TOTAL** | **19 WASM + 2 Built-in = 21** | |

---

## Common Workflows

### Workflow 1: Create Geometry + Camera

```
[primitive-sphere]
  radius: 1.0, segments: 32, rings: 16
  → vertices, normals, uvs

[vec3-construct] (camera position)
  x: 0, y: 2, z: 5
  → camera_pos

[vec3-construct] (target)
  x: 0, y: 0, z: 0
  → target_pos

[perspective-camera]
  position: camera_pos
  target: target_pos
  fov: 60, aspect: 1.777
  → view_matrix, projection_matrix
```

---

### Workflow 2: Vector Math

```
[vec3-construct]
  x: 1, y: 0, z: 0
  → vec_a

[vec3-construct]
  x: 0, y: 1, z: 0
  → vec_b

[vec-cross]
  a: vec_a
  b: vec_b
  → cross (result: 0, 0, 1)

[vec-normalize]
  vector: cross
  → normalized, length
```

---

### Workflow 3: Basic Shader Setup

```
[glsl-shader-editor] (vertex)
  type: "vertex"
  template: "Basic Vertex"
  → vertex_source

[glsl-shader-editor] (fragment)
  type: "fragment"
  template: "Basic PBR"
  → fragment_source

[render-target]
  width: 800, height: 600
  format: "rgba8"
  → config

[shader-preview]
  texture: (Phase 2 - from renderer)
  → (display)
```

---

## Type Reference

### Graphics Types (New in Phase 1)

```rust
// Vector types
vec2 { x: f32, y: f32 }
vec3 { x: f32, y: f32, z: f32 }
vec4 { x: f32, y: f32, z: f32, w: f32 }

// Matrix type (column-major)
mat4 {
    m00, m01, m02, m03,
    m10, m11, m12, m13,
    m20, m21, m22, m23,
    m30, m31, m32, m33: f32
}

// Texture type
texture-data {
    width: u32,
    height: u32,
    format: texture-format,
    data: list<u8>
}
```

### Value Variants

```rust
Value::Vec2Val(vec2)
Value::Vec3Val(vec3)
Value::Vec4Val(vec4)
Value::Mat4Val(mat4)
Value::TextureVal(texture-data)
```

---

## Performance Notes

### Component Sizes
- Vector/matrix components: ~100-120KB
- Geometry components: ~120-150KB
- Built-in nodes: Part of main binary

### Execution Times
- Vector ops: <1ms
- Matrix multiply: <5ms
- Sphere generation (32x16): ~8ms
- Sphere generation (64x32): ~25ms

### Mesh Size Guidelines
- Preview quality: 32x16 (sphere), 10 (plane subdivisions)
- Medium quality: 64x32 (sphere), 50 (plane subdivisions)
- High quality: 128x64 (sphere), 100 (plane subdivisions)
- Extreme: 256x128 (sphere) - may be slow in JSON serialization

---

**Last Updated**: 2025-11-20
**Phase**: 1 - Foundation
**Status**: Planning
