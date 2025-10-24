# Feature Specification: WASM Components Core Library

**Feature Branch**: `010-wasm-components-core`
**Created**: 2025-10-23
**Status**: Draft
**Input**: User description: "wasm components core. I want to build out a core library of wasm components.
  1. String Operations - Perfect WASM candidates
  - string-concat - Join multiple strings
  - string-split - Split on delimiter → list
  - string-length - Get character count
  - string-trim - Remove whitespace
  - string-case - Upper/lower/title case
  - string-contains - Boolean check
  - string-substring - Extract substring
  2. Comparison & Logic - Also straightforward
  - compare - Equals, NotEquals, GreaterThan, LessThan (generic across types)
  - boolean-and, boolean-or, boolean-not, boolean-xor
  - is-null, is-empty - Type checking
  3. Math Extensions - Match your existing pattern
  - math-power, math-sqrt, math-abs
  - math-min, math-max
  - math-floor, math-ceil, math-round
  - math-trig (sin, cos, tan) - could be one component with operation selector
  4. List Operations - More complex but valuable
  - list-length, list-get, list-append
  - list-join, list-slice
  - list-contains, list-index-of
  5. Data Transformation
  - json-stringify - Complement to your JSON parser
  - to-string, parse-number - Type conversions
  - format-template - String interpolation

Create a components/core/ directory with foundational components:

  components/
  ├── core/
  │   ├── string-concat/
  │   ├── string-split/
  │   ├── string-length/
  │   ├── compare/
  │   ├── boolean-and/
  │   ├── boolean-or/
  │   └── ... (all string & logic operations)
  ├── math/
  │   ├── power/
  │   ├── trig/
  │   └── ... (advanced math)
  ├── collections/
  │   ├── list-get/
  │   ├── list-length/
  │   └── ... (list operations)
  └── ... (existing components)

Eventually, you could:
  1. Migrate math operations to WASM
  2. Keep only essential built-ins (Constant, WASM Creator, Continuous)
  3. Have a clean separation: Built-ins = meta/runtime, WASM = all logic"

## Overview

This feature establishes a comprehensive core library of WebAssembly components for wasmflow, providing foundational operations for string manipulation, comparison, logic, mathematics, list operations, and data transformation. The library enables users to build complex data processing pipelines using composable, reusable WASM components organized into logical categories.

## User Scenarios & Testing

### User Story 1 - Text Processing Pipeline (Priority: P1)

As a wasmflow user, I want to process text data by chaining string operations together, so that I can clean, transform, and analyze text within my node graphs without writing custom code.

**Why this priority**: String operations are the most fundamental and frequently used operations in data processing workflows. They enable immediate value delivery for common use cases like data cleaning, formatting, and extraction.

**Independent Test**: Can be fully tested by creating a node graph that takes raw text input, applies trim/case/concat operations, and outputs formatted text. Delivers immediate value for text processing tasks.

**Acceptance Scenarios**:

1. **Given** a text input with leading/trailing whitespace, **When** the user connects it to a string-trim component, **Then** the output contains the text without whitespace
2. **Given** multiple text inputs, **When** the user connects them to a string-concat component, **Then** the output contains all inputs joined together
3. **Given** a text input, **When** the user connects it to a string-split component with a delimiter, **Then** the output contains a list of text segments
4. **Given** a text input, **When** the user connects it to a string-case component set to "uppercase", **Then** the output contains the text in uppercase
5. **Given** a text input and a search string, **When** the user connects them to a string-contains component, **Then** the output indicates whether the search string is present

---

### User Story 2 - Data Validation Pipeline (Priority: P2)

As a wasmflow user, I want to validate data using comparison and logic operations, so that I can create conditional flows and filter data based on business rules.

**Why this priority**: Logic and comparison operations enable conditional workflows and data validation, which are essential for building intelligent processing pipelines. This is the next logical step after basic text processing.

**Independent Test**: Can be fully tested by creating a validation graph that checks if values meet criteria (using compare components) and combines multiple conditions (using boolean logic components). Delivers value for data quality and conditional routing.

**Acceptance Scenarios**:

1. **Given** two numeric inputs, **When** the user connects them to a compare component set to "GreaterThan", **Then** the output indicates whether the first value is greater than the second
2. **Given** two boolean inputs, **When** the user connects them to a boolean-and component, **Then** the output is true only if both inputs are true
3. **Given** a value input, **When** the user connects it to an is-null component, **Then** the output indicates whether the value is null
4. **Given** multiple comparison results, **When** the user chain them through boolean-or components, **Then** the output represents whether any condition is met

---

### User Story 3 - Mathematical Computation Pipeline (Priority: P3)

As a wasmflow user, I want to perform advanced mathematical operations beyond basic arithmetic, so that I can build numerical processing and analysis workflows.

**Why this priority**: Advanced math operations expand the capabilities of wasmflow for scientific and analytical applications. Less commonly needed than string/logic operations but critical for numerical workflows.

**Independent Test**: Can be fully tested by creating a calculation graph using power, sqrt, trig, and rounding operations. Delivers value for scientific computing and data analysis tasks.

**Acceptance Scenarios**:

1. **Given** two numeric inputs, **When** the user connects them to a math-power component, **Then** the output contains the first value raised to the power of the second
2. **Given** a numeric input, **When** the user connects it to a math-sqrt component, **Then** the output contains the square root of the input
3. **Given** an angle in radians, **When** the user connects it to a math-trig component set to "sin", **Then** the output contains the sine of the angle
4. **Given** a decimal number, **When** the user connects it to a math-round component, **Then** the output contains the nearest integer value
5. **Given** multiple numeric inputs, **When** the user connects them to a math-max component, **Then** the output contains the largest value

---

### User Story 4 - List Manipulation Pipeline (Priority: P4)

As a wasmflow user, I want to work with lists of values by accessing, modifying, and analyzing them, so that I can process collections of data within my workflows.

**Why this priority**: List operations enable batch processing and working with collections. While powerful, they build on top of the more fundamental operations and are less frequently needed.

**Independent Test**: Can be fully tested by creating a graph that builds a list, extracts elements, slices sections, and checks for values. Delivers value for batch processing and collection manipulation.

**Acceptance Scenarios**:

1. **Given** a list input, **When** the user connects it to a list-length component, **Then** the output contains the number of elements in the list
2. **Given** a list input and an index, **When** the user connects them to a list-get component, **Then** the output contains the element at that index
3. **Given** a list input and a value, **When** the user connects them to a list-append component, **Then** the output contains a new list with the value added
4. **Given** a list input and indices, **When** the user connects them to a list-slice component, **Then** the output contains the specified range of elements
5. **Given** a list input and a delimiter, **When** the user connects them to a list-join component, **Then** the output contains a string with all elements joined by the delimiter

---

### User Story 5 - Data Transformation Pipeline (Priority: P5)

As a wasmflow user, I want to convert data between different types and formats, so that I can integrate different data sources and prepare data for output.

**Why this priority**: Type conversion and formatting operations enable interoperability between components and data sources. These are supporting operations that enhance other workflows rather than standalone value.

**Independent Test**: Can be fully tested by creating a graph that converts between types (to-string, parse-number), formats templates, and serializes to JSON. Delivers value for data integration and output formatting.

**Acceptance Scenarios**:

1. **Given** a numeric input, **When** the user connects it to a to-string component, **Then** the output contains the number represented as text
2. **Given** a text input containing a number, **When** the user connects it to a parse-number component, **Then** the output contains the numeric value
3. **Given** structured data, **When** the user connects it to a json-stringify component, **Then** the output contains a JSON text representation
4. **Given** a template string and values, **When** the user connects them to a format-template component, **Then** the output contains the template with values substituted

---

### Edge Cases

- What happens when string operations receive null or empty inputs?
- How does the system handle invalid list indices (negative, out of bounds)?
- What happens when mathematical operations encounter invalid inputs (sqrt of negative, division by zero)?
- How does the system handle type mismatches in comparison operations?
- What happens when parse-number receives non-numeric text?
- How does the system handle very large lists or strings that might impact performance?
- What happens when format-template receives mismatched placeholders and values?

## Requirements

### Functional Requirements

#### Component Organization

- **FR-001**: System MUST organize components into category directories: components/core/ (string and logic operations), components/math/ (mathematical operations), and components/collections/ (list operations)
- **FR-002**: Each component MUST reside in its own subdirectory containing WIT interface definition, Rust implementation, and build configuration
- **FR-003**: System MUST follow the existing component structure pattern established by the json-parser component

#### String Operations (Priority: P1)

- **FR-004**: System MUST provide a string-concat component that accepts multiple string inputs and outputs a single joined string
- **FR-005**: System MUST provide a string-split component that accepts a string and delimiter, outputting a list of substrings
- **FR-006**: System MUST provide a string-length component that outputs the character count of an input string
- **FR-007**: System MUST provide a string-trim component that removes leading and trailing whitespace from a string
- **FR-008**: System MUST provide a string-case component that converts strings to uppercase, lowercase, or title case based on an operation selector
- **FR-009**: System MUST provide a string-contains component that checks if a string contains a substring, outputting a boolean
- **FR-010**: System MUST provide a string-substring component that extracts a portion of a string based on start index and length

#### Comparison & Logic Operations (Priority: P2)

- **FR-011**: System MUST provide a compare component that supports Equals, NotEquals, GreaterThan, LessThan, GreaterThanOrEqual, and LessThanOrEqual operations across numeric and string types
- **FR-012**: System MUST provide boolean-and, boolean-or, boolean-not, and boolean-xor components for logical operations
- **FR-013**: System MUST provide an is-null component that checks if a value is null or undefined
- **FR-014**: System MUST provide an is-empty component that checks if a string or list is empty

#### Math Operations (Priority: P3)

- **FR-015**: System MUST provide math-power component for exponentiation
- **FR-016**: System MUST provide math-sqrt component for square root calculation
- **FR-017**: System MUST provide math-abs component for absolute value
- **FR-018**: System MUST provide math-min and math-max components that accept multiple inputs and output the minimum or maximum value
- **FR-019**: System MUST provide math-floor, math-ceil, and math-round components for rounding operations
- **FR-020**: System MUST provide a math-trig component that supports sin, cos, and tan operations based on an operation selector

#### List Operations (Priority: P4)

- **FR-021**: System MUST provide a list-length component that outputs the number of elements in a list
- **FR-022**: System MUST provide a list-get component that retrieves an element at a specified index
- **FR-023**: System MUST provide a list-append component that adds a value to a list, outputting a new list
- **FR-024**: System MUST provide a list-join component that converts a list to a string with a specified delimiter
- **FR-025**: System MUST provide a list-slice component that extracts a range of elements from a list
- **FR-026**: System MUST provide a list-contains component that checks if a list contains a value
- **FR-027**: System MUST provide a list-index-of component that returns the index of a value in a list

#### Data Transformation (Priority: P5)

- **FR-028**: System MUST provide a json-stringify component that serializes structured data to JSON text format
- **FR-029**: System MUST provide a to-string component that converts numeric and boolean values to text
- **FR-030**: System MUST provide a parse-number component that converts text to numeric values
- **FR-031**: System MUST provide a format-template component that substitutes values into a template string

#### Error Handling

- **FR-032**: Components MUST return error messages when operations fail (e.g., invalid inputs, out-of-bounds access)
- **FR-033**: Components MUST handle null and empty inputs gracefully without crashing
- **FR-034**: Mathematical components MUST handle invalid operations (e.g., sqrt of negative number, division by zero) by returning error values

#### Integration

- **FR-035**: All components MUST integrate with the existing wasmflow component loading system
- **FR-036**: Components MUST use WIT interfaces compatible with the wasmflow component model
- **FR-037**: Components MUST render appropriate UI footer views in the node editor showing their configuration and operation type

### Key Entities

- **Component Definition**: Each WASM component with its WIT interface, implementation code, and metadata (name, category, inputs, outputs, operation selectors)
- **Operation Selector**: Configuration parameter for multi-operation components (e.g., string-case operations: uppercase/lowercase/titlecase, math-trig operations: sin/cos/tan)
- **Component Category**: Organizational grouping (core, math, collections) that determines directory structure and component discovery
- **Error Result**: Structured error information returned when component operations fail, including error type and message

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can create text processing workflows using string components with results appearing in under 1 second per operation
- **SC-002**: Users can build data validation logic using comparison and boolean components that evaluate conditions correctly 100% of the time for valid inputs
- **SC-003**: Mathematical operations produce results accurate to IEEE 754 double-precision floating point standards
- **SC-004**: List operations support collections of at least 10,000 elements without performance degradation (operations complete in under 100ms)
- **SC-005**: Component error messages enable users to identify and correct issues without consulting documentation in 80% of cases
- **SC-006**: Users can discover and add any core library component to their graph in under 30 seconds
- **SC-007**: All 35+ components load successfully and are available in the component library
- **SC-008**: Zero crashes or runtime failures when valid inputs are provided to components

## Assumptions

1. **Data Serialization**: Components will use string-based serialization for complex types (lists, structured data) similar to the existing json-parser component
2. **Immutability**: List operations will follow functional programming patterns, returning new lists rather than mutating inputs
3. **Type System**: Components will handle type validation internally and return errors for type mismatches rather than crashing
4. **Unicode Support**: String operations will properly handle Unicode characters and multi-byte encodings
5. **Numeric Precision**: Mathematical operations will use 64-bit floating point (f64) for numeric calculations
6. **Error Recovery**: Component errors will propagate to connected nodes but will not crash the entire graph execution
7. **Performance**: Individual component operations will complete in under 10ms for typical inputs (strings < 1MB, lists < 1000 elements)
8. **Build System**: Components will use the same wit-bindgen and cargo-based build system as existing WASM components

## Dependencies

- Existing wasmflow component loading and execution system (wasmtime 27.0 runtime)
- WIT interface generation tooling (wit-bindgen)
- Component build system (Rust 1.75+, wasm32-wasip2 target)
- Node graph UI system for displaying component footer views

## Out of Scope

- Custom component creation UI (handled by existing WASM Creator node)
- Component versioning and updates (future enhancement)
- Component sharing or marketplace (future enhancement)
- Advanced list operations like map, filter, reduce (future enhancement)
- Regular expression support for string operations (future enhancement)
- Date/time operations (future enhancement)
- File I/O operations (out of scope for core library)
- Network operations (out of scope for core library)
