# Building Custom WasmFlow Components

This guide explains how to create custom computational nodes for WasmFlow using WebAssembly components.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Component Structure](#component-structure)
- [WIT Interface](#wit-interface)
- [Implementation Guide](#implementation-guide)
- [Building and Testing](#building-and-testing)
- [Loading Components](#loading-components)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before building custom components, ensure you have:

1. **Rust toolchain** (1.75 or later):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **WASM target**:
   ```bash
   rustup target add wasm32-wasip2
   ```

3. **cargo-component** (for building components):
   ```bash
   cargo install cargo-component
   ```

4. **wit-bindgen-cli** (optional, for manual bindings):
   ```bash
   cargo install wit-bindgen-cli
   ```

## Quick Start

### 1. Create a New Component

```bash
# Create a new component library
cargo component new my-component --lib

cd my-component
```

### 2. Define the WIT Interface

Create or edit `wit/world.wit`:

```wit
package example:my-component@1.0.0;

world component {
    // Import WasmFlow node interface
    import wasmflow:node/types@1.0.0;
    import wasmflow:node/host@1.0.0;

    // Export node interface
    export wasmflow:node/metadata@1.0.0;
    export wasmflow:node/execution@1.0.0;
}
```

### 3. Implement the Component

Edit `src/lib.rs` (see [Example Component](#example-implementation) below).

### 4. Build the Component

```bash
cargo component build --target wasm32-wasip2 --release
```

The compiled component will be at: `target/wasm32-wasip2/release/my_component.wasm`

**Important**: Always use `--target wasm32-wasip2` to build for WASI Preview 2.

### 5. Load into WasmFlow

1. Copy the `.wasm` file to WasmFlow's `components/` directory
2. In WasmFlow: **File → Reload Components**
3. Your component appears in the palette under its category

## Component Structure

A WasmFlow component consists of:

```
my-component/
├── Cargo.toml          # Package configuration
├── wit/
│   └── world.wit       # Interface definition
└── src/
    └── lib.rs          # Component implementation
```

### Cargo.toml Configuration

```toml
[package]
name = "my-component"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Required for WASM components

[dependencies]
wit-bindgen = "0.30"

[profile.release]
opt-level = "s"  # Optimize for size
lto = true
strip = true
```

## WIT Interface

All WasmFlow components must implement the `wasmflow:node` interface:

### Required Exports

#### 1. Metadata Interface

```rust
// Returns component information
fn get_info() -> ComponentInfo;

// Returns input port specifications
fn get_inputs() -> Vec<PortSpec>;

// Returns output port specifications
fn get_outputs() -> Vec<PortSpec>;

// Returns required capabilities (optional)
fn get_capabilities() -> Option<Vec<String>>;
```

#### 2. Execution Interface

```rust
// Executes the node with provided inputs
fn execute(inputs: Vec<(String, Value)>)
    -> Result<Vec<(String, Value)>, ExecutionError>;
```

### Available Imports

WasmFlow provides host functions:

```rust
// Log a message to the console
host::log(level: &str, message: &str);

// Get temporary directory path
host::get_temp_dir() -> Result<String, String>;
```

## Implementation Guide

### Example Implementation

Here's a complete example component that doubles a number:

```rust
use wasmflow::node::*;

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

    fn execute(inputs: Vec<(String, Value)>)
        -> Result<Vec<(String, Value)>, ExecutionError>
    {
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

// Export the component
wasmflow::node::export!(DoubleNumber);
```

### Supported Data Types

- `DataType::U32Type` - Unsigned 32-bit integer
- `DataType::I32Type` - Signed 32-bit integer
- `DataType::F32Type` - 32-bit floating point (use `f32` in WIT, not `float32`)
- `DataType::StringType` - UTF-8 text string
- `DataType::BinaryType` - Raw binary data
- `DataType::ListType(inner)` - Homogeneous list
- `DataType::RecordType(fields)` - Structured data
- `DataType::AnyType` - Accepts any type

**Note**: The WIT specification now uses `f32` instead of the deprecated `float32` type name.

### Error Handling

Provide helpful error messages:

```rust
Err(ExecutionError {
    message: "Division by zero".to_string(),
    input_name: Some("divisor".to_string()),
    recovery_hint: Some("Ensure divisor is not zero".to_string()),
})
```

## Building and Testing

### Build for Development

```bash
cargo component build --target wasm32-wasip2
```

### Build for Release

```bash
cargo component build --target wasm32-wasip2 --release
```

**Note**: The `--target wasm32-wasip2` flag ensures you're using WASI Preview 2, which WasmFlow requires.

### Test Component in Isolation

Create `tests/component_tests.rs`:

```rust
#[test]
fn test_double_component() {
    let inputs = vec![("input".to_string(), Value::F32Val(5.0))];
    let result = DoubleNumber::execute(inputs).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "output");
    match result[0].1 {
        Value::F32Val(v) => assert_eq!(v, 10.0),
        _ => panic!("Wrong output type"),
    }
}
```

Run tests:

```bash
cargo test
```

### Optimize Build Size

Add to `Cargo.toml`:

```toml
[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
strip = true         # Strip symbols
codegen-units = 1    # Better optimization
panic = "abort"      # Smaller panic handler
```

### Set Default Target

Create `.cargo/config.toml` in your component directory:

```toml
[build]
target = "wasm32-wasip2"

[target.wasm32-wasip2]
rustflags = ["-C", "target-feature=+bulk-memory,+mutable-globals"]
```

This allows you to build with just `cargo component build` without specifying `--target` each time.

## Loading Components

### Method 1: Manual Copy

```bash
# Build the component
cargo component build --release

# Copy to WasmFlow components directory
cp target/wasm32-wasip2/release/my_component.wasm \
   /path/to/wasmflow/components/
```

In WasmFlow: **File → Reload Components**

### Method 2: Load Component Dialog

In WasmFlow:
1. **File → Load Component...**
2. Select your `.wasm` file
3. Component appears in palette

## Capabilities and Permissions

If your component needs system access, declare capabilities:

```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "file-read:/data".to_string(),    // Read from /data
        "file-write:/output".to_string(), // Write to /output
        "network:api.example.com".to_string(), // HTTP to specific host
    ])
}
```

Available capabilities:
- `file-read:<path>` - Read files
- `file-write:<path>` - Write files
- `network:<host>` - HTTP/HTTPS access
- `process` - Execute processes (high risk)
- `env` - Access environment variables
- `time` - Access system time
- `crypto` - Cryptographic random numbers

**Note**: Users must explicitly grant capabilities when loading your component.

## Troubleshooting

### Build Errors

**Error**: `the 'float32' type has been renamed to 'f32'`

**Solution**: Update your WIT file to use `f32` instead of `float32`:
```wit
// Old (deprecated)
variant value {
    f32-val(float32),
}

// New (correct)
variant value {
    f32-val(f32),
}
```

**Error**: `error: linking with 'rust-lld' failed`

**Solution**: Ensure WASM target is installed:
```bash
rustup target add wasm32-wasip2
```

**Error**: `error: failed to parse manifest`

**Solution**: Verify `Cargo.toml` has `crate-type = ["cdylib"]`

### Runtime Errors

**Error**: "Failed to load component: Invalid WASM module"

**Solutions**:
- Build with `cargo component build` (not `cargo build`)
- Verify target is `wasm32-wasip2`
- Check component size (<50MB)

**Error**: "Component validation failed: Component must have .wasm extension"

**Solution**: Rename file to end with `.wasm`

### Type Errors

**Error**: "Type mismatch" when connecting nodes

**Solution**: Ensure port data types match:
- Check input/output specifications
- Verify `DataType` declarations
- Use `DataType::AnyType` for generic nodes

### Performance Issues

**Component takes >30 seconds**

**Solutions**:
- Optimize algorithms (avoid O(n²) operations)
- Use streaming for large data
- Break into smaller components
- Profile with `cargo flamegraph`

## Best Practices

1. **Keep components focused** - One clear responsibility per component
2. **Validate inputs** - Check for invalid values and provide helpful errors
3. **Use semantic versioning** - Increment version on breaking changes
4. **Document thoroughly** - Clear descriptions for all ports
5. **Test extensively** - Unit tests for all edge cases
6. **Minimize dependencies** - Smaller WASM size, faster load times
7. **Log thoughtfully** - Use host::log() for debugging, not spam
8. **Handle errors gracefully** - Provide recovery hints

## Example Components

See the `examples/` directory for reference implementations:

- **double-number** - Basic math operation
- **text-transform** - String manipulation
- **file-reader** - File I/O with capabilities
- **http-fetch** - Network requests

## Resources

- [WIT Specification](https://component-model.bytecodealliance.org/design/wit.html)
- [Component Model Docs](https://component-model.bytecodealliance.org/)
- [Wasmtime Guide](https://docs.wasmtime.dev/)
- [WasmFlow Constitution](../.specify/memory/constitution.md)

## Getting Help

- Check [Troubleshooting](#troubleshooting) section
- Review example components in `examples/`
- File an issue on GitHub
- Join the WasmFlow community

---

Happy component building! 🎉
