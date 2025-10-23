# Research: Component-Driven Custom UI Views

**Feature**: Component-Driven Custom UI Views
**Branch**: `003-ui-customize-currently`
**Date**: 2025-10-15

## Overview

This document captures research findings and architectural decisions for implementing a trait-based custom view interface that enables components to provide their own footer UI rendering logic.

## Research Questions & Findings

### 1. Rust Trait Design Pattern for Optional UI Views

**Question**: How should we design a trait that components can optionally implement to provide custom UI views in an egui context?

**Research Findings**:

Based on Rust trait best practices and egui patterns:

1. **Trait Definition Approach**:
   - Use a trait with a single method `render_footer(&self, ui: &mut egui::Ui, node_data: &GraphNode)`
   - Make the trait optional by not requiring all ComponentSpec to implement it
   - Use trait objects (`Box<dyn ComponentFooterView>`) for dynamic dispatch

2. **Alternative Considered: Enum-based dispatch**:
   - Could use an enum variant in ComponentSpec with footer rendering logic
   - Rejected because it couples component definition with UI code
   - Trait-based approach allows better separation of concerns

3. **Integration with egui-snarl**:
   - egui-snarl's `SnarlViewer` trait already has `has_footer()` and `show_footer()` methods
   - We keep these methods in CanvasViewer but delegate to component trait implementation
   - Canvas acts as orchestrator, components provide implementation

**Decision**: Use a trait-based approach with dynamic dispatch. Define `ComponentFooterView` trait in new `src/ui/component_view.rs` module.

**Rationale**:
- Maintains separation of concerns (component logic vs UI)
- Allows optional implementation (not all components need custom views)
- Follows Rust's composition-over-inheritance philosophy
- Enables colocation of view logic with component without coupling

### 2. Component Registry Integration Pattern

**Question**: How should ComponentRegistry store and retrieve optional footer view implementations?

**Research Findings**:

1. **Storage Options**:
   - **Option A**: Store `Option<Box<dyn ComponentFooterView>>` in ComponentSpec
     - Pros: Direct association, simple lookup
     - Cons: Requires ComponentSpec to be mutable or use Arc<dyn>

   - **Option B**: Separate registry HashMap<String, Box<dyn ComponentFooterView>>
     - Pros: Clean separation, easier to manage lifetimes
     - Cons: Two lookups required (component_id → spec, component_id → view)

   - **Option C**: Add optional view factory closure to ComponentSpec
     - Pros: Lazy creation, no lifetime issues
     - Cons: More complex API, harder to test

**Decision**: Use Option A with `Option<Arc<dyn ComponentFooterView>>` stored in ComponentSpec.

**Rationale**:
- Arc enables shared ownership without lifetime complexity
- Direct association makes API intuitive (spec.get_footer_view())
- Minimal overhead (Arc is small, view implementations are lightweight)
- Easy to test (can create test components with mock views)

**Alternatives Considered**: Separate registry rejected due to added complexity of maintaining two parallel data structures.

### 3. Error Handling for View Rendering Failures

**Question**: How should the system handle errors or panics during custom view rendering?

**Research Findings**:

1. **egui Error Handling Patterns**:
   - egui is immediate mode - rendering happens every frame
   - Panics during rendering can crash the entire application
   - Best practice: catch panics using `std::panic::catch_unwind` if view code is untrusted

2. **Rust Panic Safety**:
   - Trait method should return `Result<(), ViewError>` instead of panicking
   - Canvas can display error message in footer if rendering fails
   - Log errors for debugging without crashing app

3. **Performance Monitoring**:
   - Use `std::time::Instant` to measure render time
   - Warn if footer rendering exceeds threshold (e.g., 50ms)
   - Display warning indicator in UI if component view is slow

**Decision**:
- Trait method returns `Result<(), String>` (String error for simplicity)
- Canvas catches errors and displays "⚠️ View render failed: {error}" in footer
- Add performance logging (debug level) for render times >50ms

**Rationale**:
- Prevents single misbehaving component from crashing application
- Provides user feedback when view fails
- Helps developers debug slow view implementations

**Alternatives Considered**: Panic catching rejected as too defensive for trusted builtin components (adds complexity without value for current use case).

### 4. View State Management and Reactivity

**Question**: How should custom views handle component state changes and reactive updates?

**Research Findings**:

1. **egui Immediate Mode Paradigm**:
   - egui redraws UI every frame based on current state
   - No need for explicit update notifications - views automatically reflect current state
   - Component state lives in GraphNode, view reads it on each render

2. **State Mutation Patterns**:
   - Views should be read-only (render based on current state)
   - If view needs to mutate state (e.g., editable controls), pass mutable reference
   - Use trait method signature: `render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode)`

3. **Performance Implications**:
   - Reading state every frame is fast (no network/disk I/O)
   - State is in-memory (GraphNode structure)
   - egui's diff-based rendering minimizes actual GPU work

**Decision**:
- Views receive `&GraphNode` (immutable) for rendering
- If component needs editable controls, canvas provides `&mut GraphNode` access
- No explicit state synchronization needed (immediate mode handles it)

**Rationale**:
- Aligns with egui's immediate mode architecture
- Simplifies implementation (no state syncing code)
- Performance adequate for desktop application (<100 nodes typical)

**Alternatives Considered**: Event-based update notifications rejected as over-engineering for immediate mode UI.

### 5. Testing Strategy for Dynamic View Trait

**Question**: How can we effectively test trait-based dynamic dispatch for view rendering?

**Research Findings**:

1. **Unit Testing Approaches**:
   - Create mock component with test view implementation
   - Verify trait detection logic (has_footer returns true)
   - Test error handling with failing view implementation

2. **Integration Testing**:
   - Render test components in headless egui context
   - Verify UI output contains expected elements (requires egui_test_harness or manual inspection)
   - Test performance (measure render time)

3. **egui Testing Limitations**:
   - egui doesn't have built-in snapshot testing
   - Visual regression testing requires manual verification or custom tooling
   - Best practice: test logic separately from rendering

**Decision**:
- Unit tests: Mock ComponentFooterView implementation, test registration and lookup
- Integration tests: Create test component, verify CanvasViewer calls render_footer
- Manual testing: Visual verification of footer rendering for http_fetch migration

**Rationale**:
- Balances test coverage with implementation effort
- Focuses on testable logic (trait detection, error handling)
- Acknowledges limitations of testing UI rendering

**Alternatives Considered**: Snapshot testing rejected due to lack of egui tooling support.

## Architectural Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| View Interface Pattern | Trait with dynamic dispatch (`Arc<dyn ComponentFooterView>`) | Separation of concerns, optional implementation, follows Rust patterns |
| Registry Integration | Store `Option<Arc<dyn ComponentFooterView>>` in ComponentSpec | Direct association, simple API, minimal overhead |
| Error Handling | Return `Result<(), String>`, display errors in footer | Prevents crashes, provides user feedback |
| State Management | Immediate mode (no explicit updates), pass `&GraphNode` | Aligns with egui, simpler implementation |
| Testing Strategy | Unit + integration tests for logic, manual for visuals | Practical given egui limitations |

## Implementation Notes

1. **Module Organization**:
   - New module: `src/ui/component_view.rs` for trait definition
   - Keep view implementations colocated with component logic (e.g., in `builtin/http_fetch.rs`)

2. **Migration Path**:
   - Refactor existing hardcoded footer logic in `canvas.rs` lines 602-667
   - Extract http_fetch footer rendering to trait implementation
   - Update ComponentSpec creation for http_fetch to include view

3. **Future Extensibility**:
   - Trait design allows adding more methods later (e.g., `render_header`, `render_sidebar`)
   - Can extend to support multiple view modes (compact/detailed)
   - Foundation for plugin system (user-defined components with custom views)

## References

- egui documentation: https://docs.rs/egui/latest/egui/
- egui-snarl SnarlViewer trait: https://docs.rs/egui-snarl/latest/egui_snarl/ui/trait.SnarlViewer.html
- Rust trait objects: https://doc.rust-lang.org/book/ch17-02-trait-objects.html

## Unresolved Questions

None - all technical unknowns have been resolved through this research phase.
