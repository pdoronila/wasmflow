# GLSL Shader Nodes for WasmFlow

**Feature ID**: 012-glsl-shader-nodes
**Status**: Planning
**Category**: Graphics
**Priority**: New Feature

## Overview

This feature adds comprehensive support for authoring physically based shaders using GLSL within the WasmFlow visual node graph editor. Users will be able to create, edit, and preview GLSL shaders with a full suite of supporting nodes for vectors, matrices, geometry, cameras, lighting, and materials.

## Goals

1. Enable GLSL shader authoring directly in WasmFlow
2. Provide visual node-based workflow for PBR materials
3. Create foundation for real-time rendering in node graphs
4. Add new "Graphics" category to component palette

## Non-Goals (Phase 1)

- Full rendering engine (deferred to Phase 2)
- WebGPU/GPU integration (deferred to Phase 2)
- Texture loading from files (deferred to Phase 2)
- Advanced PBR features like IBL (deferred to Phase 4)

## Implementation Phases

### Phase 1: Foundation (This Document) ✓
**Timeline**: 3.5-5.5 weeks
**Deliverables**: 21 nodes (2 built-in + 19 WASM components)

**Scope**:
- WIT type system extensions (vec2, vec3, vec4, mat4, texture)
- GLSL shader editor node
- Vector and matrix math nodes (12 components)
- Geometry primitives (3 components)
- Camera node (1 component)
- Render target configuration (1 component)
- Shader preview node (placeholder rendering)

**See**: [PHASE1_IMPLEMENTATION_PLAN.md](./PHASE1_IMPLEMENTATION_PLAN.md)

### Phase 2: Rendering System (Future)
**Estimated Timeline**: 4-6 weeks

**Scope**:
- WebGPU integration
- Actual shader compilation and execution
- Texture loading nodes
- Basic lighting nodes
- Real-time preview rendering
- Shader program linker

### Phase 3: PBR Materials (Future)
**Estimated Timeline**: 3-4 weeks

**Scope**:
- PBR material property nodes
- Multiple light types (directional, point, spot)
- Texture sampling nodes
- Material texture maps
- Basic BRDF calculations

### Phase 4: Advanced Features (Future)
**Estimated Timeline**: 4-5 weeks

**Scope**:
- Environment maps and IBL
- Cubemap processing
- BRDF lookup tables
- Compute shaders
- Post-processing effects
- Performance optimizations

## Architecture

### Component Distribution

**Built-in Nodes** (in `src/builtin/`):
- Nodes requiring UI widgets, GPU access, or compilation
- Examples: `glsl-shader-editor`, `shader-preview`

**WASM Components** (in `components/graphics/`):
- Pure computation nodes (math, geometry, configuration)
- Examples: Vector math, matrix operations, geometry primitives

### Directory Structure

```
components/graphics/
├── math/              # Vector/matrix operations
├── primitives/        # Geometry primitives
├── camera/            # Camera nodes
├── config/            # Render configuration
└── (future: lighting, materials, textures)

src/builtin/
├── glsl_shader_editor.rs
└── shader_preview.rs
```

### WIT Type Extensions

New graphics primitive types added to `wit/wasmflow-node.wit`:
- `vec2`, `vec3`, `vec4` - Vector types
- `mat4` - 4x4 matrix
- `texture-data` - Texture pixel data
- `texture-format` - Texture format enum

## Dependencies

### New Crate Dependencies
- `naga` - GLSL validation and SPIR-V compilation
- `glam` - Vector and matrix math for components
- (Phase 2): `wgpu` - WebGPU rendering

## Use Cases

### Example: Basic PBR Shader
1. Create geometry with `primitive-sphere`
2. Set up camera with `perspective-camera`
3. Write vertex shader in `glsl-shader-editor`
4. Write PBR fragment shader in `glsl-shader-editor`
5. Configure render target with `render-target`
6. Preview output in `shader-preview`

### Example: Custom Effect Shader
1. Load texture (Phase 2)
2. Write fragment shader with custom effects
3. Apply to geometry
4. Real-time preview

## Success Metrics

### Phase 1
- All 21 nodes implemented and tested
- 80+ unit tests passing
- Integration tests demonstrate workflow
- GLSL editor validates shaders correctly
- Graphics types work throughout system

### Phase 2
- Shaders compile and execute on GPU
- Preview shows actual rendered output
- 30+ FPS preview performance target

### Future Phases
- Full PBR material system
- Multiple light sources
- Environment mapping
- Production-ready shader authoring

## Documentation

- [Phase 1 Implementation Plan](./PHASE1_IMPLEMENTATION_PLAN.md) - Detailed implementation guide
- (Future): API Reference
- (Future): Getting Started Guide
- (Future): Shader Examples Gallery

## Related Features

- `001-webassembly-based-node` - WASM component architecture
- `010-wasm-components-core` - Core component library patterns
- `005-create-wasm-component` - WASM Creator Node (similar to GLSL editor)

## Timeline Summary

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: Foundation | 3.5-5.5 weeks | Planning |
| Phase 2: Rendering | 4-6 weeks | Not Started |
| Phase 3: PBR Materials | 3-4 weeks | Not Started |
| Phase 4: Advanced | 4-5 weeks | Not Started |

**Total Estimated**: 15-20.5 weeks for complete feature

## Next Steps

1. Review Phase 1 plan
2. Approve implementation approach
3. Begin WIT type system extension
4. Implement nodes sequentially per plan
5. Create integration tests
6. Document and polish

## Questions & Decisions

### Open Questions
- GPU API choice: WebGPU vs platform-specific? → **Decision: WebGPU (Phase 2)**
- Texture format support in Phase 1? → **Decision: Basic formats only**
- Native geometry type vs JSON serialization? → **Decision: JSON in Phase 1, native in Phase 2**

### Key Decisions
- **Category Name**: "Graphics" (vs "Shader", "Rendering", "3D")
- **Editor Style**: Code editor (vs visual shader graph) for GLSL
- **Math Library**: `glam` (established, widely used)
- **Validation**: `naga` (official WebGPU shader validator)

## Contact

For questions about this feature:
- Review implementation plan: `PHASE1_IMPLEMENTATION_PLAN.md`
- Check project documentation: `/CLAUDE.md`
- Refer to core library patterns: `components/LIBRARY.md`
