# Implementation Tasks: Rectangle Selection & Node Composition

**Feature**: 007-rectangle-selection-tool
**Branch**: `007-rectangle-selection-tool`
**Generated**: 2025-10-21

## Overview

This document provides a complete task breakdown for implementing rectangle selection and WebAssembly component composition. Tasks are organized by user story to enable independent implementation and incremental delivery.

**Total Tasks**: 45
**Estimated Time**: 7-10 hours total (3-4 hours for MVP)

## Implementation Strategy

**MVP Scope** (User Stories 1 + 3): Rectangle selection with visual feedback (~3-4 hours)
- Delivers immediately useful feature (multi-select)
- Establishes foundation for composition
- Can be released independently

**Full Feature** (All Stories): Complete composition workflow (~7-10 hours)
- Adds composition capability (US2)
- Adds drill-down inspection (US4)
- Adds visual polish (US5)

## Phase 1: Project Setup & Dependencies

### T001 [X] [P] Add wac-graph dependency to Cargo.toml
**Story**: Setup (foundational)
**File**: `Cargo.toml`
**Action**: Add `wac-graph = "0.8"` to [dependencies] section
**Verification**: Run `cargo check` to ensure dependency resolves
**Time**: 2 minutes

### T002 [X] [P] Create selection module stub
**Story**: Setup (foundational)
**File**: `src/ui/selection.rs` (NEW)
**Action**: Create file with basic module structure and SelectionState struct stub
```rust
use egui::Pos2;
use std::collections::HashSet;
use crate::graph::NodeId;

pub struct SelectionState {
    // To be implemented
}
```
**Verification**: `cargo check` passes
**Time**: 2 minutes

### T003 [X] [P] Create graph validation module stub
**Story**: Setup (foundational)
**File**: `src/graph/validation.rs` (NEW)
**Action**: Create file with module structure
**Verification**: `cargo check` passes
**Time**: 2 minutes

### T004 [X] [P] Create WAC integration module stub
**Story**: Setup (foundational)
**File**: `src/runtime/wac_integration.rs` (NEW)
**Action**: Create file with ComponentComposer struct stub
**Verification**: `cargo check` passes
**Time**: 2 minutes

### T005 [X] [P] Create drill-down module stub
**Story**: Setup (foundational)
**File**: `src/graph/drill_down.rs` (NEW)
**Action**: Create file with ViewContext enum stub
**Verification**: `cargo check` passes
**Time**: 2 minutes

### T006 [X] Update module declarations
**Story**: Setup (foundational)
**Files**:
- `src/ui/mod.rs` - Add `pub mod selection;`
- `src/graph/mod.rs` - Add `pub mod validation;` and `pub mod drill_down;`
- `src/runtime/mod.rs` - Add `pub mod wac_integration;`
**Verification**: `cargo check` passes
**Time**: 3 minutes

**Phase 1 Checkpoint**: ✓ All module stubs created, dependencies added, project compiles

---

## Phase 2: User Story 1 - Rectangle Selection (Priority P1)

**Goal**: Users can click and drag on canvas to select multiple nodes

**Independent Test**: Click and drag on canvas over 5 nodes → verify all nodes within rectangle are highlighted as selected

### T007 [X] [US1] Implement SelectionState data structure
**Story**: US1 - Select Multiple Nodes with Rectangle
**File**: `src/ui/selection.rs`
**Action**: Implement complete SelectionState struct with:
- Fields: `start_pos`, `current_pos`, `selected_nodes`, `is_dragging`
- Methods: `new()`, `start_drag()`, `update_drag()`, `end_drag()`, `cancel_drag()`, `is_dragging()`, `get_selection_rect()`, `selected_nodes()`, `clear_selection()`
**Reference**: quickstart.md Step 2, data-model.md SelectionState section
**Verification**: `cargo check` passes, struct methods are public
**Time**: 15 minutes

### T008 [X] [US1] Add SelectionState to Canvas struct
**Story**: US1 - Select Multiple Nodes with Rectangle
**File**: `src/ui/canvas.rs`
**Action**:
- Import `SelectionState`
- Add `selection: SelectionState` field to Canvas struct
- Initialize in `new()` method
**Verification**: `cargo check` passes
**Time**: 5 minutes

### T009 [X] [US1] Implement mouse down handler for selection start
**Story**: US1 - Select Multiple Nodes with Rectangle
**File**: `src/ui/canvas.rs`
**Action**: In canvas update/render method:
- Detect mouse down on empty canvas (not over node)
- Call `selection.start_drag(pos)`
**Reference**: quickstart.md Step 3
**Verification**: Add debug log, test manually - log appears on empty canvas click
**Time**: 10 minutes

### T010 [X] [US1] Implement mouse drag handler for selection update
**Story**: US1 - Select Multiple Nodes with Rectangle
**File**: `src/ui/canvas.rs`
**Action**:
- Detect mouse drag while selection is active
- Call `selection.update_drag(pos)` on each drag event
**Verification**: Add debug log showing drag positions
**Time**: 10 minutes

### T011 [X] [US1] Implement mouse up handler for selection finalization
**Story**: US1 - Select Multiple Nodes with Rectangle
**File**: `src/ui/canvas.rs`
**Action**:
- Detect mouse up while dragging
- Call `find_nodes_in_rect()` helper
- Call `selection.end_drag(nodes)`
**Verification**: Debug log showing number of nodes selected
**Time**: 10 minutes

### T012 [X] [US1] Implement find_nodes_in_rect helper
**Story**: US1 - Select Multiple Nodes with Rectangle
**File**: `src/ui/canvas.rs`
**Action**: Implement method that:
- Takes a Rect as input
- Iterates through all nodes in graph
- Returns HashSet<NodeId> of nodes whose centers fall within rectangle
**Reference**: quickstart.md Step 3 `find_nodes_in_rect` implementation
**Verification**: Unit test with mock nodes at known positions
**Time**: 15 minutes

### T013 [X] [US1] Implement ESC key handler to cancel selection
**Story**: US1 - Select Multiple Nodes with Rectangle
**File**: `src/ui/canvas.rs`
**Action**:
- Detect ESC key press with `ui.input(|i| i.key_pressed(egui::Key::Escape))`
- Call `selection.cancel_drag()` and `selection.clear_selection()`
**Verification**: Test - press ESC during drag, selection should cancel
**Time**: 5 minutes

### T014 [X] [US1] Implement minimum rectangle size validation
**Story**: US1 - Select Multiple Nodes with Rectangle
**File**: `src/ui/selection.rs`
**Action**: In `end_drag()`, check if rectangle is >5×5 pixels:
- If yes: proceed with selection
- If no: clear selection instead
**Reference**: spec.md acceptance scenario 2, data-model.md validation rules
**Verification**: Test tiny drag (<5px) → no selection made
**Time**: 10 minutes

**Phase 2 Checkpoint**: ✓ US1 Complete - Users can select nodes by dragging rectangle

**Test US1 Independently**:
1. Build: `cargo run`
2. Load graph with 5+ nodes
3. Click and drag over nodes → verify selection happens
4. Make tiny drag (<5px) → verify no selection
5. Press ESC during drag → verify selection cancels

---

## Phase 3: User Story 3 - Visual Feedback (Priority P1)

**Goal**: Users see real-time visual feedback while drawing selection rectangle

**Independent Test**: Observe visual changes during click-drag → rectangle outline appears, nodes preview-highlight, finalized nodes persistently highlight

### T015 [X] [US3] Implement selection rectangle rendering
**Story**: US3 - Visual Feedback During Selection
**File**: `src/ui/canvas.rs`
**Action**: Add rendering logic:
- Check if `selection.get_selection_rect()` returns Some
- Use `ui.painter()` to draw rectangle with:
  - Semi-transparent fill: `Color32::from_rgba_unmultiplied(100, 150, 200, 50)`
  - Border stroke: `Stroke::new(1.5, Color32::from_rgb(100, 150, 200))`
**Reference**: quickstart.md Step 3 `render_selection_rectangle`
**Verification**: Visual test - drag should show blue rectangle outline
**Time**: 10 minutes

### T016 [X] [US3] Add preview highlighting for nodes within active rectangle
**Story**: US3 - Visual Feedback During Selection
**File**: `src/ui/canvas.rs`
**Action**: In node rendering loop:
- Check if `selection.is_dragging()` and node is within `get_selection_rect()`
- Apply preview highlight overlay: `Color32::from_rgba_unmultiplied(100, 200, 100, 80)`
**Verification**: Visual test - nodes inside drag rectangle show green preview tint
**Time**: 15 minutes

### T017 [X] [US3] Add persistent selection highlighting
**Story**: US3 - Visual Feedback During Selection
**File**: `src/ui/canvas.rs`
**Action**: In node rendering loop:
- Check if node ID is in `selection.selected_nodes()`
- Apply selection border: `Stroke::new(2.5, Color32::from_rgb(100, 200, 255))`
**Verification**: Visual test - selected nodes show blue border after drag complete
**Time**: 10 minutes

### T018 [X] [US3] Add SelectionTheme configuration
**Story**: US3 - Visual Feedback During Selection
**File**: `src/ui/theme.rs`
**Action**: Add `SelectionTheme` struct with configurable colors:
- `rectangle_fill`, `rectangle_stroke`
- `preview_highlight`, `selected_border`, `selected_border_width`
**Reference**: research.md section 6
**Verification**: Theme colors can be customized
**Time**: 10 minutes

### T019 [X] [US3] Ensure selection rectangle disappears on mouse up
**Story**: US3 - Visual Feedback During Selection
**File**: `src/ui/canvas.rs`
**Action**: Verify that `end_drag()` clears `start_pos` and `current_pos`, so `get_selection_rect()` returns None
**Verification**: Visual test - rectangle disappears when mouse released
**Time**: 5 minutes

**Phase 3 Checkpoint**: ✓ US3 Complete - Visual feedback for selection is clear and responsive

**Test US3 Independently**:
1. Start drag → blue rectangle outline appears
2. Drag over nodes → nodes show green preview tint
3. Release mouse → rectangle disappears, selected nodes show blue border
4. Verify <100ms response time (smooth 60 FPS)

---

## Phase 4: Foundational - Graph Validation (Required for US2)

**Goal**: Implement connected subgraph validation required for composition

### T020 [Foundation] Implement is_connected_subgraph function
**Story**: Foundation for US2
**File**: `src/graph/validation.rs`
**Action**: Implement using petgraph:
- Build petgraph Graph from selected NodeIds
- Add edges only between selected nodes
- Perform DFS from first node
- Return true if all nodes reachable
**Reference**: quickstart.md Step 5, research.md section 3
**Verification**: Unit test with connected and disconnected node sets
**Time**: 20 minutes

### T021 [Foundation] Add unit tests for validation
**Story**: Foundation for US2
**File**: `tests/unit/graph_validation_tests.rs` (NEW)
**Action**: Create tests:
- `test_validation_accepts_connected_subgraph`
- `test_validation_rejects_single_node`
- `test_validation_rejects_disconnected_nodes`
- `test_validation_rejects_empty_selection`
**Verification**: `cargo test graph_validation` passes
**Time**: 15 minutes

**Foundational Checkpoint**: ✓ Graph validation ready for composition

---

## Phase 5: Foundational - WAC Integration (Required for US2)

**Goal**: Implement WebAssembly component composition service

### T022 [Foundation] Implement ComponentComposer struct
**Story**: Foundation for US2
**File**: `src/runtime/wac_integration.rs`
**Action**: Implement:
- `ComponentComposer::new()`
- `compose(&self, socket: &Path, plugs: &[&Path]) -> Result<Vec<u8>>`
  - Create CompositionGraph
  - Register socket component with Package::from_file
  - Register plug components
  - Call `wac_graph::plug()` for each
  - Encode with validation
**Reference**: quickstart.md Step 4, research.md section 1
**Verification**: `cargo check` passes
**Time**: 30 minutes

### T023 [Foundation] Add error handling and context to composition
**Story**: Foundation for US2
**File**: `src/runtime/wac_integration.rs`
**Action**: Add detailed error context using anyhow::Context:
- File not found errors
- Component loading errors
- Registration errors
- Plug errors with file path info
**Reference**: research.md section 4
**Verification**: Test with invalid paths → meaningful error messages
**Time**: 15 minutes

### T024 [Foundation] Add composition logging
**Story**: Foundation for US2
**File**: `src/runtime/wac_integration.rs`
**Action**: Add log statements:
- `log::info!` for composition start (socket + plug count)
- `log::debug!` for each successful plug
- `log::info!` for composition complete (bytes size)
**Verification**: Run with `RUST_LOG=debug` → see composition logs
**Time**: 5 minutes

### T025 [Foundation] Add WAC integration unit tests
**Story**: Foundation for US2
**File**: `tests/unit/wac_integration_tests.rs` (NEW)
**Action**: Create tests (requires example WASM components):
- `test_basic_composition` - socket + 1 plug
- `test_multiple_plugs` - socket + 3 plugs
- `test_invalid_socket_path` - error handling
- `test_incompatible_components` - NoPlugHappened error
**Verification**: `cargo test wac_integration` passes (may need to create test fixtures)
**Time**: 25 minutes

**Foundational Checkpoint**: ✓ WAC composition service ready

---

## Phase 6: User Story 2 - Compose Selected Nodes (Priority P2)

**Goal**: Users can compose selected nodes into a single composite node

**Independent Test**: Select 3 connected nodes → click Compose → verify new composite node appears with aggregated ports and footer showing component names

### T026 [US2] Extend Node/ComponentSpec for composite nodes
**Story**: US2 - Compose Selected Nodes
**File**: `src/graph/node.rs`
**Action**: Add to ComponentSpec enum:
```rust
Composed {
    name: String,
    socket_path: PathBuf,
    plug_paths: Vec<PathBuf>,
    internal_nodes: BTreeMap<NodeId, Node>,
    internal_edges: Vec<Edge>,
    exposed_inputs: BTreeMap<String, PortMapping>,
    exposed_outputs: BTreeMap<String, PortMapping>,
    metadata: CompositionMetadata,
    #[serde(skip)]
    cached_composition: Option<Vec<u8>>,
}
```
**Reference**: data-model.md CompositeNode section
**Verification**: `cargo check` passes
**Time**: 20 minutes

### T027 [US2] Implement PortMapping and CompositionMetadata structs
**Story**: US2 - Compose Selected Nodes
**File**: `src/graph/node.rs` (or new `src/graph/composite.rs`)
**Action**: Define structs:
- `PortMapping` with fields from data-model.md
- `CompositionMetadata` with creation time, count, names, size, hash
**Verification**: Structs are Serialize + Deserialize compatible
**Time**: 15 minutes

### T028 [US2] Add ComponentComposer to App state
**Story**: US2 - Compose Selected Nodes
**File**: `src/ui/app.rs`
**Action**:
- Import `ComponentComposer`
- Add field: `composer: ComponentComposer`
- Initialize in `App::new()`
**Verification**: `cargo check` passes
**Time**: 5 minutes

### T029 [US2] Implement get_component_path helper
**Story**: US2 - Compose Selected Nodes
**File**: `src/ui/app.rs`
**Action**: Implement method:
- Takes `&Node` as input
- Extracts PathBuf from `ComponentSpec::WasmComponent { path, .. }`
- Returns Result (error if not a WASM component)
**Verification**: Unit test with mock node
**Time**: 10 minutes

### T030 [US2] Implement handle_compose_action method
**Story**: US2 - Compose Selected Nodes
**File**: `src/ui/app.rs`
**Action**: Implement full composition workflow:
1. Get selected nodes from `canvas.selection.selected_nodes()`
2. Validate: ≥2 nodes selected
3. Validate: `is_connected_subgraph()`
4. Extract socket_path (first node) and plug_paths (rest)
5. Call `composer.compose(socket, plugs)`
6. Create new CompositeNode with result
7. Add to graph
8. Remove original nodes
**Reference**: quickstart.md Step 6
**Verification**: Integration test (requires WASM components)
**Time**: 30 minutes

### T031 [US2] Add "Compose" button to UI toolbar
**Story**: US2 - Compose Selected Nodes
**File**: `src/ui/app.rs` or `src/ui/canvas.rs`
**Action**: In toolbar render:
```rust
let can_compose = self.canvas.selection.selected_nodes().len() >= 2;
ui.add_enabled_ui(can_compose, |ui| {
    if ui.button("🔗 Compose").clicked() {
        if let Err(e) = self.handle_compose_action() {
            self.show_error(&format!("Composition failed: {}", e));
        }
    }
});
```
**Reference**: quickstart.md Step 7
**Verification**: Button appears, enabled only when ≥2 nodes selected
**Time**: 10 minutes

### T032 [US2] Implement composition error dialog
**Story**: US2 - Compose Selected Nodes
**File**: `src/ui/dialogs.rs`
**Action**: Add `show_composition_error()` method:
- Display error message in egui::Window
- Parse error type and show user-friendly message
- Provide "View Details" option for technical error
**Reference**: research.md section 5 error recovery
**Verification**: Trigger composition error → friendly dialog appears
**Time**: 15 minutes

### T033 [US2] Add validation error messages
**Story**: US2 - Compose Selected Nodes
**File**: `src/ui/app.rs`
**Action**: Add user-friendly error messages:
- <2 nodes: "Select at least 2 nodes to compose"
- Disconnected: "Selected nodes must form a connected subgraph"
- WAC NoPlugHappened: "Components have incompatible interfaces"
**Verification**: Test each error case → appropriate message shown
**Time**: 10 minutes

### T034 [US2] Implement composite node rendering (basic)
**Story**: US2 - Compose Selected Nodes
**File**: `src/ui/canvas.rs`
**Action**: In node rendering:
- Detect if node is `ComponentSpec::Composed`
- Render with distinct background color
- Show node name
- (Footer in US5)
**Verification**: Composite node appears with different color
**Time**: 15 minutes

### T035 [US2] Implement port aggregation logic
**Story**: US2 - Compose Selected Nodes
**File**: `src/graph/graph.rs` (or new helper module)
**Action**: Implement function to analyze selected nodes and determine:
- Exposed inputs: inputs from selected nodes with no internal connections
- Exposed outputs: outputs from selected nodes with no internal connections
- Create PortMapping for each exposed port
**Reference**: spec.md US2 acceptance scenario 2
**Verification**: Unit test with mock connected nodes
**Time**: 25 minutes

### T036 [US2] Add composition integration test
**Story**: US2 - Compose Selected Nodes
**File**: `tests/integration/composition_tests.rs` (NEW)
**Action**: Create end-to-end test:
- Load graph with 3 connected test nodes
- Select all 3
- Call compose
- Verify composite node created
- Verify original nodes removed
- Verify exposed ports correct
**Verification**: `cargo test composition` passes
**Time**: 20 minutes

**Phase 6 Checkpoint**: ✓ US2 Complete - Users can compose selected nodes

**Test US2 Independently**:
1. Select 3 connected nodes
2. Click "Compose" button
3. Verify: new composite node appears
4. Verify: original nodes removed
5. Verify: composite has correct input/output ports
6. Test error case: select disconnected nodes → error message

---

## Phase 7: User Story 4 - Drill Into Composite Node (Priority P2)

**Goal**: Users can drill into composite nodes to view internal structure

**Independent Test**: Double-click composite node → view switches to show only internal nodes → click Exit → return to main canvas

### T037 [US4] Implement ViewContext enum
**Story**: US4 - Drill Into Composite Node
**File**: `src/graph/drill_down.rs`
**Action**: Implement:
```rust
pub enum ViewContext {
    MainCanvas,
    DrillDown {
        composite_node_id: NodeId,
        internal_graph: NodeGraphView,
    },
}
```
**Reference**: data-model.md ViewContext section
**Verification**: `cargo check` passes
**Time**: 10 minutes

### T038 [US4] Implement ViewStack struct
**Story**: US4 - Drill Into Composite Node
**File**: `src/graph/drill_down.rs`
**Action**: Implement with methods:
- `new()` - initialize with MainCanvas
- `push(context)` - enter drill-down
- `pop()` - exit drill-down
- `current()` - get current view
- `is_drilled_down()` - check if not on main canvas
**Reference**: data-model.md ViewStack section
**Verification**: Unit test push/pop operations
**Time**: 15 minutes

### T039 [US4] Add ViewStack to App state
**Story**: US4 - Drill Into Composite Node
**File**: `src/ui/app.rs`
**Action**:
- Add field: `view_stack: ViewStack`
- Initialize in `App::new()` with MainCanvas
**Verification**: `cargo check` passes
**Time**: 5 minutes

### T040 [US4] Implement double-click handler for composite nodes
**Story**: US4 - Drill Into Composite Node
**File**: `src/ui/canvas.rs`
**Action**: In node interaction handling:
- Detect double-click on node
- Check if node is `ComponentSpec::Composed`
- If yes: call `drill_into_composite(node_id)`
**Verification**: Debug log on double-click composite node
**Time**: 10 minutes

### T041 [US4] Implement drill_into_composite method
**Story**: US4 - Drill Into Composite Node
**File**: `src/ui/app.rs`
**Action**: Implement method:
- Get composite node data
- Create ViewContext::DrillDown with internal_nodes and internal_edges
- Push to view_stack
**Verification**: View stack depth increases on drill-down
**Time**: 15 minutes

### T042 [US4] Modify canvas rendering to respect ViewContext
**Story**: US4 - Drill Into Composite Node
**File**: `src/ui/canvas.rs`
**Action**: In render method:
- Check `view_stack.current()`
- If MainCanvas: render full graph
- If DrillDown: render only `internal_graph` nodes/edges
**Verification**: Visual test - only internal nodes visible in drill-down
**Time**: 20 minutes

### T043 [US4] Add "Exit Drill-Down" button/breadcrumb
**Story**: US4 - Drill Into Composite Node
**File**: `src/ui/app.rs`
**Action**: At top of UI:
- Show breadcrumb when drilled down: "Main → Composite Name"
- Add "← Exit" button
- On click: `view_stack.pop()`
**Verification**: Click Exit → returns to main canvas
**Time**: 15 minutes

### T044 [US4] Add drill-down integration test
**Story**: US4 - Drill Into Composite Node
**File**: `tests/integration/drill_down_tests.rs` (NEW)
**Action**: Test:
- Create composite node with 3 internal nodes
- Drill into composite
- Verify only internal nodes visible
- Exit drill-down
- Verify main canvas restored
**Verification**: `cargo test drill_down` passes
**Time**: 15 minutes

**Phase 7 Checkpoint**: ✓ US4 Complete - Users can drill into composite nodes

**Test US4 Independently**:
1. Create composite node (from US2)
2. Double-click composite node
3. Verify: view switches to show only 3 internal nodes
4. Verify: breadcrumb shows "Main → Composite Name"
5. Click "Exit" button
6. Verify: return to main canvas with composite node visible

---

## Phase 8: User Story 5 - Composite Node Visual Polish (Priority P3)

**Goal**: Composite nodes have distinct visual styling and show component list in footer

**Independent Test**: View composite node → footer shows component names, distinct visual styling is apparent

### T045 [US5] Add composite node footer rendering
**Story**: US5 - Manage Composite Node Representation
**File**: `src/ui/canvas.rs`
**Action**: In composite node rendering (from T034):
- Extract `component_names` from metadata
- Render footer below node with list of names
- Use smaller font, distinct background
**Reference**: spec.md US5 acceptance scenario 1
**Verification**: Visual test - footer shows component names
**Time**: 15 minutes

### T046 [US5] Add composite node visual theme
**Story**: US5 - Manage Composite Node Representation
**File**: `src/ui/theme.rs`
**Action**: Add CompositeNodeTheme:
- `composite_background`: darker/distinct color
- `composite_border`: thicker border with unique color
- `composite_badge_color`: icon/badge color
- `footer_background`, `footer_text_color`
**Reference**: research.md section 6 color palette
**Verification**: Composite nodes visually distinct from regular nodes
**Time**: 10 minutes

### T047 [US5] Apply composite node visual styling
**Story**: US5 - Manage Composite Node Representation
**File**: `src/ui/canvas.rs`
**Action**: In composite node rendering:
- Apply `composite_background` and `composite_border` from theme
- Add icon/badge (e.g., "C" or composition symbol) in corner
**Verification**: Visual test - composite nodes clearly distinguishable
**Time**: 10 minutes

### T048 [US5] Add port labels with context
**Story**: US5 - Manage Composite Node Representation
**File**: `src/ui/canvas.rs`
**Action**: For composite node ports:
- Show port name from PortMapping
- Add tooltip showing internal node name and port
- Example: "input-data (from Filter.input)"
**Reference**: spec.md US5 acceptance scenario 2
**Verification**: Hover over composite port → tooltip shows internal mapping
**Time**: 15 minutes

**Phase 8 Checkpoint**: ✓ US5 Complete - Composite nodes have polished visual representation

**Test US5 Independently**:
1. Create composite node
2. Verify: footer displays all 3 component names
3. Verify: distinct visual styling (color, border)
4. Verify: ports have descriptive labels
5. Hover port → tooltip shows internal mapping

---

## Phase 9: Polish & Cross-Cutting Concerns

### T049 [Polish] Add keyboard shortcuts
**File**: `src/ui/app.rs`
**Action**: Add shortcuts:
- `Ctrl+Shift+C`: Compose selected nodes
- `ESC`: Cancel selection (already implemented)
- `Backspace`/`Delete`: Exit drill-down view
**Verification**: Keyboard shortcuts work as expected
**Time**: 10 minutes

### T050 [Polish] Add composition progress indicator
**File**: `src/ui/app.rs`
**Action**: For compositions >5 nodes:
- Show spinner/progress during composition
- Use async composition (tokio::spawn_blocking)
**Reference**: research.md section 7 async composition
**Verification**: Large composition shows progress indicator
**Time**: 15 minutes

### T051 [Polish] Add composition undo/redo support
**File**: `src/graph/graph.rs`
**Action**: Integrate with existing undo system (if available):
- Composition creates undo checkpoint
- Undo restores original nodes
**Verification**: Compose → Undo → original nodes restored
**Time**: 20 minutes (depends on existing undo infrastructure)

### T052 [Polish] Add composition preview dialog
**File**: `src/ui/dialogs.rs`
**Action**: Before composition:
- Show dialog with list of nodes to compose
- Preview exposed inputs/outputs
- "Confirm" or "Cancel" buttons
**Reference**: research.md section 5 UI integration
**Verification**: Compose button shows preview first
**Time**: 20 minutes

### T053 [Polish] Optimize selection rendering for large graphs
**File**: `src/ui/canvas.rs`
**Action**: For graphs >100 nodes:
- Implement spatial indexing (quadtree)
- Only check visible nodes for selection
**Reference**: research.md section 7 performance optimization
**Verification**: Selection remains responsive with 500 nodes
**Time**: 30 minutes

### T054 [Polish] Add comprehensive error recovery
**File**: `src/ui/app.rs`
**Action**: Handle edge cases:
- Composition fails mid-way: rollback, don't corrupt graph
- Component files missing: clear error, allow user to locate
- WAC validation fails: explain type mismatch
**Verification**: Error cases handled gracefully, no crashes
**Time**: 20 minutes

### T055 [Polish] Add composition metrics/telemetry
**File**: `src/runtime/wac_integration.rs`
**Action**: Track metrics:
- Composition count, success rate
- Average composition time
- Component size distribution
**Verification**: Metrics can be queried/logged
**Time**: 15 minutes

---

## Task Dependencies & Execution Order

### Critical Path (Sequential Dependencies)

```
Setup (T001-T006)
  ↓
US1: Rectangle Selection (T007-T014)
  ↓
US3: Visual Feedback (T015-T019)
  ↓
Foundation: Validation (T020-T021) [P] Foundation: WAC Integration (T022-T025)
  ↓
US2: Composition (T026-T036)
  ↓
US4: Drill-Down (T037-T044)
  ↓
US5: Visual Polish (T045-T048)
  ↓
Final Polish (T049-T055)
```

### Parallel Execution Opportunities

**Phase 1 (Setup)**: T001, T002, T003, T004, T005 can all run in parallel

**Phase 2 (US1)**: T007 → (T008, T009, T010, T011 can run in parallel after T007) → T012 → T013 → T014

**Phase 3 (US3)**: T015, T016, T017, T018 can run in parallel (different aspects of rendering)

**Foundational**: T020-T021 (validation) [P] T022-T025 (WAC integration) - completely independent

**Phase 6 (US2)**:
- T026, T027 (data structures) [P] T028, T029 (app setup)
- Then T030-T033 (composition logic)
- Then T034, T035 (rendering) [P] T036 (tests)

**Phase 7 (US4)**: T037, T038 (data structures) → T039 → (T040, T041, T042, T043 can be parallelized partially)

**Phase 8 (US5)**: T045, T046, T047, T048 can run in parallel (different visual aspects)

**Phase 9 (Polish)**: Most tasks independent, can run in parallel

## Testing Strategy

**Unit Tests** (inline with implementation):
- T021: Graph validation unit tests
- T025: WAC integration unit tests
- Other unit tests embedded in implementation tasks

**Integration Tests**:
- T036: Composition end-to-end test
- T044: Drill-down end-to-end test

**Manual Testing** (after each phase checkpoint):
- Visual verification of UI changes
- Performance testing (60 FPS, <100ms response)
- Error handling verification

**No TDD Requirement**: Tests are optional and supplementary to implementation. Focus on working feature first.

## Time Estimates by Phase

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| Phase 1: Setup | T001-T006 | 15 minutes |
| Phase 2: US1 Rectangle Selection | T007-T014 | 1 hour 20 minutes |
| Phase 3: US3 Visual Feedback | T015-T019 | 50 minutes |
| **MVP Total (P1 Stories)** | **T001-T019** | **~2.5 hours** |
| Phase 4: Foundation Validation | T020-T021 | 35 minutes |
| Phase 5: Foundation WAC | T022-T025 | 1 hour 15 minutes |
| Phase 6: US2 Composition | T026-T036 | 3 hours |
| **Full Feature (P1+P2)** | **T001-T036** | **~7 hours** |
| Phase 7: US4 Drill-Down | T037-T044 | 1 hour 45 minutes |
| Phase 8: US5 Visual Polish | T045-T048 | 50 minutes |
| Phase 9: Final Polish | T049-T055 | 2 hours 10 minutes |
| **Complete Feature** | **T001-T055** | **~12 hours** |

**Note**: Times are estimates for focused development. Add 20-30% buffer for debugging and iterations.

## MVP Recommendation

**Recommended MVP**: Phases 1-3 (US1 + US3) = ~2.5 hours
- Delivers immediately useful multi-select feature
- No external dependencies (no WAC required)
- Can be released and tested independently
- Provides foundation for composition work

**Extended MVP**: Add Phases 4-6 (US2) = ~7 hours total
- Delivers complete composition workflow
- Requires WAC dependency and test components
- Provides core feature value

## Success Criteria Validation

After implementation, verify against spec.md success criteria:

- ✅ **SC-001**: Users can select 5 nodes in <2 seconds
- ✅ **SC-002**: Composition completes in <1 second for 10 nodes
- ✅ **SC-003**: Visual feedback <100ms (60 FPS maintained)
- ✅ **SC-004**: 95% of valid compositions succeed
- ✅ **SC-005**: Users identify composite nodes in <1 second
- ✅ **SC-006**: Footer displays all component names
- ✅ **SC-007**: Drill-down transition <2 seconds
- ✅ **SC-008**: Exit drill-down <1 second
- ✅ **SC-009**: Internal layout matches original

## Notes

- **Parallelization**: Tasks marked [P] can be parallelized with others in same phase
- **Story Labels**: [US1], [US2], etc. indicate which user story the task belongs to
- **File Paths**: All relative to repository root `/Users/doronila/git/wasmflow_cc/`
- **Testing**: Focus on integration tests at story boundaries, unit tests for complex logic
- **Performance**: Monitor during manual testing, optimize if <60 FPS or >100ms response
