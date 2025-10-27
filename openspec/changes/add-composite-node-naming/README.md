# Add Composite Node Naming - OpenSpec Proposal

**Change ID**: `add-composite-node-naming`
**Status**: Draft - Awaiting Approval
**Estimated Effort**: 4-6 hours

---

## Quick Summary

Enable users to name composite nodes at creation time (via modal dialog) and rename them afterward (via right-click context menu), with full undo/redo support.

---

## Key Design Decisions

Based on user preferences:

1. **Name Entry**: Modal dialog appears after clicking "Compose" (before node creation)
2. **Rename UI**: Right-click context menu with "Rename" option (composites only)
3. **Undo/Redo**: Full support via new `Command::RenameNode` variant
4. **Validation**: Non-empty strings only (no uniqueness enforcement, no character restrictions)

---

## What's Included

### 📄 Files Created

- **`proposal.md`** (73 lines): Motivation, scope, success criteria, risks
- **`specs/composite-node-naming/spec.md`** (300 lines): 5 requirements with detailed scenarios
- **`tasks.md`** (315 lines): 7 phases, 20+ implementation tasks
- **`design.md`** (465 lines): Architecture decisions, data flows, trade-offs

**Total**: 1,153 lines of specification

### 📋 Requirements (5 Total)

1. **R-CNM-001**: Name Entry at Composite Creation
   - Modal dialog with default "Composite Node"
   - Create/Cancel buttons
   - Shows after validation, before node creation

2. **R-CNM-002**: Name Validation at Creation
   - Reject empty strings (after trim)
   - Accept any non-empty string (Unicode, special chars)
   - Inline error feedback

3. **R-CNM-003**: Rename via Context Menu
   - Right-click shows "Rename" (composites only)
   - Pre-filled with current name
   - Updates `display_name` and `composition_data.name`

4. **R-CNM-004**: Undo/Redo Support
   - New `Command::RenameNode` variant
   - Ctrl+Z/Ctrl+Y support
   - Stores old/new names for reversibility

5. **R-CNM-005**: Name Propagation
   - Canvas node title shows name
   - Drill-down breadcrumb shows name
   - Port mappings use name (e.g., "Custom Name.input")
   - Persists in saved graph JSON

### 🔨 Implementation Phases

1. **Phase 1**: Data Model & Command System (1 hour)
2. **Phase 2**: UI Dialog for Naming (1.5 hours)
3. **Phase 3**: Integrate into Composition Workflow (1 hour)
4. **Phase 4**: Right-Click Context Menu (1.5 hours)
5. **Phase 5**: Name Propagation & Consistency (0.5 hours)
6. **Phase 6**: Testing & Documentation (1 hour)
7. **Phase 7**: Final Review (0.5 hours)

---

## Files to Modify

- `src/ui/dialogs.rs` - Add `CompositeNameDialog`
- `src/ui/app.rs` - Add dialog field, handle results
- `src/ui/app/composition.rs` - Integrate naming into composition flow
- `src/ui/app/commands.rs` - Add `Command::RenameNode` variant
- `src/ui/canvas.rs` or `src/ui/selection.rs` - Add context menu
- `CLAUDE.md` - Document naming guidelines

---

## Validation Checklist

Before requesting approval, verify:

- [x] All clarification questions answered (1:A, 2:A, 3:A, 4:A)
- [x] Proposal clearly states scope and motivation
- [x] Spec has at least one scenario per requirement
- [x] Tasks broken into verifiable work items
- [x] Design documents architectural decisions
- [ ] Run `openspec validate add-composite-node-naming --strict` (if tooling available)

---

## How to Review This Proposal

1. **Start with `proposal.md`**: Understand motivation and scope
2. **Read `design.md`**: Review architectural decisions and trade-offs
3. **Check `specs/composite-node-naming/spec.md`**: Validate requirements and scenarios
4. **Review `tasks.md`**: Ensure implementation plan is complete

---

## Next Steps

1. **Review**: Reviewer reads all documents and provides feedback
2. **Revise**: Address any concerns or requested changes
3. **Approve**: Reviewer approves proposal for implementation
4. **Implement**: Developer executes tasks from `tasks.md` sequentially
5. **Archive**: After deployment, move to `openspec/changes/archive/`

---

## Related Specifications

- **Composite Node Creation** (feature 007-rectangle-selection-tool)
- **Undo/Redo System** (command pattern in `src/ui/app/commands.rs`)
- **Clone Selection** (feature 011-clone-selection) - Similar undo/redo pattern

---

## Questions or Feedback?

If you have questions about this proposal, please:
1. Review the detailed scenarios in `spec.md`
2. Check design rationale in `design.md`
3. Identify specific requirements that need clarification
4. Request changes before implementation begins
