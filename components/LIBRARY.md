# WasmFlow Core Component Library - Developer Guide

**Version**: 1.1.0
**Total Components**: 43
**Categories**: 5 (Text, Logic, Math, Collections, Data)
**Target**: wasm32-wasip2
**WIT Spec**: wasmflow:node@1.1.0

## Overview

The WasmFlow Core Component Library is a comprehensive collection of pre-built WebAssembly components for common data processing operations. All components are pure computation nodes with no side effects, designed to be composed together in visual node graphs.

**Key Features**:
- **Pure WASM Components**: Compiled to wasm32-wasip2 target for cross-platform execution
- **Type-Safe Interfaces**: WIT (WebAssembly Interface Types) for clear contracts
- **Comprehensive Testing**: 148+ unit tests across all components
- **Optimized Binaries**: 60KB-1MB per component with LTO and strip optimizations
- **Minimal Dependencies**: Standard library only (except json-stringify and regex components)

## Quick Reference

### Component Count by Category

| Category | Count | Location | Description |
|----------|-------|----------|-------------|
| **Text** | 9 | `core/` + `text/` | String operations + regex pattern matching |
| **Logic** | 7 | `core/` | Comparison and boolean operations |
| **Math** | 9 | `math/` | Mathematical functions (power, sqrt, trig, etc.) |
| **Collections** | 13 | `collections/` | List operations (filter, count, reject with regex support) |
| **Data** | 5 | `data/` | Type conversion, formatting, and JSONL batch processing |

### Build Commands

```bash
# Test all components
cd components && just test-all

# Build all components
cd components && just build-all

# Install all to bin/
cd components && just install-all

# Clean all
cd components && just clean-all
```

## Component Categories

### 1. Text Processing (9 components)

**Location**: `components/core/` + `components/text/`

#### Basic String Operations (7 components - `core/`)

| Component | Inputs | Outputs | Description |
|-----------|--------|---------|-------------|
| **string-concat** | Multiple strings | result: string | Joins multiple strings |
| **string-split** | text: string, delimiter: string | parts: list<string> | Splits by delimiter |
| **string-length** | text: string | length: u32 | Unicode-aware length |
| **string-trim** | text: string | result: string | Removes whitespace |
| **string-case** | text: string, operation: string | result: string | upper/lower/title case |
| **string-contains** | text: string, substring: string | result: bool | Substring search |
| **string-substring** | text: string, start: u32, length?: u32 | result: string | Extract substring |

#### Regex Pattern Matching (2 components - `text/`)

| Component | Inputs | Outputs | Description | Dependencies |
|-----------|--------|---------|-------------|--------------|
| **regex-match** | text: string, pattern: string | matches: bool | Test if text matches regex pattern | regex 1.10 |
| **regex-match-any** | text: string, patterns: StringListVal | matches: bool, matched_pattern: string, match_count: u32 | Test if text matches ANY of multiple patterns (OR logic) | regex 1.10 |

**Integration Test**: `tests/component_tests/string_processing.json`

### 2. Logic & Validation (7 components)

**Location**: `components/core/`

| Component | Inputs | Outputs | Description |
|-----------|--------|---------|-------------|
| **compare** | left: any, right: any, operation: string | result: bool | Comparison (>, <, ==, etc.) |
| **boolean-and** | Multiple bool inputs | result: bool | Logical AND |
| **boolean-or** | Multiple bool inputs | result: bool | Logical OR |
| **boolean-not** | input: bool | result: bool | Logical NOT |
| **boolean-xor** | left: bool, right: bool | result: bool | Logical XOR |
| **is-null** | value?: any | result: bool | Null check |
| **is-empty** | value: any | result: bool | Empty string/list check |

**Integration Test**: `tests/component_tests/data_validation.json`

### 3. Mathematical Operations (9 components)

**Location**: `components/math/`

| Component | Inputs | Outputs | Description |
|-----------|--------|---------|-------------|
| **power** | base: f32, exponent: f32 | result: f32 | Exponentiation |
| **sqrt** | value: f32 | result: f32 | Square root |
| **abs** | value: f32 | result: f32 | Absolute value |
| **min** | Multiple f32 inputs | result: f32 | Minimum value |
| **max** | Multiple f32 inputs | result: f32 | Maximum value |
| **floor** | value: f32 | result: f32 | Floor function |
| **ceil** | value: f32 | result: f32 | Ceiling function |
| **round** | value: f32 | result: f32 | Rounding |
| **trig** | angle: f32, operation: string | result: f32 | sin/cos/tan |

**Integration Test**: `tests/component_tests/math_operations.json`

### 4. List Manipulation (13 components)

**Location**: `components/collections/`

**Note**: All list components currently work with `StringListVal` (Vec<String>)

#### Basic List Operations (7 components)

| Component | Inputs | Outputs | Description |
|-----------|--------|---------|-------------|
| **list-length** | list: StringListVal | length: u32 | Get list length |
| **list-get** | list: StringListVal, index: u32 | element: string | Get element at index |
| **list-append** | list: StringListVal, value: string | result: StringListVal | Append (immutable) |
| **list-join** | list: StringListVal, delimiter: string | result: string | Join to string |
| **list-slice** | list: StringListVal, start: u32, end?: u32 | result: StringListVal | Extract sublist |
| **list-contains** | list: StringListVal, value: string | result: bool | Value search |
| **list-index-of** | list: StringListVal, value: string | index: i32 | Find index (-1 if not found) |

#### Advanced List Operations with Regex (6 components)

| Component | Inputs | Outputs | Description | Dependencies |
|-----------|--------|---------|-------------|--------------|
| **list-filter-empty** | list: StringListVal | filtered: StringListVal, removed_count: u32 | Remove empty/whitespace strings | None |
| **list-filter-regex** | list: StringListVal, pattern: string | matched: StringListVal, removed_count: u32 | Keep items matching pattern (allowlist) | regex 1.10 |
| **list-filter-regex-any** | list: StringListVal, patterns: StringListVal | matched: StringListVal, removed_count: u32 | Keep items matching ANY pattern (OR logic) | regex 1.10 |
| **list-reject-regex** | list: StringListVal, pattern: string | kept: StringListVal, removed_count: u32 | Remove items matching pattern (blocklist) | regex 1.10 |
| **list-count-regex** | list: StringListVal, pattern: string | count: u32, percentage: f32, total: u32 | Count items matching pattern | regex 1.10 |
| **list-count-regex-any** | list: StringListVal, patterns: StringListVal | count: u32, percentage: f32, total: u32 | Count items matching ANY pattern (OR logic) | regex 1.10 |

**Integration Tests**:
- `tests/component_tests/list_manipulation.json`
- Kernel message engine example (see Foundational Components section)

### 5. Data Transformation (5 components)

**Location**: `components/data/`

| Component | Inputs | Outputs | Description | Dependencies |
|-----------|--------|---------|-------------|--------------|
| **json-stringify** | data: any | json: string | Serialize to JSON | serde_json 1.0 |
| **json-extract-each** | json_strings: StringListVal, field_path: string | values: StringListVal, error_count: u32, success_count: u32 | Extract field from each JSON string (JSONL batch processing) | serde_json 1.0 |
| **to-string** | value: any | text: string | Convert primitive to string | None |
| **parse-number** | text: string | number: f32 | Parse string to f32 | None |
| **format-template** | template: string, values: StringListVal | result: string | Replace {0}, {1}, etc. | None |

**Integration Tests**:
- `tests/component_tests/data_transformation.json`
- Kernel message engine example (see Foundational Components section)

## Building the Library

### Prerequisites

```bash
# Rust with wasm32-wasip2 target
rustup target add wasm32-wasip2

# Command runner
cargo install just

# WASM tools (requires network access)
cargo install cargo-component
cargo install wasm-tools
```

### Build Process

#### Individual Component

```bash
cd components/data/to-string
just test       # Run unit tests
just build      # Compile to WASM (release mode)
just install    # Copy .wasm to ../bin/
just check      # Quick compile check
just clean      # Remove build artifacts
```

#### Category-Level

```bash
# Build all components in a category
cd components/collections
just build-all
just install-all
just test-all
just clean-all
```

#### Full Library

```bash
# Build all 43 components
cd components
just build-all

# This executes:
# - just core/build-all
# - just text/build-all
# - just math/build-all
# - just collections/build-all
# - just data/build-all
```

### Build Artifacts

After building, components are located at:
```
components/
├── core/
│   ├── string-concat/target/wasm32-wasip2/release/string_concat.wasm
│   └── ...
├── math/
│   └── ...
├── collections/
│   └── ...
└── bin/
    ├── string_concat.wasm   (copied by 'just install')
    └── ...
```

## Testing

### Unit Tests

Each component includes 3-9 unit tests:

```bash
# Test single component
cd components/data/parse-number
cargo test

# Example output:
running 9 tests
test tests::test_parse_integer ... ok
test tests::test_parse_decimal ... ok
test tests::test_parse_scientific_notation ... ok
test tests::test_parse_with_whitespace ... ok
test tests::test_parse_zero ... ok
test tests::test_parse_negative ... ok
test tests::test_parse_invalid_string ... ok
test tests::test_parse_empty_string ... ok
test tests::test_parse_partial_number ... ok
```

### Integration Tests

Integration tests validate components working together:

- `string_processing.json` - Demonstrates text pipeline
- `data_validation.json` - Logic and comparison operations
- `math_operations.json` - Mathematical computations
- `list_manipulation.json` - Collection processing
- `data_transformation.json` - Type conversion and formatting
- `comprehensive_workflow.json` - All categories combined

**To run**: Load JSON files in WasmFlow UI and execute

### Test Coverage

| Category | Components | Total Tests | Coverage |
|----------|------------|-------------|----------|
| Text (basic) | 7 | 21+ | Typical, edge, error cases |
| Text (regex) | 2 | 14 | Pattern matching, multi-pattern |
| Logic | 7 | 21+ | All operators, type mismatches |
| Math | 9 | 27+ | Valid ops, NaN, infinity |
| Collections (basic) | 7 | 21+ | Empty, bounds, not found |
| Collections (regex) | 6 | 30 | Filter, count, reject patterns |
| Data | 5 | 14 | All Value variants, errors, JSONL |
| **Total** | **43** | **148+** | Comprehensive |

## Component Structure

### Standard File Layout

```
component-name/
├── Cargo.toml          # Package configuration
├── build.rs            # Build script
├── Justfile            # Build automation
├── wit/
│   └── node.wit        # WIT interface (wasmflow:node@1.1.0)
└── src/
    └── lib.rs          # Implementation + tests
```

### Standard Cargo.toml

```toml
[package]
name = "component-name"
version = "1.0.0"
edition = "2021"

[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.30"
# Add dependencies as needed (e.g., serde_json for json-stringify)

[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
strip = true         # Strip symbols
```

### Standard build.rs

```rust
fn main() {
    println!("cargo:rerun-if-changed=wit");
}
```

### Standard Justfile

```just
component_name := replace(file_name(justfile_directory()), "-", "_")

build:
    cargo build --target wasm32-wasip2 --release

install:
    cp target/wasm32-wasip2/release/{{component_name}}.wasm ../bin/

clean:
    cargo clean

test:
    cargo test

check:
    cargo check --target wasm32-wasip2

setup:
    rustup target add wasm32-wasip2
```

## Implementation Patterns

### Pattern 1: Basic Component Structure

```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component",  // Use "component-with-ui" for custom footer rendering
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Component Name".to_string(),
            version: "1.0.0".to_string(),
            description: "Clear description of what this does".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Category".to_string()),  // Text, Logic, Math, Collections, Data
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input1".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "What this input represents".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "output1".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "What this output represents".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None  // Core library components are pure computation
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Implementation here
        Ok(vec![])
    }
}

export!(Component);  // REQUIRED - exports Component for WIT discovery

#[cfg(test)]
mod tests {
    use super::*;
    // Tests here
}
```

### Pattern 2: Input Extraction with Error Handling

```rust
// Extract required input
let input = inputs
    .iter()
    .find(|(name, _)| name == "input_name")
    .ok_or_else(|| ExecutionError {
        message: "Missing required input: input_name".to_string(),
        input_name: Some("input_name".to_string()),
        recovery_hint: Some("Connect a value to this input".to_string()),
    })?;

// Type-safe value extraction
let value = match &input.1 {
    Value::StringVal(s) => s,
    _ => {
        return Err(ExecutionError {
            message: format!("Expected string for 'input_name', got {:?}", input.1),
            input_name: Some("input_name".to_string()),
            recovery_hint: Some("Provide a string value".to_string()),
        });
    }
};
```

### Pattern 3: Optional Input Handling

```rust
let optional_value = if let Some(input) = inputs.iter().find(|(name, _)| name == "optional_input") {
    match &input.1 {
        Value::U32Val(n) => *n as usize,
        _ => {
            return Err(ExecutionError {
                message: format!("Expected u32 for 'optional_input', got {:?}", input.1),
                input_name: Some("optional_input".to_string()),
                recovery_hint: Some("Provide a positive integer".to_string()),
            });
        }
    }
} else {
    default_value  // Use default when not provided
};
```

### Pattern 4: Multi-Input Processing

```rust
let mut values = Vec::new();
for input in &inputs {
    match &input.1 {
        Value::BoolVal(b) => values.push(*b),
        _ => {
            return Err(ExecutionError {
                message: format!("Expected boolean for '{}', got {:?}", input.0, input.1),
                input_name: Some(input.0.clone()),
                recovery_hint: Some("All inputs must be boolean values".to_string()),
            });
        }
    }
}

// Process collected values
let result = values.iter().all(|&x| x);  // Example: AND operation
```

### Pattern 5: Type Conversion with Validation

```rust
// Parse string to number
let number = text.trim().parse::<f32>().map_err(|e| ExecutionError {
    message: format!("Failed to parse '{}' as a number: {}", text, e),
    input_name: Some("text".to_string()),
    recovery_hint: Some("Provide a valid number string (e.g., '42', '3.14', '1.5e2')".to_string()),
})?;

// Validate range
if number.is_nan() || number.is_infinite() {
    return Err(ExecutionError {
        message: format!("Invalid number: {}", number),
        input_name: Some("text".to_string()),
        recovery_hint: Some("Number must be finite".to_string()),
    });
}
```

## Common Pitfalls and Solutions

### Problem 1: Import Errors

**Error**: `unresolved import 'exports::execution'`

**Cause**: Using incorrect WIT import paths

**Solution**: Use full package paths
```rust
// ❌ Wrong
use exports::execution::Guest as ExecutionGuest;

// ✅ Correct
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use wasmflow::node::types::*;
```

### Problem 2: Missing List Type

**Error**: `no variant named 'ListVal' found for enum 'Value'`

**Cause**: WIT spec doesn't have generic `ListVal`

**Solution**: Use specific list types
```rust
// ❌ Wrong
Value::ListVal(items)

// ✅ Correct
Value::StringListVal(items)  // Vec<String>
Value::U32ListVal(items)     // Vec<u32>
Value::F32ListVal(items)     // Vec<f32>
```

### Problem 3: Missing Export Macro

**Error**: `failed to find export of interface 'wasmflow:node/metadata'`

**Cause**: Missing `export!(Component);` macro

**Solution**: Add export macro after impl blocks
```rust
impl ExecutionGuest for Component {
    // ...
}

export!(Component);  // ← Add this

#[cfg(test)]
mod tests {
    // ...
}
```

### Problem 4: Wrong ComponentInfo Structure

**Error**: Type mismatch in ComponentInfo fields

**Solution**: Use correct field order and types
```rust
ComponentInfo {
    name: "Component Name".to_string(),
    version: "1.0.0".to_string(),        // ← version comes before description
    description: "Description".to_string(),
    author: "Author".to_string(),
    category: Some("Category".to_string()),  // ← Option<String>, not String
}
```

### Problem 5: List Join Type Confusion

**Error**: Trying to pattern match strings inside StringListVal

**Cause**: StringListVal contains Vec<String>, not Vec<Value>

**Solution**: Use direct iteration
```rust
// ❌ Wrong
for value in list_values.iter() {
    match value {
        Value::StringVal(s) => ...,  // Doesn't work!
    }
}

// ✅ Correct
for value in list_values.iter() {
    // value is already &String
    result.push(value.clone());
}
```

## Performance Characteristics

### Binary Sizes

| Category | Average Size | Range | Notes |
|----------|--------------|-------|-------|
| Text (basic) | 100KB | 80-120KB | String operations |
| **Text (regex)** | **1.0MB** | **999KB-1.0MB** | **regex crate dependency** |
| Logic | 100KB | 80-120KB | Boolean operations |
| Math | 105KB | 90-130KB | Mathematical functions |
| Collections (basic) | 100KB | 80-120KB | List operations |
| **Collections (regex)** | **1.0MB** | **1.0MB** | **regex crate dependency** |
| Data | 110KB | 90-150KB | Type conversion |
| **Data (JSON)** | **126-150KB** | **126-150KB** | **serde_json dependency** |

**Optimization Settings**:
```toml
[profile.release]
opt-level = "s"    # Optimize for size
lto = true         # Link-time optimization
strip = true       # Strip debug symbols
```

### Execution Time

All components target < 10ms for typical operations:

| Operation Type | Target | Measured |
|----------------|--------|----------|
| String operations | <10ms | 1-5ms |
| Math operations | <10ms | <1ms |
| List (100 items) | <10ms | 2-8ms |
| List (1000 items) | <100ms | 20-80ms |
| JSON serialization | <10ms | 3-8ms |

**Note**: All operations are synchronous and blocking.

### Memory Usage

- **Stack Allocation**: Most data structures use stack
- **Immutable Operations**: List/string ops create new values
- **No Shared State**: Each execution is independent
- **No Memory Leaks**: Rust ownership prevents leaks

## Documentation

### Component Documentation

Each component should document:

1. **Purpose**: One-sentence description
2. **Inputs**: Name, type, optionality, description
3. **Outputs**: Name, type, description
4. **Behavior**: What it does, edge cases, error conditions
5. **Examples**: Typical usage in tests

### Phase Documentation

**Core Library (Phases 3-7)**:
- `specs/010-wasm-components-core/PHASE3_STRING_COMPONENTS.md` - Text processing (Phase 3)
- `specs/010-wasm-components-core/PHASE4_LOGIC_COMPONENTS.md` - Logic & validation (Phase 4)
- `specs/010-wasm-components-core/PHASE5_MATH_COMPONENTS.md` - Math operations (Phase 5)
- `specs/010-wasm-components-core/PHASE6_LIST_COMPONENTS.md` - List manipulation (Phase 6)
- `specs/010-wasm-components-core/PHASE7_DATA_COMPONENTS.md` - Data transformation (Phase 7)

**Foundational Components (Advanced)**:
- `components/FOUNDATIONAL_COMPONENTS_PLAN.md` - Regex + JSONL processing (9 components)

## Contributing

### Adding New Components

1. **Choose Category**: Text, Logic, Math, Collections, or Data
2. **Create Structure**: Use template from `components/.templates/`
3. **Implement**: Follow patterns in existing components
4. **Test**: Minimum 3 tests (typical, edge, error)
5. **Document**: Update category README and Justfile
6. **Validate**: Run `just test && just build`

### Code Review Checklist

- [ ] Follows standard directory structure
- [ ] Uses correct WIT import paths
- [ ] Includes `export!(Component);` macro
- [ ] Has ComponentInfo with all required fields
- [ ] Input/output types match WIT spec
- [ ] Error messages include recovery hints
- [ ] Minimum 3 unit tests
- [ ] Tests cover typical, edge, and error cases
- [ ] Added to category Justfile
- [ ] Documentation updated

## Resources

- **WIT Templates**: `components/.templates/`
- **Example Components**: All components in `components/`
- **Integration Tests**: `tests/component_tests/`
- **Build Scripts**: Justfiles at all levels
- **Documentation**: `specs/010-wasm-components-core/`

## License

Part of the WasmFlow project. See main repository LICENSE file.
