# WasmFlow Components Directory

This directory contains custom WebAssembly components that extend WasmFlow's capabilities.

## Overview

Place compiled `.wasm` component files here to make them available in WasmFlow. Components are loaded automatically when WasmFlow starts, or manually via **File → Reload Components**.

## Quick Start

### 1. Build a Component

```bash
cd examples/double-number
cargo component build --release
```

### 2. Copy to Components Directory

```bash
cp target/wasm32-wasip2/release/double_number.wasm \
   /path/to/wasmflow/components/
```

### 3. Load in WasmFlow

- **Automatic**: Restart WasmFlow (components load on startup)
- **Manual**: In WasmFlow, select **File → Reload Components**

### 4. Use the Component

Your component appears in the node palette under its category (e.g., "Math", "Text").

## Directory Structure

```
components/
├── README.md              (this file)
├── double_number.wasm     (example: doubles a number)
├── text_transform.wasm    (example: string operations)
└── your_component.wasm    (your custom component)
```

## Component Requirements

All `.wasm` files in this directory must:

1. **Be valid WebAssembly components** built with the Component Model
2. **Export the wasmflow:node interface** (metadata + execution)
3. **Be compiled for wasm32-wasip2** target
4. **Be under 50MB** in size
5. **Have a `.wasm` extension**

## Creating Components

See the comprehensive guide: [Building Components](../docs/BUILDING_COMPONENTS.md)

Quick overview:

```bash
# Install prerequisites
rustup target add wasm32-wasip2
cargo install cargo-component

# Create new component
cargo component new my-component --lib

# Implement the WIT interface (see examples/)
# ...

# Build
cargo component build --release

# Copy here
cp target/wasm32-wasip2/release/my_component.wasm .
```

## Example Components

The `examples/` directory contains reference implementations:

- **double-number** - Basic math operation
  - Demonstrates: Simple computation, error handling, logging
  - Category: Math
  - Inputs: f32
  - Outputs: f32

More examples coming soon!

## Component Validation

When loading, WasmFlow validates:

- ✅ File extension is `.wasm`
- ✅ File size is <50MB
- ✅ WASM module is valid
- ✅ Component exports required interfaces
- ✅ WIT interface matches expected schema

Failed validation shows detailed error messages in the status bar.

## Security and Capabilities

Components declare required system access via capabilities:

```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "file-read:/data".to_string(),
        "network:api.example.com".to_string(),
    ])
}
```

Users must **explicitly approve** capabilities when loading components with system access.

### Available Capabilities

| Capability | Description | Risk |
|------------|-------------|------|
| `file-read:<path>` | Read files from path | Medium |
| `file-write:<path>` | Write files to path | High |
| `network:<host>` | HTTP/HTTPS to host | Medium |
| `process` | Execute processes | High |
| `env` | Access environment vars | Medium |
| `time` | Access system time | Low |
| `crypto` | Crypto random numbers | Low |

Components with **no capabilities** (pure computation) load without permission prompts.

## Troubleshooting

### Component doesn't appear in palette

**Check**:
1. File extension is exactly `.wasm` (not `.wasm.bak`, etc.)
2. File is in this directory (not a subdirectory)
3. Run **File → Reload Components** in WasmFlow
4. Check status bar for error messages

**Debug**:
```bash
# Verify file
ls -lh *.wasm

# Check size (<50MB)
du -h your_component.wasm

# Test with wasmtime
wasmtime your_component.wasm
```

### Component loads but execution fails

**Check**:
1. Component built with `cargo component build` (not `cargo build`)
2. Target is `wasm32-wasip2` (not `wasm32-unknown-unknown`)
3. WIT interface matches WasmFlow expectations
4. All required capabilities declared

**Debug**:
Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

### Type errors when connecting nodes

**Check**:
1. Input/output data types match (`DataType::F32Type`, etc.)
2. Port names are correct and unique
3. Optional ports are marked as `optional: true`

**Fix**: Review port specifications in your component:
```rust
fn get_inputs() -> Vec<PortSpec> {
    vec![PortSpec {
        name: "input".to_string(),
        data_type: DataType::F32Type,  // Must match connected output
        optional: false,
        description: "Clear description".to_string(),
    }]
}
```

### Performance issues

**Optimize**:
1. Build with `--release` flag
2. Add optimization flags to `Cargo.toml`:
   ```toml
   [profile.release]
   opt-level = "s"
   lto = true
   strip = true
   ```
3. Minimize dependencies
4. Profile with `cargo flamegraph`

**Note**: Components timeout after 30 seconds. Optimize algorithms for large datasets.

## Component Development Workflow

1. **Create** component from template
   ```bash
   cargo component new my-component --lib
   ```

2. **Implement** WIT interface
   - Define inputs/outputs
   - Implement execute() logic
   - Add error handling

3. **Test** in isolation
   ```bash
   cargo test
   ```

4. **Build** for release
   ```bash
   cargo component build --release
   ```

5. **Copy** to this directory
   ```bash
   cp target/wasm32-wasip2/release/my_component.wasm .
   ```

6. **Load** in WasmFlow
   - File → Reload Components

7. **Test** in a graph
   - Create test graph
   - Connect nodes
   - Execute
   - Verify output

8. **Iterate** on feedback

## Best Practices

- **Name components descriptively**: `text_uppercase.wasm`, not `component1.wasm`
- **Use semantic versioning**: Update version in component metadata
- **Document thoroughly**: Clear descriptions for all ports
- **Handle errors gracefully**: Provide helpful recovery hints
- **Test edge cases**: Null inputs, large values, empty strings
- **Minimize size**: Remove unused dependencies
- **Respect timeouts**: Optimize for <30 second execution
- **Log thoughtfully**: Debug logs help, but don't spam

## Resources

- [Building Components Guide](../docs/BUILDING_COMPONENTS.md) - Complete tutorial
- [Example Components](../examples/) - Reference implementations
- [WIT Specification](https://component-model.bytecodealliance.org/design/wit.html)
- [Wasmtime Documentation](https://docs.wasmtime.dev/)

## Getting Help

- **Check logs**: `RUST_LOG=debug cargo run`
- **Review examples**: See `examples/double-number/`
- **Read docs**: See `docs/BUILDING_COMPONENTS.md`
- **File issues**: GitHub issue tracker

---

**Ready to build?** Start with the [double-number example](../examples/double-number/README.md)!
