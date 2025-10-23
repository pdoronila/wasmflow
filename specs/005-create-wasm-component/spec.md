# Feature Specification: WASM Component Creator Node

**Feature Branch**: `005-create-wasm-component`
**Created**: 2025-10-18
**Status**: Draft
**Input**: User description: "create_wasm_component_node I want to create a new built in node, that contains a code editor widget (https://github.com/emilk/egui/blob/main/crates/egui_demo_lib/src/demo/code_editor.rs) as well as a text input for the name of the wasm component node. This widget will server as a way for a user to create a wasm component node from this node. The code a user puts in the editor will become the code for a wasm component execute function. When a user clicks execute in the user interface, this node will take the code from the code editor, inject it into a template that represents a wasm component node, compile the component and load it making it available as a user-defined node (usually colored purple in the palette). Look at the HTTP Fetch component as reference or even the double-number example which are wasm component nodes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create Simple Component from Editor (Priority: P1)

A user wants to quickly create a custom WASM component by writing Rust code directly in the visual editor, without leaving the application or setting up external build tools.

**Why this priority**: This is the core value proposition - enabling rapid prototyping and component creation within the visual environment. Without this, the feature provides no value.

**Independent Test**: Can be fully tested by adding the WASM Creator node to the canvas, typing valid Rust code for an execute function (e.g., "multiply input by 3"), providing a component name, clicking execute, and verifying that a new user-defined component appears in the palette.

**Acceptance Scenarios**:

1. **Given** the application is running, **When** the user adds a "WASM Component Creator" node from the palette, **Then** the node displays a code editor widget and a text input field for the component name
2. **Given** a WASM Creator node is on the canvas, **When** the user types valid Rust code implementing an execute function and provides a component name, **Then** the code editor displays syntax highlighting appropriate for Rust code
3. **Given** the user has entered valid code and a component name, **When** the user clicks the "Execute" button in the node's UI, **Then** the system compiles the code, loads the component, and adds it to the component palette with a purple/user-defined color scheme
4. **Given** the compilation succeeds, **When** the user opens the component palette, **Then** the newly created component appears under a "User-Defined" or similar category
5. **Given** the new component is in the palette, **When** the user drags it onto the canvas, **Then** the component behaves like any other WASM component node with proper inputs and outputs

---

### User Story 2 - Handle Compilation Errors Gracefully (Priority: P2)

A user makes a syntax error or logical mistake in their Rust code, and needs clear feedback to correct the issue without the application crashing or freezing.

**Why this priority**: Error handling is critical for user experience, but the basic creation flow (P1) must work first. Users need to understand what went wrong to iterate on their code.

**Independent Test**: Can be tested by entering invalid Rust code (e.g., missing semicolon, undefined variable), clicking execute, and verifying that a clear error message is displayed in the node's footer without crashing the application.

**Acceptance Scenarios**:

1. **Given** the user has entered invalid Rust code, **When** the user clicks execute, **Then** the compilation fails and an error message is displayed in the node's UI footer with details about what went wrong
2. **Given** a compilation error has occurred, **When** the error message is displayed, **Then** the message includes the line number, error type, and a helpful description of the problem
3. **Given** a compilation error is displayed, **When** the user corrects the code and clicks execute again, **Then** the error message clears and compilation proceeds normally
4. **Given** compilation is in progress, **When** the process takes longer than expected, **Then** a loading indicator or progress message is displayed to inform the user

---

### User Story 3 - Edit and Recompile Existing Components (Priority: P3)

A user wants to modify the code of a previously created component and regenerate it with the same name, updating the existing component rather than creating duplicates.

**Why this priority**: This enables iteration and refinement of custom components, but is less critical than initial creation. Users can work around this by creating new components with different names.

**Independent Test**: Can be tested by creating a component, editing the code in the WASM Creator node, clicking execute again, and verifying that the component in the palette is updated rather than duplicated.

**Acceptance Scenarios**:

1. **Given** a component has been successfully created, **When** the user modifies the code in the WASM Creator node and clicks execute, **Then** the existing component is replaced with the new version
2. **Given** multiple WASM Creator nodes exist with different names, **When** each is executed, **Then** each creates or updates its own distinct component without interfering with others
3. **Given** a component is in use on the canvas, **When** it is recompiled with updated code, **Then** existing instances of the component continue to function or display a message indicating they need to be refreshed
4. **Given** a WASM Creator node with the "Save Code" checkbox enabled, **When** the user saves the graph and reopens it later, **Then** the code editor contains the previously entered code
5. **Given** a WASM Creator node with the "Save Code" checkbox disabled, **When** the user saves and reopens the graph, **Then** the component name is preserved but the code editor is empty

---

### User Story 4 - Template-Based Code Generation (Priority: P2)

A user provides minimal execute function logic, and the system automatically wraps it in the proper WASM component template structure with required interfaces and exports.

**Why this priority**: Essential for usability - users shouldn't need to understand the full WASM component boilerplate. This priority is P2 because the basic creation (P1) needs the template, but the UX can be refined later.

**Independent Test**: Can be tested by entering only the core execute function body (e.g., "let result = input * 2.0; Ok(vec![...])"), clicking execute, and verifying the system generates a complete valid component.

**Acceptance Scenarios**:

1. **Given** the user enters only execute function code, **When** the execute button is clicked, **Then** the system wraps the code in a complete WASM component template including metadata interface, execution interface, and proper exports
2. **Given** the template generation completes, **When** the component is compiled, **Then** all required WIT bindings and trait implementations are correctly included
3. **Given** the user specifies input and output port types using structured code comments (e.g., `// @input name:type description` and `// @output name:type description`), **When** the component is generated, **Then** the metadata interface correctly reports the specified ports
4. **Given** no port information is provided, **When** the component is generated, **Then** default ports are created based on reasonable assumptions (e.g., single F32 input and output)

---

### Edge Cases

- What happens when the user provides an empty component name or uses special characters that are invalid in identifiers?
- How does the system handle compilation timeout if the code enters an infinite loop during compilation or execution?
- What happens when the user tries to create a component while another compilation is still in progress?
- How does the system handle disk space issues when writing temporary files for compilation?
- What happens when the generated component's code imports external crates that aren't available?
- How does the system handle very large code snippets that exceed reasonable size limits?
- What happens when multiple users or sessions try to create components with the same name?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a built-in "WASM Component Creator" node that appears in the component palette
- **FR-002**: WASM Creator node MUST display a code editor widget supporting Rust syntax highlighting and multi-line input
- **FR-003**: WASM Creator node MUST provide a text input field for specifying the component name
- **FR-004**: System MUST provide an "Execute" or "Compile" button within the WASM Creator node's UI
- **FR-005**: System MUST validate the component name to ensure it contains only valid identifier characters
- **FR-006**: System MUST wrap user-provided execute function code in a complete WASM component template including metadata interface, execution interface, and WIT bindings
- **FR-007**: System MUST compile the generated Rust code into a WASM component using the appropriate target (wasm32-wasip2)
- **FR-008**: System MUST load the compiled WASM component into the runtime and register it with the component registry
- **FR-009**: System MUST add successfully compiled components to the component palette in a user-defined category
- **FR-010**: System MUST display user-defined components with a distinct visual style (e.g., purple color) to differentiate them from built-in components
- **FR-011**: System MUST display compilation errors in the node's footer UI with line numbers and error descriptions
- **FR-012**: System MUST prevent execution of new compilations while a compilation is already in progress for the same node
- **FR-013**: System MUST replace existing components when recompiling with the same component name
- **FR-014**: System MUST provide default input and output ports when port information is not explicitly specified by the user
- **FR-015**: System MUST clean up temporary compilation artifacts after successful or failed compilation
- **FR-016**: System MUST enforce a maximum code size limit to prevent abuse and resource exhaustion
- **FR-017**: System MUST implement a compilation timeout to prevent infinite compilation times
- **FR-018**: WASM Creator node MUST always persist the component name in the graph file and MUST provide a user-controllable option (e.g., checkbox) to optionally save the code content in the graph file across application sessions
- **FR-019**: System MUST parse structured code comments (format: `// @input name:type description` and `// @output name:type description`) to extract port specifications for the generated component
- **FR-020**: System MUST support at minimum the following port types in structured comments: F32, I32, U32, String, and Boolean

### Key Entities

- **WASM Creator Node**: A built-in node type with a code editor, name input field, and execute button that generates custom WASM components
- **Component Template**: A predefined Rust code structure that includes all necessary imports, trait implementations, and exports required for a valid WASM component
- **User-Defined Component**: A dynamically created WASM component generated from user code, registered in the component palette with distinct visual styling
- **Compilation Context**: Temporary workspace containing generated Rust source code, Cargo.toml, and build artifacts during the compilation process
- **Compilation Result**: The outcome of compilation, including either a compiled WASM component binary or error messages with diagnostic information

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create a functional custom WASM component from code editor to palette in under 60 seconds for simple operations (assuming fast compilation)
- **SC-002**: Compilation errors are displayed within 2 seconds of the compilation process detecting the error
- **SC-003**: Successfully compiled components appear in the palette immediately after compilation completes
- **SC-004**: 95% of common single-operation components (e.g., multiply by N, add offset, string concatenation) compile successfully on first attempt with minimal boilerplate
- **SC-005**: The code editor supports at least 500 lines of code without performance degradation in the UI
- **SC-006**: Users can create and manage at least 20 custom components in a single session without memory or performance issues
- **SC-007**: The system handles compilation failures gracefully without crashing or requiring application restart in 100% of cases
- **SC-008**: Component creation reduces the need for external tools or build setup, measured by user ability to create components without touching the file system directly

## Assumptions

- Users have basic Rust programming knowledge and understand the structure of simple functions
- The compilation toolchain (Rust compiler with wasm32-wasip2 target) is available in the runtime environment
- Compilation times for simple components are reasonable (< 10 seconds for basic operations)
- The code editor widget referenced from egui_demo_lib provides adequate syntax highlighting and editing capabilities for Rust code
- The template structure can be based on existing examples like double-number or HTTP Fetch components
- Component metadata (inputs, outputs, category) can be specified using structured code comments in the format `// @input name:type description` and `// @output name:type description`
- When users choose not to save code in the graph file (hybrid persistence option), they understand the code will be lost when the session ends
- The "Save Code" checkbox defaults to enabled (checked) to prevent accidental data loss for new users
- The system has write access to a temporary directory for compilation artifacts
