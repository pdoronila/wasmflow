# Research: WASM Components Core Library

**Feature**: 010-wasm-components-core
**Date**: 2025-10-23
**Purpose**: Document technology choices, patterns, and design decisions for implementing 35+ core library components

## Overview

This research phase establishes the technical foundation for building a comprehensive core library of WASM components. All decisions leverage the existing wasmflow component architecture, build system, and patterns established by components like json-parser, adder, and echo.

## Technology Stack Decisions

### 1. Component Interface Definition Language

**Decision**: WIT (WebAssembly Interface Types) via wit-bindgen 0.30

**Rationale**:
- Industry standard for WASI Component Model
- Strong typing enforced at component boundaries
- Automatic binding generation for Rust
- Already established in wasmflow (10+ existing components)
- Supported by wasmtime 27.0 runtime

**Alternatives Considered**:
- **Custom IDL**: Rejected - reinventing the wheel, no tooling support
- **JSON schema**: Rejected - runtime type checking only, no compile-time safety
- **Protocol Buffers**: Rejected - not designed for WASM component boundaries

**Implementation Pattern**:
```wit
// Standard interface for all core library components
world component {
    export metadata;
    export execution;
}

interface metadata {
    get-info: func() -> component-info;
    get-inputs: func() -> list<port-spec>;
    get-outputs: func() -> list<port-spec>;
    get-capabilities: func() -> option<list<string>>;
}

interface execution {
    execute: func(inputs: list<input-value>) -> result<list<output-value>, execution-error>;
}
```

### 2. Implementation Language

**Decision**: Rust 1.75+ (stable channel, wasm32-wasip2 target)

**Rationale**:
- Native WASM support with excellent tooling
- Zero-cost abstractions for performance-critical operations
- Memory safety guarantees (crucial for sandboxed execution)
- Existing wasmflow codebase is Rust
- Strong standard library for string, math, and collection operations
- Mature ecosystem for WASM development

**Alternatives Considered**:
- **C/C++**: Rejected - memory safety concerns, manual memory management overhead
- **AssemblyScript**: Rejected - less mature tooling, limited standard library
- **TinyGo**: Rejected - larger binary sizes, GC overhead

**Performance Characteristics**:
- String operations: Native Rust String type, UTF-8 validated
- Math operations: Native WASM f64/f32 instructions (near-native performance)
- List operations: Vec<T> with pre-allocation where size known
- Binary size: 50-150KB per component with optimization flags

### 3. Build System

**Decision**: Just task runner + Nushell scripting

**Rationale**:
- Already established in wasmflow component build process
- Generic Justfile template works for all components (auto-detects component name)
- Nushell provides cross-platform scripting (Linux, macOS, Windows)
- Integrates with cargo for Rust compilation
- Top-level automation for batch operations (build-all, install-all)

**Alternatives Considered**:
- **Make**: Rejected - platform inconsistencies, complex syntax
- **Cargo workspace**: Rejected - tighter coupling between components, harder to isolate
- **Custom build script**: Rejected - maintenance burden, less community support

**Build Process**:
```bash
# Per-component build
cd components/core/string-concat
just build    # cargo build --target wasm32-wasip2 --release
just install  # cp to ../../bin/

# Batch build
cd components
just build-all    # Builds all components in all categories
just install-all  # Installs all to bin/
```

### 4. Type System for Component I/O

**Decision**: WIT primitive types + list for collections

**Rationale**:
- Matches wasmtime type system exactly
- Simple serialization/deserialization
- Type checking at graph construction time
- Existing UI handles all WIT types

**Type Mapping**:

| Operation Category | Input Types | Output Types | Notes |
|-------------------|-------------|--------------|-------|
| String Operations | string | string, u32, bool, list\<string\> | UTF-8 encoded |
| Math Operations | f32, u32, i32 | f32, u32, i32 | Use f32 for general math, preserve int types |
| Logic Operations | bool, any | bool | any type for null checks |
| List Operations | list\<any\>, u32, string | list\<any\>, any, u32, string | Generic list type |
| Type Conversion | any | string, f32 | Coercion with error handling |

**Edge Case Handling**:
- Invalid indices → return error with recovery hint
- Type mismatches → return error identifying incompatible types
- Null/empty inputs → explicit error messages
- Overflow/underflow → error or clamp to limits

### 5. Error Handling Strategy

**Decision**: Result-based errors with structured ExecutionError type

**Rationale**:
- Matches existing wasmflow error handling
- Provides actionable context (which input failed, how to fix)
- Prevents component crashes (Rust Result type enforcement)
- UI displays errors inline with node

**Error Structure**:
```rust
pub struct ExecutionError {
    pub message: String,           // Human-readable error
    pub input_name: Option<String>, // Which input caused failure
    pub recovery_hint: Option<String>, // How to fix the issue
}
```

**Example Errors**:
- String substring: "Start index 10 exceeds string length 5" (input: start_index, hint: "Provide index < 5")
- List get: "Index 100 out of bounds for list of length 50" (input: index, hint: "Use index 0-49")
- Parse number: "Cannot parse 'hello' as number" (input: text, hint: "Provide numeric text like '42' or '3.14'")

### 6. Component Organization Pattern

**Decision**: Category-based directory structure (core/, math/, collections/, data/)

**Rationale**:
- Logical grouping improves discoverability
- Maps to UI component palette categories
- Enables batch operations (test all math components)
- Easier to navigate than flat 35+ component list
- Follows principle of least surprise

**Category Definitions**:

| Category | Components | UI Palette Label | Priority |
|----------|-----------|------------------|----------|
| core/ | String ops (7) + Logic ops (7) | "Text" + "Logic" | P1-P2 |
| math/ | Math extensions (9) | "Math" | P3 |
| collections/ | List operations (7) | "Collections" | P4 |
| data/ | Type conversion (4) | "Data" | P5 |

### 7. Testing Strategy

**Decision**: Unit tests embedded in component + integration test graphs

**Rationale**:
- Unit tests validate component logic in isolation
- Integration tests validate component loading and graph execution
- Fast feedback loop (cargo test per component)
- Test graphs serve as examples and documentation

**Test Structure**:
```rust
// In each component's src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() {
        // Test typical input
    }

    #[test]
    fn test_edge_cases() {
        // Test boundary conditions
    }

    #[test]
    fn test_error_handling() {
        // Test invalid inputs
    }
}
```

**Integration Tests**:
- `tests/component_tests/string_processing.json` - Graph using 7 string components
- `tests/component_tests/math_operations.json` - Graph using 9 math components
- `tests/component_tests/list_manipulation.json` - Graph using 7 list components

### 8. Component Metadata Standards

**Decision**: Consistent naming and categorization across all components

**Rationale**:
- Improves user experience (predictable naming)
- Enables search and filtering
- Consistent documentation structure

**Naming Conventions**:
- **Package name**: `example-{category}-{operation}` (e.g., `example-string-concat`)
- **Display name**: "{Operation} {Type}" (e.g., "String Concat", "Math Power")
- **Category**: One of ["Text", "Logic", "Math", "Collections", "Data"]
- **Description**: Single sentence describing operation and return value

**Example Metadata**:
```rust
fn get_info() -> ComponentInfo {
    ComponentInfo {
        name: "String Concat".to_string(),
        description: "Joins multiple strings into a single string".to_string(),
        category: "Text".to_string(),
        version: "1.0.0".to_string(),
    }
}
```

## Design Patterns

### Pattern 1: Simple Pure Functions

**Components**: 25/35 components (string-length, math-abs, boolean-and, etc.)

**Pattern**:
```rust
// Minimal component structure
export!(Component);

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo { /* metadata */ }
    fn get_inputs() -> Vec<PortSpec> { /* 1-2 inputs */ }
    fn get_outputs() -> Vec<PortSpec> { /* 1 output */ }
    fn get_capabilities() -> Option<Vec<String>> { None }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
        // Extract input
        // Perform operation
        // Return result
    }
}
```

**Characteristics**:
- 50-100 lines of code
- Single input, single output
- No state, no side effects
- Minimal error handling

### Pattern 2: Multi-Input Operations

**Components**: 6/35 components (string-concat, math-min, math-max, compare, boolean ops)

**Pattern**:
```rust
fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
    // Accept variable number of inputs
    let values: Vec<String> = inputs.iter()
        .filter_map(|iv| match &iv.value {
            NodeValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    // Perform operation across all inputs
    let result = operation(values);

    Ok(vec![OutputValue { /* result */ }])
}
```

**Characteristics**:
- Variadic input ports (1-N inputs)
- Fold/reduce operation
- Handle mixed types gracefully

### Pattern 3: Operation Selector Components

**Components**: 2/35 components (string-case, math-trig)

**Pattern**:
```rust
fn get_inputs() -> Vec<PortSpec> {
    vec![
        PortSpec {
            name: "value".to_string(),
            data_type: DataType::String,
            // ...
        },
        PortSpec {
            name: "operation".to_string(),
            data_type: DataType::String,  // "uppercase" | "lowercase" | "titlecase"
            // ...
        },
    ]
}

fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
    let value = extract_input(&inputs, "value")?;
    let operation = extract_input(&inputs, "operation")?;

    let result = match operation.as_str() {
        "uppercase" => value.to_uppercase(),
        "lowercase" => value.to_lowercase(),
        "titlecase" => titlecase(&value),
        _ => return Err(/* invalid operation error */),
    };

    Ok(vec![/* result */])
}
```

**Characteristics**:
- Operation selector input (enum-like)
- Switch/match on operation type
- Multiple code paths in single component

### Pattern 4: Complex Type Handling

**Components**: 2/35 components (json-stringify, format-template)

**Pattern**:
```rust
use serde_json;

fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
    let data = extract_input(&inputs, "data")?;

    // Handle NodeValue::Record or NodeValue::List
    let json = match data {
        NodeValue::Record(map) => serde_json::to_string_pretty(&map),
        NodeValue::List(vec) => serde_json::to_string_pretty(&vec),
        _ => serde_json::to_string_pretty(&data),
    }?;

    Ok(vec![OutputValue {
        name: "json".to_string(),
        value: NodeValue::String(json),
    }])
}
```

**Characteristics**:
- Handles structured data (Record, List)
- Uses serde for serialization
- More complex error handling

## Best Practices Summary

### DO:
✅ Follow established component structure (Cargo.toml, build.rs, Justfile, wit/, src/)
✅ Use identical Justfile and build.rs from existing components
✅ Declare capabilities as None (pure computation)
✅ Include 3-5 unit tests per component
✅ Provide helpful error messages with recovery hints
✅ Keep component size <200KB
✅ Target <10ms execution time
✅ Use Result type for all operations that can fail
✅ Document component purpose in README.md
✅ Set appropriate category for palette organization

### DON'T:
❌ Add external dependencies without justification
❌ Perform I/O operations (file, network)
❌ Store state between executions
❌ Use panic!() in production code
❌ Return generic error messages
❌ Implement multiple unrelated operations in one component
❌ Exceed complexity budget (keep functions simple)
❌ Skip unit tests
❌ Use unwrap() without validation
❌ Assume input types (always validate)

## Performance Optimization Guidelines

### String Operations
- Use &str for read-only operations
- Pre-allocate String with capacity when size known
- Use chars().count() for Unicode-aware length
- Avoid unnecessary clones

### Math Operations
- Use appropriate numeric type (f32 vs f64, i32 vs u32)
- Leverage WASM native instructions (no external libs)
- Handle NaN/Infinity explicitly
- Use checked arithmetic for overflow detection

### List Operations
- Pre-allocate Vec with capacity
- Use iterators instead of indexed loops
- Avoid O(n²) algorithms
- Return references where possible (use clone only when necessary)

### Memory Management
- Reuse allocations where possible
- Drop large structures early
- Use stack allocation for small arrays
- Profile with cargo bloat to find size issues

## Security Considerations

All core library components are **pure computation** with zero system access:

✅ **No capabilities required** - All components return None from get_capabilities()
✅ **Sandboxed execution** - wasmtime enforces memory isolation
✅ **No side effects** - Operations are deterministic and stateless
✅ **Input validation** - All inputs validated before use
✅ **Error containment** - Errors don't crash runtime or affect other components
✅ **Type safety** - WIT types enforced at component boundaries

**Attack surface**: None - components cannot access filesystem, network, or host environment.

## Implementation Roadmap

### Phase Order (by Priority)

1. **P1: String Operations** (7 components)
   - Most frequently used
   - Simple implementations
   - High user value
   - Good starting point for pattern validation

2. **P2: Comparison & Logic** (7 components)
   - Enable conditional workflows
   - Build on string operations
   - Moderate complexity (compare component handles multiple types)

3. **P3: Math Extensions** (9 components)
   - Leverage Rust standard library
   - Simple pure functions
   - math-trig component uses operation selector pattern

4. **P4: List Operations** (7 components)
   - More complex (mutable operations, bounds checking)
   - Build on understanding of other components
   - Important for batch processing

5. **P5: Data Transformation** (4 components)
   - Most complex (json-stringify handles structured data)
   - Depends on list operations (format-template)
   - High value for interoperability

### Validation Checkpoints

After each priority level:
- ✅ All components build successfully
- ✅ All unit tests pass
- ✅ Components load in wasmflow UI
- ✅ Integration test graph executes correctly
- ✅ Performance targets met (<10ms per operation)
- ✅ Binary size targets met (<200KB per component)

## References

- **WIT Specification**: https://component-model.bytecodealliance.org/design/wit.html
- **wasmtime Component Model**: https://docs.wasmtime.dev/api/wasmtime/component/
- **Rust WASM Book**: https://rustwasm.github.io/book/
- **Existing Components**: `/home/user/wasmflow/components/`
- **Component Builder Skill**: `/home/user/wasmflow/.claude/skills/wasmflow-component-builder/`
- **Build Documentation**: `/home/user/wasmflow/docs/BUILDING_COMPONENTS.md`

## Conclusion

This research establishes a clear, repeatable pattern for building 35+ core library components. All technology choices leverage existing wasmflow infrastructure, ensuring consistency and maintainability. The phased implementation approach (P1→P5) enables incremental validation and early user feedback.

**Next Phase**: Generate data-model.md with detailed specifications for all 35+ components.
