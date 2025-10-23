# Quickstart Guide: JSON Parser Node

**Last Updated**: 2025-10-22 | **Component Version**: 0.1.0

## Overview

The JSON Parser Node is a WASM component that extracts values from JSON strings using key path notation. This guide walks through common usage patterns and integration with wasmflow_cc.

## Prerequisites

- Rust 1.75+ with `wasm32-wasip2` target installed
- cargo-component 0.13+
- wasmtime 27.0+ (for testing)
- wasmflow_cc project

### Install Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm32-wasip2 target
rustup target add wasm32-wasip2

# Install cargo-component
cargo install cargo-component --version 0.13.0

# Install wasm-tools (for WIT validation)
cargo install wasm-tools
```

---

## Quick Examples

### Example 1: Extract Simple Property

**Use Case**: Extract a version number from configuration JSON

**Input JSON**:
```json
{
  "version": 1,
  "name": "my-app"
}
```

**Node Configuration**:
- Input Port `json-string`: `"{\"version\": 1, \"name\": \"my-app\"}"`
- Input Port `key-path`: `"version"`

**Output**:
- Output Port `value`: `json-value::number(1.0)`
- Output Port `error`: `None`

**Graph Visualization**:
```
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│ Config Source│─────▶│ JSON Parser  │─────▶│ Display Node │
│  (JSON data) │      │ path="version"│     │  (shows 1.0) │
└──────────────┘      └──────────────┘      └──────────────┘
```

---

### Example 2: Extract Nested Property

**Use Case**: Extract author name from metadata

**Input JSON**:
```json
{
  "version": 1,
  "metadata": {
    "author": "me",
    "created": "2025-10-22"
  }
}
```

**Node Configuration**:
- Input Port `json-string`: `"{\"version\": 1, \"metadata\": {\"author\": \"me\", \"created\": \"2025-10-22\"}}"`
- Input Port `key-path`: `"metadata.author"`

**Output**:
- Output Port `value`: `json-value::string("me")`
- Output Port `error`: `None`

**Graph Visualization**:
```
┌──────────────┐      ┌──────────────────┐      ┌──────────────┐
│ API Response │─────▶│   JSON Parser    │─────▶│ String Node  │
│  (JSON data) │      │path="metadata.   │     │ (shows "me") │
│              │      │      author"     │     │              │
└──────────────┘      └──────────────────┘      └──────────────┘
```

---

### Example 3: Extract Array Element

**Use Case**: Get the second test run from a results array

**Input JSON**:
```json
{
  "runs": [
    {"id": 1, "time": 100},
    {"id": 2, "time": 1000}
  ]
}
```

**Node Configuration**:
- Input Port `json-string`: `"{\"runs\": [{\"id\": 1, \"time\": 100}, {\"id\": 2, \"time\": 1000}]}"`
- Input Port `key-path`: `"runs[1]"`

**Output**:
- Output Port `value`: `json-value::object("{\"id\": 2, \"time\": 1000}")`
- Output Port `error`: `None`

**Note**: Output is an object (serialized as JSON string). To extract the `time` field, use path `"runs[1].time"` instead.

---

### Example 4: Extract Property from Array Element

**Use Case**: Get execution time of the second test run

**Input JSON**:
```json
{
  "runs": [
    {"id": 1, "time": 100},
    {"id": 2, "time": 1000}
  ]
}
```

**Node Configuration**:
- Input Port `json-string`: `"{\"runs\": [{\"id\": 1, \"time\": 100}, {\"id\": 2, \"time\": 1000}]}"`
- Input Port `key-path`: `"runs[1].time"`

**Output**:
- Output Port `value`: `json-value::number(1000.0)`
- Output Port `error`: `None`

**Graph Visualization**:
```
┌──────────────┐      ┌──────────────────┐      ┌──────────────┐
│ Test Results │─────▶│   JSON Parser    │─────▶│ Number Node  │
│  (JSON data) │      │path="runs[1].    │     │ (shows 1000) │
│              │      │      time"       │     │              │
└──────────────┘      └──────────────────┘      └──────────────┘
```

---

### Example 5: Chaining Multiple Parsers

**Use Case**: Extract multiple values from the same JSON source

**Input JSON**:
```json
{
  "version": 1,
  "metadata": {
    "author": "me"
  },
  "runs": [
    {"id": 1, "time": 100},
    {"id": 2, "time": 1000}
  ]
}
```

**Graph Configuration**:
```
                        ┌──────────────────┐
                    ┌──▶│  JSON Parser 1   │───▶ (version: 1.0)
                    │   │ path="version"   │
┌──────────────┐    │   └──────────────────┘
│              │    │
│ JSON Source  │────┤   ┌──────────────────┐
│  (shared)    │    ├──▶│  JSON Parser 2   │───▶ (author: "me")
│              │    │   │path="metadata.   │
└──────────────┘    │   │     author"      │
                    │   └──────────────────┘
                    │
                    │   ┌──────────────────┐
                    └──▶│  JSON Parser 3   │───▶ (time: 1000.0)
                        │path="runs[1].    │
                        │      time"       │
                        └──────────────────┘
```

**Benefit**: Reuse the same JSON source node, extract different values with different paths.

---

## Error Handling

### Example 6: Invalid JSON

**Input**:
- `json-string`: `"{invalid json"`
- `key-path`: `"version"`

**Output**:
- `value`: `None`
- `error`: `Some(parse-error { message: "Failed to parse JSON: expected `,` at line 1 column 9", kind: invalid-json, context: None })`

**Handling in Graph**:
```
┌──────────────┐      ┌──────────────┐
│ JSON Source  │─────▶│ JSON Parser  │
└──────────────┘      └───────┬──────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
                 value               error
                    │                   │
                    ▼                   ▼
            ┌──────────────┐    ┌──────────────┐
            │ Success Path │    │  Error Path  │
            │  (if value)  │    │  (if error)  │
            └──────────────┘    └──────────────┘
```

---

### Example 7: Path Not Found

**Input**:
- `json-string`: `"{\"version\": 1}"`
- `key-path`: `"nonexistent"`

**Output**:
- `value`: `None`
- `error`: `Some(parse-error { message: "Key 'nonexistent' not found in JSON object", kind: path-not-found, context: Some("at path segment: nonexistent") })`

---

### Example 8: Index Out of Bounds

**Input**:
- `json-string`: `"{\"runs\": [{\"id\": 1}, {\"id\": 2}]}"`
- `key-path`: `"runs[999]"`

**Output**:
- `value`: `None`
- `error`: `Some(parse-error { message: "Array index 999 out of bounds (array length: 2)", kind: index-out-of-bounds, context: Some("at path segment: runs[999]") })`

---

### Example 9: Type Mismatch

**Input**:
- `json-string`: `"{\"version\": 1}"`
- `key-path`: `"version.property"`

**Output**:
- `value`: `None`
- `error`: `Some(parse-error { message: "Cannot access property 'property' on number value", kind: type-mismatch, context: Some("at path segment: version.property") })`

---

## Integration with Wasmflow

### Step 1: Build the Component

```bash
# Navigate to the builtin nodes directory
cd src/builtin

# Build the JSON parser component
cargo component build --release --target wasm32-wasip2

# Output: target/wasm32-wasip2/release/json_parser.wasm
```

### Step 2: Register the Node

In `src/builtin/mod.rs`:

```rust
mod json_parser;

pub fn register_builtin_nodes(registry: &mut NodeRegistry) {
    // ... existing nodes ...
    registry.register("json-parser", json_parser::create_node);
}
```

### Step 3: Use in Graph

```rust
// Create a JSON parser node
let parser = graph.add_node("json-parser");

// Set inputs
graph.set_input(parser, "json-string", json_data);
graph.set_input(parser, "key-path", "metadata.author");

// Execute graph
graph.execute();

// Read output
let value = graph.get_output(parser, "value");
let error = graph.get_output(parser, "error");
```

---

## Key Path Syntax Reference

### Dot Notation (Object Properties)

| Syntax | Description | Example |
|--------|-------------|---------|
| `property` | Top-level property | `"version"` |
| `obj.prop` | Nested property | `"metadata.author"` |
| `a.b.c.d` | Multi-level nesting | `"config.server.db.host"` |

### Bracket Notation (Array Indices)

| Syntax | Description | Example |
|--------|-------------|---------|
| `array[0]` | First element (zero-based) | `"runs[0]"` |
| `array[1]` | Second element | `"runs[1]"` |
| `array[999]` | Element at index 999 | `"items[999]"` |

### Combined Notation

| Syntax | Description | Example |
|--------|-------------|---------|
| `obj.array[0]` | Array in object | `"data.runs[0]"` |
| `array[0].prop` | Property in array element | `"runs[0].time"` |
| `a.b[1].c[2].d` | Complex mixed path | `"data.items[1].values[2].score"` |

### Restrictions (v1)

- No negative indices: `array[-1]` ❌
- No property escaping: Cannot access literal `"author.name"` key ❌
- No wildcards: `users[*].name` ❌
- No JSONPath: `$..author` ❌

---

## Performance Considerations

### Expected Performance

| JSON Size | Depth | Expected Time |
|-----------|-------|---------------|
| <10KB | <5 levels | <10ms |
| 100KB | <10 levels | <50ms |
| 1MB | <10 levels | <100ms |

### Tips for Optimal Performance

1. **Minimize JSON size**: Filter upstream if possible
2. **Use specific paths**: Avoid unnecessary traversal
3. **Reuse parsed JSON**: If extracting multiple values, chain parsers from same source
4. **Profile large payloads**: Use benchmarks for >1MB JSON

### Performance Testing

```bash
# Run benchmarks
cargo bench --bench json_parser_bench

# Test with large JSON
cargo test test_large_json -- --nocapture
```

---

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_extract_simple_property

# Run with output
cargo test -- --nocapture
```

### Contract Tests

```bash
# Validate WIT interface
wasm-tools component wit contracts/json-parser.wit

# Run contract tests
cargo test --test contract_tests
```

### Integration Tests

```bash
# Run graph integration tests
cargo test --test integration_tests
```

---

## Troubleshooting

### Common Issues

**Issue**: "WIT validation failed"
- **Solution**: Check WIT syntax with `wasm-tools component wit json-parser.wit`

**Issue**: "Component not found in registry"
- **Solution**: Ensure `register_builtin_nodes()` includes json-parser

**Issue**: "Type mismatch in node connection"
- **Solution**: Verify output type matches downstream input expectations

**Issue**: "Performance slower than expected"
- **Solution**: Profile with `cargo bench`, check JSON payload size

---

## Advanced Usage

### Dynamic Key Paths

Use a string input node to construct key paths dynamically:

```
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│ Path Builder │─────▶│ JSON Parser  │─────▶│   Result     │
│  (e.g., UI)  │      │  (dynamic)   │     │              │
└──────────────┘      └──────────────┘      └──────────────┘
```

### Conditional Extraction

Use branching logic to handle different JSON structures:

```
┌──────────────┐      ┌──────────────┐
│ JSON Source  │─────▶│ JSON Parser  │
└──────────────┘      └───────┬──────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
                 error                value
                    │                   │
                    ▼                   ▼
            ┌──────────────┐    ┌──────────────┐
            │ Fallback Path│    │ Success Path │
            │ (use default)│    │ (use value)  │
            └──────────────┘    └──────────────┘
```

---

## Next Steps

1. **Read the spec**: See [spec.md](./spec.md) for detailed requirements
2. **Explore data model**: See [data-model.md](./data-model.md) for type definitions
3. **Review WIT interface**: See [contracts/json-parser.wit](./contracts/json-parser.wit)
4. **Check implementation plan**: See [plan.md](./plan.md) for architecture details
5. **Run examples**: Try the examples in this guide with your own JSON data

---

## Support & Contribution

- **Issues**: Report bugs via GitHub issues
- **Feature Requests**: Submit via GitHub discussions
- **Contributing**: See CONTRIBUTING.md for development guidelines

---

## Version History

- **0.1.0** (2025-10-22): Initial release
  - Basic key path parsing (dot and bracket notation)
  - Type preservation for all JSON types
  - Comprehensive error handling
  - WIT component model integration
