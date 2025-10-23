# Implementation Plan: Four-Section Node Layout

**Branch**: `004-node-input-update` | **Date**: 2025-10-16 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/004-node-input-update/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Update the visual node editor from a three-section layout (header, body, footer) to a four-section layout (header, connections, body, footer). The connections section will display input/output pins with type information separately from the body. Both body and footer sections will support default views (auto-generated input fields and dynamic status) that can be overridden by component-provided custom views.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)
**Primary Dependencies**: egui 0.29 (UI), eframe 0.29 (app framework), egui-snarl 0.3 (node editor)
**Storage**: Graph serialization via serde + bincode (BTreeMap for deterministic order)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Desktop (Linux, macOS, Windows) via eframe native backend
**Project Type**: Single desktop application
**Performance Goals**: 60 FPS UI rendering, support 500+ node graphs without degradation
**Constraints**: <50ms per-frame rendering, <10ms per component instantiation
**Scale/Scope**: ~10 source files affected (ui/canvas.rs, ui/component_view.rs, graph/node.rs, builtin views)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Component-First Architecture ✅ PASS
- **Assessment**: This feature is a UI/UX enhancement to the node rendering system, not a new component type
- **Impact**: Improves visual presentation of existing component interfaces (input/output ports)
- **Compliance**: No changes to WIT interfaces or component model - this is purely a UI layer change

### Capability-Based Security ✅ PASS
- **Assessment**: No security implications - this feature only affects visual layout
- **Impact**: Does not modify capability system, sandbox levels, or permission flows
- **Compliance**: Security model remains unchanged

### Typed Data Flow ✅ PASS
- **Assessment**: Enhances visibility of type information by dedicating a section to connections
- **Impact**: Improves user experience with type-safe connections by making types more visible
- **Compliance**: Does not modify type checking logic - only presents existing type info more clearly

### Performance-First Execution ✅ PASS
- **Assessment**: Must maintain 60 FPS rendering with new four-section layout
- **Risk**: Additional section adds rendering overhead - need to measure impact
- **Mitigation**: Leverage existing egui immediate-mode rendering; use caching for default view generation
- **Compliance**: Performance target verified in testing (see Performance Goals in Technical Context)

### Developer Ergonomics ✅ PASS WITH ENHANCEMENT
- **Assessment**: Improves component developer experience with default views
- **Enhancement**: Components get default body/footer views automatically, reducing boilerplate
- **API Extension**: Adds ComponentBodyView trait (parallel to existing ComponentFooterView)
- **Compliance**: Backward compatible - existing components work without changes

### Composability & Modularity ✅ PASS
- **Assessment**: No impact on component composition or WAC integration
- **Impact**: Visual presentation only - does not affect component boundaries or interfaces
- **Compliance**: Modularity unchanged

### Overall Status: ✅ PASSED
No constitution violations. Feature is a UI enhancement that improves component interface visibility and developer ergonomics while maintaining all core principles.

---

## Constitution Check (Post-Design Re-Evaluation)

*Re-evaluated after Phase 1 (Design & Contracts) completion*

### Component-First Architecture ✅ PASS (Confirmed)
- **Design Impact**: ComponentBodyView trait mirrors existing ComponentFooterView pattern
- **Compliance**: No changes to WIT interfaces or WASM component model
- **Enhancement**: Default views reduce boilerplate for component developers

### Capability-Based Security ✅ PASS (Confirmed)
- **Design Impact**: No security-related changes in design artifacts
- **Compliance**: Capability system untouched

### Typed Data Flow ✅ PASS (Confirmed)
- **Design Impact**: Connections section dedicated to displaying type information improves type visibility
- **Compliance**: Type checking logic unchanged (data-model.md confirms Port structure preserved)

### Performance-First Execution ✅ PASS WITH MITIGATION
- **Design Impact**: Added performance constraints in contracts:
  - Default body: <10ms for 20 fields (hard limit)
  - Default footer: <5ms for 10 outputs (hard limit)
  - Custom views: <50ms render time (monitored + logged)
- **Mitigation Implemented**: Field count limits + performance logging in contracts
- **Compliance**: 60 FPS target maintained (15ms per node worst case < 16.67ms frame budget)

### Developer Ergonomics ✅ PASS WITH ENHANCEMENT (Confirmed)
- **Design Impact**: ComponentBodyView trait added (see contracts/component-body-view.md)
- **API Consistency**: Mirrors ComponentFooterView pattern exactly
- **Backward Compatibility**: Existing components work without changes (see migration in quickstart.md)
- **Compliance**: Reduces boilerplate, improves developer experience

### Composability & Modularity ✅ PASS (Confirmed)
- **Design Impact**: No changes to component composition or WAC integration
- **Compliance**: UI-only changes, component boundaries unchanged

### Overall Status (Post-Design): ✅ PASSED
All design artifacts reviewed. No constitution violations detected. Feature ready for implementation.

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── ui/
│   ├── canvas.rs              # SnarlViewer implementation - UPDATE for 4 sections
│   ├── component_view.rs      # ComponentFooterView trait - ADD ComponentBodyView
│   └── wit_ui_renderer.rs     # WASM component UI rendering - UPDATE for default views
├── graph/
│   └── node.rs                # Node/Port/ComponentSpec - minimal changes
├── builtin/
│   ├── math.rs                # Built-in math components - UPDATE for custom views
│   ├── constants.rs           # Constant nodes - UPDATE for custom body views
│   └── views.rs               # Custom view implementations - ADD new examples
└── runtime/
    └── engine.rs              # Execution engine - NO CHANGES (performance monitoring only)

tests/
├── unit/
│   └── component_view_tests.rs  # Unit tests for view traits - ADD body view tests
└── integration/
    └── canvas_view_tests.rs     # Integration tests for rendering - UPDATE for 4 sections
```

**Structure Decision**: Single desktop application following existing WasmFlow architecture. Changes are isolated to UI layer (src/ui/) with updates to builtin component views (src/builtin/) to demonstrate the new four-section pattern. No changes to graph execution engine or storage layer.

## Complexity Tracking

*No violations detected in Constitution Check - this section is empty.*


---

## Phase 0 & 1 Completion Summary

### Phase 0: Research (✅ Complete)

**Artifacts Generated**:
- `research.md` - Resolved all technical unknowns

**Key Decisions**:
1. **egui-snarl Integration**: Use existing pin rendering area as "connections section" (no fork needed)
2. **Default View Generation**: Immediate-mode pattern with type-driven widget selection
3. **Performance Target**: <15ms per node (within 60 FPS budget for 500-node graphs)
4. **Migration Strategy**: Zero-migration, fully backward compatible

**Technologies Confirmed**:
- egui 0.29 (immediate-mode GUI)
- egui-snarl 0.3 (node editor with SnarlViewer extensibility)
- serde (no serialization changes needed)

---

### Phase 1: Design & Contracts (✅ Complete)

**Artifacts Generated**:
- `data-model.md` - UI entity design (ComponentBodyView trait, default view helpers)
- `contracts/component-body-view.md` - ComponentBodyView trait specification
- `contracts/default-views.md` - Default body/footer view specifications
- `quickstart.md` - Developer guide with examples

**Design Highlights**:

1. **ComponentBodyView Trait** (NEW):
   - Mirrors existing `ComponentFooterView` pattern
   - Interface: `fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String>`
   - Performance: <50ms render time (monitored + logged)

2. **Default Views** (NEW):
   - **Body**: Auto-generate input fields for simple types (U32, I32, F32, String)
   - **Footer**: Auto-display output values formatted by type
   - **Limits**: 20 body fields, 10 footer outputs (performance bounds)

3. **Data Model Changes** (MINIMAL):
   - `ComponentSpec` gains optional `body_view` field (parallel to `footer_view`)
   - No changes to `GraphNode`, `Port`, or serialization format

4. **Backward Compatibility** (100%):
   - Existing components without views: Get default views automatically
   - Existing components with custom footer: Keep custom footer, get default body
   - Constant nodes: Migrate to `ComponentBodyView` (refactor, not breaking change)

**Performance Budgets Established**:
- Default body rendering: <10ms for 20 fields
- Default footer rendering: <5ms for 10 outputs
- Custom view rendering: <50ms (hard limit)
- Total per-node worst case: <15ms (under 16.67ms frame budget at 60 FPS)

**Constitution Re-Check**: ✅ PASSED (all principles confirmed)

---

### Ready for Phase 2: Tasks

The `/speckit.plan` command stops here. Next step:

```bash
/speckit.tasks
```

This will generate `tasks.md` with actionable, dependency-ordered implementation tasks based on the design artifacts created in Phase 0 & 1.

**Implementation Preview** (from quickstart.md):
1. Add `ComponentBodyView` trait to `src/ui/component_view.rs`
2. Implement `DefaultBodyView` and `DefaultFooterView` in `src/ui/canvas.rs`
3. Update `ComponentSpec` with `body_view` field in `src/graph/node.rs`
4. Update `CanvasViewer::show_body()` to use default/custom body views
5. Update `CanvasViewer::show_footer()` to use default/custom footer views
6. Migrate constant nodes to `ComponentBodyView` in `src/builtin/constants.rs`
7. Add tests for body view trait and default view rendering
8. Update documentation and examples

**Estimated Scope**: ~10 files, ~500 LOC changes (mostly UI layer)

