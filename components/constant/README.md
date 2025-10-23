# Constant Component

A WebAssembly component that outputs a constant value.

## Description

This component provides a configurable constant output. The value and type can be set in the UI:
- Type selection: U32, I32, F32, String, or Binary
- Value input: Set the constant value to output

## Inputs

None - this component has no inputs.

## Outputs

- `value` (Any Type): The configured constant value

## Configuration

The component is configured through the UI with:
1. A pulldown menu to select the output type (U32, I32, F32, String, Binary)
2. An input field to set the constant value

## Use Cases

- Provide test values for debugging
- Supply default or initial values to other nodes
- Create parameterized graphs with configurable constants

## Building

```bash
just build
```

## Installation

```bash
just install
```

This copies the built WASM file to `../bin/constant.wasm` where it can be loaded by WasmFlow.

## Example

1. Add a Constant component to your graph
2. Select "F32" from the type dropdown
3. Enter "3.14159" in the value field
4. Connect the output to other nodes that need this constant value
