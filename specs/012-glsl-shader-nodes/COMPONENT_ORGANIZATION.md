# Graphics Component Organization Chart

**Phase 1**: Foundation Nodes
**Category**: Graphics
**Total**: 21 nodes

---

## Palette View

When users open the component palette, they'll see:

```
📁 Graphics (21 components)
   ├─ 🎨 Shader Authoring
   │   ├─ GLSL Shader Editor
   │   └─ Shader Preview
   │
   ├─ 📐 Vector Math
   │   ├─ Vec2 Construct
   │   ├─ Vec3 Construct
   │   ├─ Vec4 Construct
   │   ├─ Vec2 Add
   │   ├─ Vec3 Add
   │   ├─ Vec4 Add
   │   ├─ Vec Normalize
   │   ├─ Vec Dot
   │   └─ Vec Cross
   │
   ├─ 🔢 Matrix Math
   │   ├─ Mat4 Construct
   │   └─ Mat4 Multiply
   │
   ├─ 🎨 Color
   │   └─ Color RGB
   │
   ├─ 📦 Geometry Primitives
   │   ├─ Primitive Sphere
   │   ├─ Primitive Cube
   │   └─ Primitive Plane
   │
   ├─ 📷 Camera
   │   └─ Perspective Camera
   │
   └─ ⚙️ Render Config
       └─ Render Target
```

---

## Directory Structure

```
wasmflow/
│
├─ src/builtin/
│   ├─ glsl_shader_editor.rs        [NEW] GLSL code editor
│   └─ shader_preview.rs            [NEW] Render output display
│
├─ components/graphics/              [NEW] Graphics category
│   │
│   ├─ .templates/                   Component templates
│   │   ├─ component.wit
│   │   └─ component-with-ui.wit
│   │
│   ├─ math/                         Vector & matrix operations
│   │   ├─ vec2-construct/
│   │   │   ├─ Cargo.toml
│   │   │   ├─ Justfile
│   │   │   ├─ wit/node.wit
│   │   │   └─ src/lib.rs
│   │   ├─ vec3-construct/
│   │   ├─ vec4-construct/
│   │   ├─ vec2-add/
│   │   ├─ vec3-add/
│   │   ├─ vec4-add/
│   │   ├─ vec-normalize/
│   │   ├─ vec-dot/
│   │   ├─ vec-cross/
│   │   ├─ mat4-construct/
│   │   ├─ mat4-multiply/
│   │   ├─ color-rgb/
│   │   └─ Justfile                 Category build script
│   │
│   ├─ primitives/                   Geometry generators
│   │   ├─ primitive-sphere/
│   │   ├─ primitive-cube/
│   │   ├─ primitive-plane/
│   │   └─ Justfile
│   │
│   ├─ camera/                       Camera nodes
│   │   ├─ perspective-camera/
│   │   └─ Justfile
│   │
│   ├─ config/                       Render configuration
│   │   ├─ render-target/
│   │   └─ Justfile
│   │
│   ├─ Justfile                      Top-level graphics build
│   └─ README.md                     Graphics library docs
│
├─ wit/
│   └─ wasmflow-node.wit             [MODIFIED] Add graphics types
│
└─ specs/012-glsl-shader-nodes/
    ├─ README.md
    ├─ PHASE1_IMPLEMENTATION_PLAN.md
    ├─ PHASE1_NODE_REFERENCE.md
    └─ COMPONENT_ORGANIZATION.md      (this file)
```

---

## Node Type Distribution

### By Implementation Type

```
Built-in Nodes (2):         9.5%
├─ glsl-shader-editor
└─ shader-preview

WASM Components (19):       90.5%
├─ Vector Math (9):         42.8%
├─ Matrix Math (2):          9.5%
├─ Color (1):                4.8%
├─ Geometry (3):            14.3%
├─ Camera (1):               4.8%
└─ Config (1):               4.8%
```

### By Functional Category

```
🎨 Shader Tools (2):        9.5%
   ├─ GLSL Shader Editor    (built-in)
   └─ Shader Preview        (built-in)

📐 Math Operations (12):    57.1%
   ├─ Vector Construct (3)  (WASM)
   ├─ Vector Add (3)        (WASM)
   ├─ Vector Ops (3)        (WASM)
   ├─ Matrix (2)            (WASM)
   └─ Color (1)             (WASM)

📦 Scene Setup (7):         33.3%
   ├─ Geometry (3)          (WASM)
   ├─ Camera (1)            (WASM)
   └─ Config (1)            (WASM)
```

---

## Component Dependencies

### Dependency Graph

```
┌─────────────────────────────────────────────────┐
│  wit/wasmflow-node.wit (Graphics Types)         │
│  - vec2, vec3, vec4, mat4, texture-data        │
└─────────────────┬───────────────────────────────┘
                  │
        ┌─────────┴─────────┬──────────────┐
        ▼                   ▼              ▼
  ┌─────────────┐    ┌──────────────┐  ┌────────────┐
  │  Built-in   │    │ WASM Math    │  │ WASM Geo   │
  │   Nodes     │    │  Components  │  │ Components │
  └─────────────┘    └──────────────┘  └────────────┘
        │                   │                  │
        │                   │                  │
        ▼                   ▼                  ▼
  glsl-shader-     vec2/3/4-construct    primitive-*
  editor                   │                  │
  shader-preview    vec-normalize      (uses glam)
                    vec-dot/cross
                    mat4-*
                         │
                         ▼
                    (uses glam)
```

### External Dependencies

```rust
// Built-in nodes (in main Cargo.toml)
naga = "0.14"           // GLSL validation
egui                    // Already present
egui-extras             // Already present

// WASM math components
glam = "0.25"           // Vector/matrix math

// WASM geometry components
glam = "0.25"           // For geometry generation
```

---

## Component Size Estimates

```
Built-in Nodes:
├─ glsl-shader-editor    N/A (part of main binary)
└─ shader-preview        N/A (part of main binary)

WASM Components (with LTO + strip):
├─ Vector constructors   ~100 KB each
├─ Vector operations     ~105 KB each
├─ Matrix operations     ~110 KB each
├─ Color                 ~100 KB
├─ Geometry primitives   ~140 KB each (includes glam)
├─ Camera                ~135 KB (includes glam)
└─ Render target         ~100 KB

Total WASM binaries:     ~2.0 MB (19 components)
```

---

## Build Order

Recommended build sequence to minimize errors:

```
Step 1: Foundation
├─ Update wit/wasmflow-node.wit
├─ Update src/graph/node.rs
└─ Propagate to all component deps/

Step 2: Simple Math (no dependencies)
├─ vec2/3/4-construct
├─ vec2/3/4-add
└─ color-rgb

Step 3: Advanced Math (depends on Step 2)
├─ vec-normalize
├─ vec-dot
├─ vec-cross
├─ mat4-construct
└─ mat4-multiply

Step 4: Scene Components (depends on glam)
├─ primitive-sphere
├─ primitive-cube
├─ primitive-plane
└─ perspective-camera

Step 5: Configuration
└─ render-target

Step 6: Built-in Nodes
├─ glsl-shader-editor
└─ shader-preview
```

---

## Testing Hierarchy

```
Unit Tests (each component):
├─ math/vec2-construct          4 tests
├─ math/vec3-construct          4 tests
├─ math/vec4-construct          4 tests
├─ math/vec2-add                4 tests
├─ math/vec3-add                4 tests
├─ math/vec4-add                4 tests
├─ math/vec-normalize           6 tests
├─ math/vec-dot                 5 tests
├─ math/vec-cross               5 tests
├─ math/mat4-construct          6 tests
├─ math/mat4-multiply           5 tests
├─ math/color-rgb               4 tests
├─ primitives/primitive-sphere  6 tests
├─ primitives/primitive-cube    4 tests
├─ primitives/primitive-plane   5 tests
├─ camera/perspective-camera    6 tests
└─ config/render-target         4 tests

Total Unit Tests: ~81 tests

Integration Tests:
├─ shader_editor_basic.json
├─ vector_math.json
├─ matrix_math.json
├─ geometry_generation.json
└─ camera_setup.json

Total Integration Tests: 5 graphs
```

---

## Data Flow Examples

### Example 1: Simple Vector Math

```
Input Values                    Components                    Output
─────────────────────────────────────────────────────────────────────
x: 3.0, y: 4.0, z: 0.0  ──►  [vec3-construct]  ──►  vector: (3,4,0)
                                     │
                                     ▼
                              [vec-normalize]  ──►  normalized: (0.6, 0.8, 0)
                                     │              length: 5.0
                                     ▼
```

### Example 2: Camera Setup

```
Input Values                              Components                        Output
───────────────────────────────────────────────────────────────────────────────────
x:0, y:2, z:5       ──►  [vec3-construct]  ──┬──►  camera_pos: (0,2,5)
                                              │
x:0, y:0, z:0       ──►  [vec3-construct]  ──┼──►  target: (0,0,0)
                                              │
fov: 60                                       │
aspect: 1.777       ──────────────────────────┼──►  [perspective-camera]
near: 0.1                                     │         │
far: 1000.0        ───────────────────────────┘         ▼
                                                   view_matrix: mat4
                                                   projection_matrix: mat4
                                                   view_direction: (0,0,-1)
```

### Example 3: Geometry Generation

```
Input Values                              Components                    Output
─────────────────────────────────────────────────────────────────────────────────
radius: 1.0                                                        vertices: [...]
segments: 32        ──►  [primitive-sphere]  ──►  561 vertices   normals: [...]
rings: 16                                          1024 triangles  uvs: [...]
                                                                   indices: [...]
```

---

## Palette Icon Suggestions

Future enhancement - custom icons for each category:

```
🎨 Shader Authoring
   glsl-shader-editor     📝 (code/edit icon)
   shader-preview         👁️ (eye/preview icon)

📐 Vector Math
   vec*-construct         ➕ (plus/construct icon)
   vec*-add               ➕ (addition icon)
   vec-normalize          📏 (ruler/normalize icon)
   vec-dot                • (dot product symbol)
   vec-cross              ✖️ (cross product symbol)

🔢 Matrix Math
   mat4-construct         🔢 (grid/matrix icon)
   mat4-multiply          ✖️ (multiply icon)

🎨 Color
   color-rgb              🎨 (palette icon)

📦 Geometry
   primitive-sphere       ⚪ (sphere icon)
   primitive-cube         🟦 (cube icon)
   primitive-plane        ▭ (plane icon)

📷 Camera
   perspective-camera     📷 (camera icon)

⚙️ Config
   render-target          🎯 (target icon)
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-11-20 | Initial Phase 1 organization |

---

## Related Documentation

- [Phase 1 Implementation Plan](./PHASE1_IMPLEMENTATION_PLAN.md)
- [Phase 1 Node Reference](./PHASE1_NODE_REFERENCE.md)
- [Feature Overview](./README.md)
- [Core Library Patterns](../../components/LIBRARY.md)

---

**Status**: Planning
**Next**: Begin implementation with WIT type system extension
