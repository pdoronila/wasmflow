# Tasks: Component-Driven Custom UI Views

**Input**: Design documents from `/specs/003-ui-customize-currently/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are included per constitution requirement (>80% coverage for new trait and canvas rendering logic)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- Single project: `src/`, `tests/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and module structure for new component view system

- [ ] T001 Create new module `src/ui/component_view.rs` for ComponentFooterView trait definition
- [ ] T002 Add `pub mod component_view;` to `src/ui/mod.rs` to expose the new module
- [ ] T003 [P] Create test directories: `tests/unit/` and `tests/integration/` if they don't exist

**Checkpoint**: Module structure ready for trait definition

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define core trait and extend ComponentSpec - MUST complete before ANY user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Define `ComponentFooterView` trait in `src/ui/component_view.rs` with `render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String>` method (include Send + Sync bounds)
- [ ] T005 Add `footer_view: Option<Arc<dyn ComponentFooterView>>` field to `ComponentSpec` struct in `src/graph/node.rs` with `#[serde(skip)]` attribute
- [ ] T006 Add `with_footer_view(mut self, view: Arc<dyn ComponentFooterView>) -> Self` builder method to `ComponentSpec` in `src/graph/node.rs`
- [ ] T007 Add `has_footer_view(&self) -> bool` helper method to `ComponentSpec` in `src/graph/node.rs`
- [ ] T008 Add `get_footer_view(&self) -> Option<&Arc<dyn ComponentFooterView>>` helper method to `ComponentSpec` in `src/graph/node.rs`
- [ ] T009 Update `ComponentSpec::new_builtin()` and `ComponentSpec::new_user_defined()` constructors to initialize `footer_view: None` in `src/graph/node.rs`

**Checkpoint**: Foundation ready - ComponentFooterView trait defined, ComponentSpec extended. User story implementation can now begin.

---

## Phase 3: User Story 1 - Component Provides Custom Footer View (Priority: P1) 🎯 MVP

**Goal**: Enable components to provide custom footer views via trait implementation, replacing hardcoded canvas logic

**Independent Test**: Create a single component with custom view, select it on canvas, verify footer renders via trait dispatch

### Tests for User Story 1

**NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T010 [P] [US1] Unit test: Create mock `TestFooterView` implementing `ComponentFooterView` in `tests/unit/component_view_tests.rs`
- [ ] T011 [P] [US1] Unit test: Verify `ComponentSpec.has_footer_view()` returns true when view is set in `tests/unit/component_view_tests.rs`
- [ ] T012 [P] [US1] Unit test: Verify `ComponentSpec.get_footer_view()` returns Some when view is set in `tests/unit/component_view_tests.rs`
- [ ] T013 [P] [US1] Unit test: Verify trait can be stored as `Arc<dyn ComponentFooterView>` in `tests/unit/component_view_tests.rs`
- [ ] T014 [US1] Integration test: Create test component with custom view, verify `CanvasViewer::has_footer()` detects it in `tests/integration/canvas_view_tests.rs`
- [ ] T015 [US1] Integration test: Verify `CanvasViewer::show_footer()` calls `view.render_footer()` in `tests/integration/canvas_view_tests.rs`

### Implementation for User Story 1

- [ ] T016 [US1] Refactor `CanvasViewer::has_footer()` in `src/ui/canvas.rs` (line ~602): Replace hardcoded `component_id.contains("http_fetch")` check with `self.registry.get_by_id(&node.component_id).and_then(|spec| spec.get_footer_view()).is_some()`
- [ ] T017 [US1] Refactor `CanvasViewer::show_footer()` in `src/ui/canvas.rs` (lines ~607-667): Replace hardcoded HTTP fetch rendering with trait-based dispatch: `if let Some(spec) = self.registry.get_by_id(&node_data.component_id) { if let Some(view) = spec.get_footer_view() { match view.render_footer(ui, graph_node) { ... } } }`
- [ ] T018 [US1] Add error handling in `CanvasViewer::show_footer()` in `src/ui/canvas.rs`: Display `ui.colored_label(egui::Color32::RED, "⚠️ View render failed")` and error message when `render_footer()` returns `Err`
- [ ] T019 [US1] Keep separator and width constraint logic (250px max) in `CanvasViewer::show_footer()` in `src/ui/canvas.rs`
- [ ] T020 [US1] Create `HttpFetchFooterView` struct in `src/builtin/http_fetch.rs` (or appropriate http_fetch component file)
- [ ] T021 [US1] Implement `ComponentFooterView` trait for `HttpFetchFooterView` in `src/builtin/http_fetch.rs`: Extract existing footer rendering logic from canvas.rs (Grid with output display, value truncation at 100 chars)
- [ ] T022 [US1] Update http_fetch component registration to use `.with_footer_view(Arc::new(HttpFetchFooterView))` in `src/builtin/http_fetch.rs` (or component registry location)
- [ ] T023 [US1] Verify http_fetch node still renders footer correctly after refactoring (manual test: select http_fetch node, check footer)

**Checkpoint**: User Story 1 complete - Components can provide custom footer views, http_fetch migrated to new pattern, tests pass

---

## Phase 4: User Story 2 - Multiple Components with Different Custom Views (Priority: P2)

**Goal**: Validate that multiple different component types can each provide unique custom views and switch correctly on selection

**Independent Test**: Create 2-3 test components with distinct views, select each in turn, verify correct view displays

### Tests for User Story 2

- [ ] T024 [P] [US2] Integration test: Create 2 different test components with distinct views in `tests/integration/canvas_view_tests.rs`
- [ ] T025 [US2] Integration test: Add both components to canvas, verify selecting each shows correct footer in `tests/integration/canvas_view_tests.rs`
- [ ] T026 [US2] Integration test: Verify switching selection updates footer to show new component's view in `tests/integration/canvas_view_tests.rs`
- [ ] T027 [US2] Integration test: Verify deselecting node clears footer or shows default content in `tests/integration/canvas_view_tests.rs`

### Implementation for User Story 2

- [ ] T028 [P] [US2] Create second example view: `ConstantNodeFooterView` struct in `src/builtin/constants.rs` (displays editable constant value)
- [ ] T029 [US2] Implement `ComponentFooterView` for `ConstantNodeFooterView` in `src/builtin/constants.rs`: Show current value in footer (read-only for P2)
- [ ] T030 [US2] Update constant node component registration to use `.with_footer_view(Arc::new(ConstantNodeFooterView))` in `src/builtin/constants.rs`
- [ ] T031 [P] [US2] Create third example view: `MathNodeFooterView` struct in `src/builtin/math.rs` (displays operation result summary)
- [ ] T032 [US2] Implement `ComponentFooterView` for `MathNodeFooterView` in `src/builtin/math.rs`: Show inputs and result in footer
- [ ] T033 [US2] Update math node component registration to use `.with_footer_view(Arc::new(MathNodeFooterView))` in `src/builtin/math.rs`
- [ ] T034 [US2] Verify canvas correctly switches between different component views on selection change (manual test: select http_fetch → constant → math nodes, verify footer updates)

**Checkpoint**: User Story 2 complete - Multiple component types provide unique custom views, selection switching works correctly

---

## Phase 5: User Story 3 - Component Updates Reflected in Custom View (Priority: P3)

**Goal**: Ensure custom views automatically reflect component state changes (egui immediate mode reactivity)

**Independent Test**: Create component with mutable state, modify state, verify footer view updates automatically

### Tests for User Story 3

- [ ] T035 [P] [US3] Integration test: Create test component with mutable output value in `tests/integration/canvas_view_tests.rs`
- [ ] T036 [US3] Integration test: Modify component state (e.g., change output value), verify footer reflects change in `tests/integration/canvas_view_tests.rs`
- [ ] T037 [US3] Integration test: Test rapid updates (e.g., counter incrementing) do not degrade performance (<100ms render) in `tests/integration/canvas_view_tests.rs`

### Implementation for User Story 3

- [ ] T038 [US3] Verify `render_footer()` receives fresh `&GraphNode` reference each frame in `src/ui/canvas.rs` (already implemented via `self.graph.nodes.get(&node_uuid)`)
- [ ] T039 [US3] Test constant node footer: Edit constant value, verify footer updates immediately (manual test)
- [ ] T040 [US3] Test http_fetch footer: Execute HTTP request, verify response data appears in footer (manual test)
- [ ] T041 [US3] Add performance logging (debug level) in `CanvasViewer::show_footer()` in `src/ui/canvas.rs`: Measure `view.render_footer()` time with `std::time::Instant`, log warning if >50ms
- [ ] T042 [US3] Document reactivity behavior in `ComponentFooterView` trait docs in `src/ui/component_view.rs`: Note that egui immediate mode handles updates automatically

**Checkpoint**: User Story 3 complete - Views automatically reflect state changes, performance is acceptable

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, error handling hardening, and cleanup

- [ ] T043 [P] Add comprehensive trait documentation with examples in `src/ui/component_view.rs` (match contracts/component_footer_view_trait.md)
- [ ] T044 [P] Add documentation comments to ComponentSpec view methods in `src/graph/node.rs` (with_footer_view, has_footer_view, get_footer_view)
- [ ] T045 Add error scenario tests: Test view returning Err, verify error message displays in footer in `tests/integration/canvas_view_tests.rs`
- [ ] T046 Add error scenario tests: Test component with no outputs, verify graceful handling in `tests/unit/component_view_tests.rs`
- [ ] T047 Code cleanup: Remove old hardcoded footer logic comments from `src/ui/canvas.rs`
- [ ] T048 [P] Update quickstart.md validation: Verify examples compile and run (manual walkthrough)
- [ ] T049 Run `cargo test` to verify all tests pass (>80% coverage target for new code)
- [ ] T050 Run `cargo clippy` to check for linting issues in new code
- [ ] T051 Manual testing: Load existing saved graphs, verify no serialization breakage (footer_view skipped as expected)
- [ ] T052 Manual testing: Create new graph with custom view components, save/load, verify views re-register correctly

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User Story 1 can start immediately after Foundational
  - User Story 2 can start after Foundational (does not depend on US1, but builds on same pattern)
  - User Story 3 can start after Foundational (validates reactivity of US1/US2 views)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories (MVP-ready)
- **User Story 2 (P2)**: Can start after Foundational - Independent of US1 but follows same pattern
- **User Story 3 (P3)**: Can start after Foundational - Tests reactivity of US1/US2 views but doesn't modify them

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Foundational trait definition before any view implementations
- Canvas refactoring before specific view migrations
- Core implementation (http_fetch view) before additional views (constants, math)
- Manual validation after automated tests pass

### Parallel Opportunities

**Phase 1 (Setup)**:
- T001, T002, T003 can run in parallel (different files)

**Phase 2 (Foundational)**:
- T004 can run independently (trait definition)
- T005-T009 all modify ComponentSpec sequentially (same file - no parallelization)

**Phase 3 (User Story 1) - Tests**:
- T010, T011, T012, T013 can run in parallel (all in `tests/unit/component_view_tests.rs` but independent test functions)

**Phase 3 (User Story 1) - Implementation**:
- T020 and T021 can run in parallel with T016-T019 (different files: canvas.rs vs http_fetch.rs)

**Phase 4 (User Story 2) - Tests**:
- T024, T025, T026, T027 are sequential (same test integration flow)

**Phase 4 (User Story 2) - Implementation**:
- T028, T029, T030 (constants.rs) and T031, T032, T033 (math.rs) can run in parallel (different files)

**Phase 5 (User Story 3) - Tests**:
- T035, T036, T037 are sequential (build on each other)

**Phase 6 (Polish)**:
- T043, T044, T048 can run in parallel (different files/concerns)

---

## Parallel Example: User Story 1 Implementation

```bash
# After Foundational phase completes:

# Parallel group 1 - Write failing tests:
Task: "Unit test: Create mock TestFooterView implementing ComponentFooterView in tests/unit/component_view_tests.rs"
Task: "Unit test: Verify ComponentSpec.has_footer_view() returns true in tests/unit/component_view_tests.rs"
Task: "Unit test: Verify ComponentSpec.get_footer_view() returns Some in tests/unit/component_view_tests.rs"
Task: "Unit test: Verify trait can be stored as Arc<dyn ComponentFooterView> in tests/unit/component_view_tests.rs"

# Sequential - Integration tests (build on each other):
Task: "Integration test: Create test component with custom view in tests/integration/canvas_view_tests.rs"
Task: "Integration test: Verify CanvasViewer::has_footer() detects it in tests/integration/canvas_view_tests.rs"
Task: "Integration test: Verify CanvasViewer::show_footer() calls view.render_footer() in tests/integration/canvas_view_tests.rs"

# Parallel group 2 - Implement (different files):
Task: "Refactor CanvasViewer::has_footer() in src/ui/canvas.rs" &
Task: "Create HttpFetchFooterView struct in src/builtin/http_fetch.rs" &
Task: "Implement ComponentFooterView trait for HttpFetchFooterView in src/builtin/http_fetch.rs"

# Sequential - Wire up:
Task: "Refactor CanvasViewer::show_footer() in src/ui/canvas.rs"
Task: "Add error handling in CanvasViewer::show_footer() in src/ui/canvas.rs"
Task: "Update http_fetch component registration to use .with_footer_view() in src/builtin/http_fetch.rs"
Task: "Verify http_fetch node still renders footer correctly (manual test)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T009) - CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T010-T023)
4. **STOP and VALIDATE**: Run tests, manually test http_fetch footer
5. **MVP READY**: Trait-based footer views working for http_fetch component

### Incremental Delivery

1. Setup + Foundational → Foundation ready (T001-T009)
2. Add User Story 1 → Test independently → **MVP: Single component custom view** (T010-T023)
3. Add User Story 2 → Test independently → **Multiple components with custom views** (T024-T034)
4. Add User Story 3 → Test independently → **Reactive view updates** (T035-T042)
5. Polish → **Production-ready** (T043-T052)

Each story adds value without breaking previous stories.

### Parallel Team Strategy

With multiple developers (after Foundational phase completes):

1. **Team completes Setup + Foundational together** (T001-T009)
2. Once Foundational is done:
   - **Developer A**: User Story 1 - Core trait implementation (T010-T023)
   - **Developer B** (can start in parallel): User Story 2 - Additional views (T024-T034) - Note: will need US1's trait but can work on test views
   - **Developer C** (can start in parallel): User Story 3 - Reactivity tests (T035-T042) - Note: may need US1's http_fetch view for testing
3. Stories complete and integrate independently

**Recommended**: Complete US1 first (MVP), then parallelize US2 and US3 if team capacity allows.

---

## Notes

- [P] tasks = different files or independent test functions, can run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD approach per constitution requirement)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- **Foundational phase (T004-T009) is CRITICAL** - all user stories blocked until complete
- Tests target >80% coverage for `ComponentFooterView` trait and `CanvasViewer` footer methods (per constitution)
- All existing saved graphs must continue to work (footer_view skipped in serialization)

## Summary

- **Total Tasks**: 52
- **User Story 1 (MVP)**: 14 tasks (T010-T023)
- **User Story 2**: 11 tasks (T024-T034)
- **User Story 3**: 8 tasks (T035-T042)
- **Foundational (Blocking)**: 6 tasks (T004-T009)
- **Setup**: 3 tasks (T001-T003)
- **Polish**: 10 tasks (T043-T052)
- **Parallel Opportunities**: 12 tasks marked [P]
- **MVP Scope**: Phases 1-3 (T001-T023) = 23 tasks
