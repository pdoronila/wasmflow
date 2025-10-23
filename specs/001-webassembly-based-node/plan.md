# Implementation Plan: WebAssembly Node-Based Visual Programming System

**Branch**: `001-webassembly-based-node` | **Date**: 2025-10-12 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-webassembly-based-node/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a native desktop application enabling visual programming through a node-based interface where each node is a WebAssembly component. Users compose data processing pipelines by connecting nodes visually, with full type safety at compile time and secure, capability-based execution. The system supports both built-in computational nodes and user-created WASM components with controlled system access permissions.

Primary technical approach: Rust desktop application using egui for UI, wasmtime for WASM component execution, petgraph for graph dependency resolution, and the WASI Component Model for extensibility.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)
**Primary Dependencies**: egui 0.29 (UI), eframe 0.29 (app framework), egui-snarl 0.3 (node editor), wasmtime 27.0 with component-model (WASM runtime), petgraph 0.6 (graph algorithms), serde/bincode (serialization)
**Storage**: Local filesystem (graph save files as bincode-serialized structs, component files as .wasm)
**Testing**: cargo test (unit tests), cargo-component for component testing, integration tests for graph execution
**Target Platform**: Desktop (Windows 10+, macOS 11+, Linux with X11/Wayland)
**Project Type**: Single desktop application (native binary)
**Performance Goals**: 60 FPS UI rendering, <100ms node execution overhead, <10ms component instantiation, 500+ node graph support
**Constraints**: <500MB memory for typical graphs, <3s load time for 100-node graphs, offline-capable (no network dependency)
**Scale/Scope**: Single-user desktop application, support for 500+ nodes per graph, unlimited custom components loadable

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Component-First Architecture

**Status**: ✅ PASS

- All custom nodes implemented as WASM components with WIT interfaces
- Built-in nodes (Add, Multiply, Constant, etc.) will present same component interface
- Node-based visual composition is the primary user interaction (FR-001, FR-002)
- Each component is self-contained and independently testable per constitution

**Evidence**: FR-011 (load external components), FR-012 (validate components), User Story 3 (custom nodes)

### Principle II: Capability-Based Security (NON-NEGOTIABLE)

**Status**: ✅ PASS

- Default deny: Components have no system access unless granted (FR-014)
- Permission dialog before component execution (FR-013, User Story 4)
- Capability enforcement via WASI context configuration (FR-014)
- Path scoping for file access, network allowlisting per constitution sandboxing levels
- Resource limits enforced per component

**Evidence**: FR-013, FR-014, SC-010 (100% permission dialog coverage), User Story 4 acceptance scenarios

### Principle III: Typed Data Flow

**Status**: ✅ PASS

- Type checking at graph construction time (FR-003, FR-004)
- WIT-defined data types: u32, i32, f32, string, binary, list, record (FR-007)
- Type mismatches surfaced in UI before execution (SC-005: 100% type error prevention)
- Fail-fast validation per constitution

**Evidence**: FR-003 (enforce type compatibility), FR-004 (visual feedback), User Story 1 scenario 4 (type mismatch error)

### Principle IV: Performance-First Execution

**Status**: ✅ PASS

- 60 FPS UI target (FR-019, SC-002)
- Topological sorting for dependency resolution (FR-005)
- Component instance pooling planned (FR-020: <100ms overhead)
- Lazy compilation strategy (compile on first use)
- Support for 500+ node graphs (FR-019, SC-002)

**Evidence**: FR-019, FR-020, SC-002, SC-004 (500ms for 10-node pipeline)

### Principle V: Developer Ergonomics

**Status**: ✅ PASS

- Standard cargo-component tooling for component creation
- Clear error messages with context (FR-018: which node, why)
- Component hot-reloading deferred to Phase 4 (optional)
- Structured logging and debugging support planned
- Integration tests for component contracts (User Story 3 scenario 4)

**Evidence**: FR-018, SC-007 (90% first-time user success), assumption about custom node file format specification

### Principle VI: Composability & Modularity

**Status**: ✅ PASS

- WAC composition CLI integration planned for Phase 4
- Subgraph composition via save/load (FR-009, FR-010)
- Component metadata (author, version, description) in WIT interfaces
- Node palette for discoverability (FR-015)
- Import/export via component loading (FR-011)

**Evidence**: FR-015 (node palette), FR-011 (load components), User Story 3 (custom nodes)

### Security & Capabilities Gates

**Status**: ✅ PASS

- Five sandboxing levels defined in constitution align with FR-013, FR-014
- User consent flow matches User Story 4 acceptance scenarios
- Permission revocation supported (User Story 4 scenario 4)
- Graceful failure on permission violations (SC-006: 100% no-crash rate)

### Development Standards Gates

**Status**: ✅ PASS

- Project structure matches constitution (src/ui, src/runtime, src/graph, src/builtin)
- Code quality gates: >80% coverage for graph execution logic
- Integration tests for component contracts, security, serialization
- No panics in release builds (all errors via Result/Error types)
- Dependency policy followed (wasmtime, egui, petgraph, serde, tokio)

**Overall Gate Status**: ✅ ALL GATES PASS - Proceed to Phase 0 Research

## Project Structure

### Documentation (this feature)

```
specs/001-webassembly-based-node/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── node-interface.wit
└── checklists/
    └── requirements.md  # Specification validation checklist
```

### Source Code (repository root)

```
src/
├── ui/
│   ├── app.rs           # eframe application entry point
│   ├── canvas.rs        # Node editor canvas with egui-snarl
│   ├── palette.rs       # Node palette/library browser
│   ├── dialogs.rs       # Permission, save/load dialogs
│   └── theme.rs         # Visual styling
├── runtime/
│   ├── engine.rs        # Graph execution orchestration
│   ├── wasm_host.rs     # Wasmtime component host
│   ├── instance_pool.rs # Component instance caching
│   └── capabilities.rs  # WASI context builder for permissions
├── graph/
│   ├── node.rs          # GraphNode struct and NodeValue enum
│   ├── graph.rs         # NodeGraph with petgraph integration
│   ├── connection.rs    # Connection validation and type checking
│   ├── serialization.rs # Save/load via serde + bincode
│   └── execution.rs     # Topological sort and execution order
├── builtin/
│   ├── math.rs          # Add, Subtract, Multiply, Divide nodes
│   ├── constants.rs     # Constant value nodes
│   └── text.rs          # String operations
└── lib.rs               # Library entry point and exports

tests/
├── contract/
│   └── component_interface_tests.rs  # WIT contract validation
├── integration/
│   ├── graph_execution_tests.rs      # End-to-end execution
│   ├── serialization_tests.rs        # Save/load validation
│   └── security_tests.rs             # Permission enforcement
└── unit/
    ├── type_checking_tests.rs        # Connection type validation
    ├── topology_tests.rs             # Cycle detection, sorting
    └── wasm_host_tests.rs            # Component loading/execution

components/          # User-defined WASM components (examples)
├── example_adder.wasm
└── example_file_reader.wasm

wit/                 # WIT interface definitions
└── node.wit         # Component interface specification

docs/                # Development guides and examples
└── component-development.md  # How to create custom nodes
```

**Structure Decision**: Single desktop application structure. All source code under `src/` with four primary modules: `ui` (egui/eframe interface), `runtime` (wasmtime WASM execution), `graph` (petgraph-based graph management with type checking and serialization), and `builtin` (built-in node implementations). Test directory mirrors source structure with contract, integration, and unit test categories per constitution quality gates.

## Complexity Tracking

*No violations detected - all constitution gates passed without exceptions.*

## Post-Design Constitution Re-Evaluation

*Re-check performed after Phase 1 design (research.md, data-model.md, contracts/, quickstart.md)*

### Design Validation Summary

**All constitution principles remain satisfied** after detailed design work:

#### Component-First Architecture ✅
- **WIT Interface Defined**: `contracts/node-interface.wit` specifies complete component contract
- **Metadata Interface**: Components export `get-info()`, `get-inputs()`, `get-outputs()`, `get-capabilities()`
- **Execution Interface**: Single `execute()` function with typed inputs/outputs
- **Built-in Parity**: Built-in nodes will wrap same interface (internal Rust, external WIT facade)

**New Design Detail**: WIT package versioning (`wasmflow:node@1.0.0`) enables semantic interface evolution per constitution.

#### Capability-Based Security ✅
- **CapabilityGrant Entity**: Defined in data-model.md with five sandboxing levels
- **WASI Context Mapping**: Capability sets directly map to WasiCtxBuilder configuration
- **Permission Persistence**: Grants saved with graph files, re-prompted on escalation
- **Component Declaration**: `get-capabilities()` returns capability strings before instantiation

**New Design Detail**: Host function `get-temp-dir()` provides safe temporary storage without full filesystem access.

#### Typed Data Flow ✅
- **WIT Data Types**: 7 core types defined (u32, i32, f32, string, binary, list, record)
- **Connection Validation**: Type compatibility matrix documented in data-model.md
- **Port Specifications**: `PortSpec` record includes data-type for compile-time checking
- **Any Type**: Generic `any-type` variant allows for dynamic nodes (e.g., passthrough, logger)

**New Design Detail**: `ExecutionError` record includes `input-name` and `recovery-hint` for user-friendly error messages.

#### Performance-First Execution ✅
- **Instance Pooling**: `InstancePool` struct planned in research.md (50ms → <1ms per call)
- **Lazy Compilation**: Module caching strategy with LRU eviction
- **Incremental Execution**: Dirty flag propagation for affected subgraph re-execution only
- **UI Optimization**: egui retained mode, batched connection rendering, 60 FPS target validated

**New Design Detail**: Tokio async runtime enables parallel execution of independent graph branches.

#### Developer Ergonomics ✅
- **Quickstart Guide**: Comprehensive 6-section guide created (setup → custom component → testing)
- **Component Example**: Full "Double Number" example in quickstart.md with explanatory comments
- **Host Logging**: `host::log()` function for component debugging
- **Error Context**: `ExecutionError` struct provides actionable recovery hints

**New Design Detail**: `cargo component` workflow validated; `wit-bindgen` generates bindings automatically.

#### Composability & Modularity ✅
- **Metadata Fields**: ComponentInfo includes name, version, author, category, description
- **Node Palette Categories**: Components discoverable via category (Math, Text, File I/O)
- **Graph Serialization**: NodeGraph entity fully serializable with bincode for reusable subgraphs
- **WAC Integration**: Deferred to Phase 4 but WIT interfaces designed to support composition

**New Design Detail**: Graph save format includes schema version field for forward/backward compatibility.

### Design Additions Not Covered by Constitution

The following design elements extend beyond constitution requirements but align with principles:

1. **Graph Validation Framework**: Comprehensive validation rules for cycles, types, required inputs (supports Typed Data Flow principle)
2. **Execution State Tracking**: ExecutionState enum (Idle, Running, Completed, Failed) for UI feedback
3. **Checksum Integrity**: CRC64 checksum in save file format prevents corruption (reliability)
4. **Resource Limits**: Store::limiter API for CPU/memory quotas per component (security)
5. **Timeout Enforcement**: 30-second execution limit to prevent hanging nodes (performance)

**Justification**: These additions support constitutional principles (security, performance) without introducing unjustified complexity.

### Potential Future Constitution Amendments

Design work identified areas where constitution may need updates in future versions:

1. **Streaming Data**: Current design uses discrete value passing. Streaming (async iterators) would require new WIT interface and principle for backpressure handling.
2. **Real-time Collaboration**: Constitution assumes single-user. Multi-user would need concurrency control principle.
3. **Component Marketplace**: Out of scope, but future distribution model may require new principle for component trust/signing.

**Recommendation**: Defer amendments to post-MVP based on user feedback and actual needs.

### Final Gate Status

**✅ ALL CONSTITUTION GATES REMAIN PASS POST-DESIGN**

No violations introduced during design phase. All principles validated against:
- research.md (technology decisions)
- data-model.md (entity definitions)
- contracts/node-interface.wit (WIT interface specification)
- quickstart.md (developer workflow)

**Ready to proceed to Phase 2: Task generation with `/speckit.tasks`**
