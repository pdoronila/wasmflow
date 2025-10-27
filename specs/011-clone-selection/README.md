# Clone Selection Feature Specification

**Feature ID**: 011-clone-selection
**Status**: Planning Complete → Ready for Implementation
**Created**: 2025-10-27

## Overview

This feature adds "clone selection" functionality to wasmflow, allowing users to duplicate selected nodes and their internal connections in a single operation via UI button or Ctrl+D keyboard shortcut.

## Documentation Structure

This specification is organized into multiple documents for clarity:

### 📋 [spec.md](./spec.md) - Complete Feature Specification

The master specification document containing:
- **Motivation**: Why we need this feature
- **Current state analysis**: What exists today
- **Design decisions**: What we're building and why
- **Implementation plan**: 5 phases with detailed tasks
- **Technical specifications**: Data structures, algorithms, APIs
- **Testing strategy**: Unit, integration, and manual tests
- **Success criteria**: What "done" looks like
- **Non-goals**: What we're NOT building

**Read this first** to understand the full feature scope and design.

### 📐 [plan.md](./plan.md) - High-Level Implementation Plan

Executive summary and implementation roadmap:
- **Executive summary**: Goals, design philosophy, scope
- **Architecture**: Module structure, data flow, key algorithms
- **Implementation phases**: 5 phases with time estimates
- **Risk assessment**: Potential issues and mitigations
- **Testing strategy**: Unit, integration, manual
- **Timeline**: ~12 hours (1.5 days) estimated
- **Appendices**: Decision log, constants, file changes

**Read this** for a bird's-eye view before starting implementation.

### ✅ [tasks.md](./tasks.md) - Detailed Task Breakdown

Actionable task list organized by phase:
- **Phase 1**: Core cloning logic (7 tasks)
- **Phase 2**: Command pattern for undo/redo (5 tasks)
- **Phase 3**: UI integration (4 tasks)
- **Phase 4**: Edge cases and polish (5 tasks)
- **Phase 5**: Documentation and testing (5 tasks)

Each task includes:
- File locations
- Code snippets
- Step-by-step instructions
- Acceptance criteria

**Use this** as your implementation checklist.

## Quick Start

### For Reviewers

1. Read [plan.md](./plan.md) for the high-level overview
2. Check design decisions in [spec.md](./spec.md#design-decisions)
3. Review success criteria in [spec.md](./spec.md#success-criteria)

### For Implementers

1. Read [plan.md](./plan.md) to understand the architecture
2. Follow [tasks.md](./tasks.md) phase by phase
3. Reference [spec.md](./spec.md) for detailed requirements
4. Check off tasks as you complete them

### For Testers

1. Review [spec.md](./spec.md#testing-strategy) for testing approach
2. Use manual testing checklist in [tasks.md](./tasks.md#task-55-manual-testing-checklist)
3. Run integration tests from [tasks.md](./tasks.md#task-51-write-integration-tests)

## Feature Summary

### What It Does

**For Users**:
- Select nodes in the canvas
- Click "Clone Selected" button or press Ctrl+D
- Selected nodes are duplicated with offset position
- Internal connections between cloned nodes are preserved
- Can undo/redo the operation

**Technical**:
- Clones GraphNode instances with new UUIDs
- Clones Port instances with new UUIDs
- Clones internal Connection instances with mapped UUIDs
- Integrates with existing command history (undo/redo)
- Works in both Normal and Selection modes

### Key Design Decisions

1. **Internal connections only**: External connections (one endpoint outside selection) are NOT cloned to avoid ambiguous behavior
2. **Fixed offset**: Clones appear at (+50px, +50px) from originals
3. **Name suffix**: " (Clone)" appended to display names
4. **Selection behavior**: Clones are selected, originals deselected
5. **Ctrl+D shortcut**: Standard in design tools

### Implementation Phases

| Phase | Focus | Duration | Status |
|-------|-------|----------|--------|
| 1 | Core cloning logic | 4 hours | 🔲 Not started |
| 2 | Command pattern (undo/redo) | 2 hours | 🔲 Not started |
| 3 | UI integration | 2 hours | 🔲 Not started |
| 4 | Edge cases and polish | 2 hours | 🔲 Not started |
| 5 | Documentation and testing | 2 hours | 🔲 Not started |

**Total**: ~12 hours (1.5 days)

## Success Criteria

**Must have** (all required for completion):
- ✅ Clone nodes via button and Ctrl+D
- ✅ Unique UUIDs for all cloned entities
- ✅ Internal connections preserved
- ✅ External connections NOT cloned
- ✅ Full undo/redo support
- ✅ Works in both Normal and Selection modes
- ✅ All tests pass (unit + integration)

**Quality bars**:
- Performance: <100ms for typical operations (<50 nodes)
- No clippy warnings
- No regressions in existing functionality
- CLAUDE.md updated with guidelines

## Files to Create/Modify

**New files** (2):
- `src/ui/app/duplication.rs` - Clone logic
- `tests/integration/clone_selection_test.rs` - Integration tests

**Modified files** (5):
- `src/ui/app.rs` - UI button, keyboard shortcut
- `src/graph/command.rs` - Command::CloneNodes variant
- `src/ui/app.rs` (undo/redo methods) - Handle new command
- `CLAUDE.md` - Clone Selection Guidelines
- `tests/demo/clone_selection_demo.json` - Demo graph

## Dependencies

**No new dependencies required** - Uses existing:
- `uuid` crate for UUID generation
- `egui` for UI
- Existing graph data structures
- Existing command pattern

## References

**Code to study**:
- `src/ui/app/composition.rs` - Pattern for selected node iteration
- `src/ui/selection.rs` - Selection state management
- `src/graph/command.rs` - Command pattern examples

**Related specs**:
- `007-rectangle-selection-tool` - Original selection mode

**Similar features**:
- Figma: Cmd+D (duplicate)
- Sketch: Cmd+D (duplicate)
- Adobe XD: Ctrl+D (duplicate)

## Testing Approach

### Unit Tests (15+)

Location: `src/ui/app/duplication.rs`

Tests for:
- Port cloning (UUID uniqueness, properties)
- Node cloning (UUID uniqueness, offset, naming)
- Connection filtering (internal vs external)
- Connection cloning (ID mapping)

### Integration Tests (6+)

Location: `tests/integration/clone_selection_test.rs`

Scenarios:
- Clone single node
- Clone multiple disconnected nodes
- Clone connected subgraph
- Clone partial subgraph
- Clone with external connections (verify not cloned)
- Undo/redo operations

### Manual Testing (15+ scenarios)

Checklist in [tasks.md](./tasks.md#task-55-manual-testing-checklist)

Focus:
- Special node types (WASM Creator, Composite, Continuous)
- Both modes (Normal, Selection)
- UI interactions (button, shortcut)
- Edge cases (self-loops, large selections)

## FAQ

**Q: Why not clone external connections?**
A: Ambiguous behavior - should they connect to original or clone? Skipping them keeps behavior predictable.

**Q: Why fixed offset instead of smart positioning?**
A: Simple, predictable, works for 90% of cases. Smart positioning can be added later.

**Q: Why " (Clone)" instead of "Copy of..." or " copy"?**
A: Shorter, clearer. User can rename if desired.

**Q: Why Ctrl+D instead of Ctrl+C/Ctrl+V?**
A: Ctrl+D is standard "duplicate" in design tools. Ctrl+C/V implies clipboard (not implementing yet).

**Q: Can I clone the clones?**
A: Yes! After cloning, the clones are selected, so you can immediately clone again.

**Q: What happens to continuous nodes that are running?**
A: Cloned continuous nodes have `runtime_state = None` (not running). User must start them manually.

## Next Steps

1. **Review**: Read [plan.md](./plan.md) for overview
2. **Approve**: Confirm design decisions match requirements
3. **Implement**: Follow [tasks.md](./tasks.md) phase by phase
4. **Test**: Run full test suite after each phase
5. **Document**: Update CLAUDE.md when complete

## Status Updates

**2025-10-27**: Planning complete, documents written, ready for implementation

---

**Maintained by**: Claude Code
**Questions?**: Review [spec.md](./spec.md) for detailed information
