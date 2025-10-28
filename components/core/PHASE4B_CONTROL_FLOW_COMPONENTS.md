# Phase 4B: Control Flow Logic Components

**Status**: Implementation Complete
**Date**: 2025-10-27
**Components**: 6 control flow/routing components for conditional logic and data routing

## Overview

Phase 4B adds 6 control flow components that extend the logic capabilities of WasmFlow with explicit routing and conditional data flow. These components complement the existing boolean logic components (Phase 4) by enabling value selection, conditional routing, type validation, and pattern-based switching.

### Purpose

> Enable users to create sophisticated data routing pipelines with explicit control flow, including ternary selection, conditional routing, filtering, runtime type validation, and pattern-based switching for both strings and numbers.

## Components Implemented

All 6 control flow components have been implemented with complete source code and WIT specifications:

### 1. Select (Ternary Operator) ✅
**Location**: `components/core/select/`
**Binary**: `components/bin/select.wasm` (61KB)

**Functionality**:
- Selects between two values based on a boolean condition
- Implements ternary operator pattern: `condition ? true-value : false-value`
- Supports any type for both value inputs (types can differ)

**Inputs**:
- condition (bool) - Boolean condition to evaluate
- true-value (any) - Value to return if condition is true
- false-value (any) - Value to return if condition is false

**Outputs**:
- result (any) - Selected value based on condition

**Use Cases**:
- Choosing between two configuration values
- Selecting processing paths based on conditions
- Default value fallback logic

**Tests**: 6 unit tests covering true/false conditions, numeric values, mixed types, missing inputs, and invalid types

### 2. If-Then-Else (Conditional Routing) ✅
**Location**: `components/core/if-then-else/`
**Binary**: `components/bin/if_then_else.wasm` (60KB)

**Functionality**:
- Routes different values to different outputs based on a boolean condition
- If condition is true, outputs then-value to then-output
- If condition is false, outputs else-value to else-output
- Enables graph branching with different values for each branch
- Only outputs to the selected path (then OR else, not both)

**Inputs**:
- condition (bool) - Boolean condition to evaluate
- then-value (any) - Value to send to then-output when condition is true
- else-value (any) - Value to send to else-output when condition is false

**Outputs**:
- then-output (any, optional) - Receives then-value when condition is true
- else-output (any, optional) - Receives else-value when condition is false

**Use Cases**:
- Branching execution paths with different data for each branch
- Routing success data vs. error messages to different handlers
- Conditional pipeline routing with branch-specific values
- HTTP response routing (200 OK data vs. 500 error data)

**Tests**: 8 unit tests covering routing to then/else, different types, numeric values, missing inputs, and invalid types

### 3. Conditional Pass (Filter/Gate) ✅
**Location**: `components/core/conditional-pass/`
**Binary**: `components/bin/conditional_pass.wasm` (60KB)

**Functionality**:
- Passes a value through if condition is true, otherwise blocks (no output)
- Acts as a data filter or gate
- Downstream nodes only execute when condition is true

**Inputs**:
- condition (bool) - Boolean condition to evaluate
- value (any) - Value to pass through if condition is true

**Outputs**:
- result (any, optional) - Value when condition is true, nothing when false

**Use Cases**:
- Filtering data streams based on conditions
- Gating execution of expensive operations
- Conditional data validation pipelines

**Tests**: 6 unit tests covering pass/block behavior, numbers, lists, missing inputs, and invalid types

### 4. Type Check (Runtime Validation) ✅
**Location**: `components/core/type-check/`
**Binary**: `components/bin/type_check.wasm` (61KB)

**Functionality**:
- Validates that a value matches an expected type at runtime
- Returns both validation result and actual type
- Supports all WIT value types: u32, i32, f32, string, bool, binary, string-list, u32-list, f32-list

**Inputs**:
- value (any) - Value to check
- expected-type (string) - Expected type name

**Outputs**:
- is-valid (bool) - True if value matches expected type
- actual-type (string) - Actual type of the value

**Use Cases**:
- Runtime type safety checks
- Data validation before processing
- Type-aware routing and error handling
- Debugging type mismatches in graphs

**Tests**: 6 unit tests covering valid/invalid types, all 9 type variants, and missing inputs

### 5. Switch String (String-Based Routing) ✅
**Location**: `components/core/switch-string/`
**Binary**: `components/bin/switch_string.wasm` (66KB)

**Functionality**:
- Routes data based on string pattern matching
- Compares input value against up to 4 case patterns
- Returns output associated with first matching case, or default if no match
- First match wins (cases checked in order 1-4)

**Inputs**:
- value (string) - String value to match against cases
- case1-4 (string, optional) - Patterns to match
- output1-4 (any, optional) - Values to return if corresponding case matches
- default (any) - Default value when no cases match

**Outputs**:
- result (any) - Matched output value or default

**Use Cases**:
- Status code routing ("success", "error", "pending")
- Command dispatching
- State machine transitions
- HTTP method routing

**Tests**: 7 unit tests covering all cases, default fallback, first match wins, missing inputs, and case without output errors

### 6. Switch Number (Numeric-Based Routing) ✅
**Location**: `components/core/switch-number/`
**Binary**: `components/bin/switch_number.wasm` (102KB)

**Functionality**:
- Routes data based on numeric value matching
- Handles u32, i32, and f32 inputs (auto-converts to f32 for comparison)
- Compares input against up to 4 case numbers
- Returns output associated with first matching case, or default if no match
- Uses floating-point epsilon for comparison accuracy

**Inputs**:
- value (any: u32, i32, or f32) - Numeric value to match
- case1-4 (f32, optional) - Numbers to match
- output1-4 (any, optional) - Values to return if corresponding case matches
- default (any) - Default value when no cases match

**Outputs**:
- result (any) - Matched output value or default

**Use Cases**:
- HTTP status code routing (200, 404, 500)
- Error code handling
- Numeric state routing
- Priority-based processing

**Tests**: 8 unit tests covering u32/i32/f32 inputs, all cases, default fallback, first match wins, missing/invalid inputs, and case without output errors

## Integration Test

**File**: `/Users/doronila/git/wasmflow/tests/component_tests/control_flow.json`

A comprehensive integration test graph demonstrating all 6 control flow components in practical scenarios:

### Test Scenarios:

1. **Type Validation & Selection**:
   - Type-check validates input is u32
   - Select chooses "valid" or "invalid" message based on type check result

2. **String-Based Routing**:
   - Switch-string routes "success" status to HTTP 200
   - Routes "error" status to HTTP 500
   - Falls back to 404 for unknown statuses

3. **Number-Based Routing**:
   - Switch-number routes HTTP code 200 to "OK" message
   - Routes 404 to "Error" message
   - Falls back to "Unknown" for other codes

4. **Conditional Filtering**:
   - Conditional-pass filters data based on boolean condition
   - Only passes value when condition is true

5. **If-Then-Else Routing**:
   - If-then-else routes data to then-output or else-output
   - Enables branching execution in graph

## Implementation Notes

### Design Decisions

**Separate Switch Components**: Implemented switch-string and switch-number as separate components (not a single generic switch) for:
- Clearer type safety and user expectations
- Simpler configuration without type ambiguity
- Better error messages specific to string vs. number comparison

**Default Value Approach**: Both switch components require a default value input (not error, not null) for:
- Maximum flexibility in handling unmatched cases
- Consistent behavior across all scenarios
- Allows users to provide appropriate fallback values

**First Match Wins**: Both switch components use first-match-wins semantics:
- Predictable, well-understood behavior from traditional switch statements
- Allows users to order cases by priority
- No ambiguity when multiple cases could match

**Optional Outputs**: If-then-else has optional outputs to enable:
- Flexible graph connections (don't need to connect unused paths)
- Clear execution semantics (only connected path executes)

### Binary Sizes

All components optimized for size with LTO and stripping:
- select: 61KB
- if-then-else: 60KB
- conditional-pass: 60KB
- type-check: 61KB
- switch-string: 66KB
- switch-number: 102KB (larger due to f32 conversion logic)

Total: ~410KB for all 6 components

### Build Configuration

Standard build configuration used across all components:

```toml
[profile.release]
opt-level = "s"    # Optimize for size
lto = true         # Link-time optimization
strip = true       # Strip symbols
```

### Testing Notes

Unit tests verify:
- Typical usage scenarios
- Edge cases (empty inputs, boundary values)
- Error handling (wrong types, missing inputs)
- Case ordering (first match wins)
- Optional input handling

Integration test verifies:
- All components work correctly in WasmFlow runtime
- Type checking and validation
- Routing and switching logic
- Filtering and conditional execution

## Usage Examples

### Example 1: Configuration Selection

```
[condition: is_production] → [select]
[prod_config] → [select.true-value]
[dev_config] → [select.false-value]
[select.result] → [downstream]
```

### Example 2: Error Routing

```
[is_valid] → [if-then-else.condition]
[success_data] → [if-then-else.then-value]
[error_message] → [if-then-else.else-value]
[if-then-else.then-output] → [process_success]
[if-then-else.else-output] → [handle_error]
```

### Example 3: Status Code Router

```
[http_status: "success"] → [switch-string.value]
["success"] → [switch-string.case1]
[200] → [switch-string.output1]
["error"] → [switch-string.case2]
[500] → [switch-string.output2]
[404] → [switch-string.default]
[switch-string.result] → [send_response]
```

### Example 4: Type-Safe Processing

```
[input] → [type-check.value]
["u32"] → [type-check.expected-type]
[type-check.is-valid] → [conditional-pass.condition]
[input] → [conditional-pass.value]
[conditional-pass.result] → [process_u32]
```

## Integration with Existing Components

These control flow components integrate seamlessly with Phase 4 logic components:

- **compare** outputs can drive select, if-then-else, and conditional-pass conditions
- **boolean-and/or** can combine multiple conditions before routing
- **type-check** enables type-aware routing with switch components
- **is-null/is-empty** can gate execution with conditional-pass

Example combined workflow:
```
[input] → [is-null] → [boolean-not] → [conditional-pass.condition]
[input] → [conditional-pass.value]
[conditional-pass.result] → [type-check]
[type-check.is-valid] → [if-then-else.condition]
```

## Future Enhancements

Potential additions for more advanced control flow:

1. **Multi-output router**: Route single input to one of N outputs based on conditions
2. **Range-based switch**: Match numeric ranges instead of exact values
3. **Regex switch**: String matching with regular expressions
4. **Coalesce**: Return first non-null value from multiple inputs (SQL COALESCE)
5. **Try-catch**: Error handling with fallback values

## Conclusion

Phase 4B successfully extends WasmFlow's logic capabilities with 6 production-ready control flow components. These components enable sophisticated conditional routing, type-safe data pipelines, and pattern-based switching, completing the core set of logic operations needed for complex workflows.

**Total Logic Components**: 13 (7 from Phase 4 + 6 from Phase 4B)

**Categories**:
- Boolean operations: AND, OR, NOT, XOR (4 components)
- Comparison: compare (1 component)
- Type checking: is-null, is-empty, type-check (3 components)
- Value selection: select (1 component)
- Conditional routing: if-then-else, conditional-pass (2 components)
- Pattern switching: switch-string, switch-number (2 components)
