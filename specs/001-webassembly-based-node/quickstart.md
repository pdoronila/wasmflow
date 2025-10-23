# WasmFlow Quickstart Guide

**Feature**: WebAssembly Node-Based Visual Programming System
**Audience**: Developers implementing or extending WasmFlow
**Last Updated**: 2025-10-12

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Building and Running WasmFlow](#building-and-running-wasmflow)
3. [Creating Your First Graph](#creating-your-first-graph)
4. [Creating a Custom Node Component](#creating-a-custom-node-component)
5. [Testing](#testing)
6. [Troubleshooting](#troubleshooting)

## Development Environment Setup

### Prerequisites

- **Rust**: 1.75 or later (stable channel)
- **Cargo**: Comes with Rust
- **Git**: For version control
- **Platform-specific**:
  - **Linux**: X11 or Wayland development libraries
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools or MinGW-w64

### Install Rust Toolchain

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update to latest stable
rustup update stable

# Add WASM target for component development
rustup target add wasm32-wasip2
```

### Install Component Development Tools

```bash
# Install cargo-component for building WASM components
cargo install cargo-component

# Install wac-cli for component composition (optional, Phase 4)
cargo install wac-cli

# Install wit-bindgen for manual WIT binding generation (optional)
cargo install wit-bindgen-cli
```

### Clone Repository

```bash
git clone <repository-url> wasmflow
cd wasmflow
git checkout 001-webassembly-based-node
```

## Building and Running WasmFlow

### Build the Application

```bash
# Development build (faster compilation, debug symbols)
cargo build

# Release build (optimized, for performance testing)
cargo build --release
```

**Expected Output**:
```
   Compiling wasmflow v0.1.0 (/path/to/wasmflow)
    Finished dev [unoptimized + debuginfo] target(s) in 45.23s
```

### Run the Application

```bash
# Run development build
cargo run

# Run release build
cargo run --release

# Run with debug logging
RUST_LOG=debug cargo run

# Run with trace logging for specific module
RUST_LOG=wasmflow::runtime=trace cargo run
```

**What to Expect**:
- A window opens with the WasmFlow visual editor
- Empty canvas in the center
- Node palette on the left showing built-in nodes (Add, Multiply, Constant, etc.)
- Menu bar at top with File, Edit, View options

### Verify Installation

```bash
# Run all tests to verify everything works
cargo test

# Run only unit tests
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run with output visible
cargo test -- --nocapture
```

## Creating Your First Graph

### Step 1: Add Constant Nodes

1. **Locate the Node Palette** on the left side of the window
2. **Find "Constant" node** under the "Values" category
3. **Drag Constant onto canvas** (should appear at cursor position)
4. **Double-click the Constant node** to set value to `5.0`
5. **Repeat** to create a second Constant node with value `3.0`

### Step 2: Add Math Operation

1. **Find "Add" node** under the "Math" category
2. **Drag Add onto canvas** between the two constants

### Step 3: Connect Nodes

1. **Click and drag** from the output port (right side) of Constant(5)
2. **Release** over the input port "a" (left side) of Add node
3. **Connection line appears** showing data flow
4. **Repeat** to connect Constant(3) output to Add input "b"

**Visual Feedback**:
- ✅ **Green connection line**: Types compatible
- ❌ **Red connection line**: Type mismatch (connection rejected)

### Step 4: Execute the Graph

1. **Click "Execute" button** in toolbar (or press Ctrl+E / Cmd+E)
2. **Observe output** on Add node's output port: "sum: 8.0"

**Troubleshooting Execution**:
- If execution fails, check bottom status bar for error messages
- Ensure all required inputs are connected (non-optional ports)
- Check console output (`RUST_LOG=info`) for detailed execution logs

### Step 5: Save Your Graph

1. **File → Save Graph** (or Ctrl+S / Cmd+S)
2. **Choose file location** (default: `graphs/my_first_graph.wasmflow`)
3. **Click Save**

**File Format**: Binary (bincode) with .wasmflow extension

### Step 6: Load a Saved Graph

1. **File → Load Graph** (or Ctrl+O / Cmd+O)
2. **Select .wasmflow file**
3. **Graph appears** on canvas with all nodes, connections, and values restored

## Creating a Custom Node Component

### Overview

Custom nodes are WebAssembly components implementing the WIT interface defined in `contracts/node-interface.wit`. This example creates a "Multiply by 2" node.

### Step 1: Create Component Project

```bash
# Create new component library
cargo component new double-number --lib

cd double-number
```

### Step 2: Define WIT Interface

Create or edit `wit/world.wit`:

```wit
package example:double@1.0.0;

world component {
    // Import WasmFlow node interface
    import wasmflow:node/types@1.0.0;
    import wasmflow:node/host@1.0.0;

    // Export node interface
    export wasmflow:node/metadata@1.0.0;
    export wasmflow:node/execution@1.0.0;
}
```

### Step 3: Implement Component Logic

Edit `src/lib.rs`:

```rust
use wasmflow::node::*;

// Component implementation
struct DoubleNumber;

impl Guest for DoubleNumber {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Double".to_string(),
            version: "1.0.0".to_string(),
            description: "Multiplies input by 2".to_string(),
            author: "Your Name".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "input".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Number to double".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "output".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Doubled number".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None  // No system access needed
    }

    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract input value
        let input_value = inputs.iter()
            .find(|(name, _)| name == "input")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid input".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect a number to the input port".to_string()),
            })?;

        // Compute result
        let result = input_value * 2.0;

        // Use host logging for debugging
        host::log("info", &format!("Doubling {} to {}", input_value, result));

        // Return output
        Ok(vec![("output".to_string(), Value::F32Val(result))])
    }
}

// Export component
wasmflow::node::export!(DoubleNumber);
```

### Step 4: Build Component

```bash
cargo component build --release
```

**Output Location**: `target/wasm32-wasip2/release/double_number.wasm`

### Step 5: Load Component in WasmFlow

1. **Copy WASM file** to WasmFlow's `components/` directory:
   ```bash
   cp target/wasm32-wasip2/release/double_number.wasm \
      /path/to/wasmflow/components/
   ```

2. **Restart WasmFlow** or use File → Reload Components

3. **Find "Double" node** in palette under "Math" category

4. **Drag onto canvas** and connect to test

### Step 6: Test Custom Component

Create a simple test graph:
```
Constant(7) → Double → Output (should show 14.0)
```

## Testing

### Unit Tests

Test individual modules in isolation:

```bash
# Test type checking logic
cargo test type_checking

# Test graph topology algorithms
cargo test topology

# Test WASM host functionality
cargo test wasm_host
```

### Integration Tests

Test end-to-end scenarios:

```bash
# Test graph execution
cargo test --test graph_execution_tests

# Test save/load functionality
cargo test --test serialization_tests

# Test security/permission enforcement
cargo test --test security_tests
```

### Component Contract Tests

Validate WIT interface compliance:

```bash
# Test component interface contracts
cargo test --test component_interface_tests
```

### Manual Testing Checklist

- [ ] Create graph with 10+ nodes and execute successfully
- [ ] Save graph, close app, reload graph, verify all state restored
- [ ] Load custom component with file permissions, approve, verify access
- [ ] Attempt invalid connection (type mismatch), verify rejection
- [ ] Create graph with cycle, verify execution prevented
- [ ] Execute graph with 100+ nodes, verify <3s load time, 60 FPS maintained

## Troubleshooting

### Build Errors

**Problem**: `error: linking with 'cc' failed`
- **Solution**: Install C compiler (gcc/clang on Linux, Xcode on macOS, MSVC on Windows)

**Problem**: `error: cannot find -lX11` (Linux)
- **Solution**: Install X11 development libraries
  ```bash
  # Ubuntu/Debian
  sudo apt install libx11-dev libxcursor-dev libxrandr-dev libxi-dev

  # Fedora
  sudo dnf install libX11-devel libXcursor-devel libXrandr-devel libXi-devel
  ```

### Runtime Errors

**Problem**: "Failed to load component: Invalid WASM module"
- **Solution**: Verify component built with `cargo component build` (not `cargo build`)
- **Check**: Component targets `wasm32-wasip2` (not `wasm32-unknown-unknown`)

**Problem**: "Type mismatch" when connecting nodes
- **Solution**: Check WIT interface matches expected types (F32 vs I32, etc.)
- **Debug**: Enable type checking logs with `RUST_LOG=wasmflow::graph::connection=debug`

**Problem**: "Permission denied" when component accesses file
- **Solution**: Verify capability grant in permission dialog
- **Check**: Component declared required capability in `get_capabilities()`

### Performance Issues

**Problem**: UI stutters or drops below 60 FPS
- **Solution**: Profile with `cargo build --release --features profiling`
- **Check**: Graph size (>500 nodes may need optimization)
- **Optimize**: Enable instance pooling, verify lazy compilation active

**Problem**: Component execution takes >100ms per node
- **Solution**: Review component implementation for blocking operations
- **Profile**: Use `host::log()` to measure execution time within component

### Development Tips

1. **Use RUST_LOG for debugging**:
   ```bash
   RUST_LOG=debug cargo run
   RUST_LOG=wasmflow::runtime=trace cargo run
   ```

2. **Hot-reload components**: File → Reload Components (avoids app restart)

3. **Inspect graph structure**: File → Export Graph as JSON (debug format)

4. **Component testing**: Test component in isolation before loading into app
   ```bash
   cargo component test --package double-number
   ```

5. **Validate WIT interfaces**: Use wit-bindgen to check for errors
   ```bash
   wit-bindgen rust --out-dir src/bindings wit/
   ```

## Next Steps

- **Read [data-model.md](./data-model.md)** for detailed entity descriptions
- **Review [contracts/node-interface.wit](./contracts/node-interface.wit)** for WIT interface details
- **Explore [research.md](./research.md)** for architecture decisions and best practices
- **Check [plan.md](./plan.md)** for implementation roadmap

## Getting Help

- **Check logs**: `RUST_LOG=debug cargo run` for detailed output
- **Run tests**: `cargo test` to verify system health
- **Review constitution**: `.specify/memory/constitution.md` for design principles
- **File issues**: Report bugs or ask questions in repository issue tracker
