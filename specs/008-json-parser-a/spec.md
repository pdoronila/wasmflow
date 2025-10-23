# Feature Specification: JSON Parser Node

**Feature Branch**: `008-json-parser-a`
**Created**: 2025-10-22
**Status**: Draft
**Input**: User description: "json-parser. A json parser node wasm component that takes in a string of json and parses it. This node takes an input for a key and returns the value of that key. Example json
{
  \"version\": 1,
  \"metadata\": {
    \"author\": \"me\"
  },
  \"runs\": [
    { \"id\": 1 , \"time\": 100},
    { \"id\": 2 , \"time\": 1000}
  ]
}
the input could represent a flatten key of the struct. To access version would be \"version\". Accessing metadata author would look like \"metadata.author\". To access a specific run would be \"runs[1]\". To access a specific field from a run would be \"runs[1].time\""

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Extract Simple Top-Level Values (Priority: P1)

A workflow builder connects a JSON data source to the JSON parser node and needs to extract a simple top-level value like a version number or identifier to use in subsequent workflow steps.

**Why this priority**: This is the fundamental capability of the node - extracting values from JSON using simple keys. Without this, the node provides no value.

**Independent Test**: Can be fully tested by providing a JSON string with top-level properties (e.g., `{"version": 1}`) and a simple key path (e.g., `"version"`) and verifying the correct value is returned.

**Acceptance Scenarios**:

1. **Given** a JSON parser node with input JSON `{"version": 1, "name": "test"}`, **When** the key path input is set to `"version"`, **Then** the node outputs the value `1`
2. **Given** a JSON parser node with input JSON `{"author": "me", "status": "active"}`, **When** the key path input is set to `"author"`, **Then** the node outputs the string value `"me"`
3. **Given** a JSON parser node with input JSON `{"enabled": true}`, **When** the key path input is set to `"enabled"`, **Then** the node outputs the boolean value `true`

---

### User Story 2 - Navigate Nested Object Properties (Priority: P2)

A workflow builder needs to extract values from nested JSON objects, such as extracting an author name from metadata or configuration settings from a nested structure.

**Why this priority**: Real-world JSON data is typically nested. This capability extends the basic value extraction to handle realistic data structures.

**Independent Test**: Can be fully tested by providing JSON with nested objects (e.g., `{"metadata": {"author": "me"}}`) and a dot-notation key path (e.g., `"metadata.author"`) and verifying the correct nested value is returned.

**Acceptance Scenarios**:

1. **Given** a JSON parser node with input JSON `{"metadata": {"author": "me", "version": 2}}`, **When** the key path is `"metadata.author"`, **Then** the node outputs `"me"`
2. **Given** a JSON parser node with input JSON `{"config": {"server": {"port": 8080}}}`, **When** the key path is `"config.server.port"`, **Then** the node outputs `8080`
3. **Given** a JSON parser node with input JSON `{"a": {"b": {"c": {"d": "deep"}}}}`, **When** the key path is `"a.b.c.d"`, **Then** the node outputs `"deep"`

---

### User Story 3 - Access Array Elements by Index (Priority: P2)

A workflow builder needs to extract specific items from JSON arrays, such as getting the second run from a list of test runs or the first item from a results array.

**Why this priority**: Arrays are common in JSON data structures. This allows users to access specific elements within arrays when the position is known.

**Independent Test**: Can be fully tested by providing JSON with arrays (e.g., `{"runs": [{"id": 1}, {"id": 2}]}`) and a bracket-notation key path (e.g., `"runs[1]"`) and verifying the correct array element is returned.

**Acceptance Scenarios**:

1. **Given** a JSON parser node with input JSON `{"runs": [{"id": 1}, {"id": 2}]}`, **When** the key path is `"runs[1]"`, **Then** the node outputs the object `{"id": 2}`
2. **Given** a JSON parser node with input JSON `{"values": [10, 20, 30]}`, **When** the key path is `"values[0]"`, **Then** the node outputs `10`
3. **Given** a JSON parser node with input JSON `{"items": ["first", "second", "third"]}`, **When** the key path is `"items[2]"`, **Then** the node outputs `"third"`

---

### User Story 4 - Combine Array Access with Property Navigation (Priority: P3)

A workflow builder needs to extract specific properties from objects within arrays, such as getting the execution time of the second test run or the name of the third user in a list.

**Why this priority**: This combines array indexing with nested property access, enabling complex data extraction patterns needed for real-world scenarios.

**Independent Test**: Can be fully tested by providing JSON with arrays of objects (e.g., `{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}`) and a combined key path (e.g., `"runs[1].time"`) and verifying the correct value is extracted.

**Acceptance Scenarios**:

1. **Given** a JSON parser node with input JSON `{"runs": [{"id": 1, "time": 100}, {"id": 2, "time": 1000}]}`, **When** the key path is `"runs[1].time"`, **Then** the node outputs `1000`
2. **Given** a JSON parser node with input JSON `{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}`, **When** the key path is `"users[0].name"`, **Then** the node outputs `"Alice"`
3. **Given** a JSON parser node with input JSON `{"data": {"items": [{"value": {"score": 95}}]}}`, **When** the key path is `"data.items[0].value.score"`, **Then** the node outputs `95`

---

### Edge Cases

- What happens when the key path does not exist in the JSON (e.g., `"nonexistent"` or `"metadata.missing"`)?
- How does the node handle invalid JSON input strings?
- What happens when an array index is out of bounds (e.g., `"runs[999]"` when the array has only 2 elements)?
- How does the node handle malformed key paths (e.g., `"runs[abc]"` or `"metadata..author"`)?
- What happens when the key path is an empty string?
- How does the node handle JSON null values at the specified path?
- What happens when trying to access a property on a primitive value (e.g., `"version.property"` when version is a number)?
- How does the node handle very deeply nested paths (e.g., 100+ levels deep)?
- What happens when the JSON contains special characters in property names?
- How does the node handle large JSON payloads?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Node MUST accept a JSON string as an input port
- **FR-002**: Node MUST accept a key path string as an input port specifying the location of the desired value
- **FR-003**: Node MUST parse the JSON string and extract the value at the specified key path
- **FR-004**: Node MUST support dot notation for accessing nested object properties (e.g., `"metadata.author"`)
- **FR-005**: Node MUST support bracket notation with numeric indices for accessing array elements (e.g., `"runs[1]"`)
- **FR-006**: Node MUST support combined notation for accessing properties within array elements (e.g., `"runs[1].time"`)
- **FR-007**: Node MUST output the extracted value in its native type (string, number, boolean, object, array, or null)
- **FR-008**: Node MUST provide an error output when the JSON string is invalid or cannot be parsed
- **FR-009**: Node MUST provide an error output when the key path does not exist in the parsed JSON
- **FR-010**: Node MUST provide an error output when the key path is malformed or syntactically invalid
- **FR-011**: Node MUST provide an error output when an array index is out of bounds
- **FR-012**: Node MUST handle JSON null values and distinguish them from missing values
- **FR-013**: Node MUST handle empty JSON objects `{}` and empty arrays `[]`
- **FR-014**: Node MUST handle JSON strings, numbers, booleans, objects, arrays, and null as top-level values
- **FR-015**: Node MUST be implemented as a WASM component compatible with the wasmflow_cc node system

### Key Entities

- **JSON Input**: A string containing valid JSON data that will be parsed
- **Key Path**: A string using dot notation (`.`) and bracket notation (`[n]`) to specify the location of a value within the JSON structure
- **Extracted Value**: The value found at the specified key path, preserving its original JSON type
- **Error Information**: Details about parsing failures, missing paths, or invalid key paths

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully extract simple top-level values from JSON in under 5 seconds of configuration time
- **SC-002**: Users can successfully extract nested values from JSON structures up to 10 levels deep
- **SC-003**: Users can successfully extract values from arrays with up to 1000 elements without performance degradation
- **SC-004**: 95% of valid key path queries return correct values on first attempt
- **SC-005**: All invalid inputs (malformed JSON, nonexistent paths, invalid indices) produce clear, actionable error messages
- **SC-006**: Node processes JSON payloads up to 1MB in size without timeout or failure
- **SC-007**: Users can chain multiple JSON parser nodes together to extract multiple values from the same JSON source

## Assumptions

- Array indices use zero-based indexing (standard for most programming languages)
- Dot notation and bracket notation follow JavaScript-style path syntax
- Empty key path returns an error rather than the entire JSON document
- The node outputs the raw value without additional formatting or type conversion (preserves JSON types)
- Property names in the JSON do not contain dots or brackets (if they do, this is out of scope for v1)
- The node is synchronous - it processes input immediately and produces output in the same execution cycle
- Maximum JSON payload size of 1MB is sufficient for typical workflow use cases
- Error messages are surfaced through a dedicated error output port

## Dependencies

- Wasmflow_cc node graph system and WASM component model
- WASM-compatible JSON parsing library
- Support for multiple input ports (JSON string, key path string)
- Support for multiple output ports (value output, error output)

## Out of Scope

- JSONPath query language support (e.g., `$..author` for recursive search)
- Wildcard or pattern matching in key paths
- Modifying or transforming the JSON (this is a read-only parser)
- Schema validation or JSON schema enforcement
- Filtering or mapping operations across arrays
- Support for property names containing special characters (dots, brackets)
- Custom error handling configuration
- Performance optimization for JSON payloads larger than 1MB
