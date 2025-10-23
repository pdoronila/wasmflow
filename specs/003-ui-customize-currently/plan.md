# Implementation Plan: Component-Driven Custom UI Views

**Branch**: `003-ui-customize-currently` | **Date**: 2025-10-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-ui-customize-currently/spec.md`

## Summary

Refactor the canvas footer rendering architecture to enable components to provide their own custom UI views. Currently, the canvas has hardcoded logic to detect and render custom views for specific node types (e.g., `http_fetch`). This feature introduces a trait-based interface pattern where components can optionally implement a `view()` method that the canvas calls to render footer content, enabling better code organization and colocation of component logic with its UI.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)
**Primary Dependencies**: egui 0.29 (UI framework), eframe 0.29 (app framework), egui-snarl 0.3 (node editor), wasmtime 27.0 (WASM runtime)
**Storage**: N/A (UI architecture refactoring only)
**Testing**: cargo test (unit and integration tests)
**Target Platform**: Desktop (native egui application)
**Project Type**: Single project (Rust desktop application)
**Performance Goals**: 60 FPS UI rendering, <100ms footer render time, supports 10+ different component types with custom views
**Constraints**: Must maintain existing egui-snarl integration, no breaking changes to graph serialization, component view rendering must not block UI thread
**Scale/Scope**: Refactor affects canvas rendering, component registry, and 1 existing custom view node (http_fetch)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Alignment with Core Principles

✅ **I. Component-First Architecture**
- Custom view interface will be optional, allowing components to provide UI without coupling to canvas
- Interface defined in component specification, maintaining clear separation
- No violations detected

✅ **II. Capability-Based Security (NON-NEGOTIABLE)**
- UI rendering does not affect security model
- Custom views render within existing egui context (no new system access)
- No violations detected

✅ **III. Typed Data Flow**
- UI change does not affect data flow or type checking
- Custom views may display typed data but do not modify type contracts
- No violations detected

✅ **IV. Performance-First Execution**
- Custom view rendering must not block main UI thread (addressed in design)
- Target: <100ms footer render time (documented in technical context)
- Edge case: Handle slow view rendering gracefully (documented in spec)
- No violations detected

✅ **V. Developer Ergonomics**
- This feature IMPROVES ergonomics by colocating component logic with UI
- Component developers define views in same module as component
- No violations detected

✅ **VI. Composability & Modularity**
- Custom views are optional extensions to component interface
- Does not affect component composition or reusability
- No violations detected

### Development Standards Compliance

✅ **Code Quality Gates**
- Unit tests required for trait implementation detection
- Integration tests required for canvas-component view interaction
- Test coverage target: >80% for new trait and canvas rendering logic

✅ **Dependency Policy**
- Uses existing egui ecosystem (no new dependencies)
- Leverages egui-snarl's existing footer rendering hooks

**Gate Result**: ✅ PASSED - No constitutional violations. Proceed to Phase 0.

---

## Post-Design Constitution Re-Check

*After completing Phase 1 design (research, data model, contracts, quickstart)*

### Alignment Verification

✅ **I. Component-First Architecture**
- Design uses trait-based interface (`ComponentFooterView`) maintaining component independence
- View logic colocated with component implementation (e.g., `HttpFetchFooterView` in `builtin/http_fetch.rs`)
- Optional implementation pattern preserved (not all components need custom views)
- **Compliant**: Design enhances component architecture without violations

✅ **II. Capability-Based Security (NON-NEGOTIABLE)**
- Custom views render within existing egui context (no new system access)
- Views are read-only (no state mutation beyond existing GraphNode mechanisms)
- No new capability requirements introduced
- **Compliant**: Security model unchanged

✅ **III. Typed Data Flow**
- Views display typed data from `node.outputs[].current_value` (existing type system)
- No changes to type checking or data flow validation
- **Compliant**: Type safety preserved

✅ **IV. Performance-First Execution**
- Design includes performance requirements (<50ms footer render, <100ms target documented)
- Error handling prevents blocking failures (Result-based API)
- Performance monitoring recommended (debug logging for slow views)
- Edge cases documented (slow rendering, large views)
- **Compliant**: Performance gates met

✅ **V. Developer Ergonomics**
- Quickstart guide provides clear examples for component developers
- Trait-based pattern is idiomatic Rust (familiar to developers)
- Migration path documented for existing http_fetch component
- **Compliant**: Ergonomics improved

✅ **VI. Composability & Modularity**
- Views are optional extensions (does not break existing components)
- Trait design allows future extensions (new methods with default implementations)
- Components remain composable (view is presentation-only, not business logic)
- **Compliant**: Modularity enhanced

### Development Standards Verification

✅ **Code Quality Gates**
- Unit test patterns documented in contract (test view trait, error handling)
- Integration test patterns documented (canvas-component interaction)
- Coverage target: >80% for trait and canvas rendering logic
- **Compliant**: Testing strategy defined

✅ **Dependency Policy**
- No new dependencies added (uses existing egui ecosystem)
- Leverages egui-snarl's existing footer hooks
- **Compliant**: Minimal dependency footprint

### Design Quality Assessment

**Strengths**:
1. Trait-based design follows Rust best practices
2. Clear separation of concerns (view vs component logic)
3. Backward compatible (existing components unchanged)
4. Well-documented with quickstart and contracts

**Risks Mitigated**:
1. Performance: Documented targets and monitoring
2. Error handling: Result-based API prevents crashes
3. State management: Read-only design aligns with egui immediate mode
4. Testing: Practical strategy given egui limitations

**Final Gate Result**: ✅ PASSED - Design complies with all constitutional principles and development standards. Ready for implementation (`/speckit.tasks`).

## Project Structure

### Documentation (this feature)

```
specs/003-ui-customize-currently/
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
├── graph/
│   └── node.rs              # ComponentSpec - add view trait/interface
├── ui/
│   ├── canvas.rs            # CanvasViewer - refactor footer rendering to call component views
│   └── component_view.rs    # NEW: View trait definition and helper types
└── builtin/
    └── http_fetch.rs        # Migrate existing custom view to new trait pattern

tests/
├── unit/
│   └── component_view_tests.rs  # NEW: Test view trait implementation detection
└── integration/
    └── canvas_view_tests.rs     # NEW: Test canvas-component view interaction
```

**Structure Decision**: Single project structure (existing wasmflow_cc crate). New module `src/ui/component_view.rs` for view trait definition. Refactor existing `src/ui/canvas.rs` footer rendering logic to use trait-based dispatch. Migrate `http_fetch` component to implement new trait.

## Complexity Tracking

*No constitutional violations requiring justification.*
