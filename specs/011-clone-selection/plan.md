# Clone Selection - Implementation Plan

**Feature ID**: 011-clone-selection
**Author**: Claude Code
**Created**: 2025-10-27
**Status**: Planning → Ready for Implementation

## Executive Summary

This plan outlines the implementation of a "clone selection" feature for wasmflow, allowing users to duplicate selected nodes and their internal connections in a single operation. The feature will be accessible via a UI button and Ctrl+D keyboard shortcut, with full undo/redo support.

## Goals

**Primary Goal**: Enable users to quickly duplicate graph patterns without manual recreation

**Success Metrics**:
- Users can clone 1+ selected nodes with a single action
- Internal connections between cloned nodes are preserved
- Operation is undoable/redoable
- Feature integrates seamlessly with existing selection mode

## Design Philosophy

**User Experience Principles**:
1. **Predictable**: Clones appear at fixed offset from originals (visible, not overlapping)
2. **Safe**: Only internal connections cloned (no ambiguous external connections)
3. **Reversible**: Full undo/redo support
4. **Discoverable**: Button in UI + standard keyboard shortcut (Ctrl+D)
5. **Consistent**: Works in both Normal and Selection modes

**Technical Principles**:
1. **Follow existing patterns**: Use composition.rs workflow as reference
2. **Maintain data integrity**: Generate unique UUIDs for all cloned entities
3. **Leverage command pattern**: Integrate with existing undo/redo system
4. **Minimal complexity**: Simple offset positioning (no smart layout in MVP)

## Scope

### In Scope

✅ **Core functionality**:
- Clone selected nodes (1 to N nodes)
- Clone internal connections (both endpoints selected)
- Generate unique UUIDs for nodes, ports, and connections
- Offset cloned nodes by fixed amount (+50px, +50px)
- Append " (Clone)" to display names
- Select cloned nodes after creation
- Clear original selection

✅ **UI/UX**:
- Clone button in toolbar (next to Compose)
- Ctrl+D keyboard shortcut
- Status messages for feedback
- Enable/disable based on selection state

✅ **Undo/Redo**:
- Command::CloneNodes variant
- Full undo support (removes all clones)
- Full redo support (restores all clones)

✅ **Special node types**:
- WASM Creator nodes (clone creator_data)
- Composite nodes (clone composition_data)
- Continuous nodes (reset runtime_state)
- Constant nodes (clone constant_value)

✅ **Testing**:
- Unit tests for core logic
- Integration tests for graph topologies
- Manual testing checklist
- Demo graph

### Out of Scope (Future Enhancements)

❌ **Deferred to future versions**:
- Copy/paste across graphs (clipboard serialization)
- Smart positioning (collision detection, auto-layout)
- Clone with external connections (ambiguous behavior)
- Clone to cursor position
- Incremental naming ("Clone 1", "Clone 2", etc.)
- Template library (save/reuse selections)
- Clone history tracking

## Architecture

### Module Structure

```
src/
├── ui/
│   ├── app.rs                    (Add Clone button, Ctrl+D shortcut)
│   └── app/
│       ├── composition.rs        (Reference for pattern)
│       └── duplication.rs        (NEW: Clone logic)
└── graph/
    └── command.rs                (Add Command::CloneNodes variant)
```

### Data Flow

```
User Action (Button/Ctrl+D)
    ↓
handle_clone_action()
    ↓
1. Get selected nodes
    ↓
2. Clone each node
   - Generate new node UUID
   - Clone ports with new port UUIDs
   - Build node_mapping (old_id → new_id)
   - Build port_mapping (old_port_id → new_port_id)
    ↓
3. Clone internal connections
   - Filter: only connections with both endpoints selected
   - Map node IDs via node_mapping
   - Map port IDs via port_mapping
   - Generate new connection UUIDs
    ↓
4. Create Command::CloneNodes
    ↓
5. Push to command_history
    ↓
6. Apply changes to graph
   - Clear original selection
   - Add cloned nodes (selected)
   - Add cloned connections
    ↓
7. Update status message
```

### Key Data Structures

**CloneResult** (internal helper):
```rust
struct CloneResult {
    cloned_nodes: Vec<GraphNode>,
    cloned_connections: Vec<Connection>,
    node_mapping: HashMap<Uuid, Uuid>,
    port_mapping: HashMap<Uuid, Uuid>,
}
```

**Command Variant**:
```rust
pub enum Command {
    CloneNodes {
        cloned_node_ids: Vec<Uuid>,
        cloned_nodes: Vec<GraphNode>,
        cloned_connections: Vec<Connection>,
    },
}
```

### Core Algorithms

**Node Cloning**:
```
For each selected node:
  1. Generate new_node_id = UUID::new_v4()
  2. Clone node struct
  3. For each input port:
     - Generate new_port_id = UUID::new_v4()
     - Clone port
     - Store (old_port_id → new_port_id) mapping
  4. For each output port:
     - Generate new_port_id = UUID::new_v4()
     - Clone port
     - Store (old_port_id → new_port_id) mapping
  5. Offset position += CLONE_OFFSET
  6. Append CLONE_NAME_SUFFIX to display_name
  7. Set selected = true, execution_state = None, dirty = false
  8. Store (old_node_id → new_node_id) mapping
```

**Connection Cloning**:
```
For each connection in graph:
  1. If from_node ∈ selected AND to_node ∈ selected:
     - Generate new_conn_id = UUID::new_v4()
     - Map from_node via node_mapping
     - Map to_node via node_mapping
     - Map from_port via port_mapping
     - Map to_port via port_mapping
     - Create cloned connection
  2. Else: Skip (external connection)
```

## Implementation Phases

### Phase 1: Core Cloning Logic (Est: 4 hours)

**Deliverables**:
- [ ] `src/ui/app/duplication.rs` module created
- [ ] `clone_port()` function
- [ ] `clone_node()` function
- [ ] `clone_internal_connections()` function
- [ ] `handle_clone_action()` method (without Command integration)
- [ ] Unit tests for all functions

**Testing**: Clone single node, clone connected subgraph, verify no external connections cloned

### Phase 2: Command Pattern (Est: 2 hours)

**Deliverables**:
- [ ] `Command::CloneNodes` variant added
- [ ] Undo logic implemented
- [ ] Redo logic implemented
- [ ] `handle_clone_action()` updated to use command pattern

**Testing**: Clone → Undo → Redo, verify graph state consistency

### Phase 3: UI Integration (Est: 2 hours)

**Deliverables**:
- [ ] Clone button in toolbar
- [ ] Ctrl+D keyboard shortcut
- [ ] Status messages
- [ ] Button enable/disable logic
- [ ] Tooltip

**Testing**: Button click, Ctrl+D, verify in both Normal/Selection modes

### Phase 4: Edge Cases (Est: 2 hours)

**Deliverables**:
- [ ] WASM Creator node cloning tested
- [ ] Composite node cloning tested
- [ ] Continuous node cloning tested (runtime state reset)
- [ ] Self-loop handling verified
- [ ] Error handling added

**Testing**: All special node types, edge cases, large selections

### Phase 5: Documentation (Est: 2 hours)

**Deliverables**:
- [ ] Integration tests written
- [ ] CLAUDE.md updated
- [ ] Demo graph created
- [ ] Manual testing checklist completed
- [ ] All tests passing

**Testing**: Full test suite, clippy clean, no regressions

**Total Estimated Time**: 12 hours (1.5 days)

## Risk Assessment

### High Risk

**None identified** - Feature is well-scoped, has clear reference implementation (composition.rs)

### Medium Risk

**1. Port Mapping Complexity**
- **Risk**: Incorrect port UUID mapping breaks connections
- **Mitigation**: Comprehensive unit tests, integration tests with various topologies
- **Fallback**: Add assertions to verify port IDs exist before mapping

**2. Undo/Redo State Corruption**
- **Risk**: Command pattern implementation breaks undo/redo for all operations
- **Mitigation**: Follow existing Command patterns exactly, extensive undo/redo testing
- **Fallback**: Review command history implementation if issues arise

### Low Risk

**3. UUID Collision**
- **Risk**: Two cloned entities get same UUID
- **Mitigation**: Use UUID v4 (cryptographically random, collision probability ~1 in 10^38)
- **Fallback**: Add collision detection in debug builds

**4. Performance with Large Selections**
- **Risk**: Cloning 100+ nodes is slow
- **Mitigation**: Algorithm is O(n) in nodes + connections (acceptable)
- **Fallback**: Add progress indicator if needed (unlikely for typical graphs)

## Testing Strategy

### Unit Tests (15+ tests)

**Location**: `src/ui/app/duplication.rs` inline tests

**Coverage**:
- Port cloning (UUID uniqueness, property preservation)
- Node cloning (UUID uniqueness, offset, name suffix, state reset)
- Connection filtering (internal vs external)
- Connection cloning (ID mapping, UUID generation)

### Integration Tests (6+ tests)

**Location**: `tests/integration/clone_selection_test.rs`

**Scenarios**:
1. Clone single node
2. Clone multiple disconnected nodes
3. Clone connected subgraph (A→B→C, all selected)
4. Clone partial subgraph (A→B→C, only B selected)
5. Clone with external connections (verify not cloned)
6. Undo/redo clone operation

### Manual Testing (15+ scenarios)

**Checklist**: See tasks.md Task 5.5

**Focus areas**:
- Special node types (WASM Creator, Composite, Continuous)
- Both Normal and Selection modes
- Keyboard shortcuts
- UI button states
- Large selections
- Edge cases (self-loops, no connections, single node)

## Success Criteria

### Functional Requirements (All must pass)

- ✅ User can clone selected nodes via button
- ✅ User can clone selected nodes via Ctrl+D
- ✅ Cloned nodes have unique UUIDs
- ✅ Cloned ports have unique UUIDs
- ✅ Internal connections are cloned
- ✅ External connections are NOT cloned
- ✅ Cloned nodes appear at offset position
- ✅ Clone operation is undoable
- ✅ Clone operation is redoable
- ✅ Works in both Normal and Selection modes

### UX Requirements (All must pass)

- ✅ Clone button visible when nodes selected
- ✅ Clone button disabled when no selection
- ✅ Ctrl+D documented in tooltip
- ✅ Status messages provide feedback
- ✅ No crashes or data loss
- ✅ Performance: <100ms for typical operations

### Code Quality Requirements (All must pass)

- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ No clippy warnings
- ✅ Follows existing code patterns
- ✅ CLAUDE.md updated
- ✅ No regressions

## Dependencies

**No external dependencies** - Uses existing wasmflow infrastructure:
- `uuid` crate (already used)
- `egui` (already used)
- Graph data structures (already defined)
- Command pattern (already implemented)

**Reference implementations**:
- `src/ui/app/composition.rs` - Pattern for working with selected nodes
- `src/graph/command.rs` - Pattern for undo/redo commands
- Existing node creation code - Pattern for UUID generation

## Timeline

**Estimated timeline** (full-time equivalent):

| Phase | Duration | Completion Date |
|-------|----------|----------------|
| Phase 1: Core Logic | 4 hours | Day 1 AM |
| Phase 2: Command Pattern | 2 hours | Day 1 PM |
| Phase 3: UI Integration | 2 hours | Day 1 PM |
| Phase 4: Edge Cases | 2 hours | Day 2 AM |
| Phase 5: Documentation | 2 hours | Day 2 PM |
| **Total** | **12 hours** | **~1.5 days** |

**Buffer**: Add 4 hours (33%) for unexpected issues, testing, iteration
**Total with buffer**: 16 hours (~2 days)

## Rollout Plan

### Development

1. Create feature branch: `feature/clone-selection`
2. Implement phases 1-5 sequentially
3. Run tests after each phase
4. Commit after each phase

### Testing

1. Run full test suite: `cargo test`
2. Run clippy: `cargo clippy`
3. Manual testing checklist (15 scenarios)
4. Demo graph verification

### Review

1. Self-review code against patterns in composition.rs
2. Verify all success criteria met
3. Check CLAUDE.md guidelines followed
4. Verify no regressions

### Merge

1. Ensure all tests pass
2. Ensure no clippy warnings
3. Squash commits if needed
4. Merge to main branch
5. Tag release (if appropriate)

## Future Enhancements

**Potential improvements** (post-MVP):

### Priority 1 (High value, low complexity)
- Incremental naming: "Node (Clone 1)", "Node (Clone 2)", etc.
- Ctrl+Shift+D to "duplicate in place" (no offset)

### Priority 2 (Medium value, medium complexity)
- Smart positioning: Find empty space instead of fixed offset
- Clone to cursor: Place clones at mouse position
- Clone with parameter variation: Clone and modify constant values

### Priority 3 (High value, high complexity)
- Clipboard support: Copy selection, paste in different graph
- Template library: Save selections as reusable templates
- Clone with external connections: Prompt user for connection behavior

## References

**Similar features in design tools**:
- Figma: Cmd+D (duplicate selection)
- Sketch: Cmd+D (duplicate)
- Adobe XD: Ctrl+D (duplicate)

**Relevant code**:
- `src/ui/app/composition.rs` - Selected node iteration pattern
- `src/ui/selection.rs` - Selection state management
- `src/graph/command.rs` - Command pattern examples

**Related specs**:
- 007-rectangle-selection-tool - Original selection mode spec

## Appendix A: File Changes Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `src/ui/app/duplication.rs` | CREATE | Clone logic, helper functions |
| `src/ui/app.rs` | MODIFY | Add Clone button, Ctrl+D shortcut, module import |
| `src/graph/command.rs` | MODIFY | Add Command::CloneNodes variant |
| `src/ui/app.rs` (undo/redo) | MODIFY | Handle Command::CloneNodes in undo/redo |
| `CLAUDE.md` | MODIFY | Add Clone Selection Guidelines section |
| `tests/integration/clone_selection_test.rs` | CREATE | Integration tests |
| `tests/demo/clone_selection_demo.json` | CREATE | Demo graph |

**Total files changed**: 7 (2 new, 5 modified)

## Appendix B: Constants and Configuration

```rust
// Position offset for cloned nodes
const CLONE_OFFSET: egui::Vec2 = egui::Vec2::new(50.0, 50.0);

// Suffix appended to cloned node display names
const CLONE_NAME_SUFFIX: &str = " (Clone)";

// Keyboard shortcut
// - Primary: Ctrl+D
// - macOS: Cmd+D (handled by egui automatically)

// UI Button
// - Label: "📋 Clone Selected" or "🗐 Clone"
// - Position: After Compose button in toolbar
// - Tooltip: "Clone selected nodes (Ctrl+D)"
```

## Appendix C: Decision Log

**Decision 1**: Clone internal connections only
- **Rationale**: External connections have ambiguous behavior (should they connect to original or clone?)
- **Alternative considered**: Prompt user for connection behavior (rejected: adds complexity)
- **Result**: Simple, predictable behavior

**Decision 2**: Fixed offset positioning
- **Rationale**: Simple, predictable, works for 90% of use cases
- **Alternative considered**: Smart positioning to avoid overlaps (deferred to future)
- **Result**: MVP is simple, enhancement possible later

**Decision 3**: Append " (Clone)" to name
- **Rationale**: Distinguishes clones from originals, user can rename
- **Alternative considered**: Incremental naming "Clone 1", "Clone 2" (deferred to future)
- **Result**: Simple implementation, enhanceable later

**Decision 4**: Select clones, deselect originals
- **Rationale**: Focuses attention on clones, allows immediate dragging, enables clone-the-clone
- **Alternative considered**: Keep original selection (rejected: confusing)
- **Result**: Consistent with paste behavior in design tools

**Decision 5**: Ctrl+D keyboard shortcut
- **Rationale**: Standard in design tools (Figma, Sketch, Adobe)
- **Alternative considered**: Ctrl+Alt+C (rejected: less discoverable)
- **Result**: Familiar to users of design tools

---

**Plan Status**: ✅ Complete and ready for implementation

**Next Steps**: Review plan, then begin Phase 1 implementation
