# Composite Node Naming Specification

**Capability**: `composite-node-naming`
**Status**: Proposed
**Version**: 1.0.0

## Overview

Provides user control over composite node names through creation-time dialogs and runtime rename operations with full undo/redo support.

---

## ADDED Requirements

### R-CNM-001: Name Entry at Composite Creation

**Priority**: High
**Category**: User Interaction

When the user initiates a composition operation (selecting 2+ nodes and clicking "Compose"), the system SHALL present a modal dialog requesting a name for the new composite node.

**Details**:
- Dialog appears after validation passes (connected subgraph, valid components)
- Dialog appears before composite node is created in graph
- Default value: "Composite Node"
- User can modify the default or accept it
- Dialog has "Create" and "Cancel" buttons
- Cancel button aborts the composition operation
- Create button proceeds with the entered name

#### Scenario: First-time composite creation with custom name

**Given**: User has selected 3 connected nodes (json-parser, string-concat, http-fetch)
**When**: User clicks "Compose" button
**Then**: A modal dialog titled "Name Your Composite" appears
**And**: The default name "Composite Node" is shown in an editable text field
**When**: User types "Data Processor"
**And**: User clicks "Create"
**Then**: The composite node is created with display_name = "Data Processor"
**And**: The composition_data.name field = "Data Processor"
**And**: The 3 original nodes are removed from the graph

#### Scenario: Composite creation with default name

**Given**: User has selected 2 connected nodes
**When**: User clicks "Compose" button
**Then**: A modal dialog appears with default name "Composite Node"
**When**: User clicks "Create" without modifying the name
**Then**: The composite node is created with display_name = "Composite Node"

#### Scenario: User cancels composite creation from name dialog

**Given**: User has selected 4 connected nodes
**When**: User clicks "Compose" button
**And**: The name dialog appears
**And**: User clicks "Cancel"
**Then**: The composition operation is aborted
**And**: All 4 original nodes remain in the graph unchanged
**And**: No composite node is created

---

### R-CNM-002: Name Validation at Creation

**Priority**: High
**Category**: Data Validation

The system SHALL validate composite node names before allowing creation.

**Details**:
- Empty strings are rejected (after trimming whitespace)
- Whitespace-only strings are rejected
- Any non-empty string after trimming is accepted
- No character restrictions
- No length limits
- No uniqueness enforcement
- Validation feedback shown inline in dialog

#### Scenario: Empty name rejection

**Given**: User is in the composite naming dialog
**When**: User clears the name field (empty string)
**And**: User clicks "Create"
**Then**: An error message "Name cannot be empty" appears below the text field
**And**: The "Create" button is disabled or the dialog does not close
**And**: The user can correct the name

#### Scenario: Whitespace-only name rejection

**Given**: User is in the composite naming dialog
**When**: User enters "   " (only spaces)
**And**: User clicks "Create"
**Then**: An error message "Name cannot be empty" appears
**And**: The composition does not proceed

#### Scenario: Special characters accepted

**Given**: User is in the composite naming dialog
**When**: User enters "Data → Pipeline (v2.1) [PROD]"
**And**: User clicks "Create"
**Then**: The composite node is created successfully
**And**: The name "Data → Pipeline (v2.1) [PROD]" is displayed in the node title

---

### R-CNM-003: Rename via Context Menu

**Priority**: High
**Category**: User Interaction

Users SHALL be able to rename existing composite nodes through a right-click context menu.

**Details**:
- Right-click on composite node shows context menu
- Context menu includes "Rename" option
- Option only appears for composite nodes (nodes with `composition_data.is_some()`)
- Clicking "Rename" opens a modal dialog
- Dialog pre-populated with current `display_name`
- Same validation as creation (non-empty)
- Rename updates both `display_name` and `composition_data.name`

#### Scenario: Successful rename via context menu

**Given**: A composite node named "Composite Node" exists on the canvas
**When**: User right-clicks on the composite node
**Then**: A context menu appears
**And**: The menu includes a "Rename" option
**When**: User clicks "Rename"
**Then**: A modal dialog titled "Rename Composite Node" appears
**And**: The current name "Composite Node" is pre-filled in the text field
**When**: User types "Image Processor"
**And**: User clicks "Rename"
**Then**: The composite node's display_name is updated to "Image Processor"
**And**: The composition_data.name is updated to "Image Processor"
**And**: The canvas reflects the new name immediately

#### Scenario: Context menu only for composite nodes

**Given**: One composite node and one regular WASM component node exist on canvas
**When**: User right-clicks on the WASM component node
**Then**: A context menu appears
**And**: The menu does NOT include a "Rename" option
**When**: User right-clicks on the composite node
**Then**: A context menu appears
**And**: The menu DOES include a "Rename" option

#### Scenario: Cancel rename operation

**Given**: A composite node named "Data Validator" exists
**When**: User right-clicks and selects "Rename"
**And**: The rename dialog appears
**And**: User modifies the name to "Data Processor"
**And**: User clicks "Cancel"
**Then**: The dialog closes
**And**: The composite node name remains "Data Validator" unchanged

---

### R-CNM-004: Undo/Redo Support

**Priority**: High
**Category**: Command System

Rename operations SHALL be reversible through the undo/redo system.

**Details**:
- New command variant: `Command::RenameNode { node_id, old_name, new_name }`
- Execute: Update node's `display_name` and `composition_data.name` to `new_name`
- Undo: Restore node's `display_name` and `composition_data.name` to `old_name`
- Redo: Re-apply `new_name`
- Rename commands pushed to undo stack after successful rename
- Ctrl+Z undoes rename, Ctrl+Y redoes rename

#### Scenario: Undo a rename operation

**Given**: A composite node named "Processor" exists
**When**: User renames it to "Data Pipeline"
**And**: User presses Ctrl+Z (undo)
**Then**: The composite node name reverts to "Processor"
**And**: The undo stack contains the rename command for redo

#### Scenario: Redo a rename operation

**Given**: A composite node was renamed from "Validator" to "Checker"
**And**: User pressed Ctrl+Z to undo (name is now "Validator")
**When**: User presses Ctrl+Y (redo)
**Then**: The composite node name changes back to "Checker"

#### Scenario: Rename updates dirty state

**Given**: A saved graph with a composite node named "Node A"
**When**: User renames the composite node to "Node B"
**Then**: The application marks the graph as dirty (unsaved changes)
**And**: The window title or status bar indicates unsaved changes

---

### R-CNM-005: Name Propagation

**Priority**: Medium
**Category**: Data Consistency

The composite node name SHALL be consistently reflected across all UI representations.

**Details**:
- Node title on canvas shows `display_name`
- Drill-down breadcrumb shows custom name (not hardcoded "Composite Node")
- Status messages use custom name (e.g., "Viewing internal structure of 'Data Processor'")
- Port mapping external names continue using `display_name` (e.g., "Data Processor.input")
- Serialized graph JSON preserves both `display_name` and `composition_data.name`

#### Scenario: Name appears in drill-down breadcrumb

**Given**: A composite node named "Image Processor" exists
**When**: User double-clicks to drill down into the composite
**Then**: The breadcrumb shows "Image Processor" (not "Composite Node")
**And**: The status message shows "Viewing internal structure of 'Image Processor'"

#### Scenario: Name appears in port mappings

**Given**: A composite node named "Data Validator" with exposed input "value"
**When**: User hovers over the external input port
**Then**: The port tooltip or label shows "Data Validator.value"

#### Scenario: Name persists in saved graph

**Given**: A composite node named "Custom Pipeline"
**When**: User saves the graph to disk (Ctrl+S)
**And**: User closes and reopens the application
**And**: User loads the saved graph
**Then**: The composite node appears with name "Custom Pipeline"
**And**: The drill-down breadcrumb shows "Custom Pipeline"

---

## Related Capabilities

- **Composite Node Creation** (feature 007-rectangle-selection-tool): Provides the composition workflow that this capability extends
- **Undo/Redo System** (command pattern): Provides the infrastructure for reversible operations
- **Canvas Interaction**: Provides right-click and context menu foundation

---

## Implementation Notes

### Data Model
- `GraphNode.display_name`: String field (already exists) - updated by rename
- `CompositionData.name`: String field (already exists at line 408 in `src/graph/node.rs`) - updated by rename
- Both fields must be kept in sync during rename operations

### UI Components
- New dialog: `CompositeNameDialog` in `src/ui/dialogs.rs`
- Dialog used for both creation and rename (different titles)
- Context menu logic in `src/ui/canvas.rs` or `src/ui/selection.rs`

### Command System
- Add `Command::RenameNode` variant in `src/ui/app/commands.rs`
- Implement `execute()` and `undo()` methods
- Handle edge cases: node not found, not a composite node

### Validation
- Trim whitespace from input
- Check for empty string after trim
- Display inline error in dialog
- Disable/prevent "Create"/"Rename" button when invalid

---

## Test Scenarios

### Unit Tests
- Validate name trimming logic
- Validate empty string rejection
- Test `Command::RenameNode` execute/undo/redo

### Integration Tests
- Create composite with custom name, save, reload
- Rename composite, undo, redo, save, reload
- Port mapping names reflect renamed composite

### UI Tests
- Context menu shows "Rename" only for composite nodes
- Dialog validation feedback appears for empty names
- Breadcrumb updates after rename

---

## Non-Functional Requirements

### Performance
- Dialog open latency: <50ms
- Rename execution: <10ms (in-memory update only)

### Usability
- Naming dialog must be keyboard-navigable (Tab, Enter, Escape)
- Error messages must be clear and actionable

### Compatibility
- Must not break existing graph JSON files (fields already exist)
- Works with existing undo/redo system without conflicts
