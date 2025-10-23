# Data Model: JSON Parser Node

**Feature**: JSON Parser Node | **Date**: 2025-10-22

## Overview

The JSON Parser Node is a stateless component with no persistent data storage. All data is transient - inputs are processed and outputs are produced within a single execution cycle. This document describes the internal data structures used during processing.

## Core Data Types

### 1. Input Data

#### JsonInput
**Description**: Raw JSON string provided by upstream nodes

**Structure**:
```rust
type JsonString = String;
```

**Validation Rules**:
- MUST be valid UTF-8 string
- MUST be parsable as JSON (validated during processing)
- Maximum size: 1MB (enforced by wasmflow runtime)
- Empty string is invalid (produces InvalidJson error)

**Example Values**:
```json
// Valid
"{\"version\": 1, \"metadata\": {\"author\": \"me\"}}"
"[1, 2, 3]"
"\"simple string\""
"null"

// Invalid (produces errors)
""                    // Empty
"{invalid json"       // Malformed
"not json at all"     // Invalid syntax
```

---

#### KeyPath
**Description**: Path specification using dot and bracket notation

**Structure**:
```rust
type KeyPath = String;
```

**Validation Rules**:
- MUST NOT be empty string
- MUST follow syntax: `identifier(.identifier)*([index])*`
- Identifiers: alphanumeric + underscore (no leading digit)
- Indices: non-negative integers only
- No spaces allowed (except in future escaped syntax)

**Syntax Grammar**:
```
key_path    := segment ('.' segment | '[' index ']')*
segment     := identifier
identifier  := [a-zA-Z_][a-zA-Z0-9_]*
index       := [0-9]+
```

**Valid Examples**:
```
version
metadata.author
runs[1]
runs[1].time
data.items[0].value.score
a.b.c.d.e.f.g
```

**Invalid Examples**:
```
""                    // Empty
"metadata..author"    // Double dot
"runs[abc]"           // Non-numeric index
"runs[]"              // Missing index
"123invalid"          // Leading digit
"meta data.author"    // Space in identifier
```

---

### 2. Internal Processing Types

#### Token
**Description**: Parsed components of a key path

**Structure**:
```rust
enum Token {
    Ident(String),   // Property accessor (e.g., "metadata", "author")
    Index(usize),    // Array index accessor (e.g., 1, 0, 999)
}
```

**Relationships**:
- KeyPath (String) is tokenized into Vec<Token>
- Tokens are consumed sequentially during JSON traversal

**State Transitions**:
```
KeyPath String → Tokenizer → Vec<Token> → Traversal Engine → Result<Value, Error>
```

**Example Tokenization**:
```rust
"runs[1].time"
    ↓
[
    Token::Ident("runs"),
    Token::Index(1),
    Token::Ident("time")
]
```

---

#### ParsedJson
**Description**: Intermediate representation of parsed JSON

**Structure**:
```rust
// Using serde_json::Value internally
enum ParsedJson {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<ParsedJson>),
    Object(HashMap<String, ParsedJson>),
}
```

**Relationships**:
- Created from JsonInput via `serde_json::from_str()`
- Traversed using Token sequence
- Extracted value converted to NodeValue for output

**Validation Rules**:
- All JSON types supported (per FR-014)
- Numbers represented as f64 (JSON doesn't distinguish int/float)
- Objects are unordered maps
- Arrays preserve order

---

### 3. Output Data

#### JsonValue (Success Output)
**Description**: Extracted value with preserved JSON type

**Structure** (WIT representation):
```wit
variant json-value {
    string(string),
    number(float64),
    boolean(bool),
    object(string),   // Serialized JSON
    array(string),    // Serialized JSON
    null,
}
```

**Type Mapping from serde_json::Value**:
| JSON Type | serde_json::Value | json-value variant | Example Output |
|-----------|-------------------|-------------------|----------------|
| String | Value::String | string(s) | `"me"` |
| Number | Value::Number | number(f64) | `1000.0` |
| Boolean | Value::Bool | boolean(b) | `true` |
| Object | Value::Object | object(json_str) | `"{\"id\":1}"` |
| Array | Value::Array | array(json_str) | `"[1,2,3]"` |
| Null | Value::Null | null | `null` |

**Validation Rules**:
- Type preservation: Output type MUST match JSON type at extracted path
- Object/Array serialization: MUST be valid JSON string (can be re-parsed)
- Number precision: f64 precision maintained (IEEE 754 double)

**Example Outputs**:
```rust
// For JSON: {"version": 1}
// Path: "version"
// Output: json-value::number(1.0)

// For JSON: {"metadata": {"author": "me"}}
// Path: "metadata"
// Output: json-value::object("{\"author\":\"me\"}")

// For JSON: {"runs": [{"id": 1}, {"id": 2}]}
// Path: "runs[1]"
// Output: json-value::object("{\"id\":2}")
```

---

#### ParseError (Error Output)
**Description**: Structured error information for failed extractions

**Structure** (WIT representation):
```wit
record parse-error {
    message: string,
    kind: error-kind,
    context: option<string>,
}

enum error-kind {
    invalid-json,
    path-not-found,
    malformed-path,
    index-out-of-bounds,
    type-mismatch,
}
```

**Error Kinds Mapping**:
| error-kind | Triggers When | Example |
|------------|---------------|---------|
| invalid-json | JSON parsing fails (FR-008) | `"{invalid"` |
| path-not-found | Key doesn't exist (FR-009) | `"nonexistent"` in `{}` |
| malformed-path | Invalid path syntax (FR-010) | `"runs[abc]"` |
| index-out-of-bounds | Array index too large (FR-011) | `"runs[999]"` with 2 elements |
| type-mismatch | Wrong type for operation | `"version.property"` when version is number |

**Validation Rules**:
- `message`: MUST be human-readable, actionable description
- `kind`: MUST match one of the enum variants
- `context`: OPTIONAL additional details (path segment, index value, etc.)

**Example Error Messages**:
```rust
// Invalid JSON
ParseError {
    message: "Failed to parse JSON: expected `,` at line 1 column 10",
    kind: invalid-json,
    context: None,
}

// Path not found
ParseError {
    message: "Key 'nonexistent' not found in JSON object",
    kind: path-not-found,
    context: Some("at path segment: nonexistent"),
}

// Index out of bounds
ParseError {
    message: "Array index 999 out of bounds (array length: 2)",
    kind: index-out-of-bounds,
    context: Some("at path segment: runs[999]"),
}

// Type mismatch
ParseError {
    message: "Cannot access property 'property' on number value",
    kind: type-mismatch,
    context: Some("at path segment: version.property"),
}
```

---

## Data Flow Diagram

```
┌──────────────┐
│ JsonString   │ (Input Port)
│ KeyPath      │ (Input Port)
└──────┬───────┘
       │
       ▼
┌─────────────────────────┐
│ 1. Parse JSON String    │
│    └─> ParsedJson       │
└───────┬─────────────────┘
        │
        ▼
┌─────────────────────────┐
│ 2. Tokenize KeyPath     │
│    └─> Vec<Token>       │
└───────┬─────────────────┘
        │
        ▼
┌─────────────────────────┐
│ 3. Traverse ParsedJson  │
│    using Tokens         │
└───────┬─────────────────┘
        │
        ├─(success)──────────────────┐
        │                            │
        ▼                            ▼
┌──────────────────┐      ┌─────────────────┐
│ 4a. Extract Value│      │ 4b. Create Error│
│    └─> JsonValue │      │    └─> ParseError│
└────────┬─────────┘      └────────┬────────┘
         │                         │
         ▼                         ▼
┌──────────────────────────────────────┐
│ Output Ports:                        │
│  • value: option<json-value>         │
│  • error: option<parse-error>        │
└──────────────────────────────────────┘
```

**Invariants**:
- Exactly one output port MUST be populated per execution
- If `error` is Some, then `value` MUST be None
- If `value` is Some, then `error` MUST be None
- Empty inputs produce errors (never null/empty outputs)

---

## Edge Case Handling

### 1. Empty Key Path
**Input**: `key_path = ""`
**Behavior**: Return MalformedPath error
**Rationale**: Ambiguous intent (whole document? error?), spec assumption says error

### 2. Null Values in JSON
**Input**: `json = "{\"value\": null}"`, `key_path = "value"`
**Behavior**: Return `json-value::null`
**Rationale**: Null is a valid JSON value (FR-012), distinguish from missing

### 3. Missing vs Null
**Input**: `json = "{}"`, `key_path = "missing"`
**Behavior**: Return PathNotFound error
**Comparison**: Different from `{"missing": null}` which returns `json-value::null`

### 4. Deep Nesting
**Input**: `key_path = "a.b.c.d.e.f.g.h.i.j"` (10+ levels)
**Behavior**: Process normally if JSON supports it
**Performance**: O(k) traversal where k = depth, no practical limit
**Rationale**: SC-002 specifies support for 10+ levels

### 5. Large Arrays
**Input**: `json = "[1, 2, 3, ..., 1000]"`, `key_path = "999"`
**Behavior**: Process normally, return element at index 999
**Performance**: O(1) array access via serde_json
**Rationale**: SC-003 specifies arrays up to 1000 elements

### 6. Zero Index
**Input**: `key_path = "array[0]"`
**Behavior**: Return first element (zero-based indexing)
**Rationale**: Spec assumption - standard programming convention

### 7. Negative Indices
**Input**: `key_path = "array[-1]"`
**Behavior**: Return MalformedPath error
**Rationale**: Out of scope for v1, syntax not supported

### 8. Property on Primitive
**Input**: `json = "{\"version\": 1}"`, `key_path = "version.property"`
**Behavior**: Return TypeMismatch error
**Rationale**: Cannot access properties on number values

### 9. Index on Non-Array
**Input**: `json = "{\"metadata\": {\"author\": \"me\"}}"`, `key_path = "metadata[0]"`
**Behavior**: Return TypeMismatch error
**Rationale**: Cannot index into object (use keys, not indices)

### 10. Special Characters in Keys
**Input**: `json = "{\"author.name\": \"me\"}"`, `key_path = "author.name"`
**Behavior**: Looks for nested structure `{"author": {"name": "me"}}`, not literal key
**Rationale**: Out of scope for v1 (spec assumption), escaping syntax needed for literal

---

## Type System Invariants

### JSON Type Preservation
**Invariant**: Extracted value type MUST match source JSON type
```
extract({"version": 1}, "version") → json-value::number(1.0)  ✓
extract({"version": 1}, "version") → json-value::string("1")  ✗ (wrong type)
```

### Error Exclusivity
**Invariant**: Result is EITHER value OR error, never both, never neither
```
Result::Ok(value)  → value=Some, error=None  ✓
Result::Err(error) → value=None, error=Some  ✓
Both None          → Never occurs            ✗
Both Some          → Never occurs            ✗
```

### Path Completeness
**Invariant**: All path segments MUST be consumed
```
Tokens: [Ident("runs"), Index(1)]
JSON: {"runs": [{"id": 1}, {"id": 2}]}
→ Traverse both segments, return {"id": 2}  ✓

Partial match → Error (path not found or type mismatch)  ✓
```

### Null Semantics
**Invariant**: Null value ≠ Missing value
```
{"value": null} with path "value"     → json-value::null  ✓
{}              with path "value"     → PathNotFound      ✓
{"value": null} with path "missing"   → PathNotFound      ✓
```

---

## Performance Characteristics

### Time Complexity
- **JSON Parsing**: O(n) where n = JSON string length
- **Tokenization**: O(m) where m = key path length
- **Traversal**: O(k) where k = number of path segments (depth)
- **Overall**: O(n + m + k), dominated by O(n) for large JSON

### Space Complexity
- **Parsed JSON**: O(n) for serde_json::Value
- **Tokens**: O(m) for Vec<Token>
- **Output**: O(v) where v = size of extracted value
- **Overall**: O(n) dominated by parsed JSON structure

### Expected Performance
- Small JSON (<10KB): <10ms
- Medium JSON (100KB): <50ms
- Large JSON (1MB): <100ms (SC-006 target)

---

## Testing Data

### Test Case Categories

**Category 1: Valid Paths**
- Simple property: `"version"` → expect number
- Nested property: `"metadata.author"` → expect string
- Array index: `"runs[1]"` → expect object
- Combined: `"runs[1].time"` → expect number

**Category 2: Error Cases**
- Invalid JSON: `"{invalid"` → InvalidJson
- Missing path: `"nonexistent"` → PathNotFound
- Out of bounds: `"runs[999]"` → IndexOutOfBounds
- Type mismatch: `"version.property"` → TypeMismatch
- Malformed path: `"runs[abc]"` → MalformedPath

**Category 3: Edge Cases**
- Null value: `"nullValue"` → json-value::null
- Empty object: `"{}"` with any path → PathNotFound
- Empty array: `"[]"` with `"[0]"` → IndexOutOfBounds
- Deep nesting: 10+ levels → success
- Large array: 1000+ elements → success

---

## Summary

The JSON Parser Node operates on transient data with clear type preservation and comprehensive error handling. All data structures support the functional requirements (FR-001 through FR-015) and enable testable, deterministic behavior across all edge cases.
