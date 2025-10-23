# Implementation Plan: Rectangle Selection Tool for Node Composition

**Branch**: `007-rectangle-selection-tool` | **Date**: 2025-10-21 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-rectangle-selection-tool/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement a rectangle selection tool enabling users to select multiple nodes by click-dragging a rectangle on the canvas, then compose selected nodes into a single composite node using WAC (WebAssembly Composition). The composite node exposes aggregated inputs/outputs, displays component names in footer, and supports drill-down viewing of internal structure. Selected nodes must form a connected subgraph for composition. Original nodes are removed from main canvas but preserved internally for drill-down inspection.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)
**Primary Dependencies**: egui 0.33 (UI framework), eframe 0.33 (app framework), egui-snarl (node editor), wasmtime 27.0 (WASM runtime with component-model), petgraph 0.6 (graph algorithms), serde/bincode (serialization), WAC CLI (WebAssembly Composition)
**Storage**: Graph serialization via serde + bincode (BTreeMap for deterministic order), composite node internal structure persisted in graph JSON
**Testing**: cargo test (unit tests), integration tests for graph operations, UI interaction tests
**Target Platform**: Desktop (macOS, Linux, Windows) via native egui/eframe
**Project Type**: Single desktop application with node-based visual editor
**Performance Goals**: 60 FPS UI rendering, <100ms response to mouse movement for selection rectangle, <1 second for composition of up to 10 nodes
**Constraints**: Real-time visual feedback during drag operations, must maintain component isolation and security model, composition validation via graph connectivity analysis
**Scale/Scope**: Support selection and composition of up to 10 nodes in a single operation, graphs with 500+ nodes total

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Component-First Architecture
**Status**: ⚠️ PARTIAL - Requires Justification

**Analysis**: This feature is primarily UI/UX focused (rectangle selection, visual feedback, drill-down views) rather than a composable WebAssembly component. The composition logic itself will use WAC to create component compositions.

**Justification**: Rectangle selection is a core UI interaction pattern that must be integrated into the egui canvas rendering layer. While the *output* (composite nodes) are components, the selection tool itself is host application functionality. This is similar to other UI features like the canvas, palette, and theme system which are host-implemented.

**Compliance Path**:
- Composite nodes created by this feature MUST be proper WASM components with WIT interfaces
- The WAC composition output MUST follow component-model standards
- Selection/drill-down state is UI-only (not exposed as component interface)

### Principle II: Capability-Based Security
**Status**: ✅ PASS

**Analysis**: This feature operates entirely within the UI layer and does not introduce new security concerns. Composite nodes inherit the capability declarations of their constituent components.

**Compliance**:
- No new filesystem, network, or system access required
- Composite node capability declarations will be union of constituent node capabilities
- User consent flow remains unchanged (capabilities checked at execution time)

### Principle III: Typed Data Flow
**Status**: ✅ PASS

**Analysis**: Composite nodes must correctly aggregate and type-check inputs/outputs from constituent nodes.

**Compliance**:
- WAC composition will handle WIT interface aggregation
- Type validation occurs at composition time (fail fast if incompatible)
- Internal connections within composite node validated before external exposure
- UI must display type information for composite node ports

### Principle IV: Performance-First Execution
**Status**: ✅ PASS with Performance Targets

**Analysis**: Selection and composition operations must meet 60 FPS rendering requirement.

**Compliance**:
- Selection rectangle rendering: <100ms response time (per spec)
- Composition operation: <1 second for up to 10 nodes (per spec)
- Graph connectivity validation: O(V+E) using petgraph traversal
- Drill-down view: lazy rendering, only show internal nodes when requested
- No impact on existing graph execution performance

### Principle V: Developer Ergonomics
**Status**: ✅ PASS

**Analysis**: Feature enhances developer ergonomics by enabling component reuse through composition.

**Compliance**:
- Clear visual feedback during selection (reduces user errors)
- Error messages for composition failures (disconnected subgraph, <2 nodes selected)
- Drill-down view aids debugging of composite components
- Footer displays component list for transparency

### Principle VI: Composability & Modularity
**Status**: ✅ PASS - Core Feature Goal

**Analysis**: This feature directly implements the composability principle using WAC.

**Compliance**:
- WAC CLI integration for standards-compliant composition
- Composite nodes can be saved as part of graph (reusable subgraphs)
- Composition preserves internal structure for inspection via drill-down
- Nested composition explicitly marked as out-of-scope (future consideration)

### Gate Summary
**Overall Status**: ✅ PASS with Principle I Justification

**Required Actions**:
- Document that UI/UX features are host-implemented (not component-based)
- Ensure WAC composition output follows component-model standards
- Validate type safety at composition time
- Meet performance targets (<100ms selection, <1s composition)

---

## Post-Design Constitution Re-Evaluation

**Re-evaluated**: 2025-10-21 (after Phase 1 design)

### Principle I: Component-First Architecture
**Status**: ✅ PASS - Justified

**Design Validation**:
- Composite nodes ARE proper components (using wac-graph for standards-compliant composition)
- Selection UI is host-implemented (justified as necessary for 60 FPS)
- WIT interface defined for composite node introspection (contracts/composite-node.wit)
- WAC composition via `wac-graph` crate ensures component-model compliance

**No Design Changes Needed**: Justification remains valid

### Principle II: Capability-Based Security
**Status**: ✅ PASS

**Design Validation**:
- No new system access introduced
- CompositionMetadata structure defined (data-model.md)
- Composite nodes inherit constituent capabilities (documented in research.md)

**No Design Changes Needed**

### Principle III: Typed Data Flow
**Status**: ✅ PASS

**Design Validation**:
- PortMapping structure ensures type tracking (data-model.md)
- WAC composition validates type compatibility (research.md section 2)
- WIT interface defines port-definition record with type-signature field

**No Design Changes Needed**

### Principle IV: Performance-First Execution
**Status**: ✅ PASS

**Design Validation**:
- Async composition strategy defined for >5 components (research.md section 7)
- Composition caching strategy documented (LRU cache with hash-based keys)
- Spatial indexing plan for >100 nodes (quadtree)
- Performance targets explicitly tracked in data-model.md constraints

**No Design Changes Needed**

### Principle V: Developer Ergonomics
**Status**: ✅ PASS

**Design Validation**:
- Comprehensive error handling strategy (research.md section 4)
- User-friendly error messages designed (composition_error formatting)
- Quickstart guide created with 3-4 hour implementation timeline
- Clear visual feedback states defined (data-model.md)

**No Design Changes Needed**

### Principle VI: Composability & Modularity
**Status**: ✅ PASS

**Design Validation**:
- wac-graph integration fully researched and documented
- Composite node serialization with BTreeMap for deterministic order
- Drill-down view enables inspection of internal structure
- Component metadata preserved (composition-metadata record)

**No Design Changes Needed**

### Final Gate Status
**Overall**: ✅ PASS - All principles validated against design artifacts

**Design Quality**:
- All unknowns resolved in research.md
- Complete data model with validation rules
- WIT contract defined
- Implementation guide (quickstart.md) provided
- Constitution compliance maintained throughout design

## Project Structure

### Documentation (this feature)

```
specs/007-rectangle-selection-tool/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── composite-node.wit  # WIT interface for composite nodes
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── ui/                  # UI components (egui-based)
│   ├── canvas.rs        # Main canvas - ADD rectangle selection logic
│   ├── app.rs           # App state - ADD selection state, drill-down view state
│   ├── dialogs.rs       # Dialogs - ADD composition dialog
│   ├── theme.rs         # Theming - ADD composite node styling
│   └── selection.rs     # NEW: Rectangle selection interaction module
├── graph/               # Graph data structures
│   ├── node.rs          # Node types - ADD CompositeNode variant
│   ├── graph.rs         # Graph management - ADD composition methods
│   ├── validation.rs    # NEW: Connected subgraph validation
│   └── drill_down.rs    # NEW: Drill-down view context management
├── runtime/             # WASM runtime
│   └── wac_integration.rs  # NEW: WAC CLI integration for composition
├── builtin/             # Built-in nodes (no changes for this feature)
└── lib.rs               # Library exports

tests/
├── integration/
│   ├── selection_tests.rs      # NEW: Rectangle selection tests
│   ├── composition_tests.rs    # NEW: Node composition tests
│   └── drill_down_tests.rs     # NEW: Drill-down view tests
└── unit/
    ├── graph_validation_tests.rs  # NEW: Connectivity validation tests
    └── wac_integration_tests.rs   # NEW: WAC integration tests
```

**Structure Decision**: Single desktop application structure. This feature adds new modules to existing `src/ui/` (selection.rs), `src/graph/` (validation.rs, drill_down.rs), and `src/runtime/` (wac_integration.rs) directories. Major modifications to existing canvas.rs, app.rs, node.rs, and graph.rs files to support selection state, composite nodes, and drill-down views.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Principle I: Component-First (UI feature not component-based) | Rectangle selection is host application UI interaction, not a reusable WASM component | Making selection a component would add unnecessary overhead (WASM boundary crossing on every mouse event) and violate 60 FPS requirement. UI interactions are appropriately host-implemented per existing pattern (canvas, theme, dialogs). |
