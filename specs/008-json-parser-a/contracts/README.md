# JSON Parser Component Contracts

This directory contains the interface contracts for the JSON Parser Node component.

## Files

### json-parser.wit

The WIT (WebAssembly Interface Type) definition for the JSON parser component. This file defines:

- **Input Parameters**:
  - `json-string: string` - JSON payload to parse
  - `key-path: string` - Path specification using dot/bracket notation

- **Output Types**:
  - `json-value` variant - Extracted value with type preservation
  - `parse-error` record - Structured error information

- **Function Signature**:
  ```wit
  parse: func(json-string: string, key-path: string) -> result<json-value, parse-error>;
  ```

## Usage

### Building the Component

```bash
# From the component directory
cargo component build --release
```

### Using wit-bindgen

The WIT interface will be used to generate Rust bindings:

```bash
wit-bindgen rust json-parser.wit
```

This generates Rust types and trait definitions that the component implementation must satisfy.

### Type Mappings

| WIT Type | Rust Type | JSON Type |
|----------|-----------|-----------|
| `json-value::string` | `String` | String |
| `json-value::number` | `f64` | Number |
| `json-value::boolean` | `bool` | Boolean |
| `json-value::object` | `String` (JSON) | Object |
| `json-value::array` | `String` (JSON) | Array |
| `json-value::null` | Unit variant | null |

### Error Handling

All errors are returned via the `parse-error` record:

```rust
ParseError {
    message: "Human-readable error description",
    kind: ErrorKind::PathNotFound,  // Enum variant
    context: Some("Additional context"),  // Optional details
}
```

## Contract Tests

Contract tests validate that the implementation satisfies the WIT interface:

- Located in: `tests/contract/json_parser_test.rs`
- Validate type conversions
- Ensure error serialization correctness
- Verify WIT compliance

## Validation

The WIT file can be validated using:

```bash
# Install wasm-tools if needed
cargo install wasm-tools

# Validate WIT syntax
wasm-tools component wit json-parser.wit
```

## Integration

This component integrates with wasmflow_cc via:

1. Component is compiled to WASM with component-model support
2. WIT interface is registered in the node graph system
3. Runtime instantiates component with wasmtime
4. Node inputs/outputs map to WIT function parameters/results

See `quickstart.md` for complete integration guide.
