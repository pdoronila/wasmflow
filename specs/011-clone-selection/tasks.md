# Clone Selection - Implementation Tasks

**Feature**: Clone Selection (011-clone-selection)
**Status**: Ready for implementation
**Created**: 2025-10-27

## Task Overview

This document tracks implementation progress for the clone selection feature. Tasks are organized by phase and can be completed sequentially or in parallel where noted.

---

## Phase 1: Core Cloning Logic

**Goal**: Implement node and connection cloning without UI integration

### Task 1.1: Create Duplication Module

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Create new file `src/ui/app/duplication.rs`
- [ ] Add module declaration in `src/ui/app.rs`: `mod duplication;`
- [ ] Add imports:
  ```rust
  use crate::graph::{GraphNode, Connection, Port, NodeGraph};
  use uuid::Uuid;
  use std::collections::{HashMap, HashSet};
  use egui;
  ```
- [ ] Define constants:
  ```rust
  const CLONE_OFFSET: egui::Vec2 = egui::Vec2::new(50.0, 50.0);
  const CLONE_NAME_SUFFIX: &str = " (Clone)";
  ```

**Acceptance**: File compiles, module accessible from `src/ui/app.rs`

---

### Task 1.2: Implement Port Cloning

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Implement `clone_port()` function:
  ```rust
  fn clone_port(original: &Port) -> (Port, Uuid, Uuid) {
      let new_port_id = Uuid::new_v4();
      let cloned_port = Port {
          id: new_port_id,
          name: original.name.clone(),
          port_type: original.port_type.clone(),
          allow_multiple: original.allow_multiple,
          value: original.value.clone(),
      };
      (cloned_port, original.id, new_port_id)
  }
  ```
- [ ] Add unit test `test_clone_port_generates_new_uuid()`
- [ ] Add unit test `test_clone_port_preserves_properties()`

**Acceptance**: Tests pass, port cloning generates unique UUIDs

---

### Task 1.3: Implement Node Cloning

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Implement `clone_node()` function:
  ```rust
  fn clone_node(
      original: &GraphNode,
      offset: egui::Vec2,
      name_suffix: &str,
  ) -> (GraphNode, Uuid, Uuid, HashMap<Uuid, Uuid>) {
      let new_node_id = Uuid::new_v4();
      let mut port_mapping = HashMap::new();

      // Clone inputs
      let cloned_inputs: Vec<Port> = original.inputs.iter()
          .map(|port| {
              let (cloned, old_id, new_id) = clone_port(port);
              port_mapping.insert(old_id, new_id);
              cloned
          })
          .collect();

      // Clone outputs
      let cloned_outputs: Vec<Port> = original.outputs.iter()
          .map(|port| {
              let (cloned, old_id, new_id) = clone_port(port);
              port_mapping.insert(old_id, new_id);
              cloned
          })
          .collect();

      let cloned_node = GraphNode {
          id: new_node_id,
          component_id: original.component_id.clone(),
          display_name: format!("{}{}", original.display_name, name_suffix),
          position: original.position + offset,
          inputs: cloned_inputs,
          outputs: cloned_outputs,
          metadata: original.metadata.clone(),
          capabilities: original.capabilities.clone(),
          creator_data: original.creator_data.clone(),
          composition_data: original.composition_data.clone(),
          selected: true,  // Clone is selected
          execution_state: None,  // Reset execution state
          dirty: false,  // Not dirty yet
      };

      (cloned_node, original.id, new_node_id, port_mapping)
  }
  ```
- [ ] Add unit test `test_clone_node_generates_new_uuid()`
- [ ] Add unit test `test_clone_node_offsets_position()`
- [ ] Add unit test `test_clone_node_appends_name_suffix()`
- [ ] Add unit test `test_clone_node_resets_state_fields()`

**Acceptance**: Tests pass, node cloning works correctly

---

### Task 1.4: Implement Connection Filtering

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Implement `is_internal_connection()` helper:
  ```rust
  fn is_internal_connection(
      conn: &Connection,
      selected_nodes: &HashSet<Uuid>,
  ) -> bool {
      selected_nodes.contains(&conn.from_node)
          && selected_nodes.contains(&conn.to_node)
  }
  ```
- [ ] Add unit test `test_is_internal_connection_both_selected()`
- [ ] Add unit test `test_is_internal_connection_one_external()`

**Acceptance**: Tests pass, connection filtering logic correct

---

### Task 1.5: Implement Connection Cloning

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Implement `clone_internal_connections()` function:
  ```rust
  fn clone_internal_connections(
      connections: &[Connection],
      selected_nodes: &HashSet<Uuid>,
      node_mapping: &HashMap<Uuid, Uuid>,
      port_mapping: &HashMap<Uuid, Uuid>,
  ) -> Vec<Connection> {
      connections
          .iter()
          .filter(|conn| is_internal_connection(conn, selected_nodes))
          .map(|conn| Connection {
              id: Uuid::new_v4(),
              from_node: *node_mapping.get(&conn.from_node).unwrap(),
              from_port: *port_mapping.get(&conn.from_port).unwrap(),
              to_node: *node_mapping.get(&conn.to_node).unwrap(),
              to_port: *port_mapping.get(&conn.to_port).unwrap(),
          })
          .collect()
  }
  ```
- [ ] Add unit test `test_clone_internal_connections_filters_external()`
- [ ] Add unit test `test_clone_internal_connections_maps_ids()`
- [ ] Add unit test `test_clone_internal_connections_generates_new_ids()`

**Acceptance**: Tests pass, connection cloning preserves internal connections

---

### Task 1.6: Implement Main Clone Action

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Implement `get_selected_node_ids()` helper:
  ```rust
  fn get_selected_node_ids(graph: &NodeGraph) -> HashSet<Uuid> {
      graph.nodes
          .iter()
          .filter(|(_, node)| node.selected)
          .map(|(id, _)| *id)
          .collect()
  }
  ```
- [ ] Implement `handle_clone_action()` in `impl App`:
  ```rust
  pub(super) fn handle_clone_action(&mut self) {
      // Get selected nodes
      let selected_nodes = get_selected_node_ids(&self.graph);

      if selected_nodes.is_empty() {
          self.status_message = "No nodes selected".to_string();
          return;
      }

      // Clone nodes and build mappings
      let mut node_mapping = HashMap::new();
      let mut port_mapping = HashMap::new();
      let mut cloned_nodes = Vec::new();

      for node_id in &selected_nodes {
          if let Some(original) = self.graph.nodes.get(node_id) {
              let (cloned, old_id, new_id, ports) =
                  clone_node(original, CLONE_OFFSET, CLONE_NAME_SUFFIX);
              node_mapping.insert(old_id, new_id);
              port_mapping.extend(ports);
              cloned_nodes.push(cloned);
          }
      }

      // Clone internal connections
      let cloned_connections = clone_internal_connections(
          &self.graph.connections,
          &selected_nodes,
          &node_mapping,
          &port_mapping,
      );

      // Clear original selection
      for node in self.graph.nodes.values_mut() {
          node.selected = false;
      }

      // Add cloned nodes (already selected)
      for cloned in cloned_nodes {
          self.graph.nodes.insert(cloned.id, cloned);
      }

      // Add cloned connections
      self.graph.connections.extend(cloned_connections.clone());

      // Mark graph as dirty
      self.graph_dirty = true;

      // Status message
      self.status_message = format!(
          "Cloned {} node(s) with {} connection(s)",
          node_mapping.len(),
          cloned_connections.len()
      );
  }
  ```

**Acceptance**: Method compiles, can be called from app

---

### Task 1.7: Phase 1 Testing

**Steps**:
- [ ] Run unit tests: `cargo test duplication`
- [ ] Manual test: Create graph with single node, select, call `handle_clone_action()`
- [ ] Verify: 2 nodes exist, cloned node at offset position
- [ ] Manual test: Create A→B→C, select all, clone
- [ ] Verify: 6 nodes total, A'→B'→C' exists with 2 connections
- [ ] Manual test: Create A→B→C, select only B, clone
- [ ] Verify: 4 nodes total, B' exists, no connections to B'

**Acceptance**: All unit tests pass, manual tests show correct behavior

---

## Phase 2: Command Pattern for Undo/Redo

**Goal**: Make clone operations undoable/redoable

### Task 2.1: Add Command Variant

**File**: `src/graph/command.rs`

**Steps**:
- [ ] Add `CloneNodes` variant to `Command` enum:
  ```rust
  pub enum Command {
      // ... existing variants
      CloneNodes {
          cloned_node_ids: Vec<Uuid>,
          cloned_nodes: Vec<GraphNode>,
          cloned_connections: Vec<Connection>,
      },
  }
  ```
- [ ] Verify enum compiles with new variant

**Acceptance**: Compiles without errors

---

### Task 2.2: Implement Undo Logic

**File**: `src/ui/app.rs` (or wherever `undo_command()` is implemented)

**Steps**:
- [ ] Find existing undo implementation (search for `Command::RemoveNode`)
- [ ] Add `Command::CloneNodes` case:
  ```rust
  Command::CloneNodes { cloned_node_ids, cloned_connections, .. } => {
      // Remove cloned nodes
      for id in cloned_node_ids {
          self.graph.nodes.remove(id);
      }

      // Remove cloned connections
      let cloned_conn_ids: HashSet<Uuid> = cloned_connections
          .iter()
          .map(|c| c.id)
          .collect();
      self.graph.connections.retain(|c| !cloned_conn_ids.contains(&c.id));

      self.graph_dirty = true;
  }
  ```

**Acceptance**: Undo removes cloned nodes and connections

---

### Task 2.3: Implement Redo Logic

**File**: `src/ui/app.rs` (or wherever `redo_command()` is implemented)

**Steps**:
- [ ] Find existing redo implementation
- [ ] Add `Command::CloneNodes` case:
  ```rust
  Command::CloneNodes { cloned_nodes, cloned_connections, .. } => {
      // Re-add cloned nodes
      for node in cloned_nodes {
          self.graph.nodes.insert(node.id, node.clone());
      }

      // Re-add cloned connections
      self.graph.connections.extend(cloned_connections.clone());

      self.graph_dirty = true;
  }
  ```

**Acceptance**: Redo restores cloned nodes and connections

---

### Task 2.4: Update Clone Action to Use Command

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Modify `handle_clone_action()` to create Command:
  ```rust
  // At end of handle_clone_action(), replace direct graph modification:

  // Create command
  let cloned_node_ids: Vec<Uuid> = cloned_nodes.iter().map(|n| n.id).collect();
  let command = Command::CloneNodes {
      cloned_node_ids: cloned_node_ids.clone(),
      cloned_nodes: cloned_nodes.clone(),
      cloned_connections: cloned_connections.clone(),
  };

  // Push to history
  self.command_history.push(command);

  // Apply command (add to graph)
  for cloned in cloned_nodes {
      self.graph.nodes.insert(cloned.id, cloned);
  }
  self.graph.connections.extend(cloned_connections);
  ```

**Acceptance**: Clone creates command in history

---

### Task 2.5: Phase 2 Testing

**Steps**:
- [ ] Manual test: Clone nodes, press Ctrl+Z
- [ ] Verify: Cloned nodes removed
- [ ] Manual test: Press Ctrl+Y (redo)
- [ ] Verify: Cloned nodes restored
- [ ] Manual test: Clone → Clone again → Undo once
- [ ] Verify: Only second clone removed
- [ ] Manual test: Clone → Undo → Redo → Undo → Redo
- [ ] Verify: Graph state consistent after multiple undo/redo

**Acceptance**: Undo/redo works correctly for clone operations

---

## Phase 3: UI Integration

**Goal**: Add UI button and keyboard shortcut

### Task 3.1: Add Clone Button

**File**: `src/ui/app.rs`

**Steps**:
- [ ] Find toolbar section where "Compose" button is rendered (search for "Compose")
- [ ] After Compose button, add Clone button:
  ```rust
  // After compose button code
  ui.add_space(8.0);

  let selected_count = self.graph.nodes.values().filter(|n| n.selected).count();
  let can_clone = selected_count >= 1 && self.view_stack.is_main_canvas();

  if ui
      .add_enabled(can_clone, egui::Button::new("📋 Clone Selected"))
      .on_hover_text("Clone selected nodes (Ctrl+D)")
      .clicked()
  {
      self.handle_clone_action();
  }
  ```

**Acceptance**: Button appears in toolbar, enables/disables correctly

---

### Task 3.2: Add Keyboard Shortcut

**File**: `src/ui/app.rs`

**Steps**:
- [ ] Find `handle_keyboard_shortcuts()` method (search for "consume_key")
- [ ] Add Ctrl+D handler after existing shortcuts:
  ```rust
  // Clone selected nodes
  if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::D)) {
      let selected_count = self.graph.nodes.values().filter(|n| n.selected).count();
      if selected_count >= 1 && self.view_stack.is_main_canvas() {
          self.handle_clone_action();
      }
  }
  ```

**Acceptance**: Ctrl+D triggers clone when nodes selected

---

### Task 3.3: Add Status Messages

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Update `handle_clone_action()` status messages:
  ```rust
  // At start, if no selection:
  self.status_message = "No nodes selected to clone".to_string();

  // At start, if in drill-down:
  if !self.view_stack.is_main_canvas() {
      self.status_message = "Cannot clone in drill-down view".to_string();
      return;
  }

  // At end, on success:
  self.status_message = format!(
      "Cloned {} node(s) with {} connection(s)",
      cloned_nodes.len(),
      cloned_connections.len()
  );
  ```

**Acceptance**: Status messages appear in status bar

---

### Task 3.4: Phase 3 Testing

**Steps**:
- [ ] Manual test: Select 1 node, click Clone button
- [ ] Verify: Node cloned, status message appears
- [ ] Manual test: Select multiple nodes, press Ctrl+D
- [ ] Verify: Nodes cloned
- [ ] Manual test: No selection, verify button disabled
- [ ] Manual test: Drill into composite, verify button disabled
- [ ] Manual test: Clone in Selection Mode
- [ ] Verify: Works in Selection Mode
- [ ] Manual test: Clone in Normal Mode
- [ ] Verify: Works in Normal Mode

**Acceptance**: All UI interactions work as expected

---

## Phase 4: Edge Cases and Polish

**Goal**: Handle special node types and edge cases

### Task 4.1: Test Special Node Types

**Steps**:
- [ ] Manual test: Create WASM Creator node, add code, clone
- [ ] Verify: `creator_data` cloned (code preserved)
- [ ] Manual test: Create Composite node, clone
- [ ] Verify: `composition_data` cloned (binary blob preserved)
- [ ] Manual test: Create Continuous HTTP Server, start it, clone
- [ ] Verify: Cloned node has `runtime_state = None` (not running)
- [ ] Manual test: Create Constant node with value, clone
- [ ] Verify: Constant value preserved in clone

**Acceptance**: All special node types clone correctly

---

### Task 4.2: Handle Self-Loops

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Create test graph: Node A with output→input connection to itself
- [ ] Select A, clone
- [ ] Verify: A' has self-loop (A'→A')
- [ ] Check: `is_internal_connection()` handles same from/to node

**Acceptance**: Self-loops clone correctly

---

### Task 4.3: Position Edge Case Handling

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Add viewport bounds check (optional enhancement):
  ```rust
  fn ensure_visible_position(
      position: egui::Pos2,
      viewport_rect: egui::Rect,
  ) -> egui::Pos2 {
      if viewport_rect.contains(position) {
          position
      } else {
          viewport_rect.center()
      }
  }
  ```
- [ ] Update `clone_node()` to use viewport if available
- [ ] Or: Keep simple offset (acceptable if offset is reasonable)

**Decision**: Keep simple offset for MVP (document in spec)

**Acceptance**: Cloned nodes appear at predictable location

---

### Task 4.4: Error Handling

**File**: `src/ui/app/duplication.rs`

**Steps**:
- [ ] Add error handling for port mapping failures:
  ```rust
  // In clone_internal_connections(), use ? or unwrap_or_else
  let from_node = node_mapping.get(&conn.from_node)
      .ok_or_else(|| format!("Missing node mapping for {}", conn.from_node))?;
  ```
- [ ] Add error propagation to `handle_clone_action()`
- [ ] Display errors in status bar

**Acceptance**: Errors don't crash, show helpful messages

---

### Task 4.5: Phase 4 Testing

**Steps**:
- [ ] Test all special node types (from 4.1)
- [ ] Test self-loop cloning
- [ ] Test large selection (20+ nodes)
- [ ] Test edge cases (empty graph, single node, no connections)
- [ ] Verify no crashes on any input

**Acceptance**: All edge cases handled gracefully

---

## Phase 5: Documentation and Testing

**Goal**: Document feature and add comprehensive tests

### Task 5.1: Write Integration Tests

**File**: `tests/integration/clone_selection_test.rs` (create if needed)

**Steps**:
- [ ] Create test file
- [ ] Add test: `test_clone_single_node()`
- [ ] Add test: `test_clone_multiple_disconnected_nodes()`
- [ ] Add test: `test_clone_connected_subgraph()`
- [ ] Add test: `test_clone_partial_subgraph()`
- [ ] Add test: `test_clone_with_external_connections()`
- [ ] Add test: `test_clone_undo_redo()`
- [ ] Run: `cargo test --test clone_selection_test`

**Acceptance**: All integration tests pass

---

### Task 5.2: Update CLAUDE.md

**File**: `/home/user/wasmflow/CLAUDE.md`

**Steps**:
- [ ] Add "Clone Selection Guidelines" section:
  ```markdown
  ## Clone Selection Guidelines

  **Location**: `src/ui/app/duplication.rs`

  Users can duplicate selected nodes using the Clone button or Ctrl+D shortcut.

  ### Implementation Details

  - **Offset**: Clones are placed at `(+50px, +50px)` from original position
  - **Naming**: Appends " (Clone)" to display name
  - **Connections**: Only internal connections (both nodes selected) are cloned
  - **External connections**: NOT cloned (ambiguous behavior)
  - **Selection**: Cloned nodes are selected, originals deselected
  - **Undo/Redo**: Supported via `Command::CloneNodes`

  ### Constants

  ```rust
  const CLONE_OFFSET: egui::Vec2 = egui::Vec2::new(50.0, 50.0);
  const CLONE_NAME_SUFFIX: &str = " (Clone)";
  ```

  ### Keyboard Shortcut

  - **Ctrl+D**: Clone selected nodes (available in both Normal and Selection modes)
  ```
- [ ] Commit changes

**Acceptance**: CLAUDE.md updated with guidelines

---

### Task 5.3: Create Demo Graph

**File**: `tests/demo/clone_selection_demo.json` (create if needed)

**Steps**:
- [ ] Create demo graph in app with:
  - 3 connected nodes (A→B→C)
  - 2 disconnected nodes (D, E)
  - 1 WASM Creator node (F)
- [ ] Save graph as `tests/demo/clone_selection_demo.json`
- [ ] Add README explaining demo

**Acceptance**: Demo graph loads and demonstrates feature

---

### Task 5.4: Run Full Test Suite

**Steps**:
- [ ] Run all tests: `cargo test`
- [ ] Verify: All tests pass
- [ ] Run clippy: `cargo clippy`
- [ ] Verify: No warnings
- [ ] Build release: `cargo build --release`
- [ ] Verify: Compiles without errors

**Acceptance**: All tests pass, no warnings, builds successfully

---

### Task 5.5: Manual Testing Checklist

**Steps**:
- [ ] Clone single constant node
- [ ] Clone multiple disconnected nodes
- [ ] Clone connected math pipeline (3+ nodes)
- [ ] Clone WASM Creator node (verify creator data preserved)
- [ ] Clone Composite node (verify composition binary preserved)
- [ ] Clone Continuous HTTP Server node (verify runtime state reset)
- [ ] Verify Ctrl+D shortcut works
- [ ] Verify button appears/disappears based on selection
- [ ] Test in Selection Mode
- [ ] Test in Normal Mode
- [ ] Verify undo removes clones
- [ ] Verify redo restores clones
- [ ] Verify clones are selected after creation
- [ ] Verify original selection is cleared
- [ ] Verify clones appear at offset position
- [ ] Test with large selection (10+ nodes)

**Acceptance**: All manual tests pass

---

## Final Verification

### Pre-Commit Checklist

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] No clippy warnings
- [ ] CLAUDE.md updated
- [ ] Demo graph created
- [ ] Feature works in both Normal and Selection modes
- [ ] Undo/redo works correctly
- [ ] No regressions in existing functionality
- [ ] Code follows existing patterns (composition.rs as reference)
- [ ] Status messages provide clear feedback

### Commit Message Template

```
feat: Add clone selection feature

Allows users to duplicate selected nodes and internal connections
using Clone button or Ctrl+D keyboard shortcut.

Features:
- Clone single or multiple nodes
- Preserve internal connections
- Offset cloned nodes by (50, 50) pixels
- Append " (Clone)" to display names
- Full undo/redo support via Command::CloneNodes
- Works in both Normal and Selection modes

Technical details:
- New module: src/ui/app/duplication.rs
- New command variant: Command::CloneNodes
- Keyboard shortcut: Ctrl+D
- UI: Clone button in toolbar (next to Compose)

Tests:
- Unit tests for clone logic
- Integration tests for various graph topologies
- Manual testing checklist completed

Closes #[issue-number]
```

---

## Progress Tracking

**Started**: [Date]
**Phase 1 Complete**: [Date]
**Phase 2 Complete**: [Date]
**Phase 3 Complete**: [Date]
**Phase 4 Complete**: [Date]
**Phase 5 Complete**: [Date]
**Committed**: [Date]

---

## Notes

Add implementation notes, blockers, or discoveries here as you work through tasks.

**Example**:
- Phase 1.3: Discovered `creator_data` needs deep clone (not just reference)
- Phase 2.1: Command enum required Clone trait implementation
- Phase 3.2: Ctrl+D shortcut conflicts with [other feature] - resolved by [solution]
