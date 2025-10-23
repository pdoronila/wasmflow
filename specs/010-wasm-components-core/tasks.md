# Implementation Tasks: WASM Components Core Library

**Feature**: 010-wasm-components-core
**Branch**: `010-wasm-components-core`
**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)
**Generated**: 2025-10-23

## Overview

This document provides actionable implementation tasks for building the WASM Components Core Library. Tasks are organized by user story priority (P1-P5), enabling incremental delivery and independent testing of each feature increment.

**Total Components**: 34 WASM components across 5 categories
**Total Tasks**: 155 tasks (setup, foundational, implementation, validation)
**Estimated Effort**: 5-9 days

## Task Organization

Tasks are organized into phases aligned with user story priorities:

1. **Phase 1: Setup** - Project initialization and directory structure
2. **Phase 2: Foundational** - Shared templates and build infrastructure (MUST complete before user stories)
3. **Phase 3: User Story 1 (P1)** - Text Processing Pipeline (7 string components)
4. **Phase 4: User Story 2 (P2)** - Data Validation Pipeline (7 logic components)
5. **Phase 5: User Story 3 (P3)** - Mathematical Computation Pipeline (9 math components)
6. **Phase 6: User Story 4 (P4)** - List Manipulation Pipeline (7 list components)
7. **Phase 7: User Story 5 (P5)** - Data Transformation Pipeline (4 data components)
8. **Phase 8: Polish & Integration** - Documentation, performance tuning, integration tests

## Task Notation

- **[P]**: Parallelizable (can be done concurrently with other [P] tasks in same phase)
- **[US1], [US2], etc.**: User Story label
- **T001, T002, etc.**: Sequential task numbers
- **Dependencies**: Listed at end of each phase

---

## Phase 1: Setup (Project Initialization)

**Goal**: Create directory structure and prepare development environment

### T001: Create component category directories [P]
**File**: `components/core/`, `components/math/`, `components/collections/`, `components/data/`
```bash
mkdir -p components/core
mkdir -p components/math
mkdir -p components/collections
mkdir -p components/data
```

### T002: Verify Rust toolchain setup [P]
**File**: N/A (environment check)
```bash
rustup target add wasm32-wasip2
cargo --version
just --version
nu --version
```

### T003: Create test infrastructure directories [P]
**File**: `tests/integration/`, `tests/component_tests/`
```bash
mkdir -p tests/integration
mkdir -p tests/component_tests
```

### T004: Create top-level Justfile for batch operations
**File**: `components/Justfile`
```just
# Top-level build automation for all components

# Build all components in all categories
build-all:
    @just core/build-all
    @just math/build-all
    @just collections/build-all
    @just data/build-all

# Install all components to bin/
install-all:
    @just core/install-all
    @just math/install-all
    @just collections/install-all
    @just data/install-all

# Test all components
test-all:
    @just core/test-all
    @just math/test-all
    @just collections/test-all
    @just data/test-all

# Clean all build artifacts
clean-all:
    @just core/clean-all
    @just math/clean-all
    @just collections/clean-all
    @just data/clean-all
```

### T005: Create category-level Justfiles [P]
**Files**:
- `components/core/Justfile`
- `components/math/Justfile`
- `components/collections/Justfile`
- `components/data/Justfile`

Each Justfile:
```just
# Category-level build automation

build-all:
    @cd string-concat && just build
    @cd string-split && just build
    # ... (add all components in category)

install-all:
    @cd string-concat && just install
    # ... (add all components in category)

test-all:
    @cd string-concat && just test
    # ... (add all components in category)

clean-all:
    @cd string-concat && just clean
    # ... (add all components in category)
```

**Phase 1 Complete**: Directory structure ready for component development

---

## Phase 2: Foundational (Shared Infrastructure)

**Goal**: Create reusable templates and verify build system (BLOCKING - must complete before any user story implementation)

### T006: Create shared WIT template
**File**: `components/.templates/node.wit`
```wit
// Standard WIT interface for all core library components
// Copy this file to each component's wit/ directory

world component {
    export metadata;
    export execution;
}

// Interfaces are imported from wasmflow standard types
```

### T007: Create shared Cargo.toml template
**File**: `components/.templates/Cargo.toml`
```toml
[package]
name = "example-COMPONENT-NAME"  # Replace with actual name
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.30"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

### T008: Create shared build.rs template
**File**: `components/.templates/build.rs`
```rust
fn main() {
    println!("cargo:rerun-if-changed=wit");
}
```

### T009: Create shared Justfile template
**File**: `components/.templates/Justfile`
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

### T010: Create shared lib.rs template with metadata boilerplate
**File**: `components/.templates/lib.rs`
```rust
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

export!(Component);

struct Component;

// ============================================================================
// Metadata Interface
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "COMPONENT_NAME".to_string(),  // Replace
            description: "DESCRIPTION".to_string(),  // Replace
            category: "CATEGORY".to_string(),  // Replace: Text, Logic, Math, Collections, Data
            version: "1.0.0".to_string(),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            // Define inputs
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            // Define outputs
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None  // All core library components are pure computation
    }
}

// ============================================================================
// Execution Interface
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
        // TODO: Implement component logic
        todo!()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() {
        // TODO: Test typical input
    }

    #[test]
    fn test_edge_cases() {
        // TODO: Test boundary conditions
    }

    #[test]
    fn test_error_handling() {
        // TODO: Test invalid inputs
    }
}
```

### T011: Create helper function templates
**File**: `components/.templates/helpers.rs`
```rust
// Common helper functions for extracting typed inputs

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

fn extract_u32_input(inputs: &[InputValue], name: &str) -> Result<u32, ExecutionError> {
    let input = inputs.iter()
        .find(|i| i.name == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.value {
        NodeValue::U32(n) => Ok(*n),
        _ => Err(ExecutionError {
            message: format!("Expected u32 for input '{}', got {:?}", name, input.value),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a positive integer value".to_string()),
        }),
    }
}

fn extract_f32_input(inputs: &[InputValue], name: &str) -> Result<f32, ExecutionError> {
    let input = inputs.iter()
        .find(|i| i.name == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.value {
        NodeValue::F32(n) => Ok(*n),
        _ => Err(ExecutionError {
            message: format!("Expected number for input '{}', got {:?}", name, input.value),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a numeric value".to_string()),
        }),
    }
}

fn extract_bool_input(inputs: &[InputValue], name: &str) -> Result<bool, ExecutionError> {
    let input = inputs.iter()
        .find(|i| i.name == name)
        .ok_or_else(|| ExecutionError {
            message: format!("Missing required input: {}", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Connect a value to this input".to_string()),
        })?;

    match &input.value {
        NodeValue::Bool(b) => Ok(*b),
        _ => Err(ExecutionError {
            message: format!("Expected boolean for input '{}', got {:?}", name, input.value),
            input_name: Some(name.to_string()),
            recovery_hint: Some("Provide a boolean value (true/false)".to_string()),
        }),
    }
}
```

### T012: Verify build system with test component
**File**: `components/core/.test-component/`

Create a minimal test component to validate the build pipeline:
1. Create directory structure
2. Copy templates
3. Implement minimal component
4. Run `just build`
5. Run `just install`
6. Verify .wasm file in bin/
7. Run `just test`
8. Delete test component

**Validation**: Build system works end-to-end before proceeding to user stories.

**Phase 2 Complete**: ✅ All shared infrastructure ready. Ready to begin user story implementation.

**CHECKPOINT**: Foundation complete - can now implement user stories in parallel

---

## Phase 3: User Story 1 (P1) - Text Processing Pipeline

**Goal**: Implement 7 string operation components enabling text processing workflows

**User Story**: As a wasmflow user, I want to process text data by chaining string operations together, so that I can clean, transform, and analyze text within my node graphs without writing custom code.

**Independent Test**: Create a node graph that takes raw text input, applies trim/case/concat operations, and outputs formatted text. All 7 components work correctly in isolation and when chained.

**Components**: string-concat, string-split, string-length, string-trim, string-case, string-contains, string-substring

### String Concat Component (Simple Pattern)

#### T013: [US1][P] Create string-concat component structure
**Files**: `components/core/string-concat/{Cargo.toml,build.rs,Justfile,wit/node.wit,src/lib.rs}`
- Copy templates from `.templates/`
- Update package name to `example-string-concat`

#### T014: [US1][P] Implement string-concat metadata interface
**File**: `components/core/string-concat/src/lib.rs`
```rust
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
```

#### T015: [US1][P] Implement string-concat execution logic and tests
**File**: `components/core/string-concat/src/lib.rs`

Execution logic:
```rust
fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError> {
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

    let result = strings.join("");

    Ok(vec![OutputValue {
        name: "result".to_string(),
        value: NodeValue::String(result),
    }])
}
```

Unit tests (3 minimum):
1. Test concat two strings
2. Test concat multiple strings
3. Test concat with empty string

#### T016: [US1][P] Build and validate string-concat component
```bash
cd components/core/string-concat
just test       # Run unit tests
just build      # Compile to WASM
just install    # Copy to bin/
```

### String Split Component (Simple Pattern)

#### T017: [US1][P] Create string-split component structure
**Files**: `components/core/string-split/{Cargo.toml,build.rs,Justfile,wit/node.wit,src/lib.rs}`

#### T018: [US1][P] Implement string-split metadata interface
**File**: `components/core/string-split/src/lib.rs`
- Inputs: text (string), delimiter (string)
- Outputs: parts (list<string>)

#### T019: [US1][P] Implement string-split execution logic and tests
**File**: `components/core/string-split/src/lib.rs`

Logic: Use Rust's `split()` method, handle empty delimiter (split into chars)

Tests:
1. Split on comma
2. Split on empty delimiter (chars)
3. Split with consecutive delimiters

#### T020: [US1][P] Build and validate string-split component

### String Length Component (Simple Pattern)

#### T021: [US1][P] Create string-length component structure
**Files**: `components/core/string-length/{Cargo.toml,build.rs,Justfile,wit/node.wit,src/lib.rs}`

#### T022: [US1][P] Implement string-length metadata interface
**File**: `components/core/string-length/src/lib.rs`
- Inputs: text (string)
- Outputs: length (u32)

#### T023: [US1][P] Implement string-length execution logic and tests
**File**: `components/core/string-length/src/lib.rs`

Logic: Use `.chars().count()` for Unicode-correct length

Tests:
1. ASCII string
2. Unicode string with emojis
3. Empty string

#### T024: [US1][P] Build and validate string-length component

### String Trim Component (Simple Pattern)

#### T025: [US1][P] Create string-trim component structure
#### T026: [US1][P] Implement string-trim metadata interface
#### T027: [US1][P] Implement string-trim execution logic and tests

Logic: Use `.trim()` method

Tests:
1. Leading/trailing spaces
2. Tab and newline characters
3. Middle whitespace preserved

#### T028: [US1][P] Build and validate string-trim component

### String Case Component (Operation Selector Pattern)

#### T029: [US1][P] Create string-case component structure
#### T030: [US1][P] Implement string-case metadata interface
**File**: `components/core/string-case/src/lib.rs`
- Inputs: text (string), operation (string: "uppercase"|"lowercase"|"titlecase")
- Outputs: result (string)

#### T031: [US1][P] Implement string-case execution logic and tests
**File**: `components/core/string-case/src/lib.rs`

Logic:
```rust
let operation = extract_string_input(&inputs, "operation")?;
let result = match operation.as_str() {
    "uppercase" => value.to_uppercase(),
    "lowercase" => value.to_lowercase(),
    "titlecase" => titlecase(&value),  // Implement titlecase helper
    _ => return Err(/* invalid operation error */),
};
```

Tests:
1. Uppercase operation
2. Lowercase operation
3. Titlecase operation
4. Invalid operation (error)

#### T032: [US1][P] Build and validate string-case component

### String Contains Component (Simple Pattern)

#### T033: [US1][P] Create string-contains component structure
#### T034: [US1][P] Implement string-contains metadata interface
- Inputs: text (string), substring (string)
- Outputs: result (bool)

#### T035: [US1][P] Implement string-contains execution logic and tests

Logic: Use `.contains()` method

Tests:
1. Substring found
2. Substring not found (case-sensitive)
3. Empty substring (always true)

#### T036: [US1][P] Build and validate string-contains component

### String Substring Component (Simple Pattern)

#### T037: [US1][P] Create string-substring component structure
#### T038: [US1][P] Implement string-substring metadata interface
- Inputs: text (string), start (u32), length (u32, optional)
- Outputs: result (string)

#### T039: [US1][P] Implement string-substring execution logic and tests

Logic: Character-based indexing with Unicode awareness

Tests:
1. Extract substring with length
2. Extract from start to end (no length)
3. Start beyond end (empty result)
4. Unicode characters

#### T040: [US1][P] Build and validate string-substring component

### Story 1 Integration

#### T041: [US1] Create integration test graph for string operations
**File**: `tests/component_tests/string_processing.json`

Node graph that:
1. Takes raw text input: "  hello WORLD  "
2. Applies string-trim
3. Applies string-case (lowercase)
4. Concatenates with another string
5. Splits on space
6. Gets length of first part
7. Checks if contains substring
8. Extracts substring

#### T042: [US1] Manual validation of User Story 1 acceptance scenarios

Load `string_processing.json` in wasmflow UI and verify all 5 acceptance scenarios:
1. ✅ string-trim removes whitespace
2. ✅ string-concat joins multiple strings
3. ✅ string-split outputs list of segments
4. ✅ string-case converts to uppercase
5. ✅ string-contains checks for substring

**Phase 3 Complete**: ✅ **User Story 1 (P1) - Text Processing Pipeline DELIVERED**
- All 7 string components implemented and tested
- Integration test validates component chaining
- MVP functionality available for users

**CHECKPOINT**: User Story 1 complete and independently testable

---

## Phase 4: User Story 2 (P2) - Data Validation Pipeline

**Goal**: Implement 7 comparison and logic components enabling data validation workflows

**User Story**: As a wasmflow user, I want to validate data using comparison and logic operations, so that I can create conditional flows and filter data based on business rules.

**Independent Test**: Create a validation graph that checks if values meet criteria (using compare components) and combines multiple conditions (using boolean logic components).

**Components**: compare, boolean-and, boolean-or, boolean-not, boolean-xor, is-null, is-empty

### Compare Component (Complex Type Handling)

#### T043: [US2][P] Create compare component structure
**Files**: `components/core/compare/{Cargo.toml,build.rs,Justfile,wit/node.wit,src/lib.rs}`

#### T044: [US2][P] Implement compare metadata interface
- Inputs: left (any), right (any), operation (string)
- Outputs: result (bool)
- Operations: equals, not-equals, greater-than, less-than, greater-or-equal, less-or-equal

#### T045: [US2][P] Implement compare execution logic and tests

Logic:
- Type compatibility checking (numbers can compare with numbers, strings with strings)
- Numeric comparison (handle mixed u32/i32/f32)
- Lexicographic string comparison
- Boolean equality only

Tests:
1. Compare numbers (greater-than)
2. Compare strings (less-than)
3. Mixed numeric types (valid)
4. Type mismatch (error)
5. Boolean ordering (error - only equals/not-equals)

#### T046: [US2][P] Build and validate compare component

### Boolean AND Component (Multi-Input Pattern)

#### T047: [US2][P] Create boolean-and component structure
#### T048: [US2][P] Implement boolean-and metadata interface
- Inputs: input1 (bool), input2 (bool), input3..N (bool, optional)
- Outputs: result (bool)

#### T049: [US2][P] Implement boolean-and execution logic and tests

Logic: Return true only if ALL inputs are true

Tests:
1. All true → true
2. One false → false
3. Two inputs, both true → true

#### T050: [US2][P] Build and validate boolean-and component

### Boolean OR Component (Multi-Input Pattern)

#### T051: [US2][P] Create boolean-or component structure
#### T052: [US2][P] Implement boolean-or metadata interface
#### T053: [US2][P] Implement boolean-or execution logic and tests

Logic: Return true if ANY input is true

Tests:
1. One true, others false → true
2. All false → false
3. All true → true

#### T054: [US2][P] Build and validate boolean-or component

### Boolean NOT Component (Simple Pattern)

#### T055: [US2][P] Create boolean-not component structure
#### T056: [US2][P] Implement boolean-not metadata interface
- Inputs: input (bool)
- Outputs: result (bool)

#### T057: [US2][P] Implement boolean-not execution logic and tests

Logic: Return !input

Tests:
1. true → false
2. false → true

#### T058: [US2][P] Build and validate boolean-not component

### Boolean XOR Component (Simple Pattern)

#### T059: [US2][P] Create boolean-xor component structure
#### T060: [US2][P] Implement boolean-xor metadata interface
- Inputs: left (bool), right (bool)
- Outputs: result (bool)

#### T061: [US2][P] Implement boolean-xor execution logic and tests

Logic: Return left ^ right

Tests:
1. true XOR false → true
2. true XOR true → false
3. false XOR false → false

#### T062: [US2][P] Build and validate boolean-xor component

### Is Null Component (Optional Input Pattern)

#### T063: [US2][P] Create is-null component structure
#### T064: [US2][P] Implement is-null metadata interface
- Inputs: value (any, **optional**)
- Outputs: result (bool)

#### T065: [US2][P] Implement is-null execution logic and tests

Logic: Return true if input not provided

Tests:
1. No input → true
2. Empty string provided → false
3. Zero provided → false

#### T066: [US2][P] Build and validate is-null component

### Is Empty Component (Type Checking Pattern)

#### T067: [US2][P] Create is-empty component structure
#### T068: [US2][P] Implement is-empty metadata interface
- Inputs: value (any)
- Outputs: result (bool)

#### T069: [US2][P] Implement is-empty execution logic and tests

Logic:
- For strings: length == 0
- For lists: list.len() == 0
- Other types: error

Tests:
1. Empty string → true
2. Non-empty string → false
3. Empty list → true
4. Non-empty list → false
5. Number (error)

#### T070: [US2][P] Build and validate is-empty component

### Story 2 Integration

#### T071: [US2] Create integration test graph for logic operations
**File**: `tests/component_tests/data_validation.json`

Node graph that:
1. Compares two numbers (10 > 5)
2. Compares two strings ("apple" < "banana")
3. Checks if value is null
4. Checks if string is empty
5. Combines conditions with boolean-and
6. Combines conditions with boolean-or
7. Negates result with boolean-not

#### T072: [US2] Manual validation of User Story 2 acceptance scenarios

Verify all 4 acceptance scenarios:
1. ✅ compare with GreaterThan operation
2. ✅ boolean-and returns true only if both inputs true
3. ✅ is-null detects null values
4. ✅ boolean-or combines multiple conditions

**Phase 4 Complete**: ✅ **User Story 2 (P2) - Data Validation Pipeline DELIVERED**

**CHECKPOINT**: User Story 2 complete and independently testable

---

## Phase 5: User Story 3 (P3) - Mathematical Computation Pipeline

**Goal**: Implement 9 math operation components enabling numerical processing workflows

**User Story**: As a wasmflow user, I want to perform advanced mathematical operations beyond basic arithmetic, so that I can build numerical processing and analysis workflows.

**Independent Test**: Create a calculation graph using power, sqrt, trig, and rounding operations.

**Components**: power, sqrt, abs, min, max, floor, ceil, round, trig (9 total)

### Math Power Component

#### T073: [US3][P] Create math-power component structure
**Files**: `components/math/power/{Cargo.toml,build.rs,Justfile,wit/node.wit,src/lib.rs}`

#### T074: [US3][P] Implement math-power metadata interface
- Inputs: base (f32), exponent (f32)
- Outputs: result (f32)

#### T075: [US3][P] Implement math-power execution logic and tests

Logic: Use `.powf()`, handle NaN results

Tests:
1. 2^3 = 8
2. 10^-2 = 0.01
3. 4^0.5 = 2
4. Negative base with fractional exponent (NaN error)

#### T076: [US3][P] Build and validate math-power component

### Math Square Root Component

#### T077: [US3][P] Create math-sqrt component structure
#### T078: [US3][P] Implement math-sqrt metadata interface
- Inputs: value (f32)
- Outputs: result (f32)

#### T079: [US3][P] Implement math-sqrt execution logic and tests

Logic: Use `.sqrt()`, error on negative

Tests:
1. sqrt(16) = 4
2. sqrt(2) = 1.414...
3. sqrt(-1) → error

#### T080: [US3][P] Build and validate math-sqrt component

### Math Absolute Value Component

#### T081: [US3][P] Create math-abs component structure
#### T082: [US3][P] Implement math-abs metadata interface
#### T083: [US3][P] Implement math-abs execution logic and tests

Logic: Use `.abs()`

Tests:
1. abs(-5) = 5
2. abs(5) = 5
3. abs(0) = 0

#### T084: [US3][P] Build and validate math-abs component

### Math Min Component (Multi-Input Pattern)

#### T085: [US3][P] Create math-min component structure
#### T086: [US3][P] Implement math-min metadata interface
- Inputs: input1 (f32), input2 (f32), input3..N (f32, optional)
- Outputs: result (f32)

#### T087: [US3][P] Implement math-min execution logic and tests

Logic: Find minimum of all inputs

Tests:
1. min(5, 2, 8, 1) = 1
2. min(-3, 0, 3) = -3

#### T088: [US3][P] Build and validate math-min component

### Math Max Component (Multi-Input Pattern)

#### T089: [US3][P] Create math-max component structure
#### T090: [US3][P] Implement math-max metadata interface
#### T091: [US3][P] Implement math-max execution logic and tests

Logic: Find maximum of all inputs

Tests:
1. max(5, 2, 8, 1) = 8
2. max(-3, 0, 3) = 3

#### T092: [US3][P] Build and validate math-max component

### Math Floor Component

#### T093: [US3][P] Create math-floor component structure
#### T094: [US3][P] Implement math-floor metadata interface
#### T095: [US3][P] Implement math-floor execution logic and tests

Logic: Use `.floor()`

Tests:
1. floor(3.7) = 3.0
2. floor(-2.3) = -3.0
3. floor(5.0) = 5.0

#### T096: [US3][P] Build and validate math-floor component

### Math Ceiling Component

#### T097: [US3][P] Create math-ceil component structure
#### T098: [US3][P] Implement math-ceil metadata interface
#### T099: [US3][P] Implement math-ceil execution logic and tests

Logic: Use `.ceil()`

Tests:
1. ceil(3.2) = 4.0
2. ceil(-2.7) = -2.0

#### T100: [US3][P] Build and validate math-ceil component

### Math Round Component

#### T101: [US3][P] Create math-round component structure
#### T102: [US3][P] Implement math-round metadata interface
#### T103: [US3][P] Implement math-round execution logic and tests

Logic: Use `.round()`

Tests:
1. round(3.5) = 4.0
2. round(3.4) = 3.0
3. round(-2.5) = -3.0

#### T104: [US3][P] Build and validate math-round component

### Math Trigonometry Component (Operation Selector Pattern)

#### T105: [US3][P] Create math-trig component structure
#### T106: [US3][P] Implement math-trig metadata interface
- Inputs: angle (f32), operation (string: "sin"|"cos"|"tan")
- Outputs: result (f32)

#### T107: [US3][P] Implement math-trig execution logic and tests

Logic:
```rust
match operation.as_str() {
    "sin" => angle.sin(),
    "cos" => angle.cos(),
    "tan" => angle.tan(),
    _ => return Err(/* invalid operation */),
}
```

Tests:
1. sin(0) = 0
2. sin(π/2) ≈ 1
3. cos(π) ≈ -1
4. Invalid operation (error)

#### T108: [US3][P] Build and validate math-trig component

### Story 3 Integration

#### T109: [US3] Create integration test graph for math operations
**File**: `tests/component_tests/math_operations.json`

Node graph that:
1. Calculates power (2^3)
2. Takes square root
3. Gets absolute value
4. Finds min/max of multiple values
5. Rounds, floors, ceils numbers
6. Calculates trigonometric functions

#### T110: [US3] Manual validation of User Story 3 acceptance scenarios

Verify all 5 acceptance scenarios for math operations.

**Phase 5 Complete**: ✅ **User Story 3 (P3) - Mathematical Computation Pipeline DELIVERED**

**CHECKPOINT**: User Story 3 complete and independently testable

---

## Phase 6: User Story 4 (P4) - List Manipulation Pipeline

**Goal**: Implement 7 list operation components enabling collection processing workflows

**User Story**: As a wasmflow user, I want to work with lists of values by accessing, modifying, and analyzing them, so that I can process collections of data within my workflows.

**Independent Test**: Create a graph that builds a list, extracts elements, slices sections, and checks for values.

**Components**: list-length, list-get, list-append, list-join, list-slice, list-contains, list-index-of (7 total)

### List Length Component

#### T111: [US4][P] Create list-length component structure
**Files**: `components/collections/list-length/{Cargo.toml,build.rs,Justfile,wit/node.wit,src/lib.rs}`

#### T112: [US4][P] Implement list-length metadata interface
- Inputs: list (list<any>)
- Outputs: length (u32)

#### T113: [US4][P] Implement list-length execution logic and tests

Logic: Return list.len()

Tests:
1. List of 4 elements → 4
2. Empty list → 0

#### T114: [US4][P] Build and validate list-length component

### List Get Component

#### T115: [US4][P] Create list-get component structure
#### T116: [US4][P] Implement list-get metadata interface
- Inputs: list (list<any>), index (u32)
- Outputs: element (any)

#### T117: [US4][P] Implement list-get execution logic and tests

Logic: Return list[index], error if out of bounds

Tests:
1. Get element at index 1
2. Get element at index 0
3. Index out of bounds (error)

#### T118: [US4][P] Build and validate list-get component

### List Append Component

#### T119: [US4][P] Create list-append component structure
#### T120: [US4][P] Implement list-append metadata interface
- Inputs: list (list<any>), value (any)
- Outputs: result (list<any>)

#### T121: [US4][P] Implement list-append execution logic and tests

Logic: Create new list with value appended (immutable)

Tests:
1. Append to existing list
2. Append to empty list

#### T122: [US4][P] Build and validate list-append component

### List Join Component

#### T123: [US4][P] Create list-join component structure
#### T124: [US4][P] Implement list-join metadata interface
- Inputs: list (list<string>), delimiter (string)
- Outputs: result (string)

#### T125: [US4][P] Implement list-join execution logic and tests

Logic: Join list elements with delimiter

Tests:
1. Join with comma
2. Join with space
3. Empty list → empty string

#### T126: [US4][P] Build and validate list-join component

### List Slice Component

#### T127: [US4][P] Create list-slice component structure
#### T128: [US4][P] Implement list-slice metadata interface
- Inputs: list (list<any>), start (u32), end (u32, optional)
- Outputs: result (list<any>)

#### T129: [US4][P] Implement list-slice execution logic and tests

Logic: Return list[start..end]

Tests:
1. Slice with start and end
2. Slice from start to end of list
3. Start beyond end → empty list

#### T130: [US4][P] Build and validate list-slice component

### List Contains Component

#### T131: [US4][P] Create list-contains component structure
#### T132: [US4][P] Implement list-contains metadata interface
- Inputs: list (list<any>), value (any)
- Outputs: result (bool)

#### T133: [US4][P] Implement list-contains execution logic and tests

Logic: Return true if value in list

Tests:
1. Value found → true
2. Value not found → false

#### T134: [US4][P] Build and validate list-contains component

### List Index Of Component

#### T135: [US4][P] Create list-index-of component structure
#### T136: [US4][P] Implement list-index-of metadata interface
- Inputs: list (list<any>), value (any)
- Outputs: index (i32)

#### T137: [US4][P] Implement list-index-of execution logic and tests

Logic: Return index of first occurrence, -1 if not found

Tests:
1. Value found → return index
2. First occurrence of duplicate
3. Value not found → -1

#### T138: [US4][P] Build and validate list-index-of component

### Story 4 Integration

#### T139: [US4] Create integration test graph for list operations
**File**: `tests/component_tests/list_manipulation.json`

Node graph that:
1. Gets list length
2. Gets element at index
3. Appends value
4. Slices portion
5. Joins to string
6. Checks contains
7. Gets index of value

#### T140: [US4] Manual validation of User Story 4 acceptance scenarios

Verify all 5 acceptance scenarios for list operations.

**Phase 6 Complete**: ✅ **User Story 4 (P4) - List Manipulation Pipeline DELIVERED**

**CHECKPOINT**: User Story 4 complete and independently testable

---

## Phase 7: User Story 5 (P5) - Data Transformation Pipeline

**Goal**: Implement 4 data transformation components enabling type conversion and formatting workflows

**User Story**: As a wasmflow user, I want to convert data between different types and formats, so that I can integrate different data sources and prepare data for output.

**Independent Test**: Create a graph that converts between types (to-string, parse-number), formats templates, and serializes to JSON.

**Components**: json-stringify, to-string, parse-number, format-template (4 total)

### JSON Stringify Component (Complex Type Handling)

#### T141: [US5][P] Create json-stringify component structure
**Files**: `components/data/json-stringify/{Cargo.toml,build.rs,Justfile,wit/node.wit,src/lib.rs}`

#### T142: [US5][P] Add serde_json dependency to json-stringify
**File**: `components/data/json-stringify/Cargo.toml`
```toml
[dependencies]
wit-bindgen = "0.30"
serde_json = "1.0"
```

#### T143: [US5][P] Implement json-stringify metadata interface
- Inputs: data (any - Record, List, or primitive)
- Outputs: json (string)

#### T144: [US5][P] Implement json-stringify execution logic and tests

Logic: Use serde_json to serialize

Tests:
1. Serialize Record
2. Serialize List
3. Serialize string primitive

#### T145: [US5][P] Build and validate json-stringify component

### To String Component

#### T146: [US5][P] Create to-string component structure
**Files**: `components/data/to-string/{Cargo.toml,build.rs,Justfile,wit/node.wit,src/lib.rs}`

#### T147: [US5][P] Implement to-string metadata interface
- Inputs: value (any)
- Outputs: text (string)

#### T148: [US5][P] Implement to-string execution logic and tests

Logic:
- Numbers → format as decimal
- Booleans → "true"/"false"
- Strings → pass-through
- Other → error

Tests:
1. Number to string
2. Boolean to string
3. String pass-through
4. Unsupported type (error)

#### T149: [US5][P] Build and validate to-string component

### Parse Number Component

#### T150: [US5][P] Create parse-number component structure
#### T151: [US5][P] Implement parse-number metadata interface
- Inputs: text (string)
- Outputs: number (f32)

#### T152: [US5][P] Implement parse-number execution logic and tests

Logic: Parse string to f32, handle scientific notation

Tests:
1. Parse "42"
2. Parse "3.14"
3. Parse "1.5e2"
4. Invalid input (error)

#### T153: [US5][P] Build and validate parse-number component

### Format Template Component (Complex Pattern)

#### T154: [US5][P] Create format-template component structure
#### T155: [US5][P] Implement format-template metadata interface
- Inputs: template (string), values (list<string>)
- Outputs: result (string)

#### T156: [US5][P] Implement format-template execution logic and tests

Logic: Replace {0}, {1}, {2} with values from list

Tests:
1. Format template with multiple placeholders
2. Unused placeholders remain
3. Missing values leave placeholders

#### T157: [US5][P] Build and validate format-template component

### Story 5 Integration

#### T158: [US5] Create integration test graph for data transformation
**File**: `tests/component_tests/data_transformation.json`

Node graph that:
1. Converts number to string
2. Parses number from string
3. Serializes data to JSON
4. Formats template string

#### T159: [US5] Manual validation of User Story 5 acceptance scenarios

Verify all 4 acceptance scenarios for data transformation.

**Phase 7 Complete**: ✅ **User Story 5 (P5) - Data Transformation Pipeline DELIVERED**

**CHECKPOINT**: User Story 5 complete and independently testable. ALL USER STORIES COMPLETE!

---

## Phase 8: Polish & Integration

**Goal**: Final validation, documentation, and cross-cutting improvements

### Final Validation

#### T160: Batch build all components
```bash
cd components
just build-all
just install-all
```

Verify: 34 .wasm files in `components/bin/`

#### T161: Run all unit tests
```bash
cd components
just test-all
```

Verify: All tests pass

#### T162: Load all components in wasmflow UI
1. Launch wasmflow
2. Verify all 34 components appear in palette
3. Verify correct categories:
   - Text: 7 components
   - Logic: 7 components
   - Math: 9 components
   - Collections: 7 components
   - Data: 4 components

#### T163: Execute all integration test graphs
1. Load `string_processing.json` → Execute successfully
2. Load `data_validation.json` → Execute successfully
3. Load `math_operations.json` → Execute successfully
4. Load `list_manipulation.json` → Execute successfully
5. Load `data_transformation.json` → Execute successfully

### Documentation

#### T164: Create component library README
**File**: `components/README.md`

Document:
- Overview of core library
- Component categories
- Build instructions
- Testing procedures
- Adding new components

#### T165: Update main repository README
**File**: `README.md`

Add section about core library components.

#### T166: Create performance benchmarks
**File**: `tests/benchmarks/component_performance.rs`

Benchmark each component category:
- String operations: measure <10ms target
- Math operations: measure <10ms target
- List operations with 1000 elements: measure <100ms target

### Performance Validation

#### T167: Verify binary sizes
```bash
ls -lh components/bin/*.wasm
```

Check: All components <200KB (most should be 50-150KB)

#### T168: Profile component execution times
Create test graphs with large inputs:
- 1MB string for string operations
- 1000-element lists for list operations
- Complex calculations for math operations

Verify: All operations complete in <10ms

### Final Integration

#### T169: Create comprehensive example workflow
**File**: `tests/component_tests/comprehensive_workflow.json`

Build a complex workflow using components from all categories:
1. Load text data
2. Parse and validate
3. Perform calculations
4. Transform results
5. Format output

This demonstrates the power of composing all 34 components together.

#### T170: Update CLAUDE.md with component development patterns
Document the component development workflow and patterns discovered during implementation.

**Phase 8 Complete**: ✅ **All 34 components built, tested, validated, and documented!**

---

## Dependencies & Execution Flow

### Critical Path

```
Phase 1 (Setup)
  ↓
Phase 2 (Foundational) ← BLOCKING - MUST complete before user stories
  ↓
┌─────────────────────────────────────┐
│  User Stories (Can execute in any  │
│  order after Phase 2 complete)      │
├─────────────────────────────────────┤
│  Phase 3 (US1 - P1) → 7 components  │
│  Phase 4 (US2 - P2) → 7 components  │
│  Phase 5 (US3 - P3) → 9 components  │
│  Phase 6 (US4 - P4) → 7 components  │
│  Phase 7 (US5 - P5) → 4 components  │
└─────────────────────────────────────┘
  ↓
Phase 8 (Polish & Integration)
```

### User Story Dependencies

**Independent Stories** (can be implemented in parallel after Phase 2):
- US1 (Text Processing) - No dependencies
- US2 (Data Validation) - No dependencies
- US3 (Math Computation) - No dependencies
- US4 (List Manipulation) - No dependencies
- US5 (Data Transformation) - No dependencies

**All user stories are independent!** This enables:
- Parallel development by multiple developers
- Incremental delivery (ship P1 first, then P2, etc.)
- Early user feedback on each priority level

---

## Parallel Execution Examples

### Phase 2 (Foundational)
**Can do in parallel**:
- T006-T011 [P] (All template creation tasks)

### Phase 3 (User Story 1)
**Can do in parallel** (all marked [P]):
- T013-T016 (string-concat)
- T017-T020 (string-split)
- T021-T024 (string-length)
- T025-T028 (string-trim)
- T029-T032 (string-case)
- T033-T036 (string-contains)
- T037-T040 (string-substring)

**Total parallelization**: 7 components can be built simultaneously!

### Phase 4 (User Story 2)
**Can do in parallel**:
- T043-T046 (compare)
- T047-T050 (boolean-and)
- T051-T054 (boolean-or)
- T055-T058 (boolean-not)
- T059-T062 (boolean-xor)
- T063-T066 (is-null)
- T067-T070 (is-empty)

**Total parallelization**: 7 components can be built simultaneously!

### All User Stories
Each user story phase has maximum parallelization within the phase. If you have multiple developers:
- Developer 1: Implements all P1 components (Phase 3)
- Developer 2: Implements all P2 components (Phase 4)
- Developer 3: Implements all P3 components (Phase 5)
- Developer 4: Implements P4 + P5 components (Phases 6-7)

This enables **4x parallelization** at the user story level!

---

## Implementation Strategy

### MVP Approach

**Minimum Viable Product** = User Story 1 (P1) only
- 7 string operation components
- Enables text processing workflows
- Can be shipped independently
- Provides immediate user value
- Tasks T001-T042 (42 tasks)

**Timeline**: 1-2 days for MVP

### Incremental Delivery

**Release 1** (MVP): P1 - Text Processing (7 components)
**Release 2**: P1 + P2 - Add Data Validation (14 components total)
**Release 3**: P1 + P2 + P3 - Add Math Operations (23 components total)
**Release 4**: P1-P4 - Add List Operations (30 components total)
**Release 5** (Complete): P1-P5 - All components (34 components total)

### Recommended Order

For single developer:
1. Complete Phase 1 (Setup) - 30 minutes
2. Complete Phase 2 (Foundational) - 2-3 hours
3. Implement P1 components (Phase 3) - 1 day
4. Validate and test P1 - 2 hours
5. Ship MVP Release 1
6. Implement P2 components (Phase 4) - 1 day
7. Implement P3 components (Phase 5) - 1 day
8. Implement P4 components (Phase 6) - 1 day
9. Implement P5 components (Phase 7) - 4 hours
10. Final polish (Phase 8) - 4 hours

**Total**: 5-6 days for complete implementation

---

## Success Criteria

### Per User Story

**User Story 1 (P1) - Text Processing**:
- ✅ All 7 string components build successfully
- ✅ All unit tests pass (3+ tests per component = 21+ tests)
- ✅ Components appear in "Text" category in UI
- ✅ Integration test graph executes successfully
- ✅ All 5 acceptance scenarios validated manually

**User Story 2 (P2) - Data Validation**:
- ✅ All 7 logic components build successfully
- ✅ All unit tests pass (21+ tests)
- ✅ Components appear in "Logic" category
- ✅ Integration test graph executes successfully
- ✅ All 4 acceptance scenarios validated

**User Story 3 (P3) - Math Operations**:
- ✅ All 9 math components build successfully
- ✅ All unit tests pass (27+ tests)
- ✅ Components appear in "Math" category
- ✅ Integration test graph executes successfully
- ✅ All 5 acceptance scenarios validated

**User Story 4 (P4) - List Operations**:
- ✅ All 7 list components build successfully
- ✅ All unit tests pass (21+ tests)
- ✅ Components appear in "Collections" category
- ✅ Integration test graph executes successfully
- ✅ All 5 acceptance scenarios validated

**User Story 5 (P5) - Data Transformation**:
- ✅ All 4 data components build successfully
- ✅ All unit tests pass (12+ tests)
- ✅ Components appear in "Data" category
- ✅ Integration test graph executes successfully
- ✅ All 4 acceptance scenarios validated

### Overall Project

- ✅ 34 components compiled to .wasm files in `components/bin/`
- ✅ All components <200KB in size
- ✅ 100+ unit tests passing
- ✅ 5 integration test graphs executing successfully
- ✅ All components load in wasmflow UI without errors
- ✅ Performance targets met (<10ms per operation)
- ✅ Comprehensive documentation completed
- ✅ All functional requirements (FR-001 through FR-037) satisfied
- ✅ All success criteria (SC-001 through SC-008) achieved

---

## Task Summary

| Phase | User Story | Task Count | Parallelizable | Components |
|-------|------------|------------|----------------|------------|
| 1 | Setup | 5 | 4 | 0 |
| 2 | Foundational | 7 | 6 | 0 |
| 3 | US1 (P1) | 30 | 28 | 7 |
| 4 | US2 (P2) | 30 | 28 | 7 |
| 5 | US3 (P3) | 38 | 36 | 9 |
| 6 | US4 (P4) | 30 | 28 | 7 |
| 7 | US5 (P5) | 19 | 17 | 4 |
| 8 | Polish | 11 | 0 | 0 |
| **Total** | | **170** | **147** | **34** |

**Parallelization Potential**: 86% of tasks can be executed in parallel (147/170)

---

## Quick Start

To begin implementation:

```bash
# 1. Ensure you're on the feature branch
git checkout 010-wasm-components-core

# 2. Run Phase 1 tasks (T001-T005)
mkdir -p components/{core,math,collections,data}
mkdir -p tests/{integration,component_tests}
# ... (continue with setup tasks)

# 3. Run Phase 2 tasks (T006-T012)
mkdir -p components/.templates
# Create template files (copy from quickstart.md or existing components)

# 4. Validate build system
# Create test component, build, test, delete

# 5. Begin User Story 1 (T013-T042)
# Implement first component (string-concat)
# Follow pattern for remaining 6 components

# 6. Checkpoint validation after each user story
# Run integration tests
# Manual validation of acceptance scenarios

# 7. Continue through remaining user stories (P2-P5)

# 8. Final polish and validation (Phase 8)
```

**Ready to start!** Begin with Phase 1: Setup tasks.
