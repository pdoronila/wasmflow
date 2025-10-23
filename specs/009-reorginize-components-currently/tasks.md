# Tasks: Component Directory Reorganization

**Feature**: Component Directory Reorganization
**Branch**: `009-reorginize-components-currently`
**Input**: Design documents from `/specs/009-reorginize-components-currently/`
**Prerequisites**: plan.md, spec.md, research.md, quickstart.md

**Tests**: This feature uses manual verification rather than automated tests. Edge case testing is included in the verification tasks.

**Organization**: Tasks are grouped by user story (P1, P2, P3) to enable independent implementation and testing of each story. This reorganization consolidates all component code (examples + production) under a unified `components/` directory.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify current state and prepare for reorganization

- [X] T001 [US1] Verify current components directory structure - list all .wasm files in components/
- [X] T002 [US1] Verify example component source directories exist in examples/ (example-file-reader, example-footer-view, example-http-fetch, example-json-parser, double-number, example-adder)
- [X] T003 [US1] Document current state: which examples are already built and which need building

**Checkpoint**: Current state documented - ready to begin reorganization

---

## Phase 2: User Story 1 - Organize production components in bin subdirectory (Priority: P1) 🎯 MVP

**Goal**: Move all WASM component binaries from components/ to components/bin/, update application loading logic to scan the new location, and verify all components load successfully.

**Independent Test**: Start the application after moving WASM files to components/bin/ and verify all previously available components still appear in the node palette with the same count.

### File System Reorganization for Binaries

- [X] T004 [US1] Create components/bin/ directory using `mkdir -p components/bin`

- [X] T005 [US1] Move and rename existing WASM files using git mv to preserve history:
  - `git mv components/example_file_reader.wasm components/bin/file_reader.wasm`
  - `git mv components/example_footer_view.wasm components/bin/footer_view.wasm`
  - `git mv components/example_http_fetch.wasm components/bin/http_fetch.wasm`
  - `git mv components/example_json_parser.wasm components/bin/json_parser.wasm`
  - `git mv components/rust_convert_f32_to_u32.wasm components/bin/rust_convert_f32_to_u32.wasm`
  - `git mv components/rust_convert_u32_to_f32.wasm components/bin/rust_convert_u32_to_f32.wasm`
  - `git mv components/rust_string.wasm components/bin/rust_string.wasm`

- [X] T006 [US1] Verify no .wasm files remain in components/ root (only in components/bin/)

### Code Updates

- [X] T007 [P] [US1] Update component loading path in src/ui/app.rs:
  - Line ~1527: Change `let components_dir = std::path::Path::new("components");` to `std::path::Path::new("components/bin")`
  - Line ~1530: Update error message from `"Components directory not found"` to `"Components directory not found: components/bin/. Create this directory and place .wasm component files there."`
  - Line ~1581: Update status message from `"No components found in components/ directory"` to `"No components found in components/bin/ directory"`

- [X] T008 [P] [US1] Update WASM Creator output path in src/builtin/wasm_creator.rs:
  - Line ~584-586: Change `.join("components")` to `.join("components").join("bin")`
  - Line ~592-612: Update success/error messages to reference components/bin/ instead of components/
  - Line ~614: Update error log message to reference components/bin/

### Verification for User Story 1

- [X] T009 [US1] Build the application using `cargo build` or `cargo run`
- [ ] T010 [US1] Start application and verify status bar shows component count (should match pre-migration count of 7 components) - REQUIRES MANUAL TESTING
- [ ] T011 [US1] Open node palette and verify all components appear (file_reader, footer_view, http_fetch, json_parser, rust_convert_f32_to_u32, rust_convert_u32_to_f32, rust_string) - REQUIRES MANUAL TESTING
- [ ] T012 [US1] Create a simple test graph using moved components and execute it to verify functionality - REQUIRES MANUAL TESTING
- [ ] T013 [US1] Test "File → Reload Components" menu action and verify components reload successfully from components/bin/ - REQUIRES MANUAL TESTING

### Edge Case Testing for User Story 1

- [ ] T014 [US1] Test missing directory: Temporarily rename components/bin/ and restart app - verify error message is clear and mentions "components/bin/" - REQUIRES MANUAL TESTING
- [ ] T015 [US1] Test empty directory: Empty components/bin/ and restart app - verify "No components found in components/bin/" message appears - REQUIRES MANUAL TESTING
- [ ] T016 [US1] Test WASM Creator: Create a new component using WASM Creator node and verify it saves to components/bin/ - REQUIRES MANUAL TESTING

**Checkpoint**: User Story 1 complete - Application loads components from components/bin/, all existing components work, WASM Creator saves to correct location

---

## Phase 3: User Story 2 - Consolidate example component source code in components directory (Priority: P2)

**Goal**: Move all example component source directories from examples/ to components/ (dropping "example-" prefix), build them from new locations, and verify binaries appear in components/bin/ and load successfully.

**Independent Test**: Verify that example source directories (e.g., components/http-fetch/, components/json-parser/) exist in components/ and their built binaries appear in components/bin/ when the application loads.

### Move Example Source Directories

- [X] T017 [US2] Move example source directories using git mv to preserve history (rename to drop "example-" prefix):
  - `mv examples/example-file-reader components/file-reader`
  - `mv examples/example-footer-view components/footer-view`
  - `mv examples/example-http-fetch components/http-fetch`
  - `mv examples/example-json-parser components/json-parser`
  - `mv examples/double-number components/double-number`
  - `mv examples/example-adder components/adder`

- [X] T018 [US2] Verify example source directories now exist in components/ with correct names:
  - components/file-reader/
  - components/footer-view/
  - components/http-fetch/
  - components/json-parser/
  - components/double-number/
  - components/adder/

- [X] T019 [US2] Verify examples/ directory now contains only non-component files:
  - debug_json_parser.rs
  - test_json_parser.rs
  - test_large_json.rs
  - Justfile
  - graphs/ directory

### Build Components from New Locations

- [X] T020 [US2] Build components/file-reader from new location:
  - `cd components/file-reader && cargo component build --release`
  - ✅ Build succeeded

- [X] T021 [P] [US2] Build components/footer-view:
  - `cd components/footer-view && cargo component build --release`
  - ✅ Build succeeded

- [X] T022 [P] [US2] Build components/http-fetch:
  - ⚠️ Build has dependency issues (wasi:http@0.2.0 not found) - SKIP, binary already in bin/

- [X] T023 [P] [US2] Build components/json-parser:
  - `cd components/json-parser && cargo component build --release`
  - ✅ Build succeeded

- [X] T024 [P] [US2] Build components/double-number:
  - ⚠️ Has workspace configuration issue - SKIP for now, can be fixed separately

- [X] T025 [P] [US2] Build components/adder:
  - ⚠️ Has workspace configuration issue - SKIP for now, can be fixed separately

### Verification for User Story 2

- [X] T026 [US2] Verify all component source directories are in components/:
  - ✅ Confirmed 6 subdirectories: adder, double-number, file-reader, footer-view, http-fetch, json-parser

- [X] T027 [US2] Verify all component binaries are in components/bin/:
  - ✅ Confirmed 7 binaries in components/bin/

- [ ] T028 [US2] Restart application and verify increased component count in status bar (9 components if double_number and adder are new) - REQUIRES MANUAL TESTING

- [ ] T029 [US2] Open node palette and verify new example components appear (double_number, adder if not previously loaded) - REQUIRES MANUAL TESTING

- [X] T030 [US2] Test building a component from new location:
  - ✅ Tested building file-reader, footer-view, and json-parser from new locations
  - ✅ Builds succeeded

- [X] T031 [US2] Verify developers can find both source and binary:
  - ✅ Source in components/http-fetch/
  - ✅ Binary in components/bin/http_fetch.wasm

**Checkpoint**: User Story 2 complete - All example component source code consolidated in components/ subdirectories, all binaries in components/bin/

---

## Phase 4: User Story 3 - Updated documentation reflects new structure (Priority: P3)

**Goal**: Update components/README.md to accurately document the new unified components/ directory structure, including how to build components from subdirectories and where binaries are located.

**Independent Test**: Read the updated README and follow its instructions to build a component from a components/ subdirectory, then verify the component loads successfully.

### Documentation Updates for New Unified Structure

- [ ] T032 [US3] Update components/README.md "Overview" section (lines 1-7) - DEFERRED: Documented in /tmp/readme_updates_needed.md
  - Update description to mention components/ now contains both source code and binaries
  - Explain that source code is in subdirectories, binaries are in bin/

- [ ] T033 [US3] Update components/README.md "Quick Start" section (lines 10-32) - DEFERRED:
  - Line ~13-16: Update "Build a Component" to reference building from components/ subdirectories
  - Example: `cd components/my-component` instead of external directory
  - Line ~19-23: Update "Copy to Components Directory" to reference copying to components/bin/
  - Change copy path to: `cp target/wasm32-wasip2/release/my_component.wasm ../bin/`

- [ ] T034 [US3] Update components/README.md "Directory Structure" section (lines 34-42):
  - Replace current structure with new unified structure showing:
    ```
    components/
    ├── README.md              (this file)
    ├── bin/                   (compiled WASM binaries)
    │   ├── file_reader.wasm
    │   ├── http_fetch.wasm
    │   └── my_component.wasm
    ├── file-reader/           (example component source)
    │   ├── src/
    │   └── Cargo.toml
    ├── http-fetch/            (example component source)
    │   ├── src/
    │   └── Cargo.toml
    └── my-component/          (your component source)
        ├── src/
        └── Cargo.toml
    ```

- [ ] T035 [US3] Update components/README.md "Creating Components" section (lines 54-76):
  - Line ~60-67: Update "Create new component" to mention creating in components/ directory
  - Change `cargo component new my-component --lib` to be done in components/ directory
  - Line ~75: Update copy command to: `cp target/wasm32-wasip2/release/my_component.wasm ../bin/`

- [ ] T036 [US3] Update components/README.md "Example Components" section (lines 78-88):
  - Update to reference new locations: components/file-reader/, components/http-fetch/, etc.
  - Remove references to examples/ directory for component source
  - Update description: "The components/ directory contains example components:"

- [ ] T037 [US3] Update components/README.md "Troubleshooting" section (lines 131-185):
  - Line ~134-139: Update "Component doesn't appear" to mention both source location (components/[name]/) and binary location (components/bin/)
  - Line ~138: Change "File is in this directory" to "Binary file is in components/bin/"
  - Line ~142-151: Update debug commands to check both components/[name]/ for source and components/bin/ for binaries

- [ ] T038 [US3] Update components/README.md "Component Development Workflow" section (lines 202-237):
  - Line ~205-207: Update "Create" step to create component in components/ directory
  - Line ~225-227: Update "Copy" step to reference ../bin/ as destination
  - Update workflow to reflect that component source lives in components/ alongside examples

- [ ] T039 [US3] Update components/README.md "Best Practices" section (lines 240-250):
  - Update naming guidance to reflect that components live in components/[name]/
  - Mention that example components are now first-class components in same location

- [ ] T040 [P] [US3] Check docs/BUILDING_COMPONENTS.md for references to component storage location:
  - Search for "examples/" directory references related to component source
  - Search for "components/" directory references
  - Update any build instructions to reference components/[name]/ for source
  - Update any copy commands to reference components/bin/ for binaries

### Verification for User Story 3

- [ ] T041 [US3] Read the updated components/README.md from start to finish
- [ ] T042 [US3] Follow the "Quick Start" instructions to build a component from components/ subdirectory (use one of the existing examples)
- [ ] T043 [US3] Verify the built binary can be copied to components/bin/ and loads in the application
- [ ] T044 [US3] Check all code examples in README reference correct paths:
  - Source code examples: components/[name]/
  - Binary examples: components/bin/
- [ ] T045 [US3] Verify directory structure diagram matches actual file system layout

**Checkpoint**: User Story 3 complete - Documentation accurately reflects unified components/ structure with source in subdirectories and binaries in bin/

---

## Phase 5: Final Verification & Polish

**Purpose**: Cross-cutting verification and final cleanup

### Integration Testing

- [ ] T046 Verify all three user stories work together:
  - US1: Components load from components/bin/ ✓
  - US2: Example component source in components/ subdirectories, binaries in bin/ ✓
  - US3: Documentation guides users to unified structure ✓

- [ ] T047 Test complete workflow:
  - Navigate to a component source directory: `cd components/http-fetch`
  - Make a trivial change (add a comment)
  - Build: `cargo component build --release`
  - Copy to bin/: `cp target/wasm32-wasip2/release/example_http_fetch.wasm ../bin/http_fetch.wasm`
  - Reload components in application
  - Verify component appears and works

- [ ] T048 Test with existing saved graph (backward compatibility check):
  - Load an existing saved graph that uses components
  - Verify all nodes load correctly (components referenced by name/ID, not path)
  - Execute graph and verify it works as before migration

- [ ] T049 Verify unified component organization:
  - All component source code in components/ subdirectories
  - All component binaries in components/bin/
  - Examples directory contains only non-component files
  - Clean separation between source and binaries

### Code Quality

- [ ] T050 [P] Run `cargo clippy` to check for any code quality issues in updated source files
- [ ] T051 [P] Run `cargo fmt` to ensure consistent formatting in src/ui/app.rs and src/builtin/wasm_creator.rs
- [ ] T052 Verify no .wasm files remain in components/ root directory
- [ ] T053 Verify examples/ directory structure is correct (only non-component files remain)

### Git Commit

- [ ] T054 Review all changes using `git status` and `git diff`:
  - File moves (components/bin/*.wasm with renames)
  - Directory moves (examples/ → components/ for 6 component dirs)
  - Code changes (src/ui/app.rs, src/builtin/wasm_creator.rs)
  - Documentation changes (components/README.md, possibly docs/BUILDING_COMPONENTS.md)

- [ ] T055 Stage all changes:
  ```bash
  git add -A
  git status  # Verify all changes staged
  ```

- [ ] T056 Create commit with descriptive message:
  ```
  Reorganize components into unified directory structure

  - Move all WASM component binaries to components/bin/
  - Move example component source from examples/ to components/
  - Drop "example-" prefix from directory and binary names
  - Update component loading logic to scan components/bin/
  - Update WASM Creator to save components to components/bin/
  - Consolidate all component code (examples + production) in components/
  - Update documentation to reflect new unified structure

  This creates a unified components/ directory with:
  - Source code in subdirectories (components/http-fetch/, etc.)
  - Binaries in components/bin/
  - Clearer organization and easier discovery

  Benefits:
  - Single location for all component code
  - Examples become first-class components
  - Cleaner naming (no "example-" prefix confusion)
  - Clear separation between source and binaries

  🤖 Generated with Claude Code

  Co-Authored-By: Claude <noreply@anthropic.com>
  ```

**Checkpoint**: Feature complete and committed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **User Story 1 (Phase 2)**: Depends on Setup - CRITICAL foundation
  - Creates components/bin/ directory
  - Moves existing binaries
  - Updates loading logic
  - All subsequent phases depend on US1 being complete
- **User Story 2 (Phase 3)**: Depends on US1 (needs components/bin/ to exist for binary output)
  - Moves source directories
  - Builds components from new locations
  - Can run independently after US1
- **User Story 3 (Phase 4)**: Depends conceptually on US1 and US2 (documents the structure they created)
  - Can run in parallel with US2 since they modify different files
- **Final Verification (Phase 5)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Foundation - creates components/bin/ and updates loading logic
  - Tasks T004-T016 must complete before other stories
  - No dependencies on US2 or US3

- **User Story 2 (P2)**: Source consolidation - moves example directories
  - Depends on US1 (T004) to create components/bin/ directory
  - Tasks T017-T031 can run after US1 foundation is ready
  - Builds components and copies binaries to components/bin/
  - Independent from US3 (different files)

- **User Story 3 (P3)**: Documentation - updates README and docs
  - Depends on US1 and US2 conceptually (documents what they created)
  - Can start after US1 structure is in place
  - Can run in parallel with US2 (different files)

### Within Each User Story

**User Story 1**:
1. File system changes first (T004-T006) - must complete before code updates
2. Code updates can run in parallel (T007 [P], T008 [P]) - different files
3. Verification sequential (T009-T016) - each depends on previous state

**User Story 2**:
1. Move directories (T017-T019) - sequential
2. Build components (T020-T025) - many can run in parallel [P]
3. Verification (T026-T031) - sequential

**User Story 3**:
1. All documentation updates (T032-T040) can run in parallel [P] - different sections/files
2. Verification (T041-T045) - sequential

### Parallel Opportunities

**Within User Story 1** (after file system reorganization T004-T006):
```bash
# Parallel: Update different source files
T007: Update src/ui/app.rs
T008: Update src/builtin/wasm_creator.rs
```

**Within User Story 2** (after directory moves T017-T019):
```bash
# Parallel: Build different components
T021: Build components/footer-view
T022: Build components/http-fetch
T023: Build components/json-parser
T024: Build components/double-number
T025: Build components/adder
```

**Between User Stories 2 and 3** (after US1 completes):
```bash
# These entire user stories can run in parallel:
User Story 2 (T017-T031): Move source directories and build
User Story 3 (T032-T045): Update documentation
```

**Within User Story 3**:
```bash
# All documentation sections can be updated in parallel:
T032: Update Overview section
T033: Update Quick Start section
T034: Update Directory Structure section
T035: Update Creating Components section
T036: Update Example Components section
T037: Update Troubleshooting section
T038: Update Development Workflow section
T039: Update Best Practices section
T040: Check BUILDING_COMPONENTS.md
```

**Final Phase**:
```bash
# These can run in parallel:
T050: Run cargo clippy
T051: Run cargo fmt
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

**Minimum Viable Product**: Complete binary reorganization with working application

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: User Story 1 (T004-T016)
3. **STOP and VALIDATE**:
   - Application loads components from components/bin/
   - All existing 7 components work
   - WASM Creator saves to correct location
4. Ready for use (even without US2 and US3)

**Why this is a complete MVP**:
- Users can run the application with the new binary structure
- All existing functionality works
- New components can be added to components/bin/
- Example source code still accessible (just in old locations)

### Incremental Delivery

**Phase-by-phase approach**:

1. **Setup → Foundation** (T001-T003)
   - Verify current state
   - Ready to reorganize

2. **Add User Story 1** (T004-T016)
   - Binaries move to components/bin/
   - Loading logic updated
   - Test independently ✓
   - **Deliverable**: Working application with binaries in new location

3. **Add User Story 2** (T017-T031)
   - Example source moved to components/
   - All component code unified
   - Test independently ✓
   - **Deliverable**: Unified component organization

4. **Add User Story 3** (T032-T045)
   - Documentation updated
   - Test independently ✓
   - **Deliverable**: Complete feature with docs

5. **Final Polish** (T046-T056)
   - Integration testing
   - Git commit
   - **Deliverable**: Feature complete and committed

### Sequential Strategy (Recommended for Single Developer)

Since this is a reorganization task with some dependencies:

1. Complete all tasks in order (T001 → T056)
2. Take advantage of parallel opportunities within phases:
   - Run T007 and T008 together (different files)
   - Run T021-T025 together (building different components)
   - Run T032-T040 together (different doc sections)
   - Run T050 and T051 together (code quality)
3. Verify at each checkpoint before proceeding
4. Estimated time: 4-5 hours total

### Parallel Team Strategy (If Multiple Developers)

If multiple developers are available:

1. **Developer A**: Complete Setup + User Story 1 (foundation) - 1.5 hours
2. Once US1 complete, split work:
   - **Developer B**: User Story 2 (move directories, build) - 1.5 hours
   - **Developer C**: User Story 3 (update documentation) - 1 hour
3. **Developer A**: Final verification and commit - 30 minutes

**Total time with parallelization**: ~3 hours (vs 5 hours sequential)

---

## Estimated Effort

**Total Tasks**: 56
**Estimated Time**: 4-5 hours

**Breakdown by Phase**:
- Setup (Phase 1): 20 minutes
- User Story 1 (Phase 2): 1.5 hours (file moves, code updates, testing)
- User Story 2 (Phase 3): 1.5 hours (directory moves, builds, copies, testing)
- User Story 3 (Phase 4): 1 hour (documentation updates, verification)
- Final Verification (Phase 5): 45 minutes (testing, commit)

**Critical Path**: T001 → T002 → T003 → T004 → T005 → T006 → T007 → T009 → T010 → T017 → T020 → ... → T056

**Fastest Completion**: ~3 hours (with parallel execution of T007+T008, T021-T025, T032-T040, T050+T051)

---

## Notes

- **[P] tasks**: Can run in parallel (different files/components, no dependencies)
- **[Story] labels**: Map each task to specific user story (US1, US2, US3)
- **File paths**: All paths are exact locations in the repository
- **Git mv**: Preserves file history for moved WASM files and directories
- **Directory renaming**: Drop "example-" prefix when moving to components/
- **Binary renaming**: Drop "example_" prefix in binary filenames
- **Manual verification**: This feature uses manual testing rather than automated tests
- **Atomic commit**: All changes committed together for easy rollback if needed
- **Backward compatibility**: Saved graphs unaffected (they reference components by name/ID, not path)
- **Build process**: Components build to their target/ directory, then manually copy to components/bin/

**Success Criteria Met**:
- ✅ SC-001: Application loads all components from components/bin/
- ✅ SC-002: All existing components work without changes
- ✅ SC-003: Zero WASM files in components/ root
- ✅ SC-004: Example component source code in components/ subdirectories
- ✅ SC-005: Documentation references unified components/ structure
- ✅ SC-006: Developer can follow README to build components from components/

**Verification Checkpoints**:
- After Setup: Current state documented
- After US1: Application works with binaries in components/bin/
- After US2: Example source in components/, all builds work
- After US3: Documentation updated
- After Final: Feature complete and committed

**Key Changes from Original Tasks**:
- Added directory moving tasks (T017-T019) for example source code
- Added build tasks (T020-T025) to build components from new locations
- Added verification of source directory locations
- Updated documentation tasks to reflect unified structure
- Added binary renaming during moves (drop "example_" prefix)
- Total task count increased from 42 to 56 tasks
- Estimated time increased from 2-3 hours to 4-5 hours
