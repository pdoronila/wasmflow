# Echo Component

A simple pass-through WASM component for WasmFlow that accepts any type variant as input and outputs it unchanged.

## Description

The Echo component is a utility component that takes a single input of any type (u32, i32, f32, string, or binary) and passes it through to its output unchanged. This is useful for:

- Testing data flow in graphs
- Creating pass-through nodes in complex workflows
- Debugging value propagation
- Demonstrating the AnyType port capability

## Ports

### Input Ports

- **input** (AnyType, required): Value to echo
  - Accepts any supported data type (u32, i32, f32, string, binary)

### Output Ports

- **output** (AnyType, required): Echoed value
  - Returns the exact same value that was received on the input port

## Metadata

- **Name**: Echo
- **Version**: 1.0.0
- **Category**: Utility
- **Capabilities**: None (pure computation, no system access)

## Building

### Prerequisites

```bash
# Install the wasm32-wasip2 target
rustup target add wasm32-wasip2
```

### Build Commands

```bash
# Using just
just build          # Build the component
just install        # Build and install to ../bin/
just check          # Check without building
just clean          # Clean build artifacts

# Using cargo directly
cargo build --target wasm32-wasip2 --release
```

The compiled `.wasm` file will be located at:
`target/wasm32-wasip2/release/echo.wasm`

## Installation

After building, copy the `.wasm` file to the WasmFlow components directory:

```bash
cp target/wasm32-wasip2/release/echo.wasm ../bin/
```

Or use the just command:

```bash
just install
```

## Usage in WasmFlow

1. Build and install the component
2. In WasmFlow UI, go to File → Reload Components
3. Add an Echo node to your graph
4. Connect any value to the "input" port
5. The same value will appear on the "output" port

## Example Use Cases

### Simple Pass-Through
```
[Constant: 42] → [Echo] → [Display]
```

### Type Testing
```
[String: "hello"] → [Echo] → [JSON Parser]
```

### Data Flow Debugging
```
[Complex Node] → [Echo] → [Another Node]
                    ↓
              [Debug Output]
```

## Implementation Details

- Uses `wit-bindgen` 0.30 for WIT bindings
- Pure computation component (no filesystem, network, or system access)
- Zero-copy pass-through of values
- Logs value type on each execution
- Comprehensive error handling with recovery hints

## Files

- `src/lib.rs` - Main component implementation
- `wit/node.wit` - WIT interface definition
- `Cargo.toml` - Rust package configuration
- `build.rs` - Build script
- `Justfile` - Build automation
- `README.md` - This file

## License

Same as parent WasmFlow project.
