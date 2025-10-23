# Double Number Component

A simple example WasmFlow component that multiplies an input number by 2.

## Overview

**Note**: This is a **simplified educational example** that demonstrates the structure and concepts of a WasmFlow component. For a production-ready component, you would need to use the `wit-bindgen!` macro to generate proper type-safe bindings from the WIT interface.

This example demonstrates:
- Basic WIT interface implementation
- Input/output port specification
- Simple mathematical computation
- Error handling with helpful messages
- Host function usage (commented for reference)
- Component structure and organization

## Building

### Prerequisites

```bash
# Install Rust and WASM target
rustup target add wasm32-wasip2

# Install cargo-component
cargo install cargo-component
```

### Build Commands

```bash
# Development build
cargo component build --target wasm32-wasip2

# Release build (optimized)
cargo component build --target wasm32-wasip2 --release
```

### Output Location

The compiled component will be at:
```
target/wasm32-wasip2/release/double_number.wasm
```

**Important**: Use the `--target wasm32-wasip2` flag to build for WASI Preview 2, which is what WasmFlow expects.

## Using in WasmFlow

### 1. Install the Component

```bash
# Copy to WasmFlow components directory
cp target/wasm32-wasip2/release/double_number.wasm \
   /path/to/wasmflow/components/
```

### 2. Load in WasmFlow

In WasmFlow application:
1. **File → Reload Components**
2. Find "Double" in the palette under "Math" category
3. Drag onto canvas

### 3. Test the Component

Create a simple graph:
```
Constant(7) → Double → [Output shows 14.0]
```

## Component Details

### Inputs

- **input** (f32, required): Number to double

### Outputs

- **output** (f32): Doubled number (input × 2)

### Capabilities

None - this component performs pure computation with no system access.

### Example Usage

```rust
// Input: 5.0
// Output: 10.0

// Input: -3.5
// Output: -7.0

// Input: 0.0
// Output: 0.0
```

## Implementation Notes

This is a **simplified demonstration** of the component structure. In a production environment:

1. **Use wit-bindgen macro** to generate type-safe bindings:
   ```rust
   wit_bindgen::generate!({
       world: "component",
       exports: {
           world: Component
       }
   });
   ```

2. **Proper WIT imports** from wasmflow package:
   ```wit
   import wasmflow:node/types@1.0.0;
   import wasmflow:node/host@1.0.0;
   ```

3. **Host function integration**:
   ```rust
   // Use imported host functions
   host::log("info", "Processing...");
   ```

## Extending This Example

### Add More Inputs

```rust
fn get_inputs() -> Vec<PortSpec> {
    vec![
        PortSpec {
            name: "input".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Number to double".to_string(),
        },
        PortSpec {
            name: "multiplier".to_string(),
            data_type: DataType::F32Type,
            optional: true,
            description: "Custom multiplier (default: 2.0)".to_string(),
        },
    ]
}
```

### Add Configuration

```rust
fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
    let input_value = extract_f32(&inputs, "input")?;
    let multiplier = extract_optional_f32(&inputs, "multiplier").unwrap_or(2.0);

    let result = input_value * multiplier;

    Ok(vec![("output".to_string(), Value::F32Val(result))])
}
```

### Add Validation

```rust
fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
    let input_value = extract_f32(&inputs, "input")?;

    // Validate range
    if input_value.abs() > 1e6 {
        return Err(ExecutionError {
            message: "Input value too large".to_string(),
            input_name: Some("input".to_string()),
            recovery_hint: Some("Use a value between -1,000,000 and 1,000,000".to_string()),
        });
    }

    let result = input_value * 2.0;

    Ok(vec![("output".to_string(), Value::F32Val(result))])
}
```

## Testing

### Unit Tests

Create `tests/lib_test.rs`:

```rust
use double_number::*;

#[test]
fn test_positive_number() {
    let inputs = vec![("input".to_string(), Value::F32Val(5.0))];
    let result = DoubleNumber::execute(inputs).unwrap();

    assert_eq!(result[0].0, "output");
    match result[0].1 {
        Value::F32Val(v) => assert_eq!(v, 10.0),
        _ => panic!("Wrong type"),
    }
}

#[test]
fn test_negative_number() {
    let inputs = vec![("input".to_string(), Value::F32Val(-3.5))];
    let result = DoubleNumber::execute(inputs).unwrap();

    match result[0].1 {
        Value::F32Val(v) => assert_eq!(v, -7.0),
        _ => panic!("Wrong type"),
    }
}

#[test]
fn test_missing_input() {
    let inputs = vec![];
    let result = DoubleNumber::execute(inputs);

    assert!(result.is_err());
}
```

Run tests:
```bash
cargo test
```

## Troubleshooting

### Component doesn't load

- **Check file extension**: Must be `.wasm`
- **Verify build target**: Should be `wasm32-wasip2`
- **Check file size**: Must be <50MB

### Type errors in WasmFlow

- **Ensure f32 input**: Connect from another f32 output
- **Check port names**: Input port must be named "input"

### Build fails

```bash
# Clean and rebuild
cargo clean
cargo component build --release
```

## Further Reading

- [Building Components Guide](../../docs/BUILDING_COMPONENTS.md)
- [WIT Specification](https://component-model.bytecodealliance.org/design/wit.html)
- [WasmFlow Quickstart](../../specs/001-webassembly-based-node/quickstart.md)

## License

This example is provided as-is for educational purposes.
