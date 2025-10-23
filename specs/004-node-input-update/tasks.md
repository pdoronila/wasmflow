# Tasks: Four-Section Node Layout

**Input**: Design documents from `/specs/004-node-input-update/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not explicitly requested in spec - tests marked as optional but recommended for quality

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Project Type**: Single desktop application (Rust)
- **Root**: Repository root
- **Source**: `src/`
- **Tests**: `tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: No setup required - using existing WasmFlow project structure

✅ Skipped - existing project with all dependencies (egui 0.29, egui-snarl 0.3, serde, bincode)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core trait and data model extensions that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T001 [Foundation] Add `ComponentBodyView` trait to `src/ui/component_view.rs`
  - Add trait definition parallel to existing `ComponentFooterView`
  - Interface: `fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String>`
  - Include performance requirements in trait documentation (<50ms)
  - Add Send + Sync bounds for thread safety
  - **Delivers**: Core interface for custom body views

- [X] T002 [P] [Foundation] Extend `ComponentSpec` with `body_view` field in `src/graph/node.rs`
  - Add `pub body_view: Option<Arc<dyn ComponentBodyView>>` field (marked `#[serde(skip)]`)
  - Add `with_body_view(Arc<dyn ComponentBodyView>)` builder method
  - Add `has_body_view() -> bool` check method
  - Add `get_body_view() -> Option<&Arc<dyn ComponentBodyView>>` accessor method
  - Mirror existing `footer_view` pattern exactly
  - **Delivers**: Component registration mechanism for custom body views

- [X] T003 [P] [Foundation] Update `ComponentSpec` Debug implementation in `src/graph/node.rs`
  - Add `body_view` field to manual Debug impl (display as `"<view>"` if present)
  - Maintain consistency with existing `footer_view` Debug formatting
  - **Delivers**: Debuggable ComponentSpec with new field

**Checkpoint**: Foundation ready - `ComponentBodyView` trait exists, `ComponentSpec` can register views. User story implementation can now begin.

---

## Phase 3: User Story 1 - View Node Connections Separately (Priority: P1) 🎯 MVP

**Goal**: Display input/output connections in a dedicated section with pin indicators and type information, separate from body and footer

**Independent Test**: Create a node with 2 inputs (value:u32, freq:u32) and 1 output (value:u32). Verify connections section displays all inputs on left with "o" pins, output on right with "o" pin, and type labels are visible.

### Tests for User Story 1 (Optional but Recommended)

**NOTE**: Write these tests FIRST, ensure they FAIL before implementation

- [X] T004 [P] [US1] Add unit test for connections section rendering in `tests/unit/component_view_tests.rs`
  - Test case: Node with multiple inputs/outputs renders pins in correct positions
  - Test case: Node with no connections shows empty/collapsed connections section
  - Test case: Node with input-only shows inputs on left, no outputs on right
  - **Validates**: FR-003, FR-004, FR-005 (connection display requirements)

- [X] T005 [P] [US1] Add integration test for four-section layout in `tests/integration/canvas_view_tests.rs`
  - Test case: Render node and verify four sections exist in order (header → connections → body → footer)
  - Test case: Connections section appears between header and body
  - Test case: Pin colors match data types (use CanvasViewer::type_color)
  - **Validates**: FR-001 (four-section rendering), SC-001 (connection identification)

### Implementation for User Story 1

- [X] T006 [US1] Update `CanvasViewer::show_input()` in `src/ui/canvas.rs` for connections section
  - Modify to show only `port.name` + `port.data_type.name()` (no values)
  - Remove value display logic (moves to default footer in US3)
  - Keep color-coded pins based on type (`Self::type_color()`)
  - Maintain "o" pin indicator via `PinInfo::circle()`
  - **Delivers**: Left side of connections section (inputs)

- [X] T007 [US1] Update `CanvasViewer::show_output()` in `src/ui/canvas.rs` for connections section
  - Modify to show only `port.name` + `port.data_type.name()` (no values)
  - Remove has_footer check and value display (moves to default footer in US3)
  - Keep color-coded pins based on type (`Self::type_color()`)
  - Maintain "o" pin indicator via `PinInfo::circle()`
  - **Delivers**: Right side of connections section (outputs)

- [X] T008 [US1] Add visual separation between sections in `CanvasViewer` methods in `src/ui/canvas.rs`
  - Add `ui.add_space(6.0)` before/after connections rendering
  - Consider adding `egui::Separator` between sections (optional, test visually)
  - Ensure separation is consistent with existing header/body/footer spacing
  - **Delivers**: FR-013 (visual separation between sections)

**Checkpoint**: At this point, User Story 1 should be fully functional. Nodes display connections separately from body/footer. Test independently before proceeding.

---

## Phase 4: User Story 2 - Display Default and Custom Body Content (Priority: P2)

**Goal**: Auto-generate input fields for node parameters by default, with ability for components to provide custom body views

**Independent Test**: (1) Create node without custom body - verify default input fields appear for all parameters. (2) Register component with custom body view - verify custom content overrides defaults.

### Tests for User Story 2 (Optional but Recommended)

- [X] T009 [P] [US2] Add unit test for `DefaultBodyView` rendering in `tests/unit/component_view_tests.rs`
  - Test case: Renders DragValue widget for U32/I32/F32 input ports
  - Test case: Renders TextEdit widget for String input ports
  - Test case: Shows placeholder for complex types (List, Record, Binary)
  - Test case: Limits to 20 fields max (edge case from spec)
  - Test case: Shows "(no parameters)" when node has no inputs
  - **Validates**: FR-006 (default body content), contracts/default-views.md spec

- [X] T010 [P] [US2] Add unit test for `ComponentBodyView` trait usage in `tests/unit/component_view_tests.rs`
  - Test case: Custom body view renders without error
  - Test case: Custom body view can update node.inputs values
  - Test case: Custom body view sets node.dirty = true on changes
  - Test case: Custom body view error handling (return Err displays error)
  - **Validates**: FR-007, FR-008 (custom body content), contracts/component-body-view.md

- [X] T011 [P] [US2] Add integration test for body view selection in `tests/integration/canvas_view_tests.rs`
  - Test case: Component without body_view uses DefaultBodyView
  - Test case: Component with body_view uses custom view (skips default)
  - Test case: Default body updates when input values change
  - **Validates**: FR-009 (body rendering between sections), SC-007 (default fields appear)

### Implementation for User Story 2

- [X] T012 [US2] Implement `DefaultBodyView` helper struct in `src/ui/canvas.rs`
  - Add struct with `render_for_node(ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String>` method
  - Iterate `node.inputs`, match on `data_type`:
    - DataType::U32 → egui::DragValue with speed 1.0
    - DataType::I32 → egui::DragValue with speed 1.0
    - DataType::F32 → egui::DragValue with speed 0.1
    - DataType::String → egui::TextEdit::singleline with 150px width
    - DataType::List/Record/Binary → placeholder text "(complex type - use custom view)"
    - DataType::Any → placeholder "(any type - no default editor)"
  - Set `node.dirty = true` when any widget changes
  - Limit to 20 fields max (show "... and N more" for overflow)
  - Handle empty state: show "(no parameters)" if `node.inputs.is_empty()`
  - **Delivers**: FR-006 (default body generation), contracts/default-views.md implementation

- [X] T013 [US2] Update `CanvasViewer::show_body()` in `src/ui/canvas.rs` to use default/custom views
  - Check `ComponentSpec.has_body_view()` via registry lookup
  - If true: Call `spec.get_body_view().render_body(ui, node)` (custom view)
  - If false: Call `DefaultBodyView::render_for_node(ui, node)` (default view)
  - Wrap in performance timing (similar to existing footer timing at lines 248-260)
  - Log warning if render time >50ms
  - Handle errors: Display error message in body section with red text
  - Remove hardcoded constant editing check (`if node.component_id.starts_with("builtin:constant:")`)
  - **Delivers**: FR-007, FR-008 (custom vs default body selection), FR-009 (body positioning)

- [X] T014 [P] [US2] Create example custom body view in `src/builtin/views.rs`
  - Add `ConstantF32BodyView` struct implementing `ComponentBodyView`
  - Renders TextEdit for constant value editing (migrate from hardcoded canvas.rs logic)
  - Example demonstrates ComponentBodyView pattern for other developers
  - **Delivers**: Reference implementation for quickstart.md examples

- [X] T015 [US2] Update constant nodes to use custom body view in `src/builtin/constants.rs`
  - Import `ConstantF32BodyView` from views.rs
  - Update `ComponentSpec::new_builtin()` to include `.with_body_view(Arc::new(ConstantF32BodyView))`
  - Do same for ConstantI32, ConstantU32, ConstantString variants if they exist
  - Test that constant editing still works (value changes mark node dirty)
  - **Delivers**: FR-014 (backward compatibility), migration of existing constant functionality

**Checkpoint**: At this point, User Story 2 should be fully functional. Nodes without custom views show default input fields. Constant nodes have custom body views. Test independently.

---

## Phase 5: User Story 3 - Display Default and Custom Footer Content (Priority: P3)

**Goal**: Auto-display output values in footer by default, with ability for components to provide custom footer views

**Independent Test**: (1) Create node, execute it - verify footer shows "Current {port}: {value}" for outputs. (2) Register component with custom footer view - verify custom content overrides default status.

### Tests for User Story 3 (Optional but Recommended)

- [X] T016 [P] [US3] Add unit test for `DefaultFooterView` rendering in `tests/unit/component_view_tests.rs`
  - Test case: Displays "Current {port}: {value}" for outputs with current_value
  - Test case: Uses NodeValue::format_display() for type-appropriate formatting
  - Test case: Shows "(no values computed yet)" when all outputs are None
  - Test case: Limits to 10 outputs max (edge case from spec)
  - Test case: Shows "(no outputs)" when node.outputs.is_empty()
  - **Validates**: FR-010 (default footer status), contracts/default-views.md spec

- [X] T017 [P] [US3] Add integration test for footer view selection in `tests/integration/canvas_view_tests.rs`
  - Test case: Component without footer_view uses DefaultFooterView
  - Test case: Component with footer_view uses custom view (skips default)
  - Test case: Default footer updates when output values change after execution
  - Test case: Execution state awareness (Idle, Running, Completed, Failed)
  - **Validates**: FR-011, FR-012 (custom vs default footer), SC-004 (backward compatibility)

### Implementation for User Story 3

- [X] T018 [US3] Implement `DefaultFooterView` helper struct in `src/ui/canvas.rs`
  - Add struct with `render_for_node(ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String>` method
  - Iterate `node.outputs` where `current_value.is_some()`
  - Display `ui.label(format!("Current {}: {}", port.name, value.format_display()))`
  - Show execution state indicators:
    - ExecutionState::Idle → "(awaiting execution)"
    - ExecutionState::Running → "⏳ Computing..."
    - ExecutionState::Completed → show output values (main path)
    - ExecutionState::Failed → "❌ Execution failed" in red
  - Limit to 10 outputs (show "... and N more" for overflow)
  - Handle empty states: "(no outputs)" if node.outputs.is_empty()
  - **Delivers**: FR-010 (default footer generation), contracts/default-views.md implementation

- [X] T019 [US3] Update `CanvasViewer::show_footer()` in `src/ui/canvas.rs` to use default/custom views
  - Check `ComponentSpec.has_footer_view()` via registry lookup
  - If true: Call existing `spec.get_footer_view().render_footer(ui, node)` (custom view - UNCHANGED)
  - If false: Call `DefaultFooterView::render_for_node(ui, node)` (default view - NEW)
  - Keep existing performance timing and error handling (lines 248-274)
  - Maintain 250px max width and spacing constraints
  - **Delivers**: FR-011, FR-012 (custom vs default footer selection), FR-014 (backward compat)

**Checkpoint**: All user stories should now be independently functional. Four-section layout complete with connections, default/custom body, and default/custom footer.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality improvements, edge case handling, and documentation

- [X] T020 [P] [Polish] Add performance logging for default body view rendering in `src/ui/canvas.rs`
  - Wrap DefaultBodyView::render_for_node() with std::time::Instant timing
  - Log warning if >10ms (performance budget from contracts/default-views.md)
  - Log trace for normal case (<10ms)
  - Format: "Default body rendered for '{node.display_name}' in {elapsed}ms"
  - **Delivers**: Performance monitoring per contracts

- [X] T021 [P] [Polish] Add performance logging for default footer view rendering in `src/ui/canvas.rs`
  - Wrap DefaultFooterView::render_for_node() with std::time::Instant timing
  - Log warning if >5ms (performance budget from contracts/default-views.md)
  - Log trace for normal case (<5ms)
  - Format: "Default footer rendered for '{node.display_name}' in {elapsed}ms"
  - **Delivers**: Performance monitoring per contracts

- [X] T022 [P] [Polish] Add edge case handling for many connections in `src/ui/canvas.rs`
  - Check if node has >50 input ports or >50 output ports (spec edge case)
  - If exceeded, show warning message in connections section
  - Consider truncation or scrolling hint for user
  - **Delivers**: Edge case from spec (many connections)

- [X] T023 [P] [Polish] Add edge case handling for long parameter names in `src/ui/canvas.rs`
  - Truncate port names longer than 20 characters in connections section
  - Add tooltip with full name on hover (`.on_hover_text()`)
  - **Delivers**: Edge case from spec (very long parameter names)

- [X] T024 [P] [Polish] Update HTTP Fetch example to demonstrate compatibility in `examples/example-http-fetch/src/lib.rs`
  - Verify existing custom footer view still works (no changes needed)
  - Verify default body view auto-generates URL input field
  - Add comment documenting that this example uses default body + custom footer (mixing)
  - **Delivers**: Real-world example of backward compatibility (SC-004)

- [X] T025 [P] [Polish] Run cargo clippy and fix any new warnings
  - Focus on src/ui/canvas.rs, src/ui/component_view.rs, src/graph/node.rs
  - Address any dead code warnings from removed constant editing logic
  - **Delivers**: Code quality

- [X] T026 [P] [Polish] Run cargo test and ensure all tests pass
  - Unit tests in tests/unit/component_view_tests.rs
  - Integration tests in tests/integration/canvas_view_tests.rs
  - Address any failures related to new default views
  - **Delivers**: Test suite validation

- [X] T027 [P] [Polish] Update CLAUDE.md if needed (already done by /speckit.plan)
  - ✅ Agent context already updated with egui 0.29, eframe 0.29, egui-snarl 0.3
  - **Delivers**: Up-to-date project documentation

- [X] T028 [Polish] Manual testing: Create test graph with multiple node types
  - Add constant nodes (custom body view)
  - Add math nodes (default body view)
  - Add HTTP fetch node if available (custom footer, default body)
  - Verify all render correctly with four sections
  - Test performance with 50+ nodes (should maintain 60 FPS)
  - **Delivers**: End-to-end validation of all user stories

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: ✅ Skipped (existing project)
- **Foundational (Phase 2)**: No dependencies - can start immediately - **BLOCKS all user stories**
- **User Story 1 (Phase 3)**: Depends on Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (Phase 4)**: Depends on Foundational (Phase 2) - Technically independent of US1 but builds on connections section visually
- **User Story 3 (Phase 5)**: Depends on Foundational (Phase 2) - Technically independent of US1/US2 but completes the four-section layout
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (T001-T003) - Foundation of four-section layout
- **User Story 2 (P2)**: Can start after Foundational (T001-T003) - Independent but visually builds on US1
- **User Story 3 (P3)**: Can start after Foundational (T001-T003) - Independent but completes layout started in US1

**Note**: While US2 and US3 are technically independent, they build on the visual structure established in US1. Recommended sequential implementation (P1 → P2 → P3) for clarity.

### Within Each User Story

**Foundational Phase**:
- T001 (ComponentBodyView trait) can run in parallel with T002 (ComponentSpec update)
- T003 (Debug impl) depends on T002 completing

**User Story 1**:
- T004, T005 (tests) can run in parallel - write FIRST before implementation
- T006 (show_input) can run in parallel with T007 (show_output)
- T008 (visual separation) depends on T006 and T007

**User Story 2**:
- T009, T010, T011 (tests) can run in parallel - write FIRST
- T012 (DefaultBodyView) can run in parallel with T014 (example view)
- T013 (show_body update) depends on T012 (DefaultBodyView exists)
- T015 (constant migration) depends on T014 (example exists)

**User Story 3**:
- T016, T017 (tests) can run in parallel - write FIRST
- T018 (DefaultFooterView) is independent
- T019 (show_footer update) depends on T018 (DefaultFooterView exists)

**Polish Phase**:
- T020-T027 (all marked [P]) can run in parallel
- T028 (manual testing) should be last

### Parallel Opportunities

- **Foundational**: T001 + T002 in parallel (different concepts)
- **US1 Tests**: T004 + T005 in parallel (different test files)
- **US1 Implementation**: T006 + T007 in parallel (different methods)
- **US2 Tests**: T009 + T010 + T011 in parallel (different test files)
- **US2 Implementation**: T012 + T014 in parallel (different files)
- **US3 Tests**: T016 + T017 in parallel (different test files)
- **Polish**: T020 through T027 in parallel (different concerns)

---

## Parallel Example: User Story 1

```bash
# Write tests FIRST (run in parallel):
claude-code: "Add unit test for connections section rendering in tests/unit/component_view_tests.rs per T004"
claude-code: "Add integration test for four-section layout in tests/integration/canvas_view_tests.rs per T005"

# Verify tests FAIL (expected - no implementation yet)

# Implement in parallel:
claude-code: "Update CanvasViewer::show_input() in src/ui/canvas.rs per T006"
claude-code: "Update CanvasViewer::show_output() in src/ui/canvas.rs per T007"

# Sequential (depends on T006 + T007):
claude-code: "Add visual separation between sections in src/ui/canvas.rs per T008"

# Verify tests PASS
```

---

## Parallel Example: User Story 2

```bash
# Write tests FIRST (run in parallel):
claude-code: "Add unit test for DefaultBodyView rendering in tests/unit/component_view_tests.rs per T009"
claude-code: "Add unit test for ComponentBodyView trait in tests/unit/component_view_tests.rs per T010"
claude-code: "Add integration test for body view selection in tests/integration/canvas_view_tests.rs per T011"

# Implement core + example in parallel:
claude-code: "Implement DefaultBodyView in src/ui/canvas.rs per T012"
claude-code: "Create example ConstantF32BodyView in src/builtin/views.rs per T014"

# Sequential updates (depends on T012):
claude-code: "Update CanvasViewer::show_body() in src/ui/canvas.rs per T013"

# Sequential migration (depends on T014):
claude-code: "Update constant nodes in src/builtin/constants.rs per T015"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. ✅ Phase 1: Setup (skipped - existing project)
2. Complete Phase 2: Foundational (T001-T003) - **CRITICAL blocking phase**
3. Complete Phase 3: User Story 1 (T004-T008)
4. **STOP and VALIDATE**:
   - Test: Create node with inputs/outputs
   - Verify: Connections section visible between header and body
   - Verify: Type colors display correctly
   - Verify: Four sections render in order
5. **Deploy/Demo if ready**: MVP shows improved connections visibility

### Incremental Delivery

1. Complete Foundational (T001-T003) → ComponentBodyView trait exists, ComponentSpec extensible
2. Add User Story 1 (T004-T008) → Test independently → Connections section working (MVP!)
3. Add User Story 2 (T009-T015) → Test independently → Default body views working
4. Add User Story 3 (T016-T019) → Test independently → Default footer views complete four-section layout
5. Polish (T020-T028) → Performance, edge cases, validation

Each story adds value without breaking previous stories.

### Parallel Team Strategy

With multiple developers:

1. Team completes Foundational together (T001-T003) - **MUST complete before splitting**
2. Once Foundational is done:
   - Developer A: User Story 1 (T004-T008) - Connections section
   - Developer B: User Story 2 (T009-T015) - Body views (needs T001-T003)
   - Developer C: User Story 3 (T016-T019) - Footer views (needs T001-T003)
3. Merge and integrate
4. Team collaborates on Polish (T020-T028)

**Note**: While technically parallelizable, sequential implementation (P1 → P2 → P3) recommended for visual coherence.

---

## Task Summary

**Total Tasks**: 28
- Foundational: 3 tasks (T001-T003) - **BLOCKING all user stories**
- User Story 1 (P1 - MVP): 5 tasks (T004-T008) - Connections section
- User Story 2 (P2): 7 tasks (T009-T015) - Default/custom body views
- User Story 3 (P3): 4 tasks (T016-T019) - Default/custom footer views
- Polish: 9 tasks (T020-T028) - Quality, edge cases, validation

**Parallel Opportunities**: 15 tasks marked [P]

**Independent Test Criteria**:
- US1: Create node with inputs/outputs → Verify connections section visible with type-colored pins
- US2: Create node without custom body → Verify default input fields appear; Register custom body → Verify custom view shown
- US3: Execute node → Verify footer shows "Current {port}: {value}"; Register custom footer → Verify custom view shown

**Suggested MVP Scope**: Foundational (T001-T003) + User Story 1 (T004-T008) = 8 tasks

**Estimated Effort**: ~500 LOC across 10 files (per plan.md estimate)

---

## Notes

- [P] tasks = different files or independent concerns, no dependencies
- [Story] label maps task to specific user story (US1, US2, US3) for traceability
- [Foundation] label for blocking prerequisites
- [Polish] label for quality/cross-cutting tasks
- Tests are optional but strongly recommended for quality assurance
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Performance budgets: Default body <10ms, Default footer <5ms, Custom views <50ms
- Backward compatibility: 100% - existing components work without changes
