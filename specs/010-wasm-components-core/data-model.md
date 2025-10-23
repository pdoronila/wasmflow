# Data Model: WASM Components Core Library

**Feature**: 010-wasm-components-core
**Date**: 2025-10-23
**Purpose**: Detailed specifications for all 35+ core library components

## Overview

This document defines the complete data model for the core library, specifying inputs, outputs, behavior, and error handling for each of the 35+ components. All components follow the established WIT interface pattern with metadata and execution exports.

## Common Types

### WIT Type Mapping

| WIT Type | Rust Type | Usage |
|----------|-----------|-------|
| u32 | u32 | Unsigned integers, indices, counts |
| i32 | i32 | Signed integers |
| f32 | f32 | Floating point numbers |
| string | String | UTF-8 text |
| bool | bool | Boolean values |
| list\<T\> | Vec\<T\> | Collections |
| any | NodeValue (enum) | Generic type, runtime checking |

### Port Specification Structure

```rust
pub struct PortSpec {
    pub name: String,           // Port identifier
    pub data_type: DataType,    // WIT type
    pub optional: bool,         // Can be omitted
    pub description: String,    // User-facing help text
}
```

### Error Structure

```rust
pub struct ExecutionError {
    pub message: String,           // Human-readable error
    pub input_name: Option<String>, // Which input caused failure
    pub recovery_hint: Option<String>, // How to fix the issue
}
```

## Priority 1: String Operations (7 components)

### 1.1 String Concat

**Package**: `example-string-concat`
**Category**: Text
**Description**: Joins multiple strings into a single string

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| input1 | string | No | First string |
| input2 | string | No | Second string |
| input3 | string | Yes | Third string |
| input4 | string | Yes | Fourth string |
| inputN | string | Yes | Additional strings (dynamic ports) |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | string | Concatenated string |

**Behavior**:
- Concatenates all provided input strings in order (input1 + input2 + input3 + ...)
- Empty strings are preserved in output
- No separator added between strings

**Examples**:
```
Input: ["Hello", " ", "World"]
Output: "Hello World"

Input: ["a", "b", "c", "d"]
Output: "abcd"

Input: ["", "test", ""]
Output: "test"
```

**Errors**:
- None (all inputs are strings, empty inputs allowed)

---

### 1.2 String Split

**Package**: `example-string-split`
**Category**: Text
**Description**: Splits a string on delimiter into a list of substrings

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| text | string | No | Input string to split |
| delimiter | string | No | Delimiter to split on |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| parts | list\<string\> | List of substrings |

**Behavior**:
- Splits input string on each occurrence of delimiter
- Empty delimiter splits into individual characters
- Empty string returns list with single empty string
- Multiple consecutive delimiters produce empty strings in result

**Examples**:
```
Input: text="a,b,c", delimiter=","
Output: ["a", "b", "c"]

Input: text="hello", delimiter=""
Output: ["h", "e", "l", "l", "o"]

Input: text="a,,b", delimiter=","
Output: ["a", "", "b"]
```

**Errors**:
- None (all string inputs are valid)

---

### 1.3 String Length

**Package**: `example-string-length`
**Category**: Text
**Description**: Returns the number of characters in a string

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| text | string | No | Input string |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| length | u32 | Character count (Unicode-aware) |

**Behavior**:
- Uses Rust's `.chars().count()` for Unicode-correct length
- Returns 0 for empty string
- Counts characters, not bytes (multi-byte Unicode handled correctly)

**Examples**:
```
Input: "Hello"
Output: 5

Input: "🚀🌟"
Output: 2  (not 8 bytes)

Input: ""
Output: 0
```

**Errors**:
- None

---

### 1.4 String Trim

**Package**: `example-string-trim`
**Category**: Text
**Description**: Removes leading and trailing whitespace from a string

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| text | string | No | Input string |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | string | Trimmed string |

**Behavior**:
- Uses Rust's `.trim()` method
- Removes ASCII whitespace and Unicode whitespace characters
- Does not affect whitespace in middle of string

**Examples**:
```
Input: "  hello  "
Output: "hello"

Input: "\thello\n"
Output: "hello"

Input: "  hello world  "
Output: "hello world"
```

**Errors**:
- None

---

### 1.5 String Case

**Package**: `example-string-case`
**Category**: Text
**Description**: Converts string case (uppercase, lowercase, titlecase)

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| text | string | No | Input string |
| operation | string | No | "uppercase", "lowercase", or "titlecase" |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | string | Transformed string |

**Behavior**:
- **uppercase**: Converts all characters to uppercase (`.to_uppercase()`)
- **lowercase**: Converts all characters to lowercase (`.to_lowercase()`)
- **titlecase**: Capitalizes first character of each word

**Examples**:
```
Input: text="hello world", operation="uppercase"
Output: "HELLO WORLD"

Input: text="HELLO WORLD", operation="lowercase"
Output: "hello world"

Input: text="hello world", operation="titlecase"
Output: "Hello World"
```

**Errors**:
- **Invalid operation**: "Operation must be 'uppercase', 'lowercase', or 'titlecase'" (input: operation, hint: "Use one of the three valid operations")

---

### 1.6 String Contains

**Package**: `example-string-contains`
**Category**: Text
**Description**: Checks if a string contains a substring

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| text | string | No | String to search in |
| substring | string | No | Substring to search for |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | True if substring found |

**Behavior**:
- Case-sensitive search
- Empty substring always returns true
- Uses Rust's `.contains()` method

**Examples**:
```
Input: text="Hello World", substring="World"
Output: true

Input: text="Hello World", substring="world"
Output: false  (case-sensitive)

Input: text="Hello World", substring=""
Output: true  (empty substring)
```

**Errors**:
- None

---

### 1.7 String Substring

**Package**: `example-string-substring`
**Category**: Text
**Description**: Extracts a portion of a string

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| text | string | No | Input string |
| start | u32 | No | Start index (0-based, character index) |
| length | u32 | Yes | Number of characters (if omitted, to end) |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | string | Extracted substring |

**Behavior**:
- Character-based indexing (Unicode-aware)
- If start >= string length, returns empty string
- If start + length exceeds string length, returns to end of string
- If length omitted, extracts from start to end

**Examples**:
```
Input: text="Hello World", start=0, length=5
Output: "Hello"

Input: text="Hello World", start=6
Output: "World"

Input: text="Hello", start=10
Output: ""  (start beyond end)

Input: text="🚀🌟✨", start=1, length=2
Output: "🌟✨"
```

**Errors**:
- None (out-of-bounds handled gracefully)

---

## Priority 2: Comparison & Logic Operations (7 components)

### 2.1 Compare

**Package**: `example-compare`
**Category**: Logic
**Description**: Compares two values using various operations

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| left | any | No | First value |
| right | any | No | Second value |
| operation | string | No | "equals", "not-equals", "greater-than", "less-than", "greater-or-equal", "less-or-equal" |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | Comparison result |

**Behavior**:
- **Type compatibility**:
  - Numbers (u32, i32, f32) can be compared with each other
  - Strings compared lexicographically
  - Booleans: equals/not-equals only
  - Mixed types: error (except numeric types)
- **Operations**:
  - equals: left == right
  - not-equals: left != right
  - greater-than: left > right
  - less-than: left < right
  - greater-or-equal: left >= right
  - less-or-equal: left <= right

**Examples**:
```
Input: left=10, right=5, operation="greater-than"
Output: true

Input: left="apple", right="banana", operation="less-than"
Output: true  (lexicographic)

Input: left=5.5 (f32), right=5 (u32), operation="greater-than"
Output: true  (mixed numeric types OK)
```

**Errors**:
- **Type mismatch**: "Cannot compare string with number" (input: left/right, hint: "Ensure both values are the same type or both numeric")
- **Invalid operation**: "Operation must be equals, not-equals, greater-than, less-than, greater-or-equal, or less-or-equal"
- **Ordering on booleans**: "Boolean values only support equals and not-equals" (input: operation, hint: "Use 'equals' or 'not-equals'")

---

### 2.2 Boolean AND

**Package**: `example-boolean-and`
**Category**: Logic
**Description**: Logical AND of multiple boolean inputs

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| input1 | bool | No | First boolean |
| input2 | bool | No | Second boolean |
| inputN | bool | Yes | Additional booleans |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | True if all inputs are true |

**Behavior**:
- Returns true only if all inputs are true
- Short-circuit evaluation not applicable (all inputs evaluated)
- Empty inputs (only optional inputs, none provided) returns true (identity element)

**Examples**:
```
Input: [true, true, true]
Output: true

Input: [true, false, true]
Output: false

Input: [true, true]
Output: true
```

**Errors**:
- None (all inputs are booleans)

---

### 2.3 Boolean OR

**Package**: `example-boolean-or`
**Category**: Logic
**Description**: Logical OR of multiple boolean inputs

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| input1 | bool | No | First boolean |
| input2 | bool | No | Second boolean |
| inputN | bool | Yes | Additional booleans |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | True if any input is true |

**Behavior**:
- Returns true if any input is true
- Empty inputs returns false (identity element)

**Examples**:
```
Input: [false, true, false]
Output: true

Input: [false, false, false]
Output: false
```

**Errors**:
- None

---

### 2.4 Boolean NOT

**Package**: `example-boolean-not`
**Category**: Logic
**Description**: Logical NOT (negation) of a boolean

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| input | bool | No | Boolean to negate |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | Negated value |

**Behavior**:
- Returns !input

**Examples**:
```
Input: true
Output: false

Input: false
Output: true
```

**Errors**:
- None

---

### 2.5 Boolean XOR

**Package**: `example-boolean-xor`
**Category**: Logic
**Description**: Logical XOR (exclusive or) of two booleans

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| left | bool | No | First boolean |
| right | bool | No | Second boolean |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | True if exactly one input is true |

**Behavior**:
- Returns left ^ right
- True if inputs differ, false if they match

**Examples**:
```
Input: left=true, right=false
Output: true

Input: left=true, right=true
Output: false
```

**Errors**:
- None

---

### 2.6 Is Null

**Package**: `example-is-null`
**Category**: Logic
**Description**: Checks if a value is null or undefined

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| value | any | Yes | Value to check (optional = can be null) |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | True if value is null/undefined |

**Behavior**:
- Returns true if input not provided (optional port with no connection)
- Returns false for any provided value (including empty string, 0, false)

**Examples**:
```
Input: (no connection)
Output: true

Input: ""
Output: false  (empty string is not null)

Input: 0
Output: false
```

**Errors**:
- None

---

### 2.7 Is Empty

**Package**: `example-is-empty`
**Category**: Logic
**Description**: Checks if a string or list is empty

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| value | any | No | String or list to check |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | True if value is empty |

**Behavior**:
- For strings: returns true if length == 0
- For lists: returns true if list length == 0
- For other types: error

**Examples**:
```
Input: ""
Output: true

Input: "hello"
Output: false

Input: []
Output: true

Input: [1, 2, 3]
Output: false
```

**Errors**:
- **Invalid type**: "is-empty only works with strings and lists" (input: value, hint: "Provide a string or list")

---

## Priority 3: Math Operations (9 components)

### 3.1 Math Power

**Package**: `example-math-power`
**Category**: Math
**Description**: Raises a number to a power (exponentiation)

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| base | f32 | No | Base number |
| exponent | f32 | No | Exponent |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | base ^ exponent |

**Behavior**:
- Uses Rust's `.powf()` method
- Handles negative exponents (fractional results)
- Returns NaN for invalid operations (e.g., negative base with fractional exponent)

**Examples**:
```
Input: base=2.0, exponent=3.0
Output: 8.0

Input: base=10.0, exponent=-2.0
Output: 0.01

Input: base=4.0, exponent=0.5
Output: 2.0  (square root)
```

**Errors**:
- **NaN result**: "Result is not a number (NaN)" (input: base/exponent, hint: "Check for negative base with fractional exponent")

---

### 3.2 Math Square Root

**Package**: `example-math-sqrt`
**Category**: Math
**Description**: Calculates the square root of a number

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| value | f32 | No | Input number |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | Square root |

**Behavior**:
- Uses Rust's `.sqrt()` method
- Returns NaN for negative numbers

**Examples**:
```
Input: 16.0
Output: 4.0

Input: 2.0
Output: 1.414...

Input: 0.0
Output: 0.0
```

**Errors**:
- **Negative input**: "Cannot compute square root of negative number" (input: value, hint: "Provide non-negative number")

---

### 3.3 Math Absolute Value

**Package**: `example-math-abs`
**Category**: Math
**Description**: Returns the absolute value of a number

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| value | f32 | No | Input number |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | Absolute value |

**Behavior**:
- Uses Rust's `.abs()` method
- Always returns non-negative value

**Examples**:
```
Input: -5.0
Output: 5.0

Input: 5.0
Output: 5.0

Input: 0.0
Output: 0.0
```

**Errors**:
- None

---

### 3.4 Math Min

**Package**: `example-math-min`
**Category**: Math
**Description**: Returns the smallest of multiple numbers

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| input1 | f32 | No | First number |
| input2 | f32 | No | Second number |
| inputN | f32 | Yes | Additional numbers |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | Minimum value |

**Behavior**:
- Returns smallest of all input values
- NaN values are ignored (unless all inputs are NaN)

**Examples**:
```
Input: [5.0, 2.0, 8.0, 1.0]
Output: 1.0

Input: [-3.0, 0.0, 3.0]
Output: -3.0
```

**Errors**:
- None (requires at least 2 inputs via required ports)

---

### 3.5 Math Max

**Package**: `example-math-max`
**Category**: Math
**Description**: Returns the largest of multiple numbers

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| input1 | f32 | No | First number |
| input2 | f32 | No | Second number |
| inputN | f32 | Yes | Additional numbers |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | Maximum value |

**Behavior**:
- Returns largest of all input values
- NaN values are ignored

**Examples**:
```
Input: [5.0, 2.0, 8.0, 1.0]
Output: 8.0

Input: [-3.0, 0.0, 3.0]
Output: 3.0
```

**Errors**:
- None

---

### 3.6 Math Floor

**Package**: `example-math-floor`
**Category**: Math
**Description**: Rounds a number down to the nearest integer

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| value | f32 | No | Input number |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | Floored value |

**Behavior**:
- Uses Rust's `.floor()` method
- Always rounds toward negative infinity

**Examples**:
```
Input: 3.7
Output: 3.0

Input: -2.3
Output: -3.0  (toward negative infinity)

Input: 5.0
Output: 5.0
```

**Errors**:
- None

---

### 3.7 Math Ceiling

**Package**: `example-math-ceil`
**Category**: Math
**Description**: Rounds a number up to the nearest integer

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| value | f32 | No | Input number |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | Ceiling value |

**Behavior**:
- Uses Rust's `.ceil()` method
- Always rounds toward positive infinity

**Examples**:
```
Input: 3.2
Output: 4.0

Input: -2.7
Output: -2.0  (toward positive infinity)

Input: 5.0
Output: 5.0
```

**Errors**:
- None

---

### 3.8 Math Round

**Package**: `example-math-round`
**Category**: Math
**Description**: Rounds a number to the nearest integer

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| value | f32 | No | Input number |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | Rounded value |

**Behavior**:
- Uses Rust's `.round()` method
- Rounds to nearest integer, halfway cases round away from zero

**Examples**:
```
Input: 3.5
Output: 4.0

Input: 3.4
Output: 3.0

Input: -2.5
Output: -3.0  (away from zero)
```

**Errors**:
- None

---

### 3.9 Math Trigonometry

**Package**: `example-math-trig`
**Category**: Math
**Description**: Calculates trigonometric functions (sin, cos, tan)

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| angle | f32 | No | Angle in radians |
| operation | string | No | "sin", "cos", or "tan" |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | f32 | Trigonometric result |

**Behavior**:
- **sin**: Uses `.sin()` method
- **cos**: Uses `.cos()` method
- **tan**: Uses `.tan()` method
- All operations expect angle in radians

**Examples**:
```
Input: angle=0.0, operation="sin"
Output: 0.0

Input: angle=1.5708 (π/2), operation="sin"
Output: 1.0

Input: angle=3.1416 (π), operation="cos"
Output: -1.0
```

**Errors**:
- **Invalid operation**: "Operation must be 'sin', 'cos', or 'tan'" (input: operation, hint: "Use one of the three valid operations")

---

## Priority 4: List Operations (7 components)

### 4.1 List Length

**Package**: `example-list-length`
**Category**: Collections
**Description**: Returns the number of elements in a list

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| list | list\<any\> | No | Input list |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| length | u32 | Number of elements |

**Behavior**:
- Returns list.len()
- Empty list returns 0

**Examples**:
```
Input: [1, 2, 3, 4]
Output: 4

Input: []
Output: 0

Input: ["a", "b"]
Output: 2
```

**Errors**:
- **Invalid type**: "Input must be a list" (input: list, hint: "Provide a list value")

---

### 4.2 List Get

**Package**: `example-list-get`
**Category**: Collections
**Description**: Retrieves an element at a specified index

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| list | list\<any\> | No | Input list |
| index | u32 | No | Index (0-based) |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| element | any | Element at index |

**Behavior**:
- Returns list[index]
- Index is 0-based

**Examples**:
```
Input: list=[10, 20, 30], index=1
Output: 20

Input: list=["a", "b", "c"], index=0
Output: "a"
```

**Errors**:
- **Out of bounds**: "Index 5 out of bounds for list of length 3" (input: index, hint: "Use index 0-2")

---

### 4.3 List Append

**Package**: `example-list-append`
**Category**: Collections
**Description**: Adds a value to the end of a list

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| list | list\<any\> | No | Input list |
| value | any | No | Value to append |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | list\<any\> | New list with value appended |

**Behavior**:
- Creates new list (immutable operation)
- Appends value to end
- Empty list + value = [value]

**Examples**:
```
Input: list=[1, 2], value=3
Output: [1, 2, 3]

Input: list=[], value="hello"
Output: ["hello"]
```

**Errors**:
- **Invalid type**: "Input must be a list" (input: list, hint: "Provide a list value")

---

### 4.4 List Join

**Package**: `example-list-join`
**Category**: Collections
**Description**: Converts a list to a string with a delimiter

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| list | list\<string\> | No | List of strings |
| delimiter | string | No | Separator string |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | string | Joined string |

**Behavior**:
- Joins all list elements with delimiter between them
- Empty list returns empty string
- Single element returns that element (no delimiter)

**Examples**:
```
Input: list=["a", "b", "c"], delimiter=","
Output: "a,b,c"

Input: list=["hello", "world"], delimiter=" "
Output: "hello world"

Input: list=[], delimiter=","
Output: ""
```

**Errors**:
- **Invalid type**: "All list elements must be strings" (input: list, hint: "Ensure list contains only strings")

---

### 4.5 List Slice

**Package**: `example-list-slice`
**Category**: Collections
**Description**: Extracts a range of elements from a list

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| list | list\<any\> | No | Input list |
| start | u32 | No | Start index (0-based, inclusive) |
| end | u32 | Yes | End index (exclusive, if omitted = to end) |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | list\<any\> | Sliced list |

**Behavior**:
- Returns list[start..end]
- If end omitted, returns list[start..]
- If start >= length, returns empty list
- If end > length, returns to end of list

**Examples**:
```
Input: list=[10, 20, 30, 40, 50], start=1, end=4
Output: [20, 30, 40]

Input: list=[10, 20, 30], start=1
Output: [20, 30]  (to end)

Input: list=[10, 20, 30], start=5
Output: []  (start beyond end)
```

**Errors**:
- **Invalid range**: "Start index 5 must be less than or equal to end index 3" (input: start/end, hint: "Ensure start <= end")

---

### 4.6 List Contains

**Package**: `example-list-contains`
**Category**: Collections
**Description**: Checks if a list contains a value

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| list | list\<any\> | No | Input list |
| value | any | No | Value to search for |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | bool | True if value found |

**Behavior**:
- Returns true if any element equals value
- Equality is type-aware (10 != "10")

**Examples**:
```
Input: list=[1, 2, 3], value=2
Output: true

Input: list=["a", "b"], value="c"
Output: false
```

**Errors**:
- None

---

### 4.7 List Index Of

**Package**: `example-list-index-of`
**Category**: Collections
**Description**: Returns the index of a value in a list

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| list | list\<any\> | No | Input list |
| value | any | No | Value to search for |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| index | i32 | Index of value, or -1 if not found |

**Behavior**:
- Returns index of first occurrence
- Returns -1 if value not found
- 0-based indexing

**Examples**:
```
Input: list=[10, 20, 30], value=20
Output: 1

Input: list=["a", "b", "a"], value="a"
Output: 0  (first occurrence)

Input: list=[1, 2, 3], value=5
Output: -1  (not found)
```

**Errors**:
- None

---

## Priority 5: Data Transformation (4 components)

### 5.1 JSON Stringify

**Package**: `example-json-stringify`
**Category**: Data
**Description**: Serializes structured data to JSON text format

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| data | any | No | Data to serialize (Record, List, or primitive) |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| json | string | JSON text representation |

**Behavior**:
- Uses serde_json for serialization
- Pretty-printed with indentation
- Handles Record (object), List (array), and primitives

**Examples**:
```
Input: Record { "name": "Alice", "age": 30 }
Output: "{\n  \"name\": \"Alice\",\n  \"age\": 30\n}"

Input: List [1, 2, 3]
Output: "[1, 2, 3]"

Input: "hello"
Output: "\"hello\""
```

**Errors**:
- **Serialization error**: "Failed to serialize data to JSON: {error}" (input: data, hint: "Ensure data is valid JSON-serializable type")

---

### 5.2 To String

**Package**: `example-to-string`
**Category**: Data
**Description**: Converts numeric and boolean values to text

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| value | any | No | Value to convert |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| text | string | String representation |

**Behavior**:
- Numbers: Formatted as decimal strings
- Booleans: "true" or "false"
- Strings: Pass-through unchanged
- Other types: Error

**Examples**:
```
Input: 42
Output: "42"

Input: 3.14
Output: "3.14"

Input: true
Output: "true"

Input: "hello"
Output: "hello"
```

**Errors**:
- **Unsupported type**: "Cannot convert {type} to string" (input: value, hint: "Provide number, boolean, or string")

---

### 5.3 Parse Number

**Package**: `example-parse-number`
**Category**: Data
**Description**: Converts text to numeric values

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| text | string | No | Text to parse |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| number | f32 | Parsed number |

**Behavior**:
- Parses integers and decimals
- Handles negative numbers
- Scientific notation supported (e.g., "1.5e10")

**Examples**:
```
Input: "42"
Output: 42.0

Input: "3.14"
Output: 3.14

Input: "-10"
Output: -10.0

Input: "1.5e2"
Output: 150.0
```

**Errors**:
- **Parse error**: "Cannot parse '{text}' as number" (input: text, hint: "Provide numeric text like '42' or '3.14'")

---

### 5.4 Format Template

**Package**: `example-format-template`
**Category**: Data
**Description**: Substitutes values into a template string

**Inputs**:
| Name | Type | Optional | Description |
|------|------|----------|-------------|
| template | string | No | Template with {0}, {1}, {2} placeholders |
| values | list\<string\> | No | Values to substitute |

**Outputs**:
| Name | Type | Description |
|------|------|-------------|
| result | string | Formatted string |

**Behavior**:
- Replaces {0} with values[0], {1} with values[1], etc.
- Unused placeholders remain unchanged
- Missing values leave placeholders unchanged

**Examples**:
```
Input: template="Hello {0}, you are {1} years old", values=["Alice", "30"]
Output: "Hello Alice, you are 30 years old"

Input: template="Result: {0}", values=["42"]
Output: "Result: 42"

Input: template="{0} + {1} = {2}", values=["1", "2"]
Output: "1 + 2 = {2}"  (missing value for {2})
```

**Errors**:
- **Invalid placeholder**: "Invalid placeholder format in template" (input: template, hint: "Use {0}, {1}, {2} format")

---

## Component Count Summary

| Priority | Category | Component Count | Total Lines (estimated) |
|----------|----------|----------------|------------------------|
| P1 | String Operations | 7 | ~700 (100 lines avg) |
| P2 | Comparison & Logic | 7 | ~600 (85 lines avg) |
| P3 | Math Operations | 9 | ~650 (72 lines avg) |
| P4 | List Operations | 7 | ~850 (120 lines avg) |
| P5 | Data Transformation | 4 | ~500 (125 lines avg) |
| **Total** | | **34** | **~3,300** |

**Note**: Line counts include metadata interfaces, execution logic, error handling, and unit tests. Actual implementation may vary based on complexity.

## Next Steps

With this data model defined:
1. Generate WIT contract templates in `contracts/` directory
2. Create quickstart.md with component development workflow
3. Use `/speckit.tasks` to generate actionable implementation tasks

All specifications follow the established wasmflow component pattern and are ready for implementation.
