# Feature Specification: Clone Selection

**Feature ID**: 011-clone-selection
**Status**: Planning
**Created**: 2025-10-27
**Author**: Claude Code

## Overview

Add "clone selection" functionality to the existing selection mode, allowing users to duplicate selected nodes (and their internal connections) in a single operation. This provides a faster workflow than manually recreating node configurations.

## Motivation

**User Need**: When building complex graphs, users often need to duplicate patterns or subgraphs. Currently, there's no way to copy/duplicate nodes - users must manually recreate each node and reconnect them.

**Use Cases**:
1. **Pattern replication**: User creates a processing pipeline (e.g., validate → transform → output) and wants to apply it to multiple data sources
2. **Experimentation**: User wants to test variations of a subgraph without losing the original
3. **Template instantiation**: User has a common pattern (e.g., HTTP fetch → JSON parse → error handling) used multiple times
4. **Parallel processing**: User wants identical processing chains for different inputs

**Design Tool Precedent**: Similar to Cmd+D (duplicate) in Figma, Sketch, Adobe tools

## Current State Analysis

### Existing Selection Mode Capabilities

**What Works** (as of commit c089dac):
- ✅ Rectangle selection in dedicated Selection Mode
- ✅ Visual feedback (blue rectangle overlay)
- ✅ Selected nodes tracked via `GraphNode.selected: bool`
- ✅ Selection state persists until cleared (ESC, mode switch, or composition)
- ✅ Compose button appears when ≥2 nodes selected and connected
- ✅ Composition workflow removes original nodes and creates composite

**Key Files**:
- `src/ui/selection.rs` - SelectionState, CanvasMode enums
- `src/ui/canvas.rs` - Rectangle rendering, drag handling, mode toggle
- `src/ui/canvas/selection.rs` - find_nodes_in_rect() helper
- `src/ui/app/composition.rs` - Reference for working with selected nodes
- `src/graph/node.rs` - GraphNode with `selected` field

### What's Missing

**No existing duplication functionality**:
- ❌ No copy/paste system
- ❌ No clipboard implementation
- ❌ No node duplication workflow

**What needs to be built**:
1. Node cloning logic (with new UUIDs for node and ports)
2. Connection cloning (internal connections only)
3. External connection preservation (connections to/from outside selection)
4. Position offsetting (to avoid overlap)
5. Undo/redo support via Command pattern
6. UI button and keyboard shortcut

## Design Decisions

### 1. Clone Scope

**Decision**: Clone selected nodes + internal connections ONLY

**Rationale**:
- **Internal connections** (both nodes selected): Should be cloned to preserve subgraph structure
- **External connections** (one node outside selection): Should NOT be cloned - ambiguous behavior
  - If cloned: Creates unexpected duplicate connections
  - If not cloned: Leaves cloned nodes disconnected (current decision)
- User can manually reconnect cloned nodes to external graph

**Example**:
```
Original:
  [A] → [B] → [C] → [D]
        ↑           ↑
    (selected)  (selected)

After cloning B and C:
  [A] → [B] → [C] → [D]

        [B'] → [C']     (internal B'→C' connection preserved)
                        (external A→B' and C'→D not created)
```

### 2. Position Offset

**Decision**: Offset cloned nodes by (+50px, +50px) from original positions

**Rationale**:
- Makes clones immediately visible (not hidden under originals)
- Standard offset used in design tools
- Can be adjusted if needed via constant

**Alternative considered**: Offset to mouse position (rejected - would require tracking mouse and might place outside viewport)

### 3. Display Name

**Decision**: Append " (Clone)" to `display_name` for cloned nodes

**Rationale**:
- Distinguishes clones from originals in node picker and canvas
- User can rename if desired
- Common pattern in software (Windows "Copy of...", macOS "... copy")

**Example**:
- Original: "HTTP Fetch"
- Clone: "HTTP Fetch (Clone)"

### 4. Selection Behavior After Clone

**Decision**: Clear original selection, select cloned nodes instead

**Rationale**:
- Focuses user attention on cloned nodes
- Allows immediate drag/positioning of clones
- Allows chaining clone operations (clone the clone)
- Matches behavior of paste in design tools

### 5. UI Integration

**Decision**: Add "Clone" button next to "Compose" button + Ctrl+D keyboard shortcut

**Rationale**:
- Visible in UI for discoverability
- Available in Selection Mode context
- Ctrl+D is familiar (duplicate in design tools)
- Button shows when ≥1 node selected (less restrictive than Compose)

### 6. Undo/Redo Support

**Decision**: Implement via new `Command::CloneNodes` variant

**Rationale**:
- Consistent with existing undo/redo system
- Single undo removes all cloned nodes
- Single redo restores all cloned nodes
- Maintains command history for complex workflows

## Implementation Plan

### Phase 1: Core Cloning Logic

**Goal**: Implement node and connection cloning without UI

**Tasks**:

1. **Create duplication module** (`src/ui/app/duplication.rs`)
   - Add `handle_clone_action(&mut self)` method
   - Extract selected nodes: `graph.nodes.iter().filter(|n| n.selected)`
   - Validate: at least 1 node selected, all in main canvas

2. **Implement node cloning**
   ```rust
   fn clone_node(original: &GraphNode, offset: egui::Vec2) -> (GraphNode, HashMap<Uuid, Uuid>)
   ```
   - Generate new node UUID
   - Clone all fields (component_id, display_name, metadata, etc.)
   - Append " (Clone)" to display_name
   - Offset position by `offset`
   - Generate new UUIDs for all ports (inputs and outputs)
   - Return (cloned_node, port_mapping: old_port_id → new_port_id)
   - Reset state fields: `selected = true`, `execution_state = None`, `dirty = false`

3. **Implement connection cloning**
   ```rust
   fn clone_internal_connections(
       connections: &[Connection],
       selected_nodes: &HashSet<Uuid>,
       node_mapping: &HashMap<Uuid, Uuid>,
       port_mapping: &HashMap<Uuid, Uuid>,
   ) -> Vec<Connection>
   ```
   - Filter connections where BOTH from_node and to_node are in selected set
   - For each internal connection:
     - Generate new connection UUID
     - Map from_node to cloned node ID
     - Map to_node to cloned node ID
     - Map from_port to cloned port ID
     - Map to_port to cloned port ID

4. **Integrate into graph**
   - Add cloned nodes to `self.graph.nodes`
   - Add cloned connections to `self.graph.connections`
   - Clear original selection
   - Select cloned nodes
   - Mark graph as dirty

**Success Criteria**:
- ✅ Single node can be cloned with offset position
- ✅ Node has new UUID (no conflicts)
- ✅ Ports have new UUIDs (no conflicts)
- ✅ Multiple disconnected nodes can be cloned
- ✅ Connected nodes clone with internal connections preserved
- ✅ External connections are NOT cloned

**Testing**:
- Unit tests for `clone_node()` (UUID uniqueness, offset, name)
- Unit tests for `clone_internal_connections()` (filtering, mapping)
- Integration test: Clone graph with 3 nodes (A→B→C, select B), verify 1 connection NOT cloned

### Phase 2: Command Pattern for Undo/Redo

**Goal**: Make clone operations undoable

**Tasks**:

1. **Add Command variant** (`src/graph/command.rs`)
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

2. **Implement undo** (in `src/graph/mod.rs` or `src/ui/app.rs`)
   ```rust
   Command::CloneNodes { cloned_node_ids, .. } => {
       // Remove cloned nodes
       for id in cloned_node_ids {
           self.graph.nodes.remove(id);
       }
       // Remove cloned connections
       self.graph.connections.retain(|c| !cloned_connection_ids.contains(&c.id));
   }
   ```

3. **Implement redo**
   ```rust
   Command::CloneNodes { cloned_nodes, cloned_connections, .. } => {
       // Re-add cloned nodes
       for node in cloned_nodes {
           self.graph.nodes.insert(node.id, node.clone());
       }
       // Re-add cloned connections
       self.graph.connections.extend(cloned_connections.clone());
   }
   ```

4. **Update handle_clone_action()**
   - Create Command::CloneNodes instance
   - Push to command history: `self.command_history.push(command)`
   - Apply command (add nodes/connections to graph)

**Success Criteria**:
- ✅ Ctrl+Z after clone removes all cloned nodes and connections
- ✅ Ctrl+Y (redo) restores all cloned nodes and connections
- ✅ Multiple clone operations stack in history
- ✅ Undo/redo maintains graph integrity (no orphaned connections)

**Testing**:
- Integration test: Clone → Undo → verify original selection restored
- Integration test: Clone → Undo → Redo → verify clones restored
- Integration test: Clone → Clone again → Undo → verify only second clone removed

### Phase 3: UI Integration

**Goal**: Add UI button and keyboard shortcut

**Tasks**:

1. **Add Clone button** (`src/ui/app.rs`, in toolbar section)
   - Position: Next to "Compose" button
   - Label: "📋 Clone Selected" or "🗐 Clone"
   - Enable condition: `selected_count >= 1 && self.view_stack.is_main_canvas()`
   - Action: Call `self.handle_clone_action()`

2. **Add keyboard shortcut** (`src/ui/app.rs`, in handle_keyboard_shortcuts())
   ```rust
   if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::D)) {
       if !self.graph.nodes.is_empty() {
           self.handle_clone_action();
       }
   }
   ```

3. **Add status message**
   - Success: "Cloned {count} node(s) with {conn_count} connection(s)"
   - Error: "No nodes selected" or "Cannot clone in drill-down view"

4. **Update help text** (if exists)
   - Add Ctrl+D to keyboard shortcut documentation
   - Add Clone to toolbar help text

**Success Criteria**:
- ✅ Clone button appears when ≥1 node selected in main canvas
- ✅ Clone button disabled when no selection or in drill-down mode
- ✅ Ctrl+D triggers clone operation
- ✅ Status message shows clone result
- ✅ Button uses consistent styling with existing toolbar

**Testing**:
- Manual test: Click Clone button with 1 node selected
- Manual test: Press Ctrl+D with multiple nodes selected
- Manual test: Verify button disabled in drill-down mode
- Manual test: Verify Ctrl+D works in Selection Mode

### Phase 4: Edge Cases and Polish

**Goal**: Handle edge cases and improve UX

**Tasks**:

1. **Handle special node types**
   - WASM Creator nodes: Clone `creator_data` field
   - Constant nodes: Clone `constant_value`
   - Continuous nodes: Reset `runtime_state` (don't clone running state)
   - Composite nodes: Clone `composition_data` (full binary blob)

2. **Position offset refinement**
   - Check if offset position is outside viewport
   - If outside: Center clones in current viewport
   - If overlapping many nodes: Find empty space (optional enhancement)

3. **Port type preservation**
   - Ensure `PortType` enum variants preserved
   - Ensure `allow_multiple` flag preserved
   - Ensure port metadata preserved

4. **Connection edge cases**
   - Handle self-loops (node connected to itself)
   - Handle multiple connections between same node pair
   - Verify bidirectional connections (rare but possible)

5. **Selection state preservation**
   - Ensure cloned nodes are selected after clone
   - Ensure original nodes are deselected
   - Clear selection if clone fails

6. **Error handling**
   - Handle UUID collision (extremely rare, but check)
   - Handle port mapping failures
   - Show error message in status bar

**Success Criteria**:
- ✅ WASM Creator nodes clone with full creator data
- ✅ Composite nodes clone with composition binary
- ✅ Continuous nodes don't clone running state
- ✅ Self-loops are cloned correctly
- ✅ Clones appear in viewport (not off-screen)
- ✅ Error messages guide user to resolution

**Testing**:
- Test clone WASM Creator node (verify creator_data copied)
- Test clone Continuous HTTP Server node (verify runtime_state reset)
- Test clone Composite node (verify composition_data copied)
- Test clone with self-loop (A→A becomes A'→A')
- Test clone large selection (verify offset keeps nodes visible)

### Phase 5: Documentation and Testing

**Goal**: Document feature and add comprehensive tests

**Tasks**:

1. **Add integration tests** (`tests/integration/clone_selection_test.rs`)
   - Clone single node
   - Clone multiple disconnected nodes
   - Clone connected subgraph (A→B→C, select all)
   - Clone partial subgraph (A→B→C, select B only)
   - Clone with external connections (verify not cloned)
   - Undo/redo clone operation

2. **Add unit tests**
   - `clone_node()` - UUID generation, offset, name append
   - `clone_internal_connections()` - filtering, mapping
   - Port UUID uniqueness
   - Node UUID uniqueness

3. **Update CLAUDE.md**
   - Add "Clone Selection Guidelines" section
   - Document offset constant
   - Document name suffix pattern
   - Document external connection behavior

4. **Create demo graph** (`tests/demo/clone_selection_demo.json`)
   - Pre-configured graph demonstrating clone use cases
   - Includes connected and disconnected nodes
   - Includes various node types (constant, math, WASM)

**Success Criteria**:
- ✅ All integration tests pass
- ✅ All unit tests pass
- ✅ CLAUDE.md updated with guidelines
- ✅ Demo graph loads and demonstrates feature

**Testing**:
- Run full test suite: `cargo test`
- Manual verification of demo graph

## Technical Specifications

### Data Structures

**CloneResult** (internal helper)
```rust
struct CloneResult {
    cloned_nodes: Vec<GraphNode>,
    cloned_connections: Vec<Connection>,
    node_mapping: HashMap<Uuid, Uuid>,  // old_id → new_id
    port_mapping: HashMap<Uuid, Uuid>,  // old_port_id → new_port_id
}
```

**Command Variant**
```rust
pub enum Command {
    CloneNodes {
        cloned_node_ids: Vec<Uuid>,
        cloned_nodes: Vec<GraphNode>,
        cloned_connections: Vec<Connection>,
    },
}
```

### Constants

```rust
// In src/ui/app/duplication.rs
const CLONE_OFFSET: egui::Vec2 = egui::Vec2::new(50.0, 50.0);
const CLONE_NAME_SUFFIX: &str = " (Clone)";
```

### Function Signatures

```rust
// Main entry point
pub(super) fn handle_clone_action(&mut self);

// Core cloning logic
fn clone_node(
    original: &GraphNode,
    offset: egui::Vec2,
    name_suffix: &str,
) -> (GraphNode, HashMap<Uuid, Uuid>);

fn clone_port(original: &Port) -> (Port, Uuid);

fn clone_internal_connections(
    connections: &[Connection],
    selected_nodes: &HashSet<Uuid>,
    node_mapping: &HashMap<Uuid, Uuid>,
    port_mapping: &HashMap<Uuid, Uuid>,
) -> Vec<Connection>;

// Helpers
fn is_internal_connection(
    conn: &Connection,
    selected_nodes: &HashSet<Uuid>,
) -> bool;

fn get_selected_node_ids(graph: &NodeGraph) -> HashSet<Uuid>;
```

## UI/UX Specifications

### Button Design

**Location**: Toolbar (after Compose button)
**Label**: "📋 Clone Selected" (or "🗐 Clone")
**Enabled when**:
- At least 1 node selected
- Viewing main canvas (not drill-down)
- Not during active execution

**Visual states**:
- **Enabled**: Blue button with icon
- **Disabled**: Grayed out
- **Hover**: Highlight with tooltip "Clone selected nodes (Ctrl+D)"

### Keyboard Shortcut

**Primary**: Ctrl+D (Cmd+D on macOS)
**Alternative**: None (keep it simple)

**Behavior**:
- Works in both Normal and Selection modes
- Clones currently selected nodes
- No-op if no selection

### Visual Feedback

**During clone**:
- Brief flash/highlight on cloned nodes (optional)
- Status bar message: "Cloned X node(s)"

**After clone**:
- Cloned nodes are selected (blue outline)
- Original nodes are deselected
- Can immediately drag cloned nodes

## Testing Strategy

### Unit Tests

**File**: `src/ui/app/duplication.rs` (inline tests)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_clone_node_generates_new_uuid() { }

    #[test]
    fn test_clone_node_offsets_position() { }

    #[test]
    fn test_clone_node_appends_name_suffix() { }

    #[test]
    fn test_clone_port_generates_new_uuid() { }

    #[test]
    fn test_clone_internal_connections_filters_external() { }

    #[test]
    fn test_clone_internal_connections_maps_ids() { }
}
```

### Integration Tests

**File**: `tests/integration/clone_selection_test.rs`

```rust
#[test]
fn test_clone_single_node() {
    // Setup: Graph with single node
    // Action: Clone node
    // Assert: 2 nodes in graph, cloned node has offset position
}

#[test]
fn test_clone_connected_subgraph() {
    // Setup: A→B→C (all selected)
    // Action: Clone
    // Assert: A'→B'→C' with 2 internal connections
}

#[test]
fn test_clone_partial_subgraph() {
    // Setup: A→B→C (only B selected)
    // Action: Clone
    // Assert: B' exists, no connections cloned
}

#[test]
fn test_clone_undo_redo() {
    // Setup: Graph with nodes
    // Action: Clone → Undo → Redo
    // Assert: Clones removed by undo, restored by redo
}
```

### Manual Testing Checklist

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

## Success Criteria

### Functional Requirements

- ✅ User can clone selected node(s) via button click
- ✅ User can clone selected node(s) via Ctrl+D
- ✅ Cloned nodes have unique UUIDs (no conflicts)
- ✅ Cloned nodes appear at offset position (visible)
- ✅ Internal connections are cloned
- ✅ External connections are NOT cloned
- ✅ Clone operation is undoable (Ctrl+Z)
- ✅ Clone operation is redoable (Ctrl+Y)
- ✅ Status message confirms clone operation
- ✅ Cloned nodes are selected after clone
- ✅ Works in both Normal and Selection modes

### UX Requirements

- ✅ Clone button visible when nodes selected
- ✅ Clone button disabled when no selection
- ✅ Ctrl+D shortcut documented (tooltip, help)
- ✅ Status messages guide user
- ✅ No unexpected behavior (crashes, data loss)
- ✅ Performance: <100ms for typical operations (<50 nodes)

### Code Quality Requirements

- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Code follows existing patterns (composition.rs as reference)
- ✅ Clippy warnings addressed
- ✅ Documentation complete (CLAUDE.md updated)
- ✅ No regressions in existing functionality

## Non-Goals (Out of Scope)

**Not included in this feature**:
- ❌ Copy/paste across graphs (clipboard serialization)
- ❌ Copy/paste to other applications
- ❌ Smart positioning (auto-layout, collision detection beyond simple offset)
- ❌ Clone with external connections (ambiguous behavior)
- ❌ Batch clone with variations (e.g., clone 5 times with different offsets)
- ❌ Clone history/versioning beyond undo/redo
- ❌ Clone metadata tracking (when cloned, from what, etc.)

These can be considered for future enhancements.

## Future Enhancements

**Potential improvements** (post-MVP):

1. **Smart positioning**: Find empty space instead of fixed offset
2. **Clone to cursor**: Place clones at mouse position
3. **Clone with incremental naming**: "Node (Clone 1)", "Node (Clone 2)", etc.
4. **Clone with parameter variation**: Clone node and modify constant values
5. **Clipboard support**: Copy selection, paste in different graph
6. **Template library**: Save selections as reusable templates
7. **Clone history**: Track clone genealogy for debugging

## Dependencies

**External crates**: None (uses existing wasmflow dependencies)

**Internal modules**:
- `src/graph/node.rs` - GraphNode, Port structures
- `src/graph/connection.rs` - Connection structure
- `src/graph/command.rs` - Command enum for undo/redo
- `src/ui/selection.rs` - SelectionState for getting selected nodes
- `src/ui/app.rs` - Main app state and UI integration

**Blocking issues**: None

## Risks and Mitigations

### Risk 1: UUID Collision

**Likelihood**: Very low (UUID v4 collision probability ~1 in 10^38)
**Impact**: High (data corruption, connection errors)
**Mitigation**: Use `uuid::Uuid::new_v4()` (cryptographically random), add assertion in tests

### Risk 2: Port Mapping Errors

**Likelihood**: Medium (complex logic with many mappings)
**Impact**: High (broken connections after clone)
**Mitigation**:
- Comprehensive unit tests for mapping logic
- Integration tests with various graph topologies
- Assertions to verify port IDs exist before mapping

### Risk 3: Memory Usage with Large Selections

**Likelihood**: Low (typical graphs <100 nodes)
**Impact**: Medium (slowdown, possible crash on very large graphs)
**Mitigation**:
- Clone operation is O(n) in nodes + connections (acceptable)
- No recursion (iterative cloning)
- Add performance test with 1000+ node selection (stress test)

### Risk 4: Undo/Redo State Corruption

**Likelihood**: Medium (command pattern can be tricky)
**Impact**: High (breaks undo/redo for all operations)
**Mitigation**:
- Follow existing Command patterns exactly
- Test undo/redo extensively
- Verify command history doesn't grow unbounded

## Implementation Checklist

**Phase 1: Core Cloning Logic**
- [ ] Create `src/ui/app/duplication.rs`
- [ ] Implement `clone_node()`
- [ ] Implement `clone_port()`
- [ ] Implement `clone_internal_connections()`
- [ ] Implement `handle_clone_action()`
- [ ] Add unit tests
- [ ] Manual test: Clone single node

**Phase 2: Command Pattern**
- [ ] Add `Command::CloneNodes` variant
- [ ] Implement undo logic
- [ ] Implement redo logic
- [ ] Update `handle_clone_action()` to use command
- [ ] Test undo/redo

**Phase 3: UI Integration**
- [ ] Add Clone button to toolbar
- [ ] Add Ctrl+D keyboard shortcut
- [ ] Add status messages
- [ ] Add tooltip
- [ ] Manual test: UI interactions

**Phase 4: Edge Cases**
- [ ] Handle WASM Creator nodes
- [ ] Handle Composite nodes
- [ ] Handle Continuous nodes
- [ ] Handle position edge cases
- [ ] Add error handling
- [ ] Test special node types

**Phase 5: Documentation**
- [ ] Write integration tests
- [ ] Update CLAUDE.md
- [ ] Create demo graph
- [ ] Run full test suite
- [ ] Update README (if applicable)

**Final Verification**
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Feature works in both modes
- [ ] Undo/redo works
- [ ] No regressions
- [ ] Ready for commit

## References

**Similar features in other tools**:
- Figma: Cmd+D (duplicate selection)
- Sketch: Cmd+D (duplicate)
- Adobe XD: Ctrl+D (duplicate)
- Blender: Shift+D (duplicate)
- Unity: Ctrl+D (duplicate GameObject)

**Relevant commits**:
- c089dac - Latest commit with selection mode
- Commit implementing composition workflow (reference for selected node iteration)
- Commit implementing command history (reference for undo/redo pattern)

**Related specs**:
- 007-rectangle-selection-tool - Original selection mode implementation
- Composition workflow documentation (in code comments)

---

**Status**: Ready for implementation
**Next Steps**: Begin Phase 1 - Core Cloning Logic
