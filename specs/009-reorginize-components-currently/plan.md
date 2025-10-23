# Implementation Plan: Component Directory Reorganization

**Branch**: `009-reorginize-components-currently` | **Date**: 2025-10-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/009-reorginize-components-currently/spec.md`

## Summary

Reorganize the components directory structure to:
1. Separate WASM binary files into a dedicated `components/bin/` subdirectory
2. Move all example component source code from `examples/` to `components/` (dropping "example-" prefix)
3. Update documentation to reflect the new unified component organization

This change consolidates all component-related code (both production and examples) under a single `components/` directory with clear separation between source code and binaries.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)
**Primary Dependencies**:
- std::fs (file operations)
- std::path (path manipulation)
- No new crate dependencies required

**Storage**: File system (components directory structure)
**Testing**: Manual verification (component loading on startup), potential unit tests for path construction
**Target Platform**: Desktop (macOS, Linux, Windows) - host application
**Project Type**: Single project (desktop application)
**Performance Goals**: No performance impact (file system path change only)
**Constraints**:
- Must maintain backward compatibility for component loading behavior
- Must not break existing saved graphs (they reference components by name/ID, not path)
- Zero downtime migration (file move operations)
- Must preserve git history for moved files

**Scale/Scope**:
- ~7 existing WASM components in components/
- ~6 example component source directories to move from examples/ to components/
- Build configurations in example components to update (Cargo.toml output paths)
- 1 README file to update
- 2 code locations to update (src/ui/app.rs, src/builtin/wasm_creator.rs)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Component-First Architecture
✅ **PASS** - This feature does not modify component architecture, interfaces, or behavior. It only changes the file system organization where component source code and binaries are stored. Moving examples to components/ reinforces the component-first approach by unifying all components in one location.

### Principle II: Capability-Based Security (NON-NEGOTIABLE)
✅ **PASS** - No security implications. Component loading and capability enforcement logic remains unchanged. The host application still has the same file system permissions.

### Principle III: Typed Data Flow
✅ **PASS** - No changes to type checking, data flow, or component interfaces.

### Principle IV: Performance-First Execution
✅ **PASS** - No performance impact. File system reorganization does not affect component execution, instantiation, or graph processing. Directory scanning remains a one-time startup operation.

### Principle V: Developer Ergonomics
✅ **PASS** - **SIGNIFICANTLY IMPROVES** developer experience:
- Unified location for all component code (examples + production)
- Clearer directory structure (source in components/, binaries in components/bin/)
- Easier discovery of examples (all in components/ directory)
- Simplified naming (no "example-" prefix confusion)

### Principle VI: Composability & Modularity
✅ **PASS** - No changes to component composition, WAC integration, or metadata. Component discovery mechanism updates to scan new location. Examples become first-class components.

### Code Quality Gates
✅ **PASS** - This is a refactoring/reorganization task with minimal code changes:
- File operations are already well-tested via existing component loading
- Integration test: verify components load from new location
- No new error paths or edge cases beyond existing directory handling

### Dependency Policy
✅ **PASS** - No new dependencies added. Uses existing std::fs and std::path modules.

**Overall Constitution Compliance**: ✅ **FULL COMPLIANCE** - No violations or exceptions required. This change actually **enhances** developer ergonomics (Principle V) significantly.

## Project Structure

### Documentation (this feature)

```
specs/009-reorginize-components-currently/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output
├── quickstart.md        # Phase 1 output (migration guide)
├── spec.md              # Feature specification
└── checklists/
    └── requirements.md  # Specification quality checklist
```

**Note**: No data-model.md or contracts/ needed - this is a file reorganization task with no data entities or API contracts.

### Source Code (repository root)

```
# BEFORE (current structure)
src/
├── ui/
│   └── app.rs                    # WILL MODIFY: reload_components() method
├── builtin/
│   └── wasm_creator.rs          # WILL MODIFY: components_dir path
└── ...

components/                       # Mixed: binaries + docs
├── README.md
├── example_file_reader.wasm
├── example_footer_view.wasm
├── example_http_fetch.wasm
├── example_json_parser.wasm
├── rust_convert_f32_to_u32.wasm
├── rust_convert_u32_to_f32.wasm
└── rust_string.wasm

examples/                         # Example source code
├── double-number/
│   ├── src/
│   ├── Cargo.toml
│   └── target/wasm32-wasip2/release/double_number.wasm
├── example-file-reader/
│   ├── src/
│   ├── Cargo.toml
│   └── target/...
├── example-footer-view/
│   ├── src/
│   ├── Cargo.toml
│   └── target/...
├── example-http-fetch/
│   ├── src/
│   ├── Cargo.toml
│   └── target/...
├── example-json-parser/
│   ├── src/
│   ├── Cargo.toml
│   └── target/...
└── example-adder/
    ├── src/
    ├── Cargo.toml
    └── target/...
```

```
# AFTER (new structure)
src/
├── ui/
│   └── app.rs                    # MODIFIED: scans components/bin/
├── builtin/
│   └── wasm_creator.rs          # MODIFIED: outputs to components/bin/
└── ...

components/                       # Unified: all component code
├── README.md                     # UPDATED: new structure docs
├── bin/                          # NEW: all WASM binaries
│   ├── file_reader.wasm              # RENAMED: dropped "example_"
│   ├── footer_view.wasm              # RENAMED
│   ├── http_fetch.wasm               # RENAMED
│   ├── json_parser.wasm              # RENAMED
│   ├── double_number.wasm            # NEW: from examples
│   ├── adder.wasm                    # NEW: from examples (example-adder)
│   ├── rust_convert_f32_to_u32.wasm  # MOVED from components/
│   ├── rust_convert_u32_to_f32.wasm  # MOVED
│   └── rust_string.wasm              # MOVED
├── file-reader/                  # MOVED from examples/example-file-reader
│   ├── src/
│   └── Cargo.toml                # UPDATED: output to ../bin/
├── footer-view/                  # MOVED from examples/example-footer-view
│   ├── src/
│   └── Cargo.toml                # UPDATED
├── http-fetch/                   # MOVED from examples/example-http-fetch
│   ├── src/
│   └── Cargo.toml                # UPDATED
├── json-parser/                  # MOVED from examples/example-json-parser
│   ├── src/
│   └── Cargo.toml                # UPDATED
├── double-number/                # MOVED from examples/double-number
│   ├── src/
│   └── Cargo.toml                # UPDATED
└── adder/                        # MOVED from examples/example-adder
    ├── src/
    └── Cargo.toml                # UPDATED

examples/                         # Remains for non-component examples
├── debug_json_parser.rs          # Test/debug scripts
├── test_json_parser.rs
├── test_large_json.rs
├── Justfile
└── graphs/                       # Example graphs
```

**Structure Decision**: This is a single-project desktop application. The reorganization:
1. Creates `components/bin/` for all WASM binaries
2. Moves example source directories to `components/` (dropping "example-" prefix)
3. Updates component build configs to output to `components/bin/`
4. Leaves non-component examples (test scripts, graphs) in `examples/`
5. Updates loading logic in `src/ui/app.rs` and `src/builtin/wasm_creator.rs`

## Complexity Tracking

*No Constitution violations - this section not needed.*

## Phase 0: Research & Discovery

### Research Questions

1. **Directory Move Strategy**: Git mv for source directories
   - **Decision**: Use `git mv` for example directories to preserve history
   - **Rationale**: Maintains blame/history, standard practice

2. **Naming Convention**: Drop "example-" prefix
   - **Decision**: Rename `example-http-fetch` → `http-fetch`, `example-json-parser` → `json-parser`
   - **Special Case**: `example-adder` → `adder` (becomes `adder.wasm`)
   - **Keep As-Is**: `double-number` (already has good name)
   - **Rationale**: Cleaner names, examples become first-class components

3. **Binary Output Configuration**: Update Cargo.toml in moved components
   - **Decision**: Update each component's Cargo.toml or build process to output binaries to `../bin/`
   - **Research Needed**: How to configure cargo-component output directory
   - **Alternative**: Keep default build path, add post-build copy step

4. **Non-Component Examples**: What stays in examples/?
   - **Decision**: Keep test/debug scripts and example graphs
   - **Files to Keep in examples/**:
     - `debug_json_parser.rs`
     - `test_json_parser.rs`
     - `test_large_json.rs`
     - `Justfile`
     - `graphs/` directory
   - **Rationale**: These are not components, just helper scripts

5. **Build System Impact**: Do any build scripts reference examples/?
   - **Investigation Needed**: Check for Justfile, Makefile, CI scripts
   - **Expected**: Update any scripts that build examples

### Technologies & Patterns

**File Operations**:
- Rust std::fs::read_dir() for directory scanning (already in use)
- std::path::Path::new() for path construction (already in use)
- Git mv for directory moves (preserves history)

**Cargo Component Build Configuration**:
- Research cargo-component output directory configuration
- May need to update Cargo.toml `[package.metadata.component]` section
- Or use post-build copy commands

**Documentation Updates**:
- Markdown editing for components/README.md
- Update all path references
- Update directory structure diagrams

### Best Practices

- **File Migration**: Use git mv to preserve history
- **Path Construction**: Use Path::join() for cross-platform compatibility
- **Build Configuration**: Keep build process simple (prefer standard cargo paths + copy)
- **Testing**: Verify all components build and load after migration

## Phase 1: Design & Contracts

### Data Model

**Not Applicable** - This feature does not introduce or modify data entities. It only changes file system organization.

### API Contracts

**Not Applicable** - This feature does not introduce or modify API contracts. The component loading interface remains unchanged.

### Migration Guide (quickstart.md)

A quickstart document will guide:
1. Developers on the new components/ structure
2. Where to find example source code (components/http-fetch/ etc.)
3. Where to place new component source (components/my-component/)
4. Where binaries are stored (components/bin/)
5. How to build components from new locations

### Key Design Decisions

1. **Example Directory Naming**:
   - Drop "example-" prefix for cleaner names
   - `example-http-fetch` → `http-fetch`
   - `example-json-parser` → `json-parser`
   - `example-file-reader` → `file-reader`
   - `example-footer-view` → `footer-view`
   - `example-adder` → `adder`
   - `double-number` → `double-number` (unchanged)

2. **Binary Output Location**:
   - All components build to `components/bin/`
   - Update WASM Creator to output to `components/bin/`
   - Consistent pattern for all components

3. **Build Configuration Updates**:
   - **Option A**: Update Cargo.toml to set output directory
   - **Option B**: Keep default target/ build, add manual copy step
   - **Decision**: Use Option B (manual copy) for simplicity
   - **Rationale**: Cargo-component doesn't easily support custom output dirs; manual copy is explicit and works

4. **Examples Directory Cleanup**:
   - Keep non-component files (test scripts, graphs, Justfile)
   - Remove component directories (moved to components/)
   - **Result**: examples/ becomes utility scripts location

5. **Documentation Scope**:
   - Update `components/README.md` to show new structure
   - Document how to build from new locations
   - Update directory tree diagrams
   - Show example component locations

### Files to Modify

#### Source Code (2 files)

1. **src/ui/app.rs** (line ~1527):
   - Change: `let components_dir = std::path::Path::new("components");`
   - To: `let components_dir = std::path::Path::new("components/bin");`
   - Update error messages to reference components/bin/

2. **src/builtin/wasm_creator.rs** (line ~584):
   - Change: `.join("components")`
   - To: `.join("components").join("bin")`
   - Update success/error messages

#### Directory Moves (6 moves)

Using `git mv` to preserve history:

3. `git mv examples/example-file-reader components/file-reader`
4. `git mv examples/example-footer-view components/footer-view`
5. `git mv examples/example-http-fetch components/http-fetch`
6. `git mv examples/example-json-parser components/json-parser`
7. `git mv examples/double-number components/double-number`
8. `git mv examples/example-adder components/adder`

#### WASM Binary Moves (7 moves)

9. Create `components/bin/` directory
10. Move existing WASM files to components/bin/:
    - `git mv components/example_file_reader.wasm components/bin/file_reader.wasm`
    - `git mv components/example_footer_view.wasm components/bin/footer_view.wasm`
    - `git mv components/example_http_fetch.wasm components/bin/http_fetch.wasm`
    - `git mv components/example_json_parser.wasm components/bin/json_parser.wasm`
    - `git mv components/rust_convert_f32_to_u32.wasm components/bin/`
    - `git mv components/rust_convert_u32_to_f32.wasm components/bin/`
    - `git mv components/rust_string.wasm components/bin/`

#### Documentation (1-2 files)

11. **components/README.md**:
    - Update "Directory Structure" section to show new layout
    - Update "Quick Start" to reference building from components/
    - Update all path examples
    - Show how to build components from components/ subdirectories

12. **docs/BUILDING_COMPONENTS.md** (if references component locations):
    - Update any path references

### Build Process Updates

For each moved component, create a simple build script or document the process:

**Manual Build Process** (recommended for simplicity):
```bash
# Build a component
cd components/http-fetch
cargo component build --release

# Copy to bin/
cp target/wasm32-wasip2/release/example_http_fetch.wasm ../bin/http_fetch.wasm
```

**Or add build helper** (e.g., in examples/Justfile or components/Justfile):
```justfile
# Build all components
build-all:
    for dir in components/*/; do
        (cd $dir && cargo component build --release)
    done
    # Copy binaries to bin/
    cp components/*/target/wasm32-wasip2/release/*.wasm components/bin/
```

### Testing Strategy

**Integration Test**:
1. Move directories and update code
2. Build components from new locations
3. Copy binaries to components/bin/
4. Start application
5. Verify status bar shows all components loaded
6. Test component execution in graph

**Edge Case Tests**:
1. Missing components/bin/ directory
2. Empty components/bin/ directory
3. Component source in components/ subdirectory builds correctly
4. WASM Creator outputs to components/bin/

**Documentation Test**:
1. Follow README instructions to build a component
2. Verify component loads successfully

## Next Steps

After plan approval, proceed to `/speckit.tasks` to generate the task breakdown for implementation.

**Expected Task Structure**:
- Task 1: Create components/bin/ directory
- Task 2: Move existing WASM files to components/bin/ (with renaming)
- Task 3: Move example source directories to components/ (with renaming)
- Task 4: Update src/ui/app.rs component loading logic
- Task 5: Update src/builtin/wasm_creator.rs output path
- Task 6: Build components from new locations
- Task 7: Update components/README.md documentation
- Task 8: Verify all components load successfully
- Task 9: Test WASM Creator node output location
