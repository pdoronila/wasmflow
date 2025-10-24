# WasmFlow

WebAssembly Node-Based Visual Composition System

## Overview

WasmFlow is a native desktop sandbox application for exploring WebAssembly and WebAssembly Components. It enables visual composition of components through a node-based interface, allowing users to build data processing pipelines with full type safety and secure, capability-based execution. The project also experiments with LLM-assisted workflows and GitHub Spec Kit to enhance development and prototyping ideas.

## Prerequisites

### Option 1: Using mise (Recommended)

[mise](https://mise.jdx.dev/) automatically manages all required tools and dependencies:

```bash
# Install mise (if not already installed)
curl https://mise.run | sh

# Install all dependencies (Python, Node.js, cargo-component, etc.)
mise install

# Run setup tasks (Python tools, Rust target)
mise run setup

# Verify installation
mise run verify
```

That's it! mise handles:
- ✓ Python 3.11 and `componentize-py`
- ✓ Node.js 20 and `componentize-js`
- ✓ `cargo-component` and `wasm-tools`
- ✓ `wasm32-wasip2` Rust target

### Option 2: Manual Installation

If you prefer manual setup:

1. **Rust 1.75 or later** (stable channel)
   ```bash
   rustup target add wasm32-wasip2
   ```

2. **Component compilation tools:**
   ```bash
   # Rust components
   cargo install cargo-component

   # Python components (optional)
   pip install componentize-py

   # JavaScript components (optional)
   npm install -g @bytecodealliance/componentize-js
   ```

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run

# Or use mise tasks
mise run dev      # Run in development mode
mise run build    # Build release
mise run test     # Run all tests
```

## Project Structure

```
src/
├── ui/                      # egui + egui-snarl node editor
│   ├── app/                 # Main application (modular architecture)
│   │   ├── state.rs         # File I/O, undo/redo, graph lifecycle
│   │   ├── components.rs    # WASM component loading
│   │   ├── permissions.rs   # Capability-based security dialogs
│   │   ├── composition.rs   # WAC composition, drill-down navigation
│   │   └── execution.rs     # Graph execution, continuous nodes
│   ├── canvas/              # Visual graph editor (modular)
│   │   ├── node_data.rs     # Node/port data structures
│   │   ├── viewer.rs        # egui-snarl viewer implementation
│   │   ├── footer.rs        # Node footer rendering
│   │   └── selection.rs     # Rectangle selection
│   ├── dialogs.rs           # UI dialogs (permissions, metadata, etc.)
│   ├── palette.rs           # Component palette with search
│   └── theme.rs             # Visual theming
├── runtime/                 # wasmtime execution engine
│   ├── engine.rs            # Graph execution orchestrator
│   ├── wasm_host.rs         # WASM component manager
│   ├── compiler.rs          # Component compilation (Rust/Python/JS)
│   ├── capabilities.rs      # Security and permission system
│   └── continuous.rs        # Long-running node execution
├── graph/                   # petgraph-based graph management
│   ├── graph.rs             # NodeGraph structure
│   ├── node.rs              # Node and port types
│   ├── connection.rs        # Type-safe connections
│   ├── execution.rs         # Topological sorting
│   ├── serialization.rs     # Save/load with CRC validation
│   ├── command.rs           # Undo/redo commands
│   └── drill_down.rs        # Composite node navigation
└── builtin/                 # Built-in node implementations
    ├── math.rs              # Arithmetic operations
    ├── constants.rs         # Constant values
    ├── continuous_example.rs # Long-running examples
    └── wasm_creator.rs      # In-app component builder

tests/
├── contract/                # Component WIT contract tests
├── integration/             # Graph execution integration tests
└── unit/                    # Core logic unit tests

components/bin/              # User-defined WASM components (.wasm files)
wit/                         # WIT interface definitions
docs/                        # Development guides and examples
.mise.toml                   # Tool version management
```

## Features

### Core Features
- **Visual Node Editor**: Intuitive drag-and-drop interface powered by egui-snarl
- **Type-Safe Connections**: Runtime type checking prevents incompatible connections
- **WebAssembly Components**: Load custom nodes as WASM components with hot-reload support
- **Capability-Based Security**: Fine-grained permission system for file/network access
- **Graph Serialization**: Save and load complete graphs with CRC64 checksum validation
- **Undo/Redo**: Full command history with non-destructive editing
- **Component Composition**: Compose multiple WASM components into composite nodes with WAC
- **Drill-Down Navigation**: Explore internal structure of composite nodes visually

### Built-in Nodes
- **Math Operations**: Add, Subtract, Multiply, Divide
- **Constants**: Type-specific constant values (F32, I32, U32, String)
- **Continuous Execution**: Long-running nodes with start/stop controls
- **WASM Creator**: In-app component builder with live compilation (Rust/Python/JavaScript)

### Core Component Library (New!)

WasmFlow now includes a comprehensive library of **34 pre-built components** for common data processing tasks:

**Text Processing (7 components)**
- string-concat, string-split, string-length, string-trim, string-case, string-contains, string-substring

**Logic & Validation (7 components)**
- compare, boolean-and, boolean-or, boolean-not, boolean-xor, is-null, is-empty

**Mathematical Operations (9 components)**
- power, sqrt, abs, min, max, floor, ceil, round, trig (sin/cos/tan)

**List Manipulation (7 components)**
- list-length, list-get, list-append, list-join, list-slice, list-contains, list-index-of

**Data Transformation (4 components)**
- json-stringify, to-string, parse-number, format-template

All components are:
- **Pure Computation**: No side effects, fully composable
- **Type-Safe**: WIT-based interfaces with clear contracts
- **Well-Tested**: 100+ unit tests across all components
- **Optimized**: 50-150KB binary size with LTO optimization
- **Documented**: Comprehensive guides in `components/LIBRARY.md`

**Building the Library**:
```bash
cd components
just build-all      # Build all 34 components
just test-all       # Run all tests
just install-all    # Copy to bin/
```

See **[Component Library Guide](components/LIBRARY.md)** for complete documentation, API reference, and usage examples.

### Development Features
- **Modular Architecture**: Clean separation of concerns in UI and runtime layers
- **Hot Reload**: Develop components iteratively without restarting
- **Component Search**: Fast palette search with fuzzy matching
- **Recent Files**: Quick access to recently edited graphs
- **Graph Metadata**: Track author, description, creation/modification dates

## Quick Start

### Running WasmFlow

```bash
# Clone the repository
git clone <repository-url>
cd wasmflow

# Build and run
cargo run --release
```

The application window opens with:
- **Node Palette** (left): Available components categorized by type
- **Canvas** (center): Visual graph editor
- **Menu Bar** (top): File operations, component loading, execution
- **Status Bar** (bottom): Real-time feedback and error messages

### Creating Your First Graph

1. **Add nodes**: Drag "Constant" nodes from palette (under "Constants")
2. **Set values**: Double-click constants, set one to `5.0`, another to `3.0`
3. **Add operation**: Drag "Add" node from "Math" category
4. **Connect nodes**: Click and drag from output ports to input ports
5. **Execute**: Click "▶ Execute" button in menu bar
6. **View result**: See computed sum (`8.0`) on the Add node's output

## Building Custom Components

WasmFlow supports user-defined nodes as WebAssembly components. Create powerful custom operations in Rust that integrate seamlessly into your graphs.

### Example: Double Number Component

```bash
# Navigate to example
cd components/double-number

# Build the component (WASI Preview 2)
cargo component build --target wasm32-wasip2 --release

# Copy to components directory
cp target/wasm32-wasip2/release/double_number.wasm ../bin/

# Load in WasmFlow
# File → Reload Components
```

### Component Development

See comprehensive guides:
- **[Building Components Guide](docs/BUILDING_COMPONENTS.md)** - Complete tutorial with examples
- **[Example Component](examples/double-number/README.md)** - Working reference implementation
- **[HTTP Fetch Component](examples/example-http-fetch/README.md)** - WASI HTTP network example
- **[Components Directory](components/README.md)** - Installation and troubleshooting

Key features:
- **Type-safe interfaces**: WIT-based contracts ensure correctness
- **Sandboxed execution**: WASM isolation with capability-based permissions
- **Host functions**: Logging, temp directory access, more coming
- **Hot reload**: Develop iteratively without restarting WasmFlow

### HTTP Fetch Component (WASI HTTP Example)

The HTTP Fetch component demonstrates real network capabilities using **WASI HTTP Preview 2**, allowing components to make HTTP GET requests with capability-based security.

#### Building the HTTP Fetch Component

```bash
# Navigate to HTTP Fetch example
cd components/http-fetch

# Build the component (WASI Preview 2 with HTTP support)
cargo build --target wasm32-wasip2 --release

# Copy to components directory
cp target/wasm32-wasip2/release/example_http_fetch.wasm ../bin/

# Load in WasmFlow
# File → Reload Components
```

#### Using HTTP Fetch in Graphs

The HTTP Fetch component provides network access with these features:

**Inputs:**
- `url` (String, required): HTTP/HTTPS URL to fetch
- `timeout` (U32, optional): Request timeout in seconds (default: 30)

**Outputs:**
- `body` (String): Response body as UTF-8 text
- `status` (U32): HTTP status code (200, 404, etc.)
- `headers` (String, optional): Response headers as JSON

**Capability-Based Security:**
Components declare which domains they can access via `get-capabilities()`. The runtime enforces these permissions, blocking requests to unauthorized domains.

Example component capabilities:
```rust
fn get-capabilities() -> Option<Vec<String>> {
    Some(vec![
        "network:httpbin.org",      // Allow httpbin.org and subdomains
        "network:api.example.com",  // Allow api.example.com
    ])
}
```

#### Example Graph: Fetch and Display

```
┌───────────┐        ┌──────────────┐        ┌─────────┐
│ Constant  │  url   │ HTTP Fetch   │  body  │ Display │
│ "https:// ├───────>│              ├───────>│         │
│  httpbin. │        │              │        │         │
│  org/get" │        │              │ status │         │
└───────────┘        └──────────────┴───────>│         │
                                              └─────────┘
```

**Graph workflow:**
1. Constant node provides URL string
2. HTTP Fetch makes GET request with 30s timeout
3. Response body (JSON) and status code (200) output to Display

#### Testing HTTP Components

Run comprehensive test suite:

```bash
# All tests (unit + integration + contract)
cargo test --all

# Unit tests for HTTP Fetch component
cd components/http-fetch
cargo test

# Integration tests (requires network)
cargo test --test wasi_http_execution_test

# Contract tests (WIT interface validation)
cargo test --test wasi_http_component_test
```

**Test coverage includes:**
- Unit tests (23): Helper functions, URL validation, timeout logic, error mapping
- Integration tests (19): Real HTTP requests, error scenarios, capability validation
- Contract tests (3): WIT interface compliance
- Manual tests (10): UI workflows and edge cases

#### Key Implementation Details

**WASI HTTP Integration:**
The component uses standard WASI HTTP interfaces (`wasi:http/outgoing-handler`), making it portable across WASI-compliant runtimes:

```rust
// Component imports WASI HTTP
import wasi:http/outgoing-handler@0.2.0;
import wasi:http/types@0.2.0;

// Make HTTP request in component
let request = OutgoingRequest::new(Headers::new());
request.set_method(&Method::Get)?;
request.set_authority(Some("api.example.com"))?;
request.set_path_with_query(Some("/data"))?;

// Execute with timeout (nanoseconds)
let timeout_ns = 30_000_000_000; // 30 seconds
let future_response = outgoing_handler::handle(request, Some(timeout_ns))?;
let incoming_response = future_response.get()?;

// Read response
let status = incoming_response.status();
let body_stream = incoming_response.consume()?;
let body = read_body_to_string(body_stream)?; // Max 10MB
```

**Error Handling:**
Component provides clear, actionable error messages for:
- DNS resolution failures → "Check domain spelling and internet connection"
- Connection refused → "Server may be down or blocking connections"
- Timeouts → "Server is slow. Try increasing timeout."
- Capability violations → "This component can only access: [allowed domains]"

**Security:**
- Component-side URL validation against declared capabilities
- Subdomain matching: `api.example.com` allows `*.api.example.com`
- 10MB response size limit to prevent memory exhaustion
- Timeout enforcement (1-300 seconds) to prevent indefinite blocking

#### Quickstart Guide

For a complete walkthrough of building HTTP-enabled components, see:
- **[HTTP Component Quickstart](specs/002-lets-focus-on/quickstart.md)** - 30-minute tutorial
- **[WASI HTTP Usage Guide](specs/002-lets-focus-on/contracts/wasi-http-usage.md)** - Interface reference
- **[Data Model](specs/002-lets-focus-on/data-model.md)** - Request/Response entities

**Note:** WASI HTTP is a Preview feature (0.2.0) and is experimental. This component demonstrates standards-based network access in WASM components and serves as a learning resource for component model development.

## Development

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test '*'

# With output
cargo test -- --nocapture
```

### Code Quality

```bash
# Linting
cargo clippy

# Formatting
cargo fmt --check

# Release build
cargo build --release
```

### Documentation

- **[Quickstart Guide](specs/001-webassembly-based-node/quickstart.md)** - Development setup and workflows
- **[Data Model](specs/001-webassembly-based-node/data-model.md)** - Core entities and relationships
- **[Architecture Plan](specs/001-webassembly-based-node/plan.md)** - System design and tech stack
- **[WIT Interface](specs/001-webassembly-based-node/contracts/node-interface.wit)** - Component contract

## Technology Stack

- **UI Framework**: egui 0.33, eframe 0.33, egui-snarl (latest from git)
- **WASM Runtime**: wasmtime 27.0 with component-model and async support
- **WASI Extensions**: wasmtime-wasi 27.0, wasmtime-wasi-http 27.0
- **Graph Algorithms**: petgraph 0.6 (topological sort, cycle detection)
- **Serialization**: bincode 1.3, serde/serde_json with BTreeMap for deterministic ordering
- **Data Integrity**: CRC 3.0 for CRC64 checksums
- **Composition**: wac-graph 0.8 for WebAssembly composition
- **Async Runtime**: tokio 1.40 for background execution
- **Error Handling**: anyhow 1.0, thiserror 1.0
- **Utilities**: uuid 1.6, chrono 0.4, regex 1.10

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Roadmap

### Completed ✓
- [x] Visual node editor with drag-and-drop
- [x] Type-safe node connections with runtime validation
- [x] Graph execution with topological sort
- [x] Save/load graphs with CRC64 validation
- [x] Built-in math and constant nodes
- [x] WASM component loading infrastructure with hot-reload
- [x] Capability-based security system
- [x] Permission dialog UI (view, approve, revoke, upgrade)
- [x] Component composition with WAC (WebAssembly Composition)
- [x] Drill-down navigation for composite nodes
- [x] Continuous execution nodes (long-running processes)
- [x] In-app WASM component builder (Rust/Python/JavaScript)
- [x] Rectangle selection tool
- [x] Recent files management
- [x] Graph metadata editor
- [x] Modular UI architecture refactoring
- [x] Component palette with search
- [x] **Core Component Library (34 components across 5 categories)**
  - [x] Text Processing (7): string operations
  - [x] Logic & Validation (7): comparison and boolean operations
  - [x] Mathematical Operations (9): advanced math functions
  - [x] List Manipulation (7): collection processing
  - [x] Data Transformation (4): type conversion and formatting

### In Progress 🚧
- [ ] Full async component execution with streaming I/O
- [ ] Python and JavaScript component examples
- [ ] Enhanced debugging and visualization tools

### Planned 📋
- [ ] Breakpoint debugging for graph execution
- [ ] Performance profiling and metrics
- [ ] Graph templates and snippets
- [ ] Export graphs to standalone WASM modules

## License

MIT OR Apache-2.0
