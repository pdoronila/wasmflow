# Phase 1 Implementation Plan: GLSL Shader Foundation Nodes

**Feature**: GLSL Physically Based Shader Authoring System
**Phase**: 1 - Foundation
**Category**: Graphics (new palette category)
**Created**: 2025-11-20
**Status**: Planning

## Overview

Phase 1 establishes the foundational infrastructure for GLSL shader authoring in wasmflow. This phase introduces a new "Graphics" category in the component palette and implements core nodes needed for basic shader creation, rendering, and preview capabilities.

**Key Deliverables**:
1. Extended WIT type system for graphics primitives (vec2, vec3, vec4, mat4, texture)
2. New "Graphics" category in component palette
3. GLSL shader editor (built-in node, similar to WASM Creator)
4. Vector and matrix math nodes (12 WASM components)
5. Basic geometry primitives (3 WASM components)
6. Camera node (1 WASM component)
7. Render target and preview system (2 nodes)

**Total New Nodes**: 21 nodes (2 built-in + 19 WASM components)

---

## Architecture Decisions

### 1. Component Distribution Strategy

**Built-in Nodes** (in `src/builtin/`):
- `glsl-shader-editor` - Requires code editor widget, compilation, syntax highlighting
- `shader-preview` - Requires egui texture rendering, GPU context

**WASM Components** (in `components/graphics/`):
- All math/utility nodes (vectors, matrices, colors)
- Geometry primitives (sphere, cube, plane)
- Camera nodes
- Render target configuration

**Rationale**:
- Shader editor needs direct access to egui widgets and compilation pipeline
- Preview node needs egui TextureHandle and GPU rendering
- Math/geometry nodes are pure computation - perfect for WASM isolation
- Render configuration is data transformation - suitable for WASM

### 2. Graphics Type System Design

**WIT Type Extensions** (in `wit/wasmflow-node.wit`):

```wit
// Add to types interface
variant value {
    // ... existing types ...

    // Graphics primitive types
    vec2-val(vec2),
    vec3-val(vec3),
    vec4-val(vec4),
    mat4-val(mat4),
    texture-val(texture-data),
}

// Graphics type definitions
record vec2 {
    x: f32,
    y: f32,
}

record vec3 {
    x: f32,
    y: f32,
    z: f32,
}

record vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

// 4x4 matrix (column-major, standard for OpenGL/WebGPU)
record mat4 {
    m00: f32, m01: f32, m02: f32, m03: f32,
    m10: f32, m11: f32, m12: f32, m13: f32,
    m20: f32, m21: f32, m22: f32, m23: f32,
    m30: f32, m31: f32, m32: f32, m33: f32,
}

// Texture data representation
record texture-data {
    width: u32,
    height: u32,
    format: texture-format,
    data: list<u8>,  // Raw pixel data
}

enum texture-format {
    rgba8,
    rgb8,
    r8,
    rgba32-float,
    depth24-stencil8,
}

// Data type extensions
enum data-type {
    // ... existing types ...
    vec2-type,
    vec3-type,
    vec4-type,
    mat4-type,
    texture-type,
}
```

**Update Locations**:
1. `wit/wasmflow-node.wit` - Add graphics types
2. `src/graph/node.rs` - Update `NodeValue` enum to include graphics types
3. `src/ui/component_view.rs` - Add display logic for graphics types
4. All component `wit/deps/wasmflow-node/node.wit` - Propagate updated types

### 3. Category Organization

**New Directory Structure**:

```
components/
├── graphics/              # NEW: All shader-related nodes
│   ├── .templates/        # Component templates for graphics nodes
│   │   ├── component.wit
│   │   └── component-with-ui.wit
│   ├── math/              # Vector and matrix operations
│   │   ├── vec2-construct/
│   │   ├── vec3-construct/
│   │   ├── vec4-construct/
│   │   ├── vec2-add/
│   │   ├── vec3-add/
│   │   ├── vec4-add/
│   │   ├── vec-normalize/
│   │   ├── vec-dot/
│   │   ├── vec-cross/
│   │   ├── mat4-construct/
│   │   ├── mat4-multiply/
│   │   └── color-rgb/
│   ├── primitives/        # Geometry primitives
│   │   ├── primitive-sphere/
│   │   ├── primitive-cube/
│   │   └── primitive-plane/
│   ├── camera/            # Camera nodes
│   │   └── perspective-camera/
│   ├── config/            # Render configuration
│   │   └── render-target/
│   ├── Justfile           # Category-level build
│   └── README.md          # Graphics library documentation
└── bin/                   # Compiled WASM binaries
```

**Palette Category**: All nodes will have `category: Some("Graphics".to_string())` in ComponentInfo

---

## Phase 1 Nodes Specification

### Built-in Node: GLSL Shader Editor

**Location**: `src/builtin/glsl_shader_editor.rs`

**Structure** (similar to `WasmCreatorNode`):

```rust
pub struct GlslShaderEditorNode {
    pub id: Uuid,
    pub shader_name: String,
    pub shader_type: ShaderType,  // Vertex, Fragment, Compute
    pub source_code: String,
    pub save_code: bool,
    pub compilation_state: ShaderCompilationState,
    pub last_error: Option<String>,
    pub editor_theme: CodeTheme,
}

pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

pub enum ShaderCompilationState {
    Idle,
    Compiling,
    Success,
    Failed,
}
```

**Inputs**: None (code edited in footer)

**Outputs**:
- `shader_source` (string) - GLSL source code
- `shader_type` (string) - "vertex", "fragment", or "compute"
- `entry_point` (string) - Main function name (default: "main")

**Footer UI**:
- Shader name text input
- Shader type dropdown (Vertex/Fragment/Compute)
- Code editor with GLSL syntax highlighting
- Compile button
- Error/success status display
- Save code checkbox
- Template dropdown (basic, PBR, textured, etc.)

**Templates**:

```glsl
// Vertex Shader Template
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec2 frag_uv;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 model;
    mat4 view;
    mat4 projection;
};

void main() {
    vec4 world_pos = model * vec4(position, 1.0);
    frag_position = world_pos.xyz;
    frag_normal = mat3(model) * normal;
    frag_uv = uv;
    gl_Position = projection * view * world_pos;
}
```

```glsl
// Fragment Shader Template - Basic PBR
#version 450

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform Material {
    vec4 base_color;
    float metallic;
    float roughness;
};

layout(set = 0, binding = 2) uniform Light {
    vec3 light_position;
    vec3 light_color;
};

void main() {
    vec3 N = normalize(frag_normal);
    vec3 L = normalize(light_position - frag_position);
    float NdotL = max(dot(N, L), 0.0);

    vec3 diffuse = base_color.rgb * light_color * NdotL;
    out_color = vec4(diffuse, base_color.a);
}
```

**GLSL Validation**:
- Use `naga` crate for GLSL → SPIR-V validation
- Display errors with line numbers
- Show warnings for common issues

**Implementation Tasks**:
1. Create `src/builtin/glsl_shader_editor.rs`
2. Add GLSL syntax highlighting to `CodeEditorWidget`
3. Integrate `naga` for shader validation
4. Add shader templates
5. Implement compilation state tracking
6. Add to builtin node registry

---

### Built-in Node: Shader Preview

**Location**: `src/builtin/shader_preview.rs`

**Purpose**: Display rendered shader output in node footer

**Structure**:

```rust
pub struct ShaderPreviewNode {
    pub id: Uuid,
    pub preview_size: (u32, u32),  // Width, height
    pub auto_refresh: bool,
    pub refresh_rate: f32,  // Hz
    pub texture_handle: Option<egui::TextureHandle>,
}
```

**Inputs**:
- `texture` (texture-data) - Rendered image to display
- `zoom` (f32, optional) - Display zoom level (default: 1.0)

**Outputs**: None (displays in footer)

**Footer UI**:
- Image display with egui::Image
- Size controls (actual size / fit to width / fit to height)
- Refresh controls (auto/manual, rate slider)
- Stats display (resolution, format, update time)

**Implementation Tasks**:
1. Create `src/builtin/shader_preview.rs`
2. Implement texture-data → egui::TextureHandle conversion
3. Add zoom/pan controls
4. Add to builtin node registry

---

### WASM Component: vec2-construct

**Location**: `components/graphics/math/vec2-construct/`

**Purpose**: Create a 2D vector from components

**Inputs**:
- `x` (f32) - X component
- `y` (f32) - Y component

**Outputs**:
- `vector` (vec2) - Constructed vector

**Implementation**:

```rust
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        let x = extract_f32(&inputs, "x")?;
        let y = extract_f32(&inputs, "y")?;

        Ok(vec![
            ("vector".to_string(), Value::Vec2Val(Vec2 { x, y }))
        ])
    }
}
```

**Tests**:
1. Construct with positive values
2. Construct with negative values
3. Construct with zero
4. Missing input error handling

**Build**: Standard WASM component build

---

### WASM Component: vec3-construct

**Location**: `components/graphics/math/vec3-construct/`

**Purpose**: Create a 3D vector from components

**Inputs**:
- `x` (f32)
- `y` (f32)
- `z` (f32)

**Outputs**:
- `vector` (vec3)

**Implementation**: Similar to vec2-construct

---

### WASM Component: vec4-construct

**Location**: `components/graphics/math/vec4-construct/`

**Purpose**: Create a 4D vector from components (useful for colors, homogeneous coords)

**Inputs**:
- `x` (f32)
- `y` (f32)
- `z` (f32)
- `w` (f32)

**Outputs**:
- `vector` (vec4)

---

### WASM Component: vec2-add, vec3-add, vec4-add

**Purpose**: Add two vectors component-wise

**Inputs**:
- `a` (vec2/vec3/vec4)
- `b` (vec2/vec3/vec4)

**Outputs**:
- `result` (vec2/vec3/vec4)

**Implementation**:

```rust
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        let a = extract_vec3(&inputs, "a")?;
        let b = extract_vec3(&inputs, "b")?;

        let result = Vec3 {
            x: a.x + b.x,
            y: a.y + b.y,
            z: a.z + b.z,
        };

        Ok(vec![("result".to_string(), Value::Vec3Val(result))])
    }
}
```

---

### WASM Component: vec-normalize

**Purpose**: Normalize a vector to unit length

**Inputs**:
- `vector` (vec3) - Input vector

**Outputs**:
- `normalized` (vec3) - Unit vector
- `length` (f32) - Original length

**Implementation**:

```rust
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        let v = extract_vec3(&inputs, "vector")?;

        let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();

        if length < 1e-6 {
            return Err(ExecutionError {
                message: "Cannot normalize zero-length vector".to_string(),
                input_name: Some("vector".to_string()),
                recovery_hint: Some("Ensure vector has non-zero length".to_string()),
            });
        }

        let inv_len = 1.0 / length;
        let normalized = Vec3 {
            x: v.x * inv_len,
            y: v.y * inv_len,
            z: v.z * inv_len,
        };

        Ok(vec![
            ("normalized".to_string(), Value::Vec3Val(normalized)),
            ("length".to_string(), Value::F32Val(length)),
        ])
    }
}
```

---

### WASM Component: vec-dot

**Purpose**: Calculate dot product of two vectors

**Inputs**:
- `a` (vec3)
- `b` (vec3)

**Outputs**:
- `dot` (f32) - Dot product

**Uses**: Lighting calculations (N·L), projection, angle cosine

---

### WASM Component: vec-cross

**Purpose**: Calculate cross product of two vec3s

**Inputs**:
- `a` (vec3)
- `b` (vec3)

**Outputs**:
- `cross` (vec3) - Cross product (perpendicular vector)

**Uses**: Normal calculation, coordinate system construction

---

### WASM Component: mat4-construct

**Purpose**: Create a 4x4 matrix from 16 components or 4 column vectors

**Inputs** (Option A - Components):
- `m00` through `m33` (16 x f32)

**Inputs** (Option B - Columns):
- `col0`, `col1`, `col2`, `col3` (4 x vec4)

**Outputs**:
- `matrix` (mat4)

**Implementation**: Check which inputs are provided, construct accordingly

---

### WASM Component: mat4-multiply

**Purpose**: Multiply two 4x4 matrices

**Inputs**:
- `a` (mat4)
- `b` (mat4)

**Outputs**:
- `result` (mat4) - Product matrix

**Implementation**: Standard matrix multiplication

---

### WASM Component: color-rgb

**Purpose**: Create a color from RGB components (outputs vec3)

**Inputs**:
- `r` (f32, range 0-1)
- `g` (f32, range 0-1)
- `b` (f32, range 0-1)

**Outputs**:
- `color` (vec3)

**Validation**: Clamp values to [0, 1] range

---

### WASM Component: primitive-sphere

**Purpose**: Generate sphere mesh data

**Inputs**:
- `radius` (f32)
- `segments` (u32) - Horizontal segments
- `rings` (u32) - Vertical segments

**Outputs**:
- `vertices` (StringListVal) - JSON array of vec3 positions
- `normals` (StringListVal) - JSON array of vec3 normals
- `uvs` (StringListVal) - JSON array of vec2 UVs
- `indices` (U32ListVal) - Triangle indices

**Implementation**:
- Parametric sphere generation
- UV sphere algorithm
- Serialize geometry to JSON for transport

**Note**: In future phases, add native geometry data type

---

### WASM Component: primitive-cube

**Purpose**: Generate cube mesh data

**Inputs**:
- `size` (vec3) - Dimensions (x, y, z)

**Outputs**:
- `vertices` (StringListVal)
- `normals` (StringListVal)
- `uvs` (StringListVal)
- `indices` (U32ListVal)

---

### WASM Component: primitive-plane

**Purpose**: Generate plane mesh data

**Inputs**:
- `width` (f32)
- `height` (f32)
- `subdivisions` (u32)

**Outputs**:
- `vertices` (StringListVal)
- `normals` (StringListVal)
- `uvs` (StringListVal)
- `indices` (U32ListVal)

---

### WASM Component: perspective-camera

**Purpose**: Calculate view and projection matrices for perspective camera

**Inputs**:
- `position` (vec3) - Camera position
- `target` (vec3) - Look-at target
- `up` (vec3) - Up vector (default: 0,1,0)
- `fov` (f32) - Field of view in degrees
- `aspect_ratio` (f32) - Width / height
- `near` (f32) - Near clipping plane
- `far` (f32) - Far clipping plane

**Outputs**:
- `view_matrix` (mat4)
- `projection_matrix` (mat4)
- `camera_position` (vec3)
- `view_direction` (vec3)

**Implementation**:
- Look-at matrix construction
- Perspective projection matrix
- Use `glam` crate for matrix math

---

### WASM Component: render-target

**Purpose**: Configure render target parameters

**Inputs**:
- `width` (u32)
- `height` (u32)
- `format` (string) - "rgba8", "rgba32-float", etc.
- `depth` (bool) - Enable depth buffer
- `multisample` (u32) - MSAA samples (1, 2, 4, 8)

**Outputs**:
- `config` (StringVal) - JSON config for render system

**Implementation**: Serialize configuration to JSON

---

## Implementation Sequence

### Step 1: WIT Type System Extension (2-3 days)

**Tasks**:
1. Update `wit/wasmflow-node.wit` with graphics types
2. Update `src/graph/node.rs` NodeValue enum
3. Update `src/ui/component_view.rs` display logic
4. Add serde serialization for new types
5. Propagate to all component deps directories
6. Update existing code that pattern matches on Value
7. Test serialization/deserialization

**Deliverable**: Graphics types available throughout system

**Validation**:
- All existing tests pass
- New graphics types can be created and serialized
- UI displays graphics types correctly

---

### Step 2: GLSL Shader Editor Node (3-4 days)

**Tasks**:
1. Create `src/builtin/glsl_shader_editor.rs`
2. Extend `CodeEditorWidget` with GLSL syntax highlighting
3. Add `naga` dependency for GLSL validation
4. Implement shader templates
5. Add compilation state tracking
6. Create footer UI with editor and controls
7. Add to builtin registry
8. Test shader validation and output

**Deliverable**: Working GLSL editor node in palette

**Validation**:
- Can create shader editor node
- Can write and validate GLSL code
- Outputs correct shader source
- Errors display with line numbers

---

### Step 3: Vector Math Components (2-3 days)

**Tasks**:
1. Create component template in `components/graphics/.templates/`
2. Implement vec2/3/4-construct (3 components)
3. Implement vec2/3/4-add (3 components)
4. Implement vec-normalize
5. Implement vec-dot
6. Implement vec-cross
7. Write unit tests for each (4-6 tests per component)
8. Create category Justfile
9. Build and install all

**Deliverable**: 9 vector math components

**Validation**:
- All tests pass
- Components load in UI
- Can construct and manipulate vectors in graph

---

### Step 4: Matrix and Color Components (1-2 days)

**Tasks**:
1. Implement mat4-construct
2. Implement mat4-multiply
3. Implement color-rgb
4. Add unit tests
5. Build and install

**Deliverable**: 3 matrix/color components

**Validation**:
- Can create and multiply matrices
- Color values work correctly

---

### Step 5: Geometry Primitives (2-3 days)

**Tasks**:
1. Implement primitive-sphere with UV sphere algorithm
2. Implement primitive-cube
3. Implement primitive-plane
4. Add `glam` dependency for geometry calculations
5. Test mesh generation
6. Build and install

**Deliverable**: 3 geometry primitive components

**Validation**:
- Generates valid mesh data
- Vertex counts match expectations
- UVs and normals are correct

---

### Step 6: Camera Component (1-2 days)

**Tasks**:
1. Implement perspective-camera
2. Add `glam` dependency for matrix math
3. Test view/projection matrix calculations
4. Build and install

**Deliverable**: 1 camera component

**Validation**:
- Matrices are correct for different FOV/aspect ratios
- Look-at calculation works

---

### Step 7: Render Target Component (1 day)

**Tasks**:
1. Implement render-target
2. Define render config JSON schema
3. Test serialization
4. Build and install

**Deliverable**: 1 render target component

---

### Step 8: Shader Preview Node (3-4 days)

**Tasks**:
1. Create `src/builtin/shader_preview.rs`
2. Implement texture-data → egui::TextureHandle conversion
3. Create footer UI with image display
4. Add zoom/pan controls
5. Add to builtin registry
6. Test with dummy texture data

**Deliverable**: Working preview node

**Validation**:
- Can display texture data
- Zoom/pan works
- Updates on new input

---

### Step 9: Integration Testing (2-3 days)

**Tasks**:
1. Create integration test graph:
   - GLSL shader editor → shader source
   - Camera → view/projection matrices
   - Geometry primitive → mesh data
   - Render target → config
2. Document example workflows
3. Create getting started guide
4. Update `components/graphics/README.md`

**Deliverable**: Working end-to-end example

---

### Step 10: Documentation and Polish (1-2 days)

**Tasks**:
1. Update `CLAUDE.md` with graphics guidelines
2. Create `components/graphics/README.md`
3. Document all component APIs
4. Add troubleshooting guide
5. Create video/screenshot demos

**Deliverable**: Complete documentation

---

## Dependencies

### New Rust Crates

Add to workspace `Cargo.toml`:

```toml
[workspace.dependencies]
naga = "0.14"           # GLSL validation and SPIR-V compilation
glam = "0.25"           # Vector and matrix math (for components)
```

### Component Dependencies

All graphics components need:

```toml
[dependencies]
wit-bindgen = "0.30"
glam = "0.25"  # For math components
```

---

## Testing Strategy

### Unit Tests

Each component should have **minimum 4 tests**:
1. Typical use case
2. Edge case (zero, max values, etc.)
3. Error handling (invalid input)
4. Numerical precision

**Example** (vec-normalize):

```rust
#[test]
fn test_normalize_standard_vector() {
    let inputs = vec![
        ("vector".to_string(), Value::Vec3Val(Vec3 { x: 3.0, y: 4.0, z: 0.0 }))
    ];
    let result = Component::execute(inputs).unwrap();
    let normalized = extract_vec3_from_result(&result, "normalized");
    assert!((normalized.x - 0.6).abs() < 1e-6);
    assert!((normalized.y - 0.8).abs() < 1e-6);
}

#[test]
fn test_normalize_zero_vector_error() {
    let inputs = vec![
        ("vector".to_string(), Value::Vec3Val(Vec3 { x: 0.0, y: 0.0, z: 0.0 }))
    ];
    let result = Component::execute(inputs);
    assert!(result.is_err());
}
```

### Integration Tests

Create test graphs in `tests/component_tests/`:

1. **shader_editor_basic.json** - Create shader, validate output
2. **vector_math.json** - Construct, add, normalize vectors
3. **matrix_math.json** - Construct and multiply matrices
4. **geometry_generation.json** - Generate sphere mesh
5. **camera_setup.json** - Create camera, verify matrices

---

## Performance Targets

### Component Size
- Vector/matrix components: < 120KB (similar to math components)
- Geometry primitives: < 150KB (includes glam dependency)
- Built-in nodes: N/A (compiled into binary)

### Execution Time
- Vector operations: < 1ms
- Matrix operations: < 5ms
- Geometry generation (100 vertices): < 10ms
- Shader validation: < 100ms

### Memory
- Stack-based math operations (no heap allocation in hot paths)
- Geometry data: ~100 bytes per vertex (position + normal + UV)

---

## Migration Path (Future Phases)

### Phase 2: Rendering System
- WebGPU integration
- Actual shader compilation and execution
- Texture loading
- Light nodes

### Phase 3: PBR Materials
- Material property nodes
- Texture sampling
- BRDF calculations
- Environment maps

### Phase 4: Advanced Features
- Compute shaders
- Post-processing
- Custom render passes
- Performance optimization

---

## Success Criteria

Phase 1 is complete when:

- [ ] All 21 nodes implemented and tested
- [ ] Graphics types work throughout system
- [ ] GLSL editor can validate shaders
- [ ] Vector/matrix math works correctly
- [ ] Geometry primitives generate valid meshes
- [ ] Camera produces correct matrices
- [ ] Preview node can display placeholder images
- [ ] All unit tests pass (80+ tests)
- [ ] Integration tests demonstrate workflow
- [ ] Documentation complete
- [ ] No regressions in existing features

---

## Risk Mitigation

### Risk 1: Graphics Type System Complexity
**Impact**: High - Affects entire codebase
**Mitigation**:
- Implement incrementally
- Test each type addition thoroughly
- Keep backward compatibility

### Risk 2: Shader Validation Performance
**Impact**: Medium - Could slow down editor
**Mitigation**:
- Make validation async
- Cache validation results
- Debounce validation triggers

### Risk 3: Geometry Data Size
**Impact**: Medium - Large meshes could strain JSON serialization
**Mitigation**:
- Phase 1: Accept JSON serialization overhead
- Phase 2: Add binary geometry format
- Document mesh size recommendations

### Risk 4: Preview Without GPU
**Impact**: High - Can't test rendering without GPU access
**Mitigation**:
- Phase 1: Preview accepts texture-data but displays placeholder
- Phase 2: Add WebGPU integration for actual rendering
- Provide CPU-based fallback renderer

---

## Timeline Estimate

| Step | Duration | Dependencies |
|------|----------|--------------|
| 1. WIT Types | 2-3 days | None |
| 2. GLSL Editor | 3-4 days | Step 1 |
| 3. Vector Math | 2-3 days | Step 1 |
| 4. Matrix/Color | 1-2 days | Step 1, 3 |
| 5. Geometry | 2-3 days | Step 1, 4 |
| 6. Camera | 1-2 days | Step 1, 4 |
| 7. Render Target | 1 day | Step 1 |
| 8. Preview | 3-4 days | Step 1 |
| 9. Integration | 2-3 days | All above |
| 10. Docs | 1-2 days | All above |

**Total**: 18-27 days (3.5-5.5 weeks)

**Optimistic**: 18 days (~3.5 weeks)
**Realistic**: 22 days (~4.5 weeks)
**Pessimistic**: 27 days (~5.5 weeks)

---

## Next Steps

1. Review and approve this plan
2. Create OpenSpec proposal if formal spec needed
3. Start with Step 1: WIT Type System Extension
4. Implement steps sequentially
5. Review after each milestone

---

## Appendix A: Complete Node List

### Built-in Nodes (2)
1. `glsl-shader-editor` - Shader code editor
2. `shader-preview` - Render output display

### WASM Components - Math (12)
3. `vec2-construct` - Create vec2
4. `vec3-construct` - Create vec3
5. `vec4-construct` - Create vec4
6. `vec2-add` - Add vec2s
7. `vec3-add` - Add vec3s
8. `vec4-add` - Add vec4s
9. `vec-normalize` - Normalize vector
10. `vec-dot` - Dot product
11. `vec-cross` - Cross product
12. `mat4-construct` - Create mat4
13. `mat4-multiply` - Multiply matrices
14. `color-rgb` - Create color

### WASM Components - Geometry (3)
15. `primitive-sphere` - Generate sphere
16. `primitive-cube` - Generate cube
17. `primitive-plane` - Generate plane

### WASM Components - Camera (1)
18. `perspective-camera` - Camera matrices

### WASM Components - Config (1)
19. `render-target` - Render config

**Total**: 19 WASM components + 2 built-in = 21 nodes

---

## Appendix B: Example Workflow

**Goal**: Create a basic shader with a rotating sphere

**Graph Flow**:

```
1. [primitive-sphere]
   radius: 1.0, segments: 32, rings: 16
   → vertices, normals, uvs, indices

2. [vec3-construct]
   x: 0.0, y: 0.0, z: 5.0
   → camera_position

3. [vec3-construct]
   x: 0.0, y: 0.0, z: 0.0
   → look_at_target

4. [perspective-camera]
   position: camera_position
   target: look_at_target
   fov: 60.0
   aspect_ratio: 1.777
   → view_matrix, projection_matrix

5. [glsl-shader-editor] (Vertex Shader)
   template: "Basic Vertex"
   → vertex_shader_source

6. [glsl-shader-editor] (Fragment Shader)
   template: "Basic PBR"
   → fragment_shader_source

7. [render-target]
   width: 800, height: 600
   format: "rgba8"
   → render_config

8. [shader-preview]
   texture: (from renderer - Phase 2)
   → displays output
```

**Phase 1 Result**: All nodes connect, data flows correctly, preview shows placeholder
**Phase 2 Result**: Actual rendered sphere appears in preview

---

## Appendix C: Code Style Guidelines

### Graphics Components

Follow existing core library patterns:

```rust
// Standard structure
wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Component Name".to_string(),
            version: "1.0.0".to_string(),
            description: "Clear description".to_string(),
            author: "WasmFlow Graphics Library".to_string(),
            category: Some("Graphics".to_string()),  // ← Graphics category
        }
    }
    // ...
}

export!(Component);
```

### Helper Functions

Create shared helper module for graphics math:

```rust
// components/graphics/math/src/helpers.rs
pub fn extract_vec3(inputs: &[(String, Value)], name: &str)
    -> Result<Vec3, ExecutionError>
{
    let input = inputs.iter()
        .find(|(n, _)| n == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a vec3 value".to_string()),
        })?;

    match &input.1 {
        Value::Vec3Val(v) => Ok(v.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected vec3 for '{}', got {:?}", name, input.1),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a vec3 value".to_string()),
        })
    }
}
```

---

**End of Phase 1 Implementation Plan**
