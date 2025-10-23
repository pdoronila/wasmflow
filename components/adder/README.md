# Example Adder Component

A basic WasmFlow component that adds two numbers together.

## Overview

This component demonstrates:
- **Pure computation** (no special capabilities required)
- **Multiple inputs** (two F32 values)
- **Single output** (sum of inputs)
- **Error handling** with helpful recovery hints
- **Logging** for debugging

## Ports

### Inputs
- **a** (F32): First number to add
- **b** (F32): Second number to add

### Outputs
- **sum** (F32): The sum of a and b

## Capabilities

None required - this is a pure computation component.

## Building

```bash
cd examples/example-adder
cargo component build --release
cp target/wasm32-wasip1/release/example_adder.wasm ../../components/
```

## Usage in WasmFlow

1. **Load the component**: File → Reload Components
2. **Add to canvas**: Find "Add Numbers" in the Math category
3. **Connect inputs**: Connect F32 values to both 'a' and 'b' ports
4. **Execute**: Click Execute to run the graph
5. **View result**: The sum appears on the 'sum' output port

## Example Graph

```
[Constant: 5.0] → [a]
                      [Add Numbers] → [sum] → [Display]
[Constant: 3.0] → [b]

Result: 8.0
```

## Source Code

See `src/lib.rs` for the complete implementation.

Key features:
- Input validation with clear error messages
- Logging of operation details
- Proper WIT interface implementation
