# Design Document: Add Composite Node Naming

**Change ID**: `add-composite-node-naming`
**Created**: 2025-10-27
**Status**: Proposed

---

## Architecture Overview

This change extends the existing composition workflow with user-controlled naming through two interaction paths:

1. **Creation-time naming**: Modal dialog intercepts composition workflow before node creation
2. **Runtime renaming**: Context menu triggers rename dialog, updates via command pattern

Both paths converge on updating two synchronized fields:
- `GraphNode.display_name` (String)
- `CompositionData.name` (String)

---

## Design Decisions

### Decision 1: Modal Dialog vs Inline Editing

**Chosen**: Modal dialog for both creation and rename

**Rationale**:
- **Consistency**: Same UI pattern for both creation and rename operations
- **Focus**: Modal ensures user makes deliberate naming choice
- **Validation**: Centralized validation logic in one dialog component
- **Limitation**: egui-snarl node editor doesn't support inline text editing in node headers

**Alternatives Considered**:
- Inline editing (double-click on title): Rejected due to egui-snarl constraints
- Side panel with text field: Rejected due to poor discoverability

**Trade-offs**:
- ✅ Simple, predictable UX
- ✅ Easy to implement validation
- ❌ One extra click compared to inline editing

---

### Decision 2: Context Menu Trigger for Rename

**Chosen**: Right-click context menu with "Rename" option

**Rationale**:
- **Discoverability**: Right-click is standard interaction for object-specific actions
- **Contextual**: Only shown for composite nodes (filtering reduces UI clutter)
- **Precedent**: Common pattern in desktop applications (file explorers, IDEs)

**Alternatives Considered**:
- Menu bar option (Edit → Rename): Rejected due to extra clicks and hidden location
- Button in node footer: Rejected to avoid UI clutter in footer area
- Keyboard shortcut (F2): Could be added later, but context menu is sufficient

**Trade-offs**:
- ✅ Familiar interaction pattern
- ✅ Scoped to composite nodes only
- ❌ Requires right-click (less discoverable for trackpad-only users)

---

### Decision 3: Command Pattern Integration

**Chosen**: New `Command::RenameNode` variant with undo/redo support

**Rationale**:
- **Consistency**: All editing operations (clone, delete, move) use command pattern
- **Expectations**: Users expect Ctrl+Z to work for rename
- **Complexity**: Minimal - only store old/new names and node ID

**Alternatives Considered**:
- Direct mutation without undo: Rejected due to inconsistency with other operations
- Transaction log approach: Overkill for simple rename operation

**Trade-offs**:
- ✅ Full undo/redo support
- ✅ Consistent with existing patterns
- ❌ Slight memory overhead (stores old/new names in undo stack)

**Implementation Pattern**:
```rust
pub enum Command {
    // ... existing variants
    RenameNode {
        node_id: Uuid,
        old_name: String,
        new_name: String,
    },
}

impl Command {
    pub fn execute(&self, app: &mut WasmFlowApp) -> Result<(), String> {
        match self {
            Command::RenameNode { node_id, new_name, .. } => {
                let node = app.graph.nodes.get_mut(node_id)
                    .ok_or("Node not found")?;

                node.display_name = new_name.clone();
                if let Some(ref mut comp_data) = node.composition_data {
                    comp_data.name = new_name.clone();
                }

                app.dirty = true;
                Ok(())
            }
            // ... other variants
        }
    }

    pub fn undo(&self, app: &mut WasmFlowApp) -> Result<(), String> {
        match self {
            Command::RenameNode { node_id, old_name, .. } => {
                let node = app.graph.nodes.get_mut(node_id)
                    .ok_or("Node not found")?;

                node.display_name = old_name.clone();
                if let Some(ref mut comp_data) = node.composition_data {
                    comp_data.name = old_name.clone();
                }

                app.dirty = true;
                Ok(())
            }
            // ... other variants
        }
    }
}
```

---

### Decision 4: Validation Strategy

**Chosen**: Non-empty strings only (no other restrictions)

**Rationale**:
- **Simplicity**: Minimal validation reduces edge cases
- **Flexibility**: Users can name composites however they want (Unicode, special chars, etc.)
- **No Uniqueness**: Duplicate names are allowed (users may want "Processor 1", "Processor 2")
- **Trim Whitespace**: Prevent accidental whitespace-only names

**Validation Rules**:
1. Trim leading/trailing whitespace
2. Check if result is empty
3. If empty: show error "Name cannot be empty"
4. If valid: accept any string

**Alternatives Considered**:
- Enforce unique names: Rejected due to added complexity and limited value
- Character restrictions: Rejected to support international characters
- Length limits: Rejected as unnecessary (no technical constraint)

**Trade-offs**:
- ✅ Simple validation logic
- ✅ Supports all languages/scripts
- ❌ Users could create confusing duplicate names (acceptable)

---

### Decision 5: Dialog Reuse

**Chosen**: Single `CompositeNameDialog` for both creation and rename

**Rationale**:
- **Code reuse**: Same validation, same layout, only title differs
- **Maintainability**: One dialog to test and maintain
- **Flexibility**: Mode enum (`DialogMode::Create` vs `DialogMode::Rename`) controls behavior

**Dialog State**:
```rust
pub struct CompositeNameDialog {
    is_open: bool,
    name: String,                      // Editable name
    mode: DialogMode,                   // Create or Rename
    validation_error: Option<String>,   // "Name cannot be empty"
    result: Option<CompositeNameAction>, // Confirmed(String) or Cancelled
}

pub enum DialogMode {
    Create,
    Rename,
}

pub enum CompositeNameAction {
    Confirmed(String),
    Cancelled,
}
```

**Trade-offs**:
- ✅ Less code duplication
- ✅ Consistent UX between creation and rename
- ❌ Slightly more complex state management (mode enum)

---

## Data Flow

### Creation Flow

```
User: Select nodes + Click "Compose"
  ↓
App: Validate selection (connected subgraph, WASM components)
  ↓
App: Open CompositeNameDialog in Create mode
  ↓
Dialog: Show "Name Your Composite" with default "Composite Node"
  ↓
User: Edit name + Click "Create" (or Cancel)
  ↓
Dialog: Validate (trim, check empty)
  ↓
Dialog: Return CompositeNameAction::Confirmed(name)
  ↓
App: Continue composition with custom name
  ↓
App: Create CompositionData with name field
  ↓
App: Create GraphNode with display_name = name
  ↓
Canvas: Render node with custom name
```

### Rename Flow

```
User: Right-click composite node
  ↓
Canvas: Show context menu with "Rename" option
  ↓
User: Click "Rename"
  ↓
App: Open CompositeNameDialog in Rename mode with current name
  ↓
Dialog: Show "Rename Composite Node" with pre-filled name
  ↓
User: Edit name + Click "Rename" (or Cancel)
  ↓
Dialog: Validate (trim, check empty)
  ↓
Dialog: Return CompositeNameAction::Confirmed(new_name)
  ↓
App: Create Command::RenameNode { node_id, old_name, new_name }
  ↓
App: Execute command (updates display_name + composition_data.name)
  ↓
App: Push command to undo stack
  ↓
Canvas: Render node with new name
```

### Undo Flow

```
User: Press Ctrl+Z
  ↓
App: Pop command from undo stack
  ↓
App: Call command.undo()
  ↓
Command: Restore old_name to display_name + composition_data.name
  ↓
App: Push command to redo stack
  ↓
Canvas: Render node with restored old name
```

---

## Component Interactions

### Components Modified

1. **`src/ui/dialogs.rs`**
   - New: `CompositeNameDialog` struct
   - New: `DialogMode` enum
   - New: `CompositeNameAction` enum
   - New: `show()` method with validation

2. **`src/ui/app.rs`**
   - Add: `composite_name_dialog: CompositeNameDialog` field
   - Modify: `update()` to handle dialog results
   - Add: State tracking for rename context (clicked node ID)

3. **`src/ui/app/composition.rs`**
   - Modify: `handle_compose_action()` to show dialog before creation
   - Add: `handle_composite_name_confirmed()` for creation flow
   - Verify: `handle_drill_down()` uses `display_name` (already does)

4. **`src/ui/app/commands.rs`**
   - Add: `Command::RenameNode` variant
   - Implement: `execute()` and `undo()` for rename

5. **`src/ui/canvas.rs` or `src/ui/selection.rs`**
   - Add: Right-click detection
   - Add: Context menu rendering
   - Add: "Rename" option for composite nodes only

### Dependencies

- **egui**: Dialog rendering, text input, validation UI
- **Existing undo/redo system**: Command pattern infrastructure
- **Existing composition workflow**: Hooks into creation flow
- **Existing node data model**: Uses existing `display_name` and `composition_data.name` fields

---

## Error Handling

### Edge Cases

1. **Empty name in dialog**
   - Detection: `name.trim().is_empty()`
   - Handling: Show inline error, prevent confirmation
   - Recovery: User edits name or cancels

2. **Node deleted during rename**
   - Detection: `graph.nodes.get(&node_id)` returns `None`
   - Handling: Command execution returns `Err("Node not found")`
   - Recovery: Show error toast, discard command

3. **Non-composite node rename attempt**
   - Prevention: Context menu only shows "Rename" for composites
   - Fallback: If somehow triggered, command checks `composition_data.is_some()`

4. **Cancelled operations**
   - Dialog: Returns `CompositeNameAction::Cancelled`
   - App: Clears state, does nothing
   - No side effects

### Validation Errors

All validation errors shown inline in dialog:
- "Name cannot be empty" - red text below input field
- No error state for valid inputs

---

## Performance Considerations

### Memory

- **Dialog state**: ~200 bytes (String + enums)
- **Command in undo stack**: ~150 bytes (Uuid + 2 Strings)
- **Impact**: Negligible (< 1KB per rename operation)

### CPU

- **Dialog validation**: <1ms (trim + empty check)
- **Rename execution**: <1ms (2 field updates)
- **Context menu**: <1ms (node type check)
- **No performance concerns**

### UI Responsiveness

- **Dialog open**: <50ms (egui window creation)
- **Canvas update after rename**: <10ms (immediate re-render)
- **No perceived latency**

---

## Testing Strategy

### Unit Tests

1. **Command tests** (`src/ui/app/commands.rs`):
   - `test_rename_node_execute()` - Updates both fields
   - `test_rename_node_undo()` - Restores old name
   - `test_rename_node_not_found()` - Error handling

2. **Dialog tests** (`src/ui/dialogs.rs`):
   - `test_validation_rejects_empty()` - Empty string rejected
   - `test_validation_trims_whitespace()` - "  name  " becomes "name"
   - `test_validation_accepts_special_chars()` - Unicode accepted

### Integration Tests

1. **End-to-end creation**:
   - Create composite with custom name
   - Verify `display_name` and `composition_data.name` match
   - Save graph, reload, verify persistence

2. **End-to-end rename**:
   - Rename via context menu
   - Verify name updates in canvas and breadcrumb
   - Undo, verify old name restored
   - Redo, verify new name reapplied

3. **Edge cases**:
   - Cancel creation, verify original nodes remain
   - Cancel rename, verify name unchanged
   - Empty name rejected in both modes

### Manual UI Tests

- Right-click on regular node: no "Rename" option
- Right-click on composite: "Rename" option appears
- Keyboard navigation: Tab, Enter, Escape work
- Port mappings show updated name
- Status messages show updated name

---

## Migration & Compatibility

### Backward Compatibility

- ✅ **Serialization format unchanged**: `display_name` and `composition_data.name` already exist
- ✅ **Old graphs load correctly**: Default "Composite Node" preserved if present
- ✅ **No breaking changes**: Existing functionality unaffected

### Forward Compatibility

- ✅ **New graphs load in old versions**: Extra dialog ignored, composites still functional
- ⚠️ **Old versions can't rename**: Feature unavailable but graphs still work

---

## Security Considerations

### Input Validation

- **XSS**: Not applicable (desktop app, not web)
- **Path traversal**: Not applicable (names not used as file paths)
- **SQL injection**: Not applicable (no database)
- **Length limits**: No enforced limit (egui handles display truncation)

### Resource Limits

- **Memory**: Unbounded name length could consume memory
  - **Mitigation**: Practical limit is UI text field size (~4KB input)
- **Undo stack**: Many renames could fill stack
  - **Mitigation**: Existing undo stack has reasonable limit (configurable)

**Conclusion**: No significant security concerns for this feature.

---

## Future Enhancements

Potential future additions (out of scope for this change):

1. **Name suggestions**: Auto-generate names from internal nodes (e.g., "json-parser + http-fetch")
2. **Name templates**: User-defined naming patterns with variables
3. **Bulk rename**: Rename multiple composites at once
4. **Uniqueness enforcement**: Optional setting to prevent duplicate names
5. **Keyboard shortcut**: F2 to rename selected composite
6. **Rename regular nodes**: Extend to all node types (requires broader design)

---

## Open Questions & Decisions

All design questions resolved:
- ✅ Interaction pattern chosen (modal + context menu)
- ✅ Validation rules defined (non-empty only)
- ✅ Undo/redo support confirmed
- ✅ Data model unchanged (fields already exist)

No open questions remaining.
