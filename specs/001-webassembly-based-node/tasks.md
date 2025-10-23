# Tasks: WebAssembly Node-Based Visual Programming System

**Input**: Design documents from `/specs/001-webassembly-based-node/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), data-model.md, contracts/node-interface.wit

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root
- Paths reference plan.md structure (src/ui/, src/runtime/, src/graph/, src/builtin/)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure needed by all user stories

- [x] T001 Initialize Rust project with Cargo.toml dependencies (egui 0.29, eframe 0.29, egui-snarl 0.3, wasmtime 27.0, petgraph 0.6, serde, bincode, anyhow, thiserror, tokio, uuid)
- [x] T002 [P] Create project directory structure: src/ui/, src/runtime/, src/graph/, src/builtin/, tests/unit/, tests/integration/, tests/contract/, wit/, components/, docs/
- [x] T003 [P] Configure Cargo.toml with workspace settings, release profile optimizations (opt-level=3, lto=true for performance targets)
- [x] T004 [P] Add wasm32-wasip2 target support and cargo-component dev dependency for component development

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Define core data types in src/graph/node.rs: NodeValue enum (U32, I32, F32, String, Binary, List, Record) with serde derives
- [x] T006 Define Port struct in src/graph/node.rs: id, name, data_type, direction (Input/Output enum), optional, current_value fields
- [x] T007 Define Connection struct in src/graph/connection.rs: id, source_node, source_port, target_node, target_port, validated fields with serde derives
- [x] T008 [P] Define GraphNode struct in src/graph/node.rs: id, component_id, display_name, position (egui::Pos2), inputs, outputs, metadata, capabilities, execution_state with serde derives
- [x] T009 [P] Define ComponentSpec struct in src/graph/node.rs: id, name, description, author, version, component_type (Builtin/UserDefined enum), input_spec, output_spec, required_capabilities
- [x] T010 [P] Define CapabilitySet enum in src/runtime/capabilities.rs: None, FileRead, FileWrite, FileReadWrite, Network, Full variants with path/host restrictions
- [x] T011 Define NodeGraph struct in src/graph/graph.rs: id, name, nodes HashMap, connections Vec, metadata, version, execution_order_cache
- [x] T012 Copy WIT interface from specs/001-webassembly-based-node/contracts/node-interface.wit to wit/node.wit at repository root
- [x] T013 Create error types in src/lib.rs: GraphError (CycleDetected, TypeMismatch, InvalidConnection), ComponentError (LoadFailed, ValidationFailed, ExecutionError, PermissionDenied), SerializationError using thiserror

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Create and Execute Simple Data Flow (Priority: P1) 🎯 MVP

**Goal**: Enable users to build simple data processing pipelines by visually connecting pre-built computational nodes and seeing results immediately

**Independent Test**: Open application, drag Add node onto canvas, drag two Constant nodes, connect Constant(5) → Add.a, Constant(3) → Add.b, execute, observe sum=8.0 on output port

### Implementation for User Story 1

- [x] T014 [P] [US1] Implement type compatibility checking in src/graph/connection.rs: is_compatible(source_type, target_type) function with exact match, Any type handling, list compatibility per data-model.md matrix
- [x] T015 [P] [US1] Implement Connection validation in src/graph/connection.rs: validate_connection() checking source is output, target is input, no self-connections, type compatibility
- [x] T016 [US1] Implement NodeGraph::add_connection() in src/graph/graph.rs with type validation, preventing duplicate connections to same input port
- [x] T017 [US1] Implement NodeGraph::remove_connection() in src/graph/graph.rs updating connections Vec and clearing execution_order_cache
- [x] T018 [P] [US1] Create builtin Add node in src/builtin/math.rs implementing ComponentSpec with inputs (a: F32, b: F32), output (sum: F32), execute logic
- [x] T019 [P] [US1] Create builtin Subtract node in src/builtin/math.rs with inputs (a: F32, b: F32), output (difference: F32)
- [x] T020 [P] [US1] Create builtin Multiply node in src/builtin/math.rs with inputs (a: F32, b: F32), output (product: F32)
- [x] T021 [P] [US1] Create builtin Divide node in src/builtin/math.rs with inputs (a: F32, b: F32), output (quotient: F32), division-by-zero error handling
- [x] T022 [P] [US1] Create builtin Constant node in src/builtin/constants.rs with user-configurable value output (supports U32, I32, F32, String types)
- [x] T023 [US1] Implement ComponentRegistry in src/graph/node.rs: HashMap storage, register_builtin(), register_component(), get_by_id(), list_all() methods
- [x] T024 [US1] Implement topological sort in src/graph/execution.rs using petgraph DiGraph, returns Vec<NodeId> in dependency order
- [x] T025 [US1] Implement cycle detection in src/graph/execution.rs using petgraph::algo::is_cyclic_directed, returns CycleDetected error with node IDs
- [x] T026 [US1] Implement ExecutionEngine in src/runtime/engine.rs: execute_graph() orchestrating topological sort, sequential node execution, result propagation
- [x] T027 [US1] Implement node execution logic in src/runtime/engine.rs: execute_node() reading input values from connections, invoking component, writing outputs to ports
- [x] T028 [US1] Create eframe application in src/ui/app.rs: WasmFlowApp struct with NodeGraph, ComponentRegistry, selected state, initialize window with 1280x720 size
- [x] T029 [US1] Integrate egui-snarl in src/ui/canvas.rs: SnarlViewer trait implementation mapping GraphNode/Connection to snarl data model
- [x] T030 [US1] Implement node rendering in src/ui/canvas.rs: draw nodes with title, input/output ports, port labels, execution state color coding
- [x] T031 [US1] Implement connection rendering in src/ui/canvas.rs: draw connection lines, color code by type compatibility (green=valid, red=invalid)
- [x] T032 [US1] Implement node palette in src/ui/palette.rs: scrollable list of available components grouped by category (Math, Constants), drag-to-canvas support
- [x] T033 [US1] Implement add node interaction in src/ui/canvas.rs: detect palette drop, create GraphNode at cursor position, assign UUID, add to graph
- [x] T034 [US1] Implement create connection interaction in src/ui/canvas.rs: drag from output port, visual feedback during drag, validate on drop, add to graph or show error
- [x] T035 [US1] Implement delete node/connection in src/ui/canvas.rs: right-click context menu, remove from graph, update UI
- [x] T036 [US1] Add Execute button to toolbar in src/ui/app.rs: trigger ExecutionEngine, handle errors, display status message
- [x] T037 [US1] Implement output display in src/ui/canvas.rs: render output port values on nodes after execution, format NodeValue variants for display
- [x] T038 [US1] Implement error message display in src/ui/app.rs: show GraphError/ComponentError in bottom status bar with actionable context (FR-018)
- [x] T039 [US1] Add undo/redo command history in src/graph/graph.rs: Command pattern for AddNode, RemoveNode, AddConnection, RemoveConnection, with undo stack (FR-016)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently (visual calculator MVP)

---

## Phase 4: User Story 2 - Save and Load Processing Pipelines (Priority: P2)

**Goal**: Enable users to save composed node graphs to disk and reload them later, preserving all state

**Independent Test**: Create 3-node graph (Constant → Add → Constant), save to file, close app, reopen, load file, verify nodes/connections/values restored

### Implementation for User Story 2

- [x] T040 [P] [US2] Implement GraphSaveFormat struct in src/graph/serialization.rs: version u32, graph NodeGraph, checksum u64 with serde derives (using BTreeMap for deterministic serialization)
- [x] T041 [US2] Implement NodeGraph serialization in src/graph/serialization.rs: to_bytes() using bincode, prepend magic bytes "WASMFLOW", version=1, compute CRC64 checksum (with BTreeMap)
- [x] T042 [US2] Implement NodeGraph deserialization in src/graph/serialization.rs: from_bytes() validating magic bytes, version compatibility, checksum integrity, returns SerializationError on failure (with BTreeMap)
- [x] T043 [US2] Implement post-load validation in src/graph/serialization.rs: verify all connection node IDs exist, ports exist on nodes, types compatible, no dangling references (with BTreeMap)
- [x] T044 [US2] Add Save Graph menu item in src/ui/app.rs: open native file dialog (rfd crate), call NodeGraph::to_bytes(), write to .wasmflow file, show success message
- [x] T045 [US2] Add Load Graph menu item in src/ui/app.rs: open file dialog, read bytes, call NodeGraph::from_bytes(), replace current graph, refresh UI
- [x] T046 [US2] Implement unsaved changes tracking in src/ui/app.rs: dirty flag set on graph modifications, cleared on save
- [x] T047 [US2] Add close/exit confirmation dialog in src/ui/dialogs.rs: check dirty flag, show "Save changes before closing?" with Save/Discard/Cancel buttons
- [x] T048 [US2] Handle corrupted file errors in src/ui/app.rs: catch SerializationError, display user-friendly message explaining issue (magic bytes mismatch, version incompatible, checksum failed)
- [x] T049 [US2] Add File menu with New Graph, Open, Save, Save As, Recent Files in src/ui/app.rs using egui menu bar
- [x] T050 [US2] Implement Recent Files list in src/ui/app.rs: store last 10 files in local config (dirs crate), display in File menu, handle missing files gracefully

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently (persistence enabled)

---

## Phase 5: User Story 3 - Use Custom Extension Nodes (Priority: P3)

**Goal**: Enable users to extend system capabilities by loading custom-built computational nodes from external WASM files

**Independent Test**: Provide pre-built double-number.wasm component, load via menu, add to graph, connect Constant(7) → Double, execute, verify output=14.0

### Implementation for User Story 3

- [x] T051 [P] [US3] Implement wasmtime Store initialization in src/runtime/wasm_host.rs: create Store<HostState>, configure resource limiter (500MB memory, fuel limits)
- [x] T052 [P] [US3] Implement wasmtime Linker setup in src/runtime/wasm_host.rs: add host function bindings for host::log(), host::get-temp-dir()
- [x] T053 [US3] Implement host::log() in src/runtime/wasm_host.rs: receive level/message from component, write to stdout with timestamp and component ID prefix
- [x] T054 [US3] Implement host::get-temp-dir() in src/runtime/wasm_host.rs: return temp directory path or error if no temp storage capability granted
- [x] T055 [US3] Implement component loading in src/runtime/wasm_host.rs: Module::from_file() reading .wasm, compile with wasmtime, cache Module in ComponentSpec
- [x] T056 [US3] Implement WIT interface binding in src/runtime/wasm_host.rs: use bindgen!() macro to generate Rust bindings for wasmflow:node world
- [x] T057 [US3] Implement component metadata extraction in src/runtime/wasm_host.rs: instantiate component, call get-info(), get-inputs(), get-outputs(), get-capabilities(), populate ComponentSpec
- [x] T058 [US3] Implement component validation in src/runtime/wasm_host.rs: check WIT interface exports match expected, validate version format, check file size <50MB, verify imports satisfied
- [x] T059 [US3] Add Load Component menu item in src/ui/app.rs: open file dialog filtering .wasm files, call component loading, add to ComponentRegistry, refresh palette
- [x] T060 [US3] Implement component execute() in src/runtime/engine.rs: instantiate component with Store, convert NodeValue inputs to WIT value format, call execute(), convert outputs back to NodeValue
- [x] T061 [US3] Add component execution timeout in src/runtime/engine.rs: wrap component.execute() with tokio::time::timeout(30s), return ExecutionError on timeout
- [x] T062 [US3] Handle component errors in src/runtime/engine.rs: catch ExecutionError from component, extract message/input-name/recovery-hint, display in UI with context
- [x] T063 [US3] Implement component hot-reload in src/ui/app.rs: File → Reload Components menu item, re-scan components/ directory, re-load all .wasm files, update registry and palette
- [x] T064 [US3] Add component info display in src/ui/palette.rs: show tooltip on hover with author, version, description from ComponentInfo metadata
- [x] T065 [US3] Handle malformed component files in src/runtime/wasm_host.rs: catch wasmtime errors during Module::from_file(), validate WIT exports, return ComponentError::LoadFailed with specific reason

**Checkpoint**: All user stories (1, 2, 3) should now be independently functional (extensibility enabled)

---

## Phase 6: User Story 4 - Manage Node Permissions for Safe Execution (Priority: P4)

**Goal**: Enable users to review and control what system resources custom nodes can access, ensuring security

**Independent Test**: Load component requesting file-read:/data capability, review permission dialog showing directory, grant access, execute, verify component can only read from /data directory

### Implementation for User Story 4

- [X] T066 [P] [US4] Implement CapabilityGrant struct in src/runtime/capabilities.rs: node_id, capability_set, granted_at timestamp, scope description with serde derives
- [X] T067 [P] [US4] Implement WASI context builder in src/runtime/capabilities.rs: to_wasi_ctx() converting CapabilitySet to WasiCtxBuilder with preopened dirs, network access
- [X] T068 [US4] Implement FileRead capability enforcement in src/runtime/capabilities.rs: WasiCtxBuilder::preopened_dir(path, DirPerms::READ) for each path in FileRead variant
- [X] T069 [US4] Implement FileWrite capability enforcement in src/runtime/capabilities.rs: preopened_dir(path, DirPerms::WRITE) for FileWrite paths
- [X] T070 [US4] Implement Network capability enforcement in src/runtime/capabilities.rs: WasiCtxBuilder::inherit_network() + allowlist validation in outgoing handler (block non-allowlisted hosts)
- [X] T071 [US4] Parse capability requests in src/runtime/wasm_host.rs: parse get-capabilities() string format "file-read:/path", "network:host.com" into CapabilitySet variants
- [X] T072 [US4] Create permission dialog in src/ui/dialogs.rs: PermissionDialog widget showing component name, requested capabilities list, checkbox approval, Approve/Deny buttons
- [X] T073 [US4] Trigger permission dialog in src/ui/app.rs: on first add custom node to graph, show dialog with requested capabilities, wait for user response before adding node
- [X] T074 [US4] Store capability grants in NodeGraph in src/graph/graph.rs: add capability_grants: BTreeMap<NodeId, CapabilityGrant> field, serialize with graph save file
- [X] T075 [US4] Implement permission enforcement in src/runtime/engine.rs: before execute_node(), build WASI context from CapabilityGrant, configure Store, pass to component instantiation
- [X] T076 [US4] Handle permission violations in src/runtime/engine.rs: catch WASI permission errors (EPERM, EACCES), convert to PermissionDenied error with node ID and capability type
- [X] T077 [US4] Display permission errors in src/ui/app.rs: show PermissionDenied with message like "Node 'FileReader' denied access to /etc - not in approved paths"
- [X] T078 [US4] Add View Permissions in node context menu in src/ui/canvas.rs: right-click node → View Permissions → show current CapabilityGrant in dialog, allow Revoke button
- [X] T079 [US4] Implement capability revocation in src/ui/dialogs.rs: remove CapabilityGrant from graph via Revoke button in permissions view dialog, next execution will fail with PermissionDenied until re-approved
- [X] T080 [US4] Implement capability escalation detection in src/ui/app.rs: compare component requested capabilities to existing CapabilityGrant when adding nodes, re-prompt if capabilities differ, reuse grant if same
- [X] T081 [US4] Add Full capability warning in src/ui/dialogs.rs: if component requests Full access, show red warning box with explicit opt-in checkbox and security disclaimer, disable Approve until acknowledged

**Checkpoint**: All user stories (1, 2, 3, 4) should be independently functional (security controls complete)

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories, final quality polish

- [X] T082 [P] Implement instance pooling in src/runtime/instance_pool.rs: InstancePool struct with HashMap<ComponentId, Vec<InstancePre>>, get()/return() methods, max 10 instances per component
- [X] T083 [P] Implement lazy compilation in src/runtime/wasm_host.rs: compile components on first use, cache Module in ComponentSpec, LRU eviction after 50 cached modules
- [X] T084 [P] Implement incremental execution in src/graph/execution.rs: add dirty flags to GraphNode, propagate_dirty() on input changes, execute_dirty_subgraph() for affected nodes only
- [X] T085 [P] Add UI performance optimizations in src/ui/canvas.rs: batch connection line rendering, only repaint on graph changes, use egui retained mode for static elements
- [X] T086 [P] Implement logging framework in src/lib.rs: configure env_logger with RUST_LOG support, structured logging for execution, component loading, errors
- [X] T087 [P] Add application theme in src/ui/theme.rs: define color palette (node background, port colors by type, connection valid/invalid), apply via egui::Style
- [X] T088 [P] Create example components in components/ directory: example_adder.wasm, example_file_reader.wasm (with file-read capability), example_http_fetch.wasm (with network capability)
- [X] T089 [P] Write component development guide in docs/component-development.md: cargo component setup, WIT interface tutorial, testing workflow, capability declaration
- [X] T090 [P] Add canvas navigation in src/ui/canvas.rs: pan with middle-mouse drag, zoom with scroll wheel, fit-to-screen button, reset view button
- [X] T091 [P] Implement node search in src/ui/palette.rs: filter text box, fuzzy search by name/category/description, keyboard navigation
- [X] T092 [P] Add graph metadata editor in src/ui/dialogs.rs: edit graph name, author, description, created/modified timestamps displayed read-only
- [X] T093 Add performance benchmarks in tests/integration/performance_tests.rs: benchmark 100-node graph execution (<500ms target), 500-node graph rendering (60 FPS target), load 100-node graph (<3s target)
- [X] T094 Create integration test in tests/integration/graph_execution_tests.rs: create graph programmatically, execute, verify outputs match expected values
- [X] T095 Create serialization roundtrip test in tests/integration/serialization_tests.rs: save graph, load, verify all fields identical (nodes, connections, positions, values)
- [X] T096 Create security test in tests/integration/security_tests.rs: create component requesting file access, deny permission, verify execution fails with PermissionDenied
- [X] T097 Create cycle detection test in tests/unit/topology_tests.rs: construct graph with cycle, verify is_cyclic_directed returns true, execute fails with CycleDetected error
- [X] T098 Create type checking test in tests/unit/type_checking_tests.rs: attempt invalid connection (F32→I32), verify is_compatible returns false, add_connection fails
- [X] T099 [P] Add CLI argument parsing in src/main.rs: support --graph <file> to open graph on startup, --no-palette to hide palette, --log-level <level>
- [X] T100 Add About dialog in src/ui/dialogs.rs: display WasmFlow version, Rust version, dependencies versions, constitution link, GitHub repo link

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational completion - No dependencies on other stories
- **User Story 2 (Phase 4)**: Depends on Foundational completion - No dependencies on other stories (can run in parallel with US1)
- **User Story 3 (Phase 5)**: Depends on Foundational completion - Builds on US1 (needs working graph creation/execution)
- **User Story 4 (Phase 6)**: Depends on Foundational + US3 completion (needs component loading to demonstrate permissions)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independent of US1 (serialization is self-contained)
- **User Story 3 (P3)**: Requires US1 complete (needs working graph execution to demonstrate custom components)
- **User Story 4 (P4)**: Requires US3 complete (permissions only relevant for custom components)

### Within Each User Story

- **US1**: Type checking (T014-T015) → Graph operations (T016-T017) → Builtin nodes (T018-T022) → Registry (T023) → Execution (T024-T027) → UI (T028-T039)
- **US2**: Serialization (T040-T043) → UI integration (T044-T050)
- **US3**: WASM host (T051-T058) → Component execution (T059-T065)
- **US4**: Capability infrastructure (T066-T071) → Permission UI (T072-T081)

### Parallel Opportunities

- **Setup (Phase 1)**: T002, T003, T004 can run in parallel
- **Foundational (Phase 2)**: T005-T006 sequential, then T008-T010 can run in parallel
- **US1**: T014-T015 parallel, T018-T022 parallel (all builtin nodes), T030-T032 parallel (UI components)
- **US2**: T040-T041 sequential, T044-T045 sequential, T049-T050 parallel
- **US3**: T051-T054 parallel (host setup), T063-T065 parallel (UI polish)
- **US4**: T066-T070 parallel (capability variants), T072-T073 parallel (UI), T078-T081 parallel (advanced features)
- **Polish (Phase 7)**: T082-T092 all parallel, T099-T100 parallel

---

## Parallel Example: User Story 1

```bash
# Launch all builtin nodes together (different files):
Task: "Create builtin Add node in src/builtin/math.rs"
Task: "Create builtin Subtract node in src/builtin/math.rs"
Task: "Create builtin Multiply node in src/builtin/math.rs"
Task: "Create builtin Divide node in src/builtin/math.rs"

# Launch type checking and validation together (different files):
Task: "Implement type compatibility checking in src/graph/connection.rs"
Task: "Implement Connection validation in src/graph/connection.rs"

# Launch UI components together (different files):
Task: "Implement node rendering in src/ui/canvas.rs"
Task: "Implement connection rendering in src/ui/canvas.rs"
Task: "Implement node palette in src/ui/palette.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T013) - CRITICAL - blocks all stories
3. Complete Phase 3: User Story 1 (T014-T039)
4. **STOP and VALIDATE**: Test User Story 1 independently
   - Open app → Add nodes → Connect → Execute → See results
   - Verify 60 FPS rendering, type checking, error messages
5. Deploy/demo if ready (visual calculator MVP is valuable on its own)

### Incremental Delivery

1. Complete Setup + Foundational (T001-T013) → Foundation ready
2. Add User Story 1 (T014-T039) → Test independently → Deploy/Demo (MVP!)
   - **Value delivered**: Visual programming calculator, immediate utility
3. Add User Story 2 (T040-T050) → Test independently → Deploy/Demo
   - **Value delivered**: Persistence, users can save/share graphs
4. Add User Story 3 (T051-T065) → Test independently → Deploy/Demo
   - **Value delivered**: Extensibility, users can create custom nodes
5. Add User Story 4 (T066-T081) → Test independently → Deploy/Demo
   - **Value delivered**: Security, safe execution of untrusted components
6. Add Polish (T082-T100) → Final release
   - **Value delivered**: Performance, UX polish, documentation

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T013)
2. Once Foundational is done:
   - **Developer A**: User Story 1 (T014-T039) - Core graph execution
   - **Developer B**: User Story 2 (T040-T050) - Serialization (can start immediately, independent)
   - **Developer C**: Prepare User Story 3 infrastructure (research wasmtime, test examples)
3. Once US1 complete, Developer C starts US3 (T051-T065)
4. Once US3 complete, Developer A or B starts US4 (T066-T081)
5. Polish phase: All developers work on T082-T100 in parallel

---

## Notes

- [P] tasks = different files, no dependencies, can run concurrently
- [Story] label maps task to specific user story for traceability (US1, US2, US3, US4)
- Each user story is independently completable and testable
- Stop at any checkpoint to validate story independently before proceeding
- Commit after each task or logical group (e.g., all builtin nodes)
- Tests not included per spec (no explicit test request in spec.md)
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

## Critical Success Metrics (from spec.md)

- **SC-001**: Users create 5-node graph within 2 minutes (US1 complete)
- **SC-002**: 60 FPS with 500 nodes (US1 + Polish T085)
- **SC-003**: Load 100-node graph in <3s (US2 + Polish T083)
- **SC-004**: Execute 10-node pipeline in <500ms (US1 + Polish T082, T084)
- **SC-005**: 100% type error prevention (US1 T014-T015, T031)
- **SC-006**: 100% graceful permission failures (US4 T076-T077)
- **SC-007**: 90% first-time user success (US1 complete, US2 complete)
- **SC-008**: <500MB memory (US1 + Polish T082, resource limits in T051)
- **SC-009**: Zero data loss in saves (US2 T041-T043 checksum)
- **SC-010**: 100% permission dialogs shown (US4 T072-T073)
