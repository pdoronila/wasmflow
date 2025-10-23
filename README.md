# WasmFlow

WebAssembly Node-Based Visual Composition System

## Overview

WasmFlow is a native desktop sandbox application for exploring WebAssembly and WebAssembly Components. It enables visual composition of components through a node-based interface, allowing users to build data processing pipelines with full type safety and secure, capability-based execution. The project also experiments with LLM-assisted workflows and GitHub Spec Kit to enhance development and prototyping ideas.

## Prerequisites

- Rust 1.75 or later (stable channel)
- For component development:
  ```bash
  rustup target add wasm32-wasip2
  cargo install cargo-component
  ```

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run
```

## Project Structure

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
wit/                 # WIT interface definitions
docs/                # Development guides and examples
```

## Features

- **Visual Node Editor**: Intuitive drag-and-drop interface powered by egui-snarl
- **Type-Safe Connections**: Compile-time type checking prevents incompatible connections
- **WebAssembly Components**: Load custom nodes as WASM components with hot-reload support
- **Capability-Based Security**: Fine-grained permission system for file/network access
- **Graph Serialization**: Save and load complete graphs with checksum validation
- **Built-in Nodes**: Math operations (Add, Subtract, Multiply, Divide), Constants
- **Undo/Redo**: Full command history with non-destructive editing

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
cd examples/double-number

# Build the component (WASI Preview 2)
cargo component build --target wasm32-wasip2 --release

# Copy to components directory
cp target/wasm32-wasip2/release/double_number.wasm ../../components/

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
cd examples/example-http-fetch

# Build the component (WASI Preview 2 with HTTP support)
cargo build --target wasm32-wasip2 --release

# Copy to components directory
cp target/wasm32-wasip2/release/example_http_fetch.wasm ../../components/

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
cd examples/example-http-fetch
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

## Project Structure

```
wasmflow/
├── src/
│   ├── ui/              # egui + egui-snarl node editor
│   │   ├── app.rs       # Main application state
│   │   ├── canvas.rs    # Visual graph editor
│   │   ├── palette.rs   # Component palette
│   │   └── dialogs.rs   # UI dialogs
│   ├── runtime/         # wasmtime execution engine
│   │   ├── engine.rs    # Graph execution orchestrator
│   │   ├── wasm_host.rs # WASM component manager
│   │   └── capabilities.rs # Security system
│   ├── graph/           # petgraph-based graph management
│   │   ├── graph.rs     # NodeGraph structure
│   │   ├── node.rs      # Node and port types
│   │   ├── connection.rs # Type-safe connections
│   │   ├── execution.rs # Topological sorting
│   │   ├── serialization.rs # Save/load with validation
│   │   └── command.rs   # Undo/redo commands
│   └── builtin/         # Built-in node implementations
│       ├── math.rs      # Arithmetic operations
│       └── constants.rs # Constant values
├── tests/
│   ├── contract/        # Component WIT contract tests
│   ├── integration/     # Graph execution integration tests
│   └── unit/            # Core logic unit tests
├── examples/
│   └── double-number/   # Example custom component
├── components/          # User-defined WASM components (.wasm files)
├── wit/                 # WIT interface definitions
├── docs/                # Development guides
│   └── BUILDING_COMPONENTS.md  # Component development guide
└── specs/               # Feature specifications
    └── 001-webassembly-based-node/
        ├── quickstart.md      # Getting started guide
        ├── plan.md            # Architecture and design
        ├── data-model.md      # Entity relationships
        └── contracts/         # WIT interfaces
```

## Technology Stack

- **UI**: egui 0.32, eframe 0.32, egui-snarl 0.8
- **WASM Runtime**: wasmtime 27.0 with component-model support
- **Graph**: petgraph 0.6 for algorithms (topological sort, cycle detection)
- **Serialization**: bincode 1.3 with CRC64 checksums for integrity
- **Async**: tokio 1.40 for future WASM execution
- **Error Handling**: anyhow 1.0, thiserror 1.0

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Roadmap

- [x] Visual node editor with drag-and-drop
- [x] Type-safe node connections
- [x] Graph execution with topological sort
- [x] Save/load graphs with validation
- [x] Built-in math and constant nodes
- [x] WASM component loading infrastructure
- [x] Capability-based security system
- [ ] Full async component execution
- [ ] Permission dialog UI
- [ ] Additional built-in nodes (text, logic, I/O)
- [ ] Graph debugging and visualization

## License

MIT OR Apache-2.0
