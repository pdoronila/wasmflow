# Implementation Tasks: Add Composite Node Naming

**Change ID**: `add-composite-node-naming`
**Estimated Effort**: 4-6 hours

---

## Phase 1: Data Model & Command System

### Task 1.1: Add RenameNode command variant
- [ ] Open `src/ui/app/commands.rs`
- [ ] Add `RenameNode { node_id: Uuid, old_name: String, new_name: String }` variant to `Command` enum
- [ ] Implement `execute()` method:
  - Find node by `node_id` in graph
  - Verify node is a composite (has `composition_data`)
  - Update `node.display_name` to `new_name`
  - Update `node.composition_data.name` to `new_name`
  - Mark graph as dirty
- [ ] Implement `undo()` method:
  - Find node by `node_id`
  - Update `node.display_name` to `old_name`
  - Update `node.composition_data.name` to `old_name`
  - Mark graph as dirty
- [ ] Add unit tests for execute/undo/redo flow

**Validation**:
- Rename command compiles without errors
- Unit tests pass: `cargo test commands::tests::rename_node`

**Files Modified**: `src/ui/app/commands.rs`

---

## Phase 2: UI Dialog for Naming

### Task 2.1: Create CompositeNameDialog struct
- [ ] Open `src/ui/dialogs.rs`
- [ ] Add `CompositeNameDialog` struct with fields:
  - `is_open: bool`
  - `name: String` (editable name)
  - `mode: DialogMode` (enum: Create or Rename)
  - `validation_error: Option<String>`
  - `result: Option<CompositeNameAction>` (enum: Confirmed, Cancelled)
- [ ] Add `DialogMode` enum: `Create` and `Rename`
- [ ] Add `CompositeNameAction` enum: `Confirmed(String)` and `Cancelled`
- [ ] Implement `CompositeNameDialog::new()` constructor
- [ ] Implement `open_for_creation(&mut self, default_name: String)` method
- [ ] Implement `open_for_rename(&mut self, current_name: String)` method

**Validation**: Code compiles, dialog struct is well-formed

**Files Modified**: `src/ui/dialogs.rs`

---

### Task 2.2: Implement dialog UI with validation
- [ ] In `src/ui/dialogs.rs`, implement `show(&mut self, ctx: &egui::Context) -> Option<CompositeNameAction>`
- [ ] Dialog title: "Name Your Composite" (Create) or "Rename Composite Node" (Rename)
- [ ] Add text field for name editing with `ui.text_edit_singleline(&mut self.name)`
- [ ] Implement validation on button click:
  - Trim whitespace: `let trimmed = self.name.trim();`
  - Check for empty: `if trimmed.is_empty() { set error }`
  - If valid: return `Some(CompositeNameAction::Confirmed(trimmed.to_string()))`
- [ ] Show inline error message if `validation_error.is_some()`
- [ ] Add "Create"/"Rename" button (label depends on mode)
- [ ] Add "Cancel" button that returns `Some(CompositeNameAction::Cancelled)`
- [ ] Make dialog keyboard-navigable (Enter to confirm, Escape to cancel)
- [ ] Use `.anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)` for centering

**Validation**:
- Dialog appears correctly in UI
- Validation rejects empty names with error message
- Special characters are accepted

**Files Modified**: `src/ui/dialogs.rs`

---

### Task 2.3: Add dialog instance to WasmFlowApp
- [ ] Open `src/ui/app.rs`
- [ ] Add field `composite_name_dialog: CompositeNameDialog` to `WasmFlowApp` struct
- [ ] Initialize in `WasmFlowApp::new()`: `composite_name_dialog: CompositeNameDialog::new()`
- [ ] In `update()` method, call `self.composite_name_dialog.show(ctx)` and handle returned action
- [ ] Store dialog result for use in composition workflow

**Validation**: App compiles with new dialog field

**Files Modified**: `src/ui/app.rs`

---

## Phase 3: Integrate Naming into Composition Workflow

### Task 3.1: Add name dialog to composition flow
- [ ] Open `src/ui/app/composition.rs`
- [ ] In `handle_compose_action()`, after validation passes (line 60), check if dialog is open
- [ ] If dialog not open yet, open dialog: `self.composite_name_dialog.open_for_creation("Composite Node".to_string())`
- [ ] Return early from `handle_compose_action()` to wait for user input
- [ ] Add new method `handle_composite_name_confirmed(&mut self, name: String)` that:
  - Continues composition with selected nodes
  - Uses `name` for both `display_name` and `composition_data.name` (line 141, 175)
  - Removes hardcoded "Composite Node" string
- [ ] In `src/ui/app.rs` `update()`, when dialog returns `Confirmed(name)`, call `handle_composite_name_confirmed(name)`
- [ ] When dialog returns `Cancelled`, clear composition state and return

**Validation**:
- Click "Compose" shows name dialog
- Entering name creates composite with custom name
- Cancel aborts composition

**Files Modified**: `src/ui/app/composition.rs`, `src/ui/app.rs`

---

## Phase 4: Right-Click Context Menu for Rename

### Task 4.1: Add context menu infrastructure
- [ ] Investigate where right-click handling exists (likely `src/ui/canvas.rs` or `src/ui/selection.rs`)
- [ ] If context menu doesn't exist, add right-click detection:
  - `if response.secondary_clicked() { /* context menu */ }`
  - Store right-clicked node ID
- [ ] Create context menu using `egui::menu::menu_button()` or `egui::popup_below_widget()`

**Validation**: Right-click on node shows a basic context menu

**Files Modified**: `src/ui/canvas.rs` or `src/ui/selection.rs`

---

### Task 4.2: Add "Rename" option for composite nodes
- [ ] In context menu rendering code, check if clicked node is composite:
  - `if let Some(node) = graph.nodes.get(&clicked_node_id)`
  - `if node.composition_data.is_some()`
- [ ] If composite, show "Rename" menu item
- [ ] If non-composite, do NOT show "Rename" option
- [ ] When "Rename" clicked:
  - Get current node name
  - Open dialog: `self.composite_name_dialog.open_for_rename(node.display_name.clone())`
  - Store clicked node ID for later use

**Validation**:
- Right-click on composite node shows "Rename"
- Right-click on regular node does NOT show "Rename"
- Clicking "Rename" opens dialog with current name pre-filled

**Files Modified**: `src/ui/canvas.rs` or `src/ui/selection.rs`

---

### Task 4.3: Handle rename confirmation
- [ ] In `src/ui/app.rs` `update()`, handle dialog result when in Rename mode
- [ ] When dialog returns `Confirmed(new_name)`:
  - Get old name from the clicked node
  - Create command: `Command::RenameNode { node_id, old_name, new_name }`
  - Execute command: `self.execute_command(command)`
  - Clear clicked node ID state
- [ ] When dialog returns `Cancelled`, clear state and do nothing

**Validation**:
- Rename via context menu updates node name
- Change is visible immediately on canvas
- Undo (Ctrl+Z) reverts the rename

**Files Modified**: `src/ui/app.rs`

---

## Phase 5: Name Propagation & Consistency

### Task 5.1: Update drill-down breadcrumb
- [ ] Open `src/graph/drill_down.rs` or wherever breadcrumb is rendered
- [ ] Find where breadcrumb text is set (currently uses hardcoded string or `node.display_name`)
- [ ] Ensure breadcrumb uses `node.display_name` (not hardcoded "Composite Node")
- [ ] Test: Drill down shows custom name in breadcrumb

**Validation**: Drill-down breadcrumb shows "Data Processor" not "Composite Node"

**Files Modified**: `src/graph/drill_down.rs` or `src/ui/app/composition.rs`

---

### Task 5.2: Update status messages
- [ ] Open `src/ui/app/composition.rs`
- [ ] In `handle_drill_down()` (line 359), verify status message uses `node.display_name`
- [ ] Check any other status messages that reference composite nodes
- [ ] Replace any hardcoded "Composite Node" strings with `node.display_name`

**Validation**: Status bar shows "Viewing internal structure of 'Custom Name'"

**Files Modified**: `src/ui/app/composition.rs`

---

### Task 5.3: Verify port mapping names
- [ ] Open `src/ui/app/composition.rs`
- [ ] Review `aggregate_boundary_ports()` method (line 249, 287)
- [ ] Verify external port names use `node.display_name` (already does)
- [ ] Test: Renamed composite shows updated port names (e.g., "New Name.input")

**Validation**: Port mappings reflect renamed composite name

**Files Modified**: None (already correct)

---

## Phase 6: Testing & Documentation

### Task 6.1: Write unit tests
- [ ] Add test in `src/ui/app/commands.rs`:
  - `test_rename_node_command()`
  - `test_rename_node_undo_redo()`
  - `test_rename_non_composite_fails()`
- [ ] Add test in `src/ui/dialogs.rs`:
  - `test_composite_name_dialog_validation()`

**Validation**: `cargo test` passes all tests

**Files Modified**: `src/ui/app/commands.rs`, `src/ui/dialogs.rs`

---

### Task 6.2: Integration testing
- [ ] Manual test: Create composite with custom name "Image Processor"
- [ ] Manual test: Save graph, reload, verify name persists
- [ ] Manual test: Rename composite via context menu
- [ ] Manual test: Undo rename, redo rename
- [ ] Manual test: Drill down shows custom name in breadcrumb
- [ ] Manual test: Empty name rejected with error message
- [ ] Manual test: Special characters in name accepted
- [ ] Manual test: Context menu only shows "Rename" for composites

**Validation**: All manual tests pass

---

### Task 6.3: Update CLAUDE.md guidelines
- [ ] Open `CLAUDE.md`
- [ ] Add section "Composite Node Naming Guidelines" after "Clone Selection Guidelines"
- [ ] Document:
  - Users can name composites at creation time
  - Users can rename via right-click context menu
  - Undo/redo supported via `Command::RenameNode`
  - Validation: non-empty strings only
  - Name stored in both `display_name` and `composition_data.name`

**Validation**: Documentation is clear and complete

**Files Modified**: `CLAUDE.md`

---

## Phase 7: Final Review

### Task 7.1: Code review checklist
- [ ] All hardcoded "Composite Node" strings replaced with variables
- [ ] Dialog validation prevents empty names
- [ ] Undo/redo works correctly
- [ ] No memory leaks (dialog state cleaned up)
- [ ] No clippy warnings: `cargo clippy`
- [ ] All tests pass: `cargo test`

**Validation**: `cargo clippy` and `cargo test` pass cleanly

---

### Task 7.2: Build and smoke test
- [ ] Run `cargo build --release`
- [ ] Launch application
- [ ] Create 3-node workflow and compose with custom name
- [ ] Rename composite via context menu
- [ ] Test undo/redo
- [ ] Save and reload graph
- [ ] Verify all features work end-to-end

**Validation**: Release build works without errors

---

## Rollback Plan

If critical issues discovered:
1. Revert changes to `src/ui/app/composition.rs` (restore hardcoded "Composite Node")
2. Disable context menu "Rename" option
3. Keep `Command::RenameNode` for future use

The change is non-breaking since `CompositionData.name` field already exists.

---

## Estimated Time Breakdown

- Phase 1 (Command System): 1 hour
- Phase 2 (Dialog UI): 1.5 hours
- Phase 3 (Composition Integration): 1 hour
- Phase 4 (Context Menu): 1.5 hours
- Phase 5 (Name Propagation): 0.5 hours
- Phase 6 (Testing): 1 hour
- Phase 7 (Review): 0.5 hours

**Total**: ~6 hours

---

## Dependencies Between Tasks

- Task 2.3 depends on 2.1, 2.2
- Task 3.1 depends on 2.3
- Task 4.2 depends on 4.1
- Task 4.3 depends on 4.2 and 1.1
- Task 6.1 depends on all implementation tasks
- Task 7.1 depends on all tasks

**Parallelizable**:
- Phase 1 and Phase 2 can be done in parallel
- Task 5.1, 5.2, 5.3 can be done in parallel
