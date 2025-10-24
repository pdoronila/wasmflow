# Quickstart: Building Core Library Components

**Feature**: 010-wasm-components-core
**Date**: 2025-10-23
**Purpose**: Developer guide for implementing the WASM components core library

## Overview

This guide walks through the process of building the 35+ core library components, from setup to deployment. All components follow an established pattern, making development fast and consistent.

## Prerequisites

### Required Tools

```bash
# Rust toolchain with wasm32-wasip2 target
rustup target add wasm32-wasip2

# Just task runner
cargo install just

# Nushell (for build scripts)
# Installation: https://www.nushell.sh/book/installation.html

# Verify installations
rustc --version  # Should be 1.75+
just --version
nu --version
```

### Repository Setup

```bash
# Clone repository
cd /home/user/wasmflow

# Ensure you're on the feature branch
git checkout 010-wasm-components-core

# Verify existing component structure
ls components/
# Should see: adder/, double-number/, echo/, json-parser/, bin/, etc.
```

## Development Workflow

### Step 1: Component Scaffolding

#### Option A: Manual Creation

```bash
# Create component directory structure
mkdir -p components/core/string-concat/{src,wit}

cd components/core/string-concat

# Copy boilerplate files from existing component
cp ../../json-parser/Cargo.toml ./
cp ../../json-parser/build.rs ./
cp ../../json-parser/Justfile ./
cp ../../json-parser/wit/node.wit ./wit/

# Edit Cargo.toml to update package name
# package name = "example-string-concat"
```

#### Option B: Using Scaffold Script (if available)

```bash
# Use the wasmflow component builder skill script
.claude/skills/wasmflow-component-builder/scripts/new_component.sh \
  --name string-concat \
  --category core \
  --description "Joins multiple strings"
```

### Step 2: Define WIT Interface

Edit `wit/node.wit`:

```wit
// Import standard component interface
// (This is already in the template - just verify it's correct)

world component {
    export metadata;
    export execution;
}

// metadata and execution interfaces are imported from wasmflow standard types
```

**Note**: The WIT file is generic across all components. Component-specific behavior is defined in Rust code via the metadata interface.

### Step 3: Implement Component

Edit `src/lib.rs`:

```rust
// Template for string-concat component
wit_bindgen::generate!({
    world: "component",
    path: "../wit",
});

use exports::metadata::Guest as MetadataGuest;
use exports::execution::Guest as ExecutionGuest;
use exports::{
    ComponentInfo, PortSpec, DataType, InputValue, OutputValue,
    ExecutionError, NodeValue,
};

// Export component
export!(Component);

struct Component;

// ============================================================================
// Metadata Interface
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "String Concat".to_string(),
            description: "Joins multiple strings into a single string".to_string(),
            category: "Text".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input1".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "First string".to_string(),
            },
            PortSpec {
                name: "input2".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Second string".to_string(),
            },
            PortSpec {
                name: "input3".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Third string".to_string(),
            },
            // Add more optional inputs as needed
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "result".to_string(),
            data_type: DataType::StringType,
            optional: false,
            description: "Concatenated string".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None  // Pure computation, no capabilities needed
    }
}

// ============================================================================
// Execution Interface
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
        // Extract string inputs
        let mut strings = Vec::new();

        for input in &inputs {
            match &input.value {
                NodeValue::String(s) => strings.push(s.clone()),
                _ => {
                    return Err(ExecutionError {
                        message: format!("Expected string, got {:?}", input.value),
                        input_name: Some(input.name.clone()),
                        recovery_hint: Some("Provide string values".to_string()),
                    });
                }
            }
        }

        // Concatenate all strings
        let result = strings.join("");

        // Return output
        Ok(vec![OutputValue {
            name: "result".to_string(),
            value: NodeValue::String(result),
        }])
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_two_strings() {
        let inputs = vec![
            InputValue {
                name: "input1".to_string(),
                value: NodeValue::String("Hello".to_string()),
            },
            InputValue {
                name: "input2".to_string(),
                value: NodeValue::String(" World".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "Hello World"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_concat_multiple_strings() {
        let inputs = vec![
            InputValue {
                name: "input1".to_string(),
                value: NodeValue::String("a".to_string()),
            },
            InputValue {
                name: "input2".to_string(),
                value: NodeValue::String("b".to_string()),
            },
            InputValue {
                name: "input3".to_string(),
                value: NodeValue::String("c".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "abc"),
            _ => panic!("Expected string output"),
        }
    }

    #[test]
    fn test_concat_with_empty_string() {
        let inputs = vec![
            InputValue {
                name: "input1".to_string(),
                value: NodeValue::String("".to_string()),
            },
            InputValue {
                name: "input2".to_string(),
                value: NodeValue::String("test".to_string()),
            },
        ];

        let result = Component::execute(inputs).unwrap();
        match &result[0].value {
            NodeValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string output"),
        }
    }
}
```

### Step 4: Build and Test

```bash
# From component directory (e.g., components/core/string-concat/)

# Run unit tests
just test

# Build component
just build

# Install to bin/ directory
just install

# Verify .wasm file created
ls ../../bin/string_concat.wasm
```

### Step 5: Test in Wasmflow UI

```bash
# Run wasmflow application
cd /home/user/wasmflow
cargo run

# In the UI:
# 1. Component should appear in "Text" category palette
# 2. Drag component to canvas
# 3. Connect constant nodes to inputs
# 4. Execute graph to verify behavior
```

## Implementation Strategy

### Priority-Based Development

Follow the priority order defined in the specification:

**Phase 1: P1 - String Operations (7 components)**
- Start with simple components: string-length, string-trim
- Then multi-input: string-concat
- Then complex: string-split, string-case, string-substring, string-contains

**Phase 2: P2 - Comparison & Logic (7 components)**
- Start with simple: boolean-not, is-null, is-empty
- Then binary ops: boolean-and, boolean-or, boolean-xor
- Then complex: compare (handles multiple types)

**Phase 3: P3 - Math Operations (9 components)**
- Start with single-input: sqrt, abs, floor, ceil, round
- Then binary: power
- Then multi-input: min, max
- Then operation selector: trig

**Phase 4: P4 - List Operations (7 components)**
- Start with simple: list-length, list-contains
- Then indexed access: list-get
- Then transformations: list-append, list-slice, list-join, list-index-of

**Phase 5: P5 - Data Transformation (4 components)**
- Simple conversions: to-string, parse-number
- Complex: json-stringify, format-template

### Batch Operations

Build multiple components efficiently:

```bash
# Build all components in a category
cd components/core
just build-all     # Builds all in core/

cd ../math
just build-all     # Builds all in math/

# Or from top level
cd components
just build-all     # Builds ALL components
just install-all   # Installs all to bin/
```

## Component Patterns

### Pattern 1: Simple Pure Function (25/35 components)

**Characteristics**:
- Single input, single output
- 50-100 lines total
- Minimal error handling

**Template**: See string-length, math-abs, boolean-not

### Pattern 2: Multi-Input Operation (6/35 components)

**Characteristics**:
- Variable number of inputs
- Fold/reduce pattern
- Type checking for each input

**Template**: See string-concat, boolean-and, math-min

**Key Implementation Detail**:
```rust
// Accept all inputs (required + optional)
for input in &inputs {
    match &input.value {
        NodeValue::String(s) => values.push(s.clone()),
        _ => { /* error */ }
    }
}

// Perform operation
let result = values.join("");  // or other operation
```

### Pattern 3: Operation Selector (2/35 components)

**Characteristics**:
- Operation parameter determines behavior
- Switch/match on operation string
- Shared input/output structure

**Template**: See string-case, math-trig

**Key Implementation Detail**:
```rust
let operation = extract_string_input(&inputs, "operation")?;

let result = match operation.as_str() {
    "uppercase" => value.to_uppercase(),
    "lowercase" => value.to_lowercase(),
    "titlecase" => titlecase(&value),
    _ => return Err(invalid_operation_error()),
};
```

### Pattern 4: Complex Type Handling (2/35 components)

**Characteristics**:
- Handles structured data (Record, List)
- Requires serde dependencies
- More complex error handling

**Template**: See json-stringify, format-template

**Key Implementation Detail**:
```rust
// In Cargo.toml, add:
// serde = "1.0"
// serde_json = "1.0"

use serde_json;

match data {
    NodeValue::Record(map) => serde_json::to_string_pretty(&map),
    NodeValue::List(vec) => serde_json::to_string_pretty(&vec),
    _ => serde_json::to_string_pretty(&data),
}?;
```

## Common Issues and Solutions

### Issue 1: WIT Binding Errors

**Error**: `cannot find type NodeValue in this scope`

**Solution**: Ensure WIT file is in `wit/node.wit` and contains standard interface definitions. The `wit_bindgen::generate!` macro must point to correct path.

### Issue 2: Component Not Loading in UI

**Error**: Component doesn't appear in palette

**Solutions**:
1. Verify .wasm file in `components/bin/` directory
2. Check component name matches expected pattern (snake_case.wasm)
3. Reload components in UI (File → Reload Components)
4. Check console logs for loading errors

### Issue 3: Type Mismatch Errors

**Error**: `expected String, found NodeValue::U32`

**Solution**: Add explicit type checking and helpful error messages:

```rust
fn extract_string_input(inputs: &[InputValue], name: &str) -> Result<String, ExecutionError> {
    let input = inputs.iter()
        .find(|i| i.name == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.value {
        NodeValue::String(s) => Ok(s.clone()),
        _ => Err(ExecutionError {
            message: format!("Expected string for input '{}', got {:?}", name, input.value),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a string value".to_string()),
        }),
    }
}
```

### Issue 4: Binary Size Too Large

**Error**: Component .wasm file > 200KB

**Solutions**:
1. Verify Cargo.toml has release optimizations:
   ```toml
   [profile.release]
   opt-level = "s"   # Optimize for size
   lto = true        # Link-time optimization
   strip = true      # Strip debug symbols
   ```
2. Check for unnecessary dependencies
3. Use `cargo bloat --release --target wasm32-wasip2` to find size issues

## Testing Strategy

### Unit Tests (Required)

Every component must have at least 3 unit tests:
1. Basic operation test (typical input)
2. Edge case test (boundary conditions)
3. Error handling test (invalid input)

```bash
# Run tests for single component
cd components/core/string-concat
just test

# Run tests for all components
cd components
just test-all  # (if defined in top-level Justfile)
```

### Integration Tests

Create test graphs that combine multiple components:

```bash
# Location: tests/component_tests/
# - string_processing.json
# - math_operations.json
# - list_manipulation.json

# These can be loaded in wasmflow UI to verify component interaction
```

### Manual Testing Checklist

For each component:
- [ ] Component appears in correct palette category
- [ ] Input ports accept correct types
- [ ] Execution produces expected output
- [ ] Error messages are helpful and actionable
- [ ] Component works when chained with others
- [ ] Performance is acceptable (<10ms for typical inputs)

## Validation Checkpoints

After completing each priority level:

```bash
# Build all components
cd components
just build-all
just install-all

# Verify component count
ls bin/*.wasm | wc -l
# P1: Should see 7 new components
# P2: Should see 14 total (7 + 7)
# P3: Should see 23 total (14 + 9)
# P4: Should see 30 total (23 + 7)
# P5: Should see 34 total (30 + 4)

# Run wasmflow and verify:
cargo run
# 1. All components load without errors
# 2. Components appear in correct categories
# 3. Create test graph using new components
# 4. Execute graph successfully
```

## Performance Optimization

### Tips for Fast Components

1. **Avoid unnecessary clones**: Use references where possible
   ```rust
   // Bad
   let s = input.value.clone();
   process(s.clone());

   // Good
   let s = &input.value;
   process(s);
   ```

2. **Pre-allocate collections**: When size is known
   ```rust
   // Bad
   let mut result = String::new();
   for s in strings { result.push_str(&s); }

   // Good
   let capacity: usize = strings.iter().map(|s| s.len()).sum();
   let mut result = String::with_capacity(capacity);
   for s in strings { result.push_str(&s); }
   ```

3. **Use iterators**: More efficient than loops
   ```rust
   // Bad
   for i in 0..list.len() {
       if list[i] == value { return true; }
   }

   // Good
   list.iter().any(|item| item == &value)
   ```

4. **Minimize allocations**: Reuse buffers when possible
   ```rust
   // For simple operations, return references or use stack
   // Only allocate when necessary
   ```

## Next Steps

After completing all components:

1. **Update CLAUDE.md**: Run script to update project conventions
2. **Create Documentation**: Add component reference to docs/
3. **Performance Benchmarking**: Measure execution times
4. **User Testing**: Get feedback on component usability
5. **Migration Planning**: Consider moving built-in math ops to WASM

## Resources

- **Component Templates**: `/home/user/wasmflow/.claude/skills/wasmflow-component-builder/references/component_templates.md`
- **WIT Reference**: `/home/user/wasmflow/.claude/skills/wasmflow-component-builder/references/wit_interface_reference.md`
- **Troubleshooting**: `/home/user/wasmflow/.claude/skills/wasmflow-component-builder/references/troubleshooting.md`
- **Existing Components**: `/home/user/wasmflow/components/` (json-parser, adder, echo)
- **Build Documentation**: `/home/user/wasmflow/docs/BUILDING_COMPONENTS.md`

## Summary

This quickstart provides everything needed to implement the core library:
- ✅ Setup instructions
- ✅ Step-by-step workflow
- ✅ Code templates for all patterns
- ✅ Testing strategy
- ✅ Validation checkpoints
- ✅ Optimization tips

**Estimated Timeline**:
- P1 (7 components): 1-2 days
- P2 (7 components): 1-2 days
- P3 (9 components): 1-2 days
- P4 (7 components): 1-2 days
- P5 (4 components): 1 day
- **Total**: 5-9 days for complete implementation

Ready to start! Begin with P1: String Operations.
