# Feature Specification: Component Directory Reorganization

**Feature Branch**: `009-reorginize-components-currently`
**Created**: 2025-10-22
**Status**: Draft
**Input**: User description: "reorginize-components. Currently when the app starts up it loads components from the components folder. I'd like to move all these wasm files to the components/bin folder and update the application to load from there. After lets move all the example components into the components directory, so all wasm components can be found in a single place. Also update the README.md in the components folder with this update."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Organize production components in bin subdirectory (Priority: P1)

When application maintainers want to separate built artifacts from source-level documentation and configuration, they need a clear directory structure that distinguishes runtime binaries from other files.

**Why this priority**: This establishes the foundational directory structure that all other changes depend on. Without this, users cannot access any components.

**Independent Test**: Can be fully tested by starting the application after moving WASM files to components/bin/ and verifying all previously available components still appear in the node palette.

**Acceptance Scenarios**:

1. **Given** the application is starting up, **When** it scans for components, **Then** it loads all .wasm files from components/bin/ directory
2. **Given** WASM files exist in both components/ and components/bin/, **When** the application loads, **Then** it only loads components from components/bin/
3. **Given** the components/bin/ directory does not exist, **When** the application starts, **Then** it displays an appropriate message indicating no components were found

---

### User Story 2 - Consolidate example component source code in components directory (Priority: P2)

Component developers reference example implementations scattered across the examples/ directory structure. To simplify discovery and organization, all example component source code should be moved into the components/ directory alongside production components, with binaries built to components/bin/.

**Why this priority**: This improves developer experience and creates a unified location for all component code (both examples and production). The application can run without example components.

**Independent Test**: Can be tested by verifying that example source directories (e.g., components/http-fetch/, components/json-parser/) exist in components/ and their built binaries appear in components/bin/ when loaded by the application.

**Acceptance Scenarios**:

1. **Given** example source directories exist in examples/, **When** they are moved to components/ with "example-" prefix removed, **Then** developers can find example source code in components/ subdirectories
2. **Given** example components are built from components/ subdirectories, **When** their binaries are placed in components/bin/, **Then** they appear in the node palette alongside production components
3. **Given** both example source (in components/http-fetch/) and binaries (in components/bin/) exist, **When** a developer wants to reference an example, **Then** they can find the source in components/http-fetch/ and the built artifact in components/bin/http_fetch.wasm

---

### User Story 3 - Updated documentation reflects new structure (Priority: P3)

Users consulting the README for component loading instructions need accurate information about where to place WASM files and how the directory is structured.

**Why this priority**: Documentation is important but can be updated independently. The application can function correctly even if documentation is temporarily outdated.

**Independent Test**: Can be tested by reading the README and following its instructions to place a new component, then verifying the component loads successfully.

**Acceptance Scenarios**:

1. **Given** a user reads the components/README.md, **When** they follow the instructions to add a new component, **Then** they successfully place it in components/bin/ and it loads in the application
2. **Given** the README describes the directory structure, **When** a user reads it, **Then** they understand that WASM binaries go in bin/ while documentation stays in the root components/ directory
3. **Given** the README provides examples, **When** showing file paths, **Then** all paths correctly reference components/bin/ for WASM files

---

### Edge Cases

- What happens when components/bin/ directory doesn't exist on first startup?
- How does the system handle WASM files that remain in the old components/ location after migration?
- What happens if a WASM file exists in both components/ and components/bin/ with the same name?
- How does the system respond when the components directory exists but bin/ subdirectory is empty?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST load WASM components from components/bin/ directory instead of components/ directory
- **FR-002**: System MUST scan only the components/bin/ directory for .wasm files (not components/ root or other subdirectories)
- **FR-003**: System MUST display an appropriate status message when components/bin/ directory does not exist or is empty
- **FR-004**: All existing production WASM components (rust_convert_f32_to_u32.wasm, rust_convert_u32_to_f32.wasm, rust_string.wasm, etc.) MUST be moved to components/bin/
- **FR-005**: All example component source directories from examples/ (example-file-reader, example-footer-view, example-http-fetch, example-json-parser, etc.) MUST be moved to components/ with "example-" prefix removed (becoming components/file-reader/, components/footer-view/, components/http-fetch/, components/json-parser/, etc.)
- **FR-005a**: Example component binaries MUST be built to components/bin/ from their new locations
- **FR-006**: Components README.md MUST be updated to reflect that WASM files are placed in components/bin/ directory
- **FR-007**: README.md directory structure documentation MUST show bin/ subdirectory containing WASM files
- **FR-008**: README.md quick start instructions MUST reference components/bin/ as the destination for copying WASM files
- **FR-009**: README.md troubleshooting sections MUST reference components/bin/ when discussing file location issues
- **FR-010**: The "Reload Components" menu action MUST scan components/bin/ directory

### Key Entities

- **Component Binary File**: A .wasm file containing compiled WebAssembly component code, located in components/bin/
- **Components Directory**: Root directory at components/ containing documentation (README.md) and the bin/ subdirectory
- **Bin Subdirectory**: Subdirectory at components/bin/ containing all WASM component binaries
- **Example Component**: A WASM component built from source in examples/ directory, with binary copied to components/bin/

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Application successfully loads all components from components/bin/ directory on startup
- **SC-002**: All existing components that previously loaded from components/ now load from components/bin/ without functionality changes
- **SC-003**: Zero WASM files remain in components/ root directory after reorganization (all moved to bin/ subdirectory)
- **SC-004**: All example WASM components are accessible in components/bin/ alongside production components
- **SC-005**: Documentation accurately describes the new structure with 100% of code examples and file paths referencing components/bin/
- **SC-006**: A developer following README instructions can successfully place a component in components/bin/ and have it load on first attempt

## Assumptions *(mandatory)*

1. **Directory Creation**: The components/bin/ directory will need to be created as part of this change (it does not currently exist)
2. **Example Component Selection**: Only the final release builds from examples/*/target/wasm32-wasip2/release/*.wasm will be copied (not intermediate build artifacts in deps/)
3. **File Permissions**: The application has read permissions for components/bin/ directory
4. **Backward Compatibility**: No existing graphs or configurations reference the physical location of component files (they reference components by name/ID, not file path)
5. **Component Loading Logic**: The component loading code in src/ui/app.rs (reload_components method) is the only place that needs updating for directory path changes
6. **Documentation Scope**: Only components/README.md needs updating; other documentation (docs/BUILDING_COMPONENTS.md, example READMEs) remain unchanged unless they specifically reference component storage location

## Dependencies *(mandatory)*

1. **No External Dependencies**: This is a pure reorganization task with no external library or tool requirements
2. **Build System**: Requires that example components be built before copying (assumes `cargo component build --release` has been run in example directories)
3. **Version Control**: Changes should preserve git history for moved files where possible

## Scope *(mandatory)*

### In Scope

- Moving all WASM files from components/ to components/bin/
- Moving example component source directories from examples/ to components/ (renaming to remove "example-" prefix)
- Updating component loading logic in src/ui/app.rs to scan components/bin/
- Updating build configurations in moved example components to output to components/bin/
- Updating components/README.md with new directory structure and paths
- Testing that all components load successfully from new location

### Out of Scope

- Changing how components are built or compiled
- Modifying the component interface or WIT specifications
- Updating example source code or build configurations
- Changing capability permission system or security model
- Modifying docs/BUILDING_COMPONENTS.md or example-specific READMEs (unless they contain specific references to where to copy final WASM files)
- Creating automated migration scripts for end users
- Changing the "File → Reload Components" menu workflow (only the underlying directory path changes)
