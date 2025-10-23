<!--
Sync Impact Report:
- Version change: N/A → 1.0.0 (initial constitution)
- Modified principles: N/A (initial creation)
- Added sections: All core principles, Security & Capabilities, Development Standards, Governance
- Removed sections: N/A
- Templates requiring updates:
  ✅ plan-template.md - Constitution Check section ready for validation
  ✅ spec-template.md - Requirements align with component model and security principles
  ✅ tasks-template.md - Task organization supports component-based development
- Follow-up TODOs: None
-->

# WasmFlow Constitution

## Core Principles

### I. Component-First Architecture

Every feature MUST be implemented as a WebAssembly component using the WASI Component Model.

- Components MUST define WIT interfaces for all inputs, outputs, and behavior
- Components MUST be self-contained and independently testable
- Node-based visual composition is the primary user interaction model
- Builtin nodes MAY be implemented in Rust but MUST present the same component interface as user-defined nodes
- Clear functional purpose required—no organizational-only components

**Rationale**: Component isolation enables secure execution, composition flexibility, and user extensibility without recompilation of the host application.

### II. Capability-Based Security (NON-NEGOTIABLE)

Default deny for all system access; explicit user-granted capabilities required.

- Components MUST declare required capabilities (file I/O, network, etc.) in metadata
- Runtime MUST enforce capability restrictions via WASI context configuration
- Users MUST explicitly grant permissions before component execution
- Path scoping for file access—no unrestricted filesystem access
- Network access MUST be allowlist-based (specific hosts only)
- Resource limits (CPU, memory) MUST be enforced per component

**Rationale**: Security-first design prevents malicious or buggy user-created components from compromising the system. Inspired by Upload Labs' controlled execution environment.

### III. Typed Data Flow

All node connections MUST be type-checked before graph execution.

- WIT-defined data types enforce type safety at component boundaries
- Type validation occurs at graph construction time (fail fast)
- Support for primitive types (u32, i32, f32, string), collections (list), and structured data (record)
- Binary data supported for efficient large payloads
- Type mismatches MUST be surfaced to users in the UI with clear error messages

**Rationale**: Type safety prevents runtime errors in composed graphs and provides clear contracts for component developers.

### IV. Performance-First Execution

60 FPS UI rendering and near-native execution speed are mandatory.

- Graph execution engine MUST use topological sorting for dependency resolution
- Component instance pooling and reuse to minimize instantiation overhead (< 10ms per invocation)
- Parallel execution of independent graph branches where possible
- Lazy compilation of WASM components (compile on first use, cache thereafter)
- Incremental graph updates—only re-execute affected nodes on data changes
- Target: support 500+ node graphs without performance degradation

**Rationale**: Desktop-native performance expectations; visual programming tools MUST feel responsive to be usable.

### V. Developer Ergonomics

Component development workflow MUST be streamlined and well-documented.

- Standard cargo-component tooling for component creation
- Clear WIT interface examples and templates
- Component hot-reloading during development (when possible)
- Structured logging and debugging support for component execution
- Error messages MUST provide actionable context (which node, which input, why)
- Integration tests for component contracts required before publishing

**Rationale**: User extensibility is a core feature; lowering barriers for component authors is critical for ecosystem growth.

### VI. Composability & Modularity

Components MUST support composition to create higher-level abstractions.

- WAC (WebAssembly Composition) CLI integration for component composition
- Composite components can be saved as reusable subgraphs
- Component metadata includes versioning (author, version, description)
- Node categories and search functionality for discoverability
- Import/export of component libraries across projects

**Rationale**: Visual programming scales through abstraction; composition enables building complex systems from simple, verified components.

## Security & Capabilities

### Sandboxing Levels

Components operate under one of five sandboxing levels:

1. **Strict**: No system access (pure computation)
2. **Limited**: Read-only file access to specific directories
3. **Standard**: Read/write to designated user data directory
4. **Extended**: Network access + standard file access
5. **Developer**: Full access (requires explicit opt-in, warning to user)

**Enforcement**: Each level mapped to WASI context configuration; runtime enforces via wasmtime security model.

### Capability Declaration

Components MUST declare capabilities in WIT metadata:

```wit
// Example capability declaration
capability file-read = { paths: ["/data/input"] }
capability network = { hosts: ["api.example.com"] }
```

**User Consent Flow**:
1. User adds component to graph
2. UI displays requested capabilities
3. User reviews and approves/denies
4. Approved capabilities stored with graph save file
5. Re-prompt on capability escalation

## Development Standards

### Project Structure

```
src/
├── ui/              # egui + egui-snarl node editor
├── runtime/         # wasmtime execution engine
├── graph/           # petgraph-based graph management
└── builtin/         # built-in node implementations

tests/
├── contract/        # Component WIT contract tests
├── integration/     # Graph execution integration tests
└── unit/            # Core logic unit tests

components/          # User-defined WASM components
docs/                # Development guides and examples
wit/                 # WIT interface definitions
```

### Code Quality Gates

- All graph execution logic MUST have unit tests (>80% coverage target)
- Component contracts MUST have WIT validation tests
- Integration tests required for:
  - New component interface features
  - Security capability changes
  - Graph serialization/deserialization
- Performance benchmarks for execution engine (regression detection)
- No panics in release builds—all errors via Result/Error types

### Dependency Policy

- WASM runtime: wasmtime with component-model feature
- UI: egui ecosystem (eframe, egui-snarl)
- Graph: petgraph for dependency analysis
- Serialization: serde + bincode for performance-critical paths
- Async: tokio for concurrent component execution
- Minimize dependency tree—avoid heavy crates where possible

## Governance

### Amendment Process

1. Proposal documented in issue or RFC
2. Technical review by core maintainers
3. Impact analysis on existing components and graphs
4. Approval required before merge
5. Migration guide provided for breaking changes
6. Constitution version incremented per semantic versioning

### Versioning Rules

- **MAJOR**: Breaking changes to WIT interfaces, security model, or graph serialization format
- **MINOR**: New component capabilities, optional features, expanded APIs
- **PATCH**: Bug fixes, documentation improvements, clarifications

### Compliance Verification

- All PRs MUST verify compliance with security principles
- Component submissions MUST pass capability declaration validation
- Performance regressions blocked unless justified with complexity tracking
- Breaking changes to core principles require MAJOR version bump and explicit migration path

### Runtime Guidance

For day-to-day development questions not covered by this constitution, refer to:
- `technical_spec.md` for architectural details
- `docs/component-development.md` for component authoring (when created)
- WIT specification documentation at component-model.bytecodealliance.org

**Version**: 1.0.0 | **Ratified**: 2025-10-12 | **Last Amended**: 2025-10-12
