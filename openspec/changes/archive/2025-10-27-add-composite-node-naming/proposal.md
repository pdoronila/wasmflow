# Proposal: Add Composite Node Naming

**Change ID**: `add-composite-node-naming`
**Status**: Draft
**Created**: 2025-10-27
**Author**: Claude Code

## Summary

Enable users to set custom names for composite nodes both at creation time and after creation through a rename operation. This improves graph readability and organization by allowing meaningful names instead of the generic "Composite Node" default.

## Motivation

Currently, when users compose multiple nodes into a composite node, the system assigns a hardcoded name "Composite Node" (line 175 in `src/ui/app/composition.rs`). This creates several problems:

1. **Poor Graph Readability**: Multiple composite nodes in the same graph are indistinguishable by name
2. **Lost Semantic Context**: The name doesn't reflect what the composite does (e.g., "Image Processor", "Data Validator")
3. **Difficult Navigation**: Users must drill down to understand what each composite contains
4. **Inconsistent UX**: Regular nodes can have meaningful names, but composites cannot be renamed

## Scope

### In Scope
- Modal dialog for naming new composite nodes at creation time
- Right-click context menu option to rename existing composite nodes
- Undo/redo support for rename operations via `Command::RenameNode`
- Basic validation (non-empty string requirement)
- Update to `CompositionData.name` field (already exists in data model)
- Update drill-down breadcrumb to show custom name

### Out of Scope
- Renaming non-composite nodes (regular WASM component nodes)
- Name uniqueness enforcement across graph
- Complex naming conventions or character restrictions
- Bulk rename operations
- Auto-generated naming suggestions

## Success Criteria

1. Users can name a composite node during creation via modal dialog
2. Users can rename existing composite nodes via right-click context menu
3. Custom names appear in node title, drill-down breadcrumb, and port mappings
4. Rename operations support undo/redo
5. Empty names are rejected with user feedback
6. Changes persist when saving/loading graph files

## Dependencies

- Existing UI dialog infrastructure (`src/ui/dialogs.rs`)
- Existing command/undo system (`src/ui/app/commands.rs`)
- Existing composition workflow (`src/ui/app/composition.rs`)
- Canvas interaction system (`src/ui/canvas.rs`)

## Risks and Mitigations

**Risk**: Users expect unique name enforcement
**Mitigation**: Document that duplicate names are allowed; can add uniqueness validation in future if needed

**Risk**: Context menu adds UI complexity
**Mitigation**: Only show "Rename" option for composite nodes; keep interaction simple

**Risk**: Breaking changes to serialization format
**Mitigation**: `CompositionData.name` field already exists; no schema changes needed

## Alternatives Considered

1. **Inline editing (double-click)**: Rejected due to egui-snarl node editor limitations
2. **Rename via node footer**: Rejected to keep context menu as primary rename UI
3. **No undo/redo**: Rejected to maintain consistency with other editing operations (clone, delete)

## Open Questions

None - all design decisions clarified with user.
