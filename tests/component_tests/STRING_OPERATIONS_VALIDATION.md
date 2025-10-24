# String Operations Integration Test - Manual Validation Guide

**Feature**: WASM Components Core Library - User Story 1 (P1)
**Components**: String operations (7 components)
**Test File**: `string_processing.json`
**Created**: 2025-10-23

## Overview

This document provides step-by-step instructions for manually validating the 7 string operation components in the wasmflow UI. This validation corresponds to **Task T042** in the implementation plan.

## Prerequisites

Before beginning validation:

1. **Build all components**:
   ```bash
   cd /home/user/wasmflow/components/core

   # Build each component individually
   cd string-concat && cargo build --target wasm32-wasip2 --release && cd ..
   cd string-split && cargo build --target wasm32-wasip2 --release && cd ..
   cd string-length && cargo build --target wasm32-wasip2 --release && cd ..
   cd string-trim && cargo build --target wasm32-wasip2 --release && cd ..
   cd string-case && cargo build --target wasm32-wasip2 --release && cd ..
   cd string-contains && cargo build --target wasm32-wasip2 --release && cd ..
   cd string-substring && cargo build --target wasm32-wasip2 --release && cd ..
   ```

2. **Copy WASM files to wasmflow components directory** (location TBD based on wasmflow configuration)

3. **Launch wasmflow**:
   ```bash
   cd /home/user/wasmflow
   cargo run --release
   ```

## Test Scenarios

### Scenario 1: Individual Component Testing

Test each component in isolation before integration testing.

#### 1.1 String Trim

**Setup**:
- Add a Constant (String) node with value: `"  Hello World  "`
- Add a String Trim component
- Connect Constant.value → String Trim.text

**Execute & Validate**:
- ✅ Expected output: `"Hello World"` (no leading/trailing spaces)
- ✅ Verify whitespace is removed from both ends
- ✅ Verify middle whitespace is preserved

**Test Variations**:
- Test with tabs and newlines: `"\t\nHello\t\n"`
- Test with empty string: `""`
- Test with only whitespace: `"   "`

#### 1.2 String Case

**Setup**:
- Add a Constant (String) node with value: `"Hello World"`
- Add a Constant (String) node with value: `"lowercase"` (operation)
- Add a String Case component
- Connect inputs accordingly

**Execute & Validate**:
- ✅ Expected output: `"hello world"`

**Test Variations**:
- Test uppercase operation: Expected `"HELLO WORLD"`
- Test titlecase operation: Expected `"Hello World"`
- Test invalid operation: Should produce error with helpful message

#### 1.3 String Concat

**Setup**:
- Add two Constant (String) nodes: `"Hello"` and `" World"`
- Add a String Concat component
- Connect both strings to text1 and text2 inputs

**Execute & Validate**:
- ✅ Expected output: `"Hello World"`

**Test Variations**:
- Test with 3+ optional inputs
- Test with empty strings
- Test with Unicode: `"🚀"` + `"🌟"`

#### 1.4 String Split

**Setup**:
- Add a Constant (String) node with value: `"one,two,three"`
- Add a Constant (String) node with value: `","` (delimiter)
- Add a String Split component
- Connect inputs

**Execute & Validate**:
- ✅ Expected output: List containing `["one", "two", "three"]`
- ✅ Verify output is a List type
- ✅ Verify correct element count

**Test Variations**:
- Test with empty delimiter (should split into characters)
- Test with consecutive delimiters: `"a,,b"` → `["a", "", "b"]`
- Test with delimiter not found: Should return single-element list

#### 1.5 String Length

**Setup**:
- Add a Constant (String) node with value: `"Hello World"`
- Add a String Length component
- Connect input

**Execute & Validate**:
- ✅ Expected output: `11` (U32)
- ✅ Verify count is Unicode characters, not bytes

**Test Variations**:
- Test with Unicode emojis: `"🚀🌟"` → Expected `2` (not 8 bytes)
- Test with empty string: Expected `0`
- Test with multi-byte characters: `"Café"` → Expected `4`

#### 1.6 String Contains

**Setup**:
- Add a Constant (String) node with value: `"Hello World"`
- Add a Constant (String) node with value: `"World"`
- Add a String Contains component
- Connect inputs

**Execute & Validate**:
- ✅ Expected output: `true` (Bool)

**Test Variations**:
- Test case sensitivity: `"world"` → Expected `false`
- Test substring not found: `"xyz"` → Expected `false`
- Test empty substring: `""` → Expected `true` (always contained)

#### 1.7 String Substring

**Setup**:
- Add a Constant (String) node with value: `"Hello World"`
- Add a Constant (U32) node with value: `0` (start)
- Add a Constant (U32) node with value: `5` (length)
- Add a String Substring component
- Connect inputs

**Execute & Validate**:
- ✅ Expected output: `"Hello"`

**Test Variations**:
- Test without length (to end): start=6 → Expected `"World"`
- Test start beyond end: start=100 → Expected `""` (empty string)
- Test Unicode: `"🚀🌟✨"`, start=1, length=2 → Expected `"🌟✨"`

---

### Scenario 2: Integration Pipeline Testing

Use the provided `string_processing.json` graph to test all components working together.

#### 2.1 Load Integration Test Graph

**Steps**:
1. In wasmflow UI, select "File" → "Import Graph" (or equivalent)
2. Navigate to `/home/user/wasmflow/tests/component_tests/string_processing.json`
3. Load the graph
4. Verify all 14 nodes are loaded and connected correctly

**Expected Layout**:
```
Input Text → Trim → Case → Concat ──→ Split
                              ↓       ↓
                           Length  Contains
                              ↓       ↓
                          Substring  [outputs]
```

#### 2.2 Execute Integration Test

**Steps**:
1. Click the "Execute" button in wasmflow UI
2. Observe execution flow (nodes should highlight as they execute)
3. Check all nodes reach "Completed" state

**Expected Results**:

| Node | Output Port | Expected Value | Validation |
|------|-------------|----------------|------------|
| Raw Input Text | value | `"  Hello WORLD! Welcome to WasmFlow.  "` | ✅ Initial input |
| String Trim | result | `"Hello WORLD! Welcome to WasmFlow."` | ✅ Whitespace removed |
| String Case | result | `"hello world! welcome to wasmflow."` | ✅ Converted to lowercase |
| String Concat | result | `"hello world! welcome to wasmflow. - processed by wasmflow"` | ✅ Suffix appended |
| String Split | result | `["hello", "world!", "welcome", "to", "wasmflow.", "-", "processed", "by", "wasmflow"]` | ✅ 9 elements |
| String Length | result | `57` | ✅ Character count |
| String Contains | result | `true` | ✅ Found "wasmflow" |
| String Substring | result | `"hello world! we"` | ✅ First 15 chars |

#### 2.3 Verify Data Types

For each output, verify the data type matches the specification:
- String outputs should display as text
- U32 outputs should display as positive integers
- Bool outputs should display as true/false
- List outputs should display as array notation

---

### Scenario 3: Error Handling Validation

Test that components produce helpful error messages.

#### 3.1 Missing Required Input

**Setup**:
- Add a String Trim component without connecting the text input
- Attempt to execute

**Expected Behavior**:
- ✅ Execution fails with error
- ✅ Error message identifies missing input: `"Missing or invalid 'text' input"`
- ✅ Error provides recovery hint: `"Provide a string value"`

#### 3.2 Invalid Input Type

**Setup**:
- Add a String Length component
- Connect a U32 constant (number) instead of String to text input
- Execute

**Expected Behavior**:
- ✅ Execution fails with type mismatch error
- ✅ Error message is clear about expected type

#### 3.3 Invalid Operation

**Setup**:
- Add a String Case component
- Set operation to `"invalid"` (not uppercase/lowercase/titlecase)
- Execute

**Expected Behavior**:
- ✅ Execution fails with validation error
- ✅ Error message lists valid operations
- ✅ Recovery hint provided

---

### Scenario 4: Unicode Handling Validation

Verify all components handle Unicode correctly.

#### 4.1 Multi-byte Characters

**Setup**:
- Create a pipeline: Input `"Café ☕"` → Length → Substring
- Set substring start=0, length=4

**Expected Results**:
- ✅ Length outputs `6` (not byte count)
- ✅ Substring outputs `"Café"` (correct character extraction)

#### 4.2 Emoji Handling

**Setup**:
- Create a pipeline: Input `"🚀 Launch"` → Split (delimiter=" ") → Length
- Measure length of `"🚀"`

**Expected Results**:
- ✅ Split produces two elements: `["🚀", "Launch"]`
- ✅ Length of `"🚀"` is `1` (not 4 bytes)

---

## Acceptance Criteria Checklist

Use this checklist to confirm User Story 1 is complete:

### Functional Requirements
- [ ] All 7 string components load successfully in wasmflow
- [ ] Each component appears in the component palette
- [ ] Component metadata (name, description, version) displays correctly
- [ ] All input ports are correctly labeled and typed
- [ ] All output ports are correctly labeled and typed

### Execution Requirements
- [ ] Individual components execute without errors
- [ ] Components handle required inputs correctly
- [ ] Components handle optional inputs correctly
- [ ] Integration pipeline executes end-to-end
- [ ] Execution produces correct output values
- [ ] Execution completes within reasonable time (<1s for test data)

### Error Handling Requirements
- [ ] Missing inputs produce clear error messages
- [ ] Invalid input types produce clear error messages
- [ ] Invalid operations produce clear error messages
- [ ] Errors include input_name for debugging
- [ ] Errors include recovery_hint for user guidance

### Data Integrity Requirements
- [ ] Unicode characters are handled correctly (not byte-based)
- [ ] Empty strings are handled correctly
- [ ] Whitespace-only strings are handled correctly
- [ ] Very long strings (>1000 chars) execute correctly
- [ ] Special characters don't cause crashes

### Integration Requirements
- [ ] Components can be chained together
- [ ] Data flows correctly between components
- [ ] Complex pipelines (5+ components) execute correctly
- [ ] Multiple outputs from same node work correctly
- [ ] Branching pipelines work correctly

### Documentation Requirements
- [ ] This validation guide is complete and accurate
- [ ] Integration test graph is documented
- [ ] Expected results are clearly specified
- [ ] Error cases are documented

---

## Test Results Template

Use this template to record validation results:

```
## Test Execution Report

**Date**: _______________
**Tester**: _______________
**Wasmflow Version**: _______________
**Rust Version**: _______________

### Scenario 1: Individual Component Testing
- [ ] 1.1 String Trim: ______ (PASS/FAIL)
- [ ] 1.2 String Case: ______ (PASS/FAIL)
- [ ] 1.3 String Concat: ______ (PASS/FAIL)
- [ ] 1.4 String Split: ______ (PASS/FAIL)
- [ ] 1.5 String Length: ______ (PASS/FAIL)
- [ ] 1.6 String Contains: ______ (PASS/FAIL)
- [ ] 1.7 String Substring: ______ (PASS/FAIL)

### Scenario 2: Integration Pipeline Testing
- [ ] 2.1 Load Integration Test Graph: ______ (PASS/FAIL)
- [ ] 2.2 Execute Integration Test: ______ (PASS/FAIL)
- [ ] 2.3 Verify Data Types: ______ (PASS/FAIL)

### Scenario 3: Error Handling Validation
- [ ] 3.1 Missing Required Input: ______ (PASS/FAIL)
- [ ] 3.2 Invalid Input Type: ______ (PASS/FAIL)
- [ ] 3.3 Invalid Operation: ______ (PASS/FAIL)

### Scenario 4: Unicode Handling Validation
- [ ] 4.1 Multi-byte Characters: ______ (PASS/FAIL)
- [ ] 4.2 Emoji Handling: ______ (PASS/FAIL)

### Issues Found
(List any issues discovered during testing)

### Overall Result
- [ ] User Story 1 (P1) - ACCEPTED
- [ ] User Story 1 (P1) - REJECTED (requires fixes)
```

---

## Notes

- **Component Loading**: If components don't appear in the palette, verify .wasm files are in the correct directory
- **Execution Failures**: Check wasmflow logs for detailed error messages
- **Performance**: String operations should be near-instantaneous for test data (<100ms)
- **Memory**: Monitor memory usage during long string operations

## Next Steps

After completing this validation:
1. Record results in the test execution report
2. File issues for any failures or unexpected behavior
3. Update component implementations if needed
4. Re-run validation after fixes
5. Once all tests pass, mark User Story 1 as **COMPLETE**
6. Proceed to User Story 2 (Math Operations) if continuing beyond MVP

---

**Last Updated**: 2025-10-23
**Status**: Ready for validation
**Related Tasks**: T041 (Integration Test Graph), T042 (Manual Validation)
