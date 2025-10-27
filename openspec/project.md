# Project Context

## Purpose
WasmFlow is a visual programming application that enables users to create and execute computational workflows using WebAssembly components in a node-based graph editor. The project allows users to:
- Build data flow graphs by connecting WASM component nodes
- Execute both synchronous and continuous (long-running) workflows
- Create custom WASM components with rich UI rendering capabilities
- Compose complex workflows from a library of 34+ core components across text, math, logic, collections, and data transformation categories

## Tech Stack
**Core Runtime**
- Rust 1.75+ (stable channel with wasm32-wasip2 target)
- wasmtime 27.0 (WASM runtime with component-model support)
- tokio (async runtime for continuous execution)

**UI Framework**
- egui 0.29-0.33 (immediate mode UI library)
- eframe 0.29-0.33 (application framework)
- egui-snarl 0.3 (node graph editor)

**Graph & Data**
- petgraph 0.6 (graph algorithms)
- serde/bincode (serialization with deterministic BTreeMap)
- crc (CRC64 checksums for graph validation)

**WebAssembly**
- wit-bindgen 0.30 (WIT interface generation)
- wasmtime-wasi-http 27.0 (WASI HTTP Preview support)
- WAC CLI (WebAssembly Composition)

## Project Conventions

### Code Style
**Rust Conventions**
- Follow standard Rust conventions and idioms
- Use `cargo clippy` for linting
- Edition: 2021

**Naming Conventions**
- Component directories: `kebab-case` (e.g., `string-concat`, `json-parser`)
- Rust code: `snake_case` for functions, variables, modules
- Component names: Title Case in metadata (e.g., "String Concat")

**Component Structure**
- All WASM components follow standard structure with MetadataGuest, ExecutionGuest traits
- Always include `export!(Component)` macro
- Place unit tests in `#[cfg(test)] mod tests`

### Architecture Patterns
**Data Structure Guidelines**
- **Use BTreeMap for all serialized data structures** (e.g., NodeGraph.nodes, NodeValue::Record) to ensure deterministic serialization and enable CRC64 checksum validation
- **Use HashMap for runtime-only structures** marked with `#[serde(skip)]` where non-deterministic ordering is acceptable
- Performance difference is negligible for <1000 nodes

**Node Layout Constraints**
- Node footers MUST have both width and height constraints at canvas level to prevent infinite growth
- Use `ui.set_max_width()` and `ui.set_max_height()` in `show_footer()` method
- ScrollArea must use `.auto_shrink([false, true])` to prevent horizontal shrinking
- WIT renderer vertical layouts must set minimum width with `ui.set_min_width(ui.available_width())`

**Component World Selection**
- Standard `component` world: Pure computation, simple I/O, no custom UI
- `component-with-ui` world: Components needing custom footer rendering with `impl UiGuest`

**Continuous Execution**
- Use `ContinuousNodeConfig` with `runtime_state` marked `#[serde(skip)]`
- Follow state machine: Idle → Starting → Running → Stopping → Stopped/Error
- 3-phase shutdown: 1.5s graceful + 0.5s forced + cleanup

### Testing Strategy
**Component Tests**
- Minimum 3 tests per component: typical usage, edge cases, error handling
- Current coverage: 122+ tests across 34 components
- Use `cargo test` for unit tests

**Integration Tests**
- JSON-based graph tests in `tests/component_tests/`
- Categories: string_processing, data_validation, math_operations, list_manipulation, data_transformation, comprehensive_workflow

**Quality Checks**
- `cargo clippy` for linting
- `cargo test` for all tests
- Build verification: `cargo build --release`

### Git Workflow
**Branching**
- Main branch: `main`
- Feature branches: descriptive names (e.g., `claude/implement-selection-mode-*`)

**Commits**
- Use conventional commit style when appropriate
- Prefix with type: `feat:`, `fix:`, `docs:`, `refactor:`
- Example: "feat: Implement clone selection feature with full undo/redo support"

**Pull Requests**
- PR branches merge into `main`
- Include clear description of changes
- Reference related issues or specifications

## Domain Context

### Visual Programming Concepts
- **Node Graph**: A directed graph where nodes represent computations and edges represent data flow
- **Ports**: Inputs and outputs on nodes that can be connected together
- **Execution**: Nodes execute when all required inputs are available
- **Continuous Nodes**: Long-running nodes that execute repeatedly (e.g., timers, HTTP servers)
- **Composite Nodes**: Encapsulated subgraphs that appear as single nodes

### WebAssembly Component Model
- **WIT (WebAssembly Interface Types)**: Interface definition language for WASM components
- **Component Worlds**: Define the imports and exports of a component
  - Standard world: metadata + execution exports
  - With-UI world: adds UI export for custom footer rendering
- **Host Interface**: Rust application imports exposed to WASM components
- **Guest Traits**: Traits that WASM components implement (MetadataGuest, ExecutionGuest, UiGuest)

### Node Types
- **Builtin Nodes**: Hardcoded in Rust (math, constant, WASM creator)
- **WASM Component Nodes**: Loaded dynamically from .wasm files
- **Composite Nodes**: User-created subgraphs
- **Continuous Nodes**: Support long-running execution with state management

### Value System
The type system supports these value types:
- Primitives: U32, I32, F32, Bool, String
- Collections: StringListVal, U32ListVal, F32ListVal
- Complex: BinaryVal, RecordVal (BTreeMap-based)

### Component Library Categories
- **Text**: String operations (concat, trim, case, substring, etc.)
- **Math**: Arithmetic and comparison operations
- **Logic**: Boolean operations and conditional logic
- **Collections**: List operations (filter, map, join, slice, etc.)
- **Data**: JSON parsing, HTTP requests, type conversions

## Important Constraints

### Technical Constraints
- **Deterministic Serialization**: All persisted data structures must use BTreeMap to ensure consistent graph checksums
- **WASM Target**: Components must compile to `wasm32-wasip2` target
- **UI Layout**: Node footers require strict size constraints to prevent infinite growth in egui-snarl
- **Component Binary Size**: Target 50-150KB per component with LTO and strip optimizations
- **Execution Model**: Synchronous node execution (except continuous nodes which use async tokio)

### Design Constraints
- **No Persistent Storage**: Graph state lives only in memory and user-saved JSON files
- **Single Selection vs Multi-Selection**: Two distinct interaction modes (Normal vs Selection)
- **Component Immutability**: WASM components are stateless; all state flows through ports
- **WIT Interface Versioning**: Components must match expected WIT interface versions

### Safety Constraints
- **No Arbitrary Code Execution**: Only approved WASM components can be loaded
- **Resource Limits**: Continuous nodes have timeout and shutdown mechanisms
- **Type Safety**: All value conversions are explicit with error handling

## External Dependencies

### Build Tools
- **cargo**: Rust build tool and package manager
- **wac CLI**: WebAssembly Composition tool for component linking
- **wit-bindgen**: Code generator for WIT interfaces

### Runtime Dependencies
- **wasmtime**: WASM runtime engine (version 27.0 with component-model)
- **tokio**: Async runtime for continuous node execution

### Optional Development Tools
- **just**: Command runner (Justfiles for component builds)
- **gh**: GitHub CLI for PR workflows

### Component Dependencies
Individual components may use:
- **serde_json**: JSON parsing (json-parser, json-stringify components)
- **WASI HTTP**: Network requests (http-fetch component)

### No External Services
The application runs entirely locally with no cloud dependencies, APIs, or external services required for core functionality.
