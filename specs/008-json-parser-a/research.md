# Research: JSON Parser Node

**Feature**: JSON Parser Node | **Date**: 2025-10-22

## Research Questions Resolved

### 1. JSON Parsing Library for WASM Components

**Decision**: Use `serde_json` crate

**Rationale**:
- Industry-standard JSON library in Rust ecosystem with excellent WASM support
- Zero-copy parsing capabilities for performance
- Robust error handling with detailed error messages
- Full JSON type preservation (strings, numbers, booleans, objects, arrays, null)
- Well-tested and maintained by the Rust community
- Already used in wasmflow_cc ecosystem (see `src/node/value.rs`)

**Alternatives Considered**:
- `json` crate: Less feature-rich, smaller community
- `simd-json`: Better performance but requires unsafe code and SIMD support (not guaranteed in WASM)
- Custom parser: Unnecessary complexity for standard JSON parsing

### 2. Key Path Parsing Strategy

**Decision**: Implement custom key path parser with tokenization

**Rationale**:
- Simple grammar: identifiers, dots (`.`), and brackets (`[n]`)
- Regex-based tokenizer can handle: `metadata.runs[1].time`
- State machine approach for unambiguous parsing
- Clear error messages for malformed paths (e.g., `runs[abc]`, `metadata..author`)
- ~100 LOC for parser, highly testable

**Alternatives Considered**:
- JSONPath library: Overly complex for simple dot/bracket notation, adds dependencies
- String splitting: Insufficient for handling mixed notation like `runs[1].time`
- External DSL: Unnecessary complexity for well-defined simple syntax

**Implementation Approach**:
```rust
// Tokenize: "runs[1].time" -> [Ident("runs"), Index(1), Ident("time")]
enum Token {
    Ident(String),  // Property name
    Index(usize),   // Array index
}

// Parse token stream and traverse JSON value
fn extract_value(json: &serde_json::Value, tokens: &[Token]) -> Result<Value, Error>
```

### 3. Type Representation for Extracted Values

**Decision**: Use `serde_json::Value` internally, convert to wasmflow `NodeValue` for output

**Rationale**:
- `serde_json::Value` preserves all JSON types (String, Number, Bool, Object, Array, Null)
- Wasmflow's `NodeValue` already has variants for JSON-compatible types
- Conversion layer maps JSON types to node graph types
- Objects and arrays serialized as JSON strings for output (allows further processing)

**Type Mapping**:
```rust
serde_json::Value::String -> NodeValue::String
serde_json::Value::Number -> NodeValue::Number (f64)
serde_json::Value::Bool -> NodeValue::Bool
serde_json::Value::Object -> NodeValue::String (serialized JSON)
serde_json::Value::Array -> NodeValue::String (serialized JSON)
serde_json::Value::Null -> NodeValue::Null (if exists, or Option::None)
```

**Alternatives Considered**:
- Custom JSON type enum: Reinvents the wheel, duplicates serde_json functionality
- Always return strings: Loses type information, forces downstream nodes to parse
- Structured NodeValue with nested records: Too complex for v1, can be future enhancement

### 4. Error Handling Strategy

**Decision**: Use Rust `Result<T, JsonParserError>` with detailed error variants

**Rationale**:
- Functional Requirements FR-008 through FR-011 require distinct error cases
- Enum-based errors enable pattern matching and specific error messages
- Error context includes location information (path segment where failure occurred)

**Error Types**:
```rust
enum JsonParserError {
    InvalidJson(String),           // FR-008: JSON parsing failed
    PathNotFound(String),          // FR-009: Key path doesn't exist
    MalformedPath(String),         // FR-010: Invalid path syntax
    IndexOutOfBounds(usize, usize), // FR-011: Array index too large
    TypeMismatch(String),          // Tried to index non-array or access property on primitive
}
```

**Alternatives Considered**:
- Generic error strings: Poor user experience, hard to debug
- Result<Option<Value>, Error>: Confusing semantics (null vs missing vs error)
- Panic on errors: Violates WASM component best practices

### 5. WIT Interface Design

**Decision**: Define clear input/output ports with variant type for values

**Rationale**:
- WIT supports variant types for representing multiple possible return types
- Aligns with WASI Component Model best practices
- Type-safe interface enforced at compile time

**WIT Interface** (draft):
```wit
package wasmflow:json-parser@0.1.0;

interface json-parser {
    // JSON value variants
    variant json-value {
        string(string),
        number(float64),
        boolean(bool),
        object(string),   // Serialized JSON
        array(string),    // Serialized JSON
        null,
    }

    // Error information
    record parse-error {
        message: string,
        kind: error-kind,
    }

    enum error-kind {
        invalid-json,
        path-not-found,
        malformed-path,
        index-out-of-bounds,
        type-mismatch,
    }

    // Main extraction function
    parse: func(json-string: string, key-path: string) -> result<json-value, parse-error>;
}
```

**Alternatives Considered**:
- Separate functions for each error type: Too many exports, clutters interface
- String-only returns: Loses type information and structure
- Throwing exceptions: Not idiomatic in WIT/WASM

### 6. Performance Optimization Strategy

**Decision**: Lazy parsing with single-pass traversal

**Rationale**:
- Only parse JSON once using `serde_json::from_str`
- Traverse parsed structure following key path tokens
- No need to build intermediate structures
- Target: 1MB JSON in <100ms easily achievable with serde_json

**Performance Characteristics**:
- Parsing: O(n) where n = JSON string length
- Traversal: O(k) where k = key path depth
- Memory: O(n) for parsed JSON structure
- Expected: <10ms for typical payloads (<10KB)

**Alternatives Considered**:
- Streaming parser: Unnecessary complexity for in-memory JSON
- Cache parsed JSON: Stateless component design prevents caching
- Lazy JSON parsing (json-rust): Marginal gains, less robust error handling

### 7. Testing Strategy

**Decision**: Three-tier testing approach

**Rationale**:
- Unit tests: Key path parser logic (tokenization, validation)
- Contract tests: WIT interface compliance
- Integration tests: End-to-end node graph scenarios

**Test Coverage**:

**Unit Tests** (`tests/unit/json_parser_unit.rs`):
- Key path tokenization (20+ test cases for valid/invalid paths)
- Value extraction for each JSON type
- Error case handling (all error variants)
- Edge cases: empty paths, deep nesting, large arrays

**Contract Tests** (`tests/contract/json_parser_test.rs`):
- WIT interface validation
- Type conversion correctness
- Error serialization

**Integration Tests** (`tests/integration/json_parser_graph.rs`):
- Node graph integration
- Chaining multiple JSON parser nodes
- Connection type validation
- Real-world JSON examples

**Benchmark Tests**:
- 1KB JSON parsing
- 1MB JSON parsing
- 100-level deep nesting
- 1000-element array access

**Alternatives Considered**:
- Only integration tests: Insufficient granularity for debugging failures
- Fuzz testing: Valuable but deferred to post-v1 hardening
- Property-based testing: Valuable but adds complexity for v1

## Dependencies Confirmed

### Primary Dependencies
- `serde = { version = "1.0", features = ["derive"] }` - Serialization framework
- `serde_json = "1.0"` - JSON parsing and manipulation
- `wit-bindgen = "0.30"` - WIT bindings generation
- `thiserror = "1.0"` - Error type derivation

### Development Dependencies
- `cargo-component = "0.13"` - WASM component build tooling
- `criterion = "0.5"` - Performance benchmarking

### System Dependencies
- Rust 1.75+ with `wasm32-wasip2` target installed
- wasmtime 27.0+ for component model runtime testing

## Best Practices Applied

### WASM Component Model
- Follow cargo-component project structure
- Use `wit-bindgen` for automatic binding generation
- Export single well-defined interface
- No global state (pure functions only)
- Explicit error handling via Result types

### Rust Best Practices
- Use `thiserror` for ergonomic error types
- Leverage `serde_json::Value` for JSON manipulation
- Avoid `unsafe` code (not needed for this component)
- Use `#[cfg(test)]` for test-only code
- Comprehensive error messages with context

### Testing Best Practices
- Test all acceptance scenarios from spec.md
- Cover all edge cases listed in spec.md
- Use table-driven tests for key path parsing
- Benchmark performance against success criteria (SC-006)
- Integration tests validate WIT contract compliance

### Documentation Best Practices
- Inline code documentation for public APIs
- Examples in quickstart.md for common use cases
- Error message guidance (what went wrong, how to fix)
- WIT interface documentation with comments

## Open Questions / Future Enhancements

### Post-V1 Considerations (out of scope for current feature)
1. **Escaped property names**: How to access keys containing dots or brackets?
   - Potential: Escape syntax like `metadata["author.name"]`
   - Deferred: Rare use case, adds parser complexity

2. **JSONPath support**: Full JSONPath query language
   - Potential: Add separate JSONPath parser node
   - Deferred: Significant scope expansion, separate feature

3. **Schema validation**: Validate JSON against JSON Schema
   - Potential: Separate schema validator node
   - Deferred: Different use case, orthogonal to value extraction

4. **Streaming JSON**: Support for very large JSON files
   - Potential: Streaming parser for multi-GB payloads
   - Deferred: Out of scope for 1MB limit (SC-006)

5. **Performance optimization**: SIMD JSON parsing
   - Potential: Use simd-json for 2-3x speed improvement
   - Deferred: Current performance target easily met with serde_json

## Risk Assessment

### Technical Risks

**Risk**: Key path parser bugs (malformed paths, edge cases)
- **Mitigation**: Comprehensive unit tests (20+ test cases), property-based testing
- **Impact**: Medium (functional errors)
- **Likelihood**: Low (simple grammar, well-tested)

**Risk**: Performance degradation on large JSON
- **Mitigation**: Benchmark tests for 1MB payloads, performance regression tests
- **Impact**: Medium (fails SC-006)
- **Likelihood**: Low (serde_json is well-optimized)

**Risk**: Type conversion errors (JSON to NodeValue)
- **Mitigation**: Contract tests for all type conversions, integration tests
- **Impact**: High (incorrect results)
- **Likelihood**: Low (straightforward mapping)

### Integration Risks

**Risk**: WIT interface changes in wasmtime/component-model
- **Mitigation**: Pin wasmtime version, follow component-model updates
- **Impact**: High (breaking changes)
- **Likelihood**: Low (stable APIs in wasmtime 27.0)

**Risk**: NodeValue type incompatibility
- **Mitigation**: Review existing NodeValue implementation, add variants if needed
- **Impact**: Medium (requires NodeValue refactor)
- **Likelihood**: Low (NodeValue already supports JSON types)

## Summary

All research questions resolved with clear technical decisions. No blocking unknowns remain. Component design aligns with constitution principles and follows established patterns in the wasmflow_cc codebase. Ready to proceed to Phase 1 (Design & Contracts).
