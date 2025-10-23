# Research & Technology Decisions

**Feature**: WebAssembly Node-Based Visual Programming System
**Branch**: 001-webassembly-based-node
**Date**: 2025-10-12

## Overview

This document captures technology research, architectural decisions, and best practices for implementing a native desktop visual programming system with WebAssembly component support.

## Key Technology Decisions

### 1. UI Framework: egui + egui-snarl

**Decision**: Use egui 0.29 with eframe for the application framework and egui-snarl 0.3 for node editor functionality.

**Rationale**:
- **egui**: Immediate mode GUI written in pure Rust, excellent performance (easily hits 60 FPS target), cross-platform (Windows, macOS, Linux), no external dependencies beyond graphics backend
- **egui-snarl**: Purpose-built node editor widget for egui with built-in support for:
  - Node dragging, connection drawing, pin management
  - Type-aware connection validation hooks
  - Clean separation between UI and graph data model
  - Active maintenance and good API design
- **eframe**: Official egui application framework, handles window creation, event loop, and graphics backend selection (glow for OpenGL/WebGL)

**Alternatives Considered**:
- **iced**: More opinionated Elm-architecture GUI. Rejected because node editor support is immature and immediate mode of egui provides better performance for frequently updating canvases.
- **druid**: More mature but development has slowed. egui has more active ecosystem and better WASM support for potential future web builds.
- **Custom node editor**: Building from scratch would require significant effort for basic features (snapping, connection routing, zoom/pan) that egui-snarl provides out of the box.

**Best Practices**:
- Separate graph data model from UI state (egui-snarl provides SnarlViewer trait for clean separation)
- Use egui's Context for efficient repainting (only redraw when needed)
- Implement custom node rendering for visual feedback on type compatibility
- Cache layout calculations to avoid re-computing node positions every frame

### 2. WASM Runtime: wasmtime with Component Model

**Decision**: Use wasmtime 27.0 with component-model and async features.

**Rationale**:
- **Component Model Support**: First-class support for WASI Preview 2 and WIT interfaces, essential for type-safe component boundaries
- **Performance**: Cranelift JIT compiler provides near-native performance (<10ms instantiation overhead achievable)
- **Security**: Built-in sandboxing via WASI capabilities system, direct mapping to constitution's security model
- **Maturity**: Production-ready, used by major projects (Fastly Compute, Docker WASM, Spin framework)
- **Instance Pre-compilation**: InstancePre API enables instance pooling for sub-millisecond invocation times
- **Async Support**: Tokio integration allows parallel execution of independent graph branches

**Alternatives Considered**:
- **wasmer**: Good performance but component model support is less mature. wasmtime has better WASI Preview 2 implementation.
- **wasm3**: Interpreter-based, much slower execution. Doesn't meet <100ms overhead target.
- **Native plugins (dylib)**: No sandboxing, cross-platform compilation challenges, security nightmare for user-created plugins.

**Best Practices**:
- Use InstancePre for component instance pooling (pre-compile components, reuse instances)
- Configure WasiCtxBuilder per component with minimal capabilities (principle of least privilege)
- Set resource limits via Store::limiter (memory, CPU time) to prevent runaway components
- Use wasmtime::Linker to provide host functions consistently across all components
- Enable async support for parallel graph execution but maintain deterministic ordering within dependency chains

### 3. Graph Management: petgraph

**Decision**: Use petgraph 0.6 for graph data structure and algorithms.

**Rationale**:
- **Topological Sort**: Built-in toposort for dependency-ordered execution (FR-005)
- **Cycle Detection**: is_cyclic_directed() for preventing circular dependencies (FR-017)
- **Flexible Representations**: DiGraph supports directed edges with metadata (type annotations, port IDs)
- **Performance**: Optimized for common graph algorithms, suitable for 500+ node graphs
- **Mature Library**: Battle-tested, comprehensive documentation, active maintenance

**Alternatives Considered**:
- **Custom Graph Structure**: Would need to reimplement topological sort, cycle detection. Not worth effort when petgraph provides this.
- **Static Dependency Analysis**: Could precompute execution order at save time, but dynamic approach allows for runtime graph modifications and partial re-execution.

**Best Practices**:
- Use DiGraph<NodeId, ConnectionMetadata> for storing node graph
- Store node data in BTreeMap<NodeId, GraphNode> for deterministic serialization (O(log n) lookup is acceptable for <1000 nodes)
- Cache topological sort result until graph structure changes (add/remove connection)
- Use petgraph's visit traits for custom graph traversal algorithms
- Implement incremental execution: track dirty nodes, re-execute only affected subgraphs

### 4. Serialization: serde + bincode + BTreeMap

**Decision**: Use serde 1.0 for serialization traits with bincode 1.3 for binary format, and BTreeMap for all serialized collections.

**Rationale**:
- **Performance**: bincode is fastest Rust serialization format (~10x faster than JSON), critical for <3s load time (SC-003)
- **Size**: Binary format produces smaller files than JSON/TOML, important for large graphs
- **Type Safety**: serde's derive macros provide compile-time serialization validation
- **Compatibility**: bincode maintains backward compatibility within major versions
- **Ecosystem**: Works seamlessly with all Rust types (egui::Pos2, petgraph structures, custom enums)
- **Determinism**: BTreeMap ensures consistent serialization order, enabling reliable checksum validation

**BTreeMap vs HashMap Decision**:
- **Why BTreeMap**: HashMap has non-deterministic iteration order, causing the same graph to serialize to different byte sequences, breaking checksum validation
- **Performance Trade-off**: O(log n) vs O(1) lookup is negligible for graphs with <1000 nodes (<0.1ms difference)
- **Benefit**: Enables CRC64 integrity checking without false positives
- **Where Used**: Only for serialized structures (NodeGraph.nodes, NodeValue::Record). Runtime-only structures can still use HashMap

**Alternatives Considered**:
- **JSON (serde_json)**: Human-readable but 3-5x slower and larger files. Not suitable for large graphs (100+ nodes).
- **MessagePack**: Faster than JSON but still slower than bincode, minimal benefit over bincode for desktop use.
- **Custom binary format**: Not worth maintenance burden when bincode provides excellent performance.
- **HashMap with disabled checksums**: Eliminates corruption detection, unacceptable for data integrity

**Best Practices**:
- Derive Serialize/Deserialize for all persistent data structures
- Use BTreeMap for all serialized collections that need deterministic order
- Use HashMap for runtime-only structures (marked with #[serde(skip)])
- Use #[serde(skip)] for cached fields (execution state, UI viewport)
- Version graph file format with a schema version field for future migrations
- Implement validation after deserialization (verify node IDs exist, connections are valid)
- CRC64 checksum for data integrity verification (enabled with BTreeMap)

### 5. Type System: WIT-Defined Data Types

**Decision**: Use WIT (WebAssembly Interface Types) for defining component interfaces and data types.

**Rationale**:
- **Interoperability**: WIT is the standard for WASI components, ensures compatibility with future ecosystem tools
- **Type Safety**: Strongly typed interface definitions prevent type mismatches at component boundaries
- **Tooling**: wit-bindgen generates Rust bindings automatically, reduces manual marshaling code
- **Versioning**: WIT supports semantic versioning of interfaces, important for component evolution
- **Documentation**: WIT serves as machine-readable API documentation for components

**Type Mapping** (WIT ↔ Rust ↔ UI):
```
WIT Type       → Rust Type           → NodeValue Enum
─────────────────────────────────────────────────────
u32            → u32                 → NodeValue::U32(u32)
i32            → i32                 → NodeValue::I32(i32)
f32            → f32                 → NodeValue::F32(f32)
string         → String              → NodeValue::String(String)
list<u8>       → Vec<u8>             → NodeValue::Binary(Vec<u8>)
list<T>        → Vec<T>              → NodeValue::List(Vec<NodeValue>)
record { ... } → Rust struct         → NodeValue::Record(BTreeMap<String, NodeValue>)
```

**Best Practices**:
- Define common data types in a shared WIT package (upload-labs:types)
- Use record types for structured data instead of multiple discrete outputs
- Prefer binary (list<u8>) for large data to avoid serialization overhead
- Document expected ranges/constraints in WIT comments (e.g., "// must be positive")
- Version WIT interfaces semantically (breaking changes = major version bump)

### 6. Async Runtime: Tokio

**Decision**: Use tokio 1.40 with "full" features for async component execution.

**Rationale**:
- **Parallel Execution**: Execute independent graph branches concurrently (constitution Principle IV)
- **Wasmtime Integration**: wasmtime's async support requires Tokio (async Store, async component calls)
- **Future-Proofing**: Async architecture enables future features (streaming data between nodes, background compilation)
- **Ecosystem**: De facto standard for async Rust, extensive documentation and tooling

**Best Practices**:
- Use tokio::spawn for executing independent subgraphs in parallel
- Maintain deterministic ordering within dependency chains (await in topological order)
- Set timeouts for component execution to handle hanging nodes (edge case: 10+ second execution)
- Use async channels (tokio::sync::mpsc) for inter-node communication in streaming scenarios
- Profile with tokio-console to identify blocking operations in async contexts

### 7. Error Handling Strategy

**Decision**: Use thiserror 1.0 for error types, anyhow 1.0 for application-level errors.

**Rationale**:
- **thiserror**: Ergonomic derive macros for custom error types, provides good error context for library code
- **anyhow**: Context-rich error chains for application code, excellent for debugging (FR-018: clear error messages)
- **No Panics**: Constitution requires all errors via Result types, both libraries support this
- **Error Context**: anyhow's .context() provides actionable context (which node, what input, why failed)

**Error Taxonomy**:
```
GraphError           # Graph structure errors (cycle, invalid connection)
  ├─ CycleDetected(Vec<NodeId>)
  ├─ TypeMismatch { from: DataType, to: DataType }
  └─ InvalidConnection { source, target }

ComponentError       # Component loading/execution errors
  ├─ LoadFailed(PathBuf, String)
  ├─ ValidationFailed(String)
  ├─ ExecutionError { node: NodeId, cause: String }
  └─ PermissionDenied { node: NodeId, capability: String }

SerializationError   # Save/load errors
  ├─ SaveFailed(PathBuf, io::Error)
  └─ LoadFailed(PathBuf, bincode::Error)
```

**Best Practices**:
- Surface errors to UI with actionable messages (FR-018)
- Log full error chain for debugging while showing simplified message to user
- Never unwrap() in production code paths (use Result propagation or graceful fallback)
- Provide error recovery suggestions in UI (e.g., "Type mismatch: Connect to a Number input instead")

## Performance Optimization Strategies

### Instance Pooling

**Strategy**: Pre-instantiate components and reuse instances across invocations.

**Implementation**:
```rust
struct InstancePool {
    pools: HashMap<ComponentId, Vec<Instance>>, // HashMap OK here (not serialized)
    max_per_component: usize,
}
```

**Expected Impact**: Reduces per-invocation overhead from ~50ms (cold start) to <1ms (warm instance).

### Lazy Compilation

**Strategy**: Compile WASM components on first use, cache compiled modules.

**Implementation**: Use wasmtime::Module::from_file + in-memory cache with LRU eviction.

**Expected Impact**: Eliminates upfront compilation cost, spreads load across graph construction.

### Incremental Execution

**Strategy**: Track dirty nodes (inputs changed), re-execute only affected subgraph.

**Implementation**: Maintain node dirty flags, propagate downstream on input change, execute dirty subgraph only.

**Expected Impact**: 10-100x faster re-execution for localized changes vs. full graph execution.

### UI Rendering Optimizations

**Strategy**: Minimize redraws, use egui's retained mode optimizations.

**Implementation**:
- Only request repaint when graph changes or during animations
- Use egui::Area with fixed positions for nodes (avoid recalculation)
- Batch connection line drawing with single painter call

**Expected Impact**: Maintains 60 FPS even with 500+ nodes (constitution target).

## Security Considerations

### WASI Capability Enforcement

**Approach**: Build WasiCtx per component with minimal capabilities.

**Example Capability Mapping**:
```rust
Strict      → WasiCtxBuilder::new() (empty context)
Limited     → .preopened_dir(path, DirPerms::READ)
Standard    → .preopened_dir(user_data_dir, DirPerms::READ | DirPerms::WRITE)
Extended    → + .inherit_network() with allowlist enforcement
Developer   → .inherit_stdio().inherit_env() (warning required)
```

**Validation**: Integration tests verify that components cannot access denied resources (SC-006).

### Component Validation

**Pre-Load Checks**:
1. WIT interface compatibility (correct exports, types match)
2. Module size limits (reject > 50MB components)
3. Import requirements (ensure host can satisfy all imports)
4. Metadata validation (author, version fields present)

**Runtime Checks**:
1. Resource limits via Store::limiter (memory, fuel)
2. Timeout enforcement (kill execution after 30s)
3. Return value validation (types match WIT declaration)

## Development Workflow

### Component Development Cycle

1. **Create Component**: `cargo component new my-node --lib`
2. **Define Interface**: Edit `wit/world.wit` to match node interface
3. **Implement**: Write Rust code implementing Guest trait
4. **Build**: `cargo component build --release`
5. **Test**: Copy .wasm to `components/`, load in application
6. **Iterate**: Hot-reload during development (Phase 4 feature)

### Testing Strategy

**Unit Tests**: Test individual modules (type checking, topology, serialization)
**Contract Tests**: Validate WIT interface implementations
**Integration Tests**: End-to-end graph execution, security enforcement
**Performance Benchmarks**: Criterion benchmarks for execution overhead, load times

## Open Questions & Future Research

### Streaming Data

**Question**: Should nodes support streaming data for large datasets?

**Current Decision**: Deferred to future phases. Initial implementation uses discrete value passing (load entire dataset into memory).

**Rationale**: Adds complexity (async iterators, backpressure) without clear MVP requirement. Can be added later via new WIT interface type.

### Graph Diffing for Undo/Redo

**Question**: Use command pattern or graph snapshots for undo/redo (FR-016)?

**Current Decision**: Command pattern for graph edits (AddNode, RemoveConnection commands).

**Rationale**: More memory efficient than snapshots for large graphs, enables fine-grained undo history.

### Component Distribution

**Question**: How will users discover and install third-party components?

**Current Decision**: Out of scope for MVP. Users manually place .wasm files in `components/` directory.

**Future Considerations**: Component registry (similar to crates.io), package manager integration, in-app component browser.

## References

- [WASI Component Model](https://github.com/WebAssembly/component-model)
- [wasmtime Guide](https://docs.wasmtime.dev/)
- [egui Documentation](https://docs.rs/egui/latest/egui/)
- [egui-snarl Repository](https://github.com/zakarumych/egui-snarl)
- [petgraph Documentation](https://docs.rs/petgraph/latest/petgraph/)
- [WIT Format Specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
