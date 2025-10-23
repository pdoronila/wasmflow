# Implementation Plan: Continuous Execution Nodes

**Branch**: `006-continuous-node-can` | **Date**: 2025-10-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-continuous-node-can/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Enable nodes to run continuously (like web servers or event listeners) rather than executing once and stopping. Users can start/stop these nodes with dedicated play/stop controls. The feature supports server-like functionality (HTTP listeners, socket servers) and continuous input processing for reactive workflows.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel with wasm32-wasip2 target)
**Primary Dependencies**: egui 0.29 (UI), eframe 0.29 (app framework), egui-snarl 0.3 (node editor), wasmtime 27.0 with component-model (WASM runtime), tokio (async runtime for continuous execution)
**Storage**: Graph serialization via serde + bincode (BTreeMap for deterministic order), persistence of execution state in node metadata
**Testing**: cargo test (unit tests for execution engine), cargo clippy (linting)
**Target Platform**: Desktop (native via egui/eframe), potential WebAssembly target for browser
**Project Type**: Single desktop application with embedded WASM runtime
**Performance Goals**: 60 FPS UI rendering, <100ms response to start/stop controls, support for multiple concurrent continuous nodes, <10ms per node execution cycle overhead
**Constraints**: Must not block UI thread during continuous execution, graceful shutdown within 2 seconds, proper async task cleanup, memory limits per continuous node (WASI resource limits)
**Scale/Scope**: Support 10+ concurrent continuous nodes in a single graph, 500+ total nodes per graph, handle long-running operations (hours/days) without memory leaks

**Unknowns/Research Needed**:
- NEEDS CLARIFICATION: Async execution model for continuous nodes - how to integrate tokio tasks with egui's event loop without blocking UI
- NEEDS CLARIFICATION: State management for running continuous nodes - how to persist/restore running state across app restarts
- NEEDS CLARIFICATION: Thread safety for shared node state between UI thread and async execution threads
- NEEDS CLARIFICATION: Resource cleanup strategy - graceful shutdown of async tasks, handling of WASM instance cleanup
- NEEDS CLARIFICATION: Error propagation from async continuous execution to UI layer

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Component-First Architecture ✓ PASS

- Continuous execution capability will be implemented as component metadata/interface extensions
- Continuous nodes MUST still define WIT interfaces for inputs/outputs
- Both builtin and WASM component nodes can support continuous execution mode
- Play/stop controls are UI-level additions that interact with component lifecycle

**Compliance**: Feature extends component model without violating component-first principles.

### II. Capability-Based Security ✓ PASS

- Continuous nodes that run HTTP servers or socket listeners MUST declare network capabilities
- Long-running execution increases security surface area - REQUIRES extra scrutiny
- Resource limits (CPU, memory) MUST be enforced for continuous nodes to prevent DoS
- Network access MUST remain allowlist-based even for continuous server nodes

**Compliance**: No violations. Feature requires enhanced capability enforcement for long-running operations.

### III. Typed Data Flow ✓ PASS

- Continuous nodes process inputs using same WIT-defined types as single-execution nodes
- Type validation remains at graph construction time
- Continuous execution does not change type checking requirements

**Compliance**: Type safety model unchanged.

### IV. Performance-First Execution ⚠️ ATTENTION REQUIRED

- 60 FPS UI rendering MUST NOT be degraded by continuous node background execution
- Async execution MUST run on separate threads/tasks to avoid blocking UI
- Multiple concurrent continuous nodes could impact overall graph performance
- Target: <10ms overhead per continuous node execution cycle

**Action Required**: Research async execution patterns that maintain UI responsiveness. Phase 0 must validate egui + tokio integration approach.

### V. Developer Ergonomics ✓ PASS

- Continuous execution mode should be simple to enable (metadata flag or interface extension)
- Error messages must clearly indicate which continuous node failed and why
- Debugging running continuous nodes requires logging/tracing infrastructure
- Component hot-reloading becomes more complex with running continuous nodes

**Compliance**: Feature adds complexity but follows existing ergonomic patterns. Hot-reloading for running nodes is a known challenge.

### VI. Composability & Modularity ✓ PASS

- Continuous nodes can be composed into subgraphs like regular nodes
- Component metadata will include execution mode (single vs continuous)
- Subgraphs containing continuous nodes require special handling for start/stop

**Compliance**: Composition model extends naturally to continuous nodes.

### Security & Capabilities - Enhanced Requirements ⚠️ ATTENTION REQUIRED

- Continuous nodes running servers/listeners are high-risk attack surface
- MUST enforce stricter sandboxing for continuous network-accessible nodes
- Resource limits critical to prevent runaway continuous nodes (memory leaks, CPU spin)
- Capability re-validation on continuous node restart (prevent capability escalation)

**Action Required**: Phase 1 design must specify capability model for continuous execution lifecycle.

### Summary

**Gates**: 2 attention items, 0 blockers
- Performance verification needed (async + UI integration)
- Security model enhancement required (continuous execution capabilities)

**Verdict**: ✓ PROCEED to Phase 0 with research focus on async execution and security model.

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── ui/                      # egui + egui-snarl node editor
│   ├── node_ui.rs           # Node rendering (add play/stop controls)
│   └── execution_status.rs  # Visual state indicators for running nodes
├── runtime/                 # wasmtime execution engine
│   ├── executor.rs          # Graph execution logic
│   ├── continuous.rs        # NEW: Continuous execution manager
│   └── async_runtime.rs     # NEW: Tokio integration for continuous nodes
├── graph/                   # petgraph-based graph management
│   ├── node.rs              # Node data structures (add execution mode field)
│   └── state.rs             # NEW: Runtime state tracking for continuous nodes
└── builtin/                 # built-in node implementations
    └── continuous_example.rs # NEW: Example continuous node (HTTP server)

tests/
├── contract/                # Component WIT contract tests
│   └── continuous_wit.rs    # NEW: Continuous node WIT interface tests
├── integration/             # Graph execution integration tests
│   ├── continuous_exec.rs   # NEW: Continuous execution lifecycle tests
│   └── async_ui.rs          # NEW: UI responsiveness tests with running nodes
└── unit/                    # Core logic unit tests
    ├── continuous_manager.rs # NEW: Continuous execution manager tests
    └── state_management.rs  # NEW: State tracking tests
```

**Structure Decision**: Single project (existing structure). This feature extends the existing runtime and UI layers without requiring new top-level projects. Key additions:
- `runtime/continuous.rs` - manages async task lifecycle for continuous nodes
- `runtime/async_runtime.rs` - integrates tokio with egui event loop
- `graph/state.rs` - tracks runtime state (running/stopped/error) per node
- `ui/execution_status.rs` - visual indicators for continuous node states
- `builtin/continuous_example.rs` - reference implementation (HTTP server node)

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

N/A - No constitution violations requiring justification. The two attention items (performance and security) are compliance enhancements, not violations.

---

## Post-Design Constitution Re-Evaluation

*Re-checked after Phase 1 design completion*

### IV. Performance-First Execution - ✅ RESOLVED

**Initial Concern**: UI responsiveness with continuous background execution

**Design Resolution**:
- Background thread execution with channel-based communication (research.md)
- `try_recv()` non-blocking polling maintains 60 FPS UI rendering
- Measured <1ms channel overhead, <100ns RwLock reads
- Pattern already proven in existing codebase (`app.rs` lines 368-418)

**Verdict**: ✅ Performance requirements met through existing proven patterns.

### Security & Capabilities - ✅ RESOLVED

**Initial Concern**: Enhanced security model for long-running network-accessible nodes

**Design Resolution**:
- Capability grants persisted and re-validated on node start (data-model.md)
- WIT interface requires explicit capability declarations (contracts/continuous-execution.wit)
- 2-second graceful shutdown + forced termination prevents resource exhaustion
- CancellationToken + cleanup pattern ensures resource release (research.md)

**Verdict**: ✅ Security model adequately addresses continuous execution risks.

### Final Assessment

**All constitution requirements satisfied**:
- ✅ Component-First Architecture: Continuous mode as component interface extension
- ✅ Capability-Based Security: Enhanced enforcement for long-running operations
- ✅ Typed Data Flow: WIT-defined interfaces maintain type safety
- ✅ Performance-First Execution: 60 FPS guaranteed through background execution
- ✅ Developer Ergonomics: Simple interface, clear error messages, WIT contracts
- ✅ Composability & Modularity: Continuous nodes compose like regular nodes

**No blockers to implementation.**
