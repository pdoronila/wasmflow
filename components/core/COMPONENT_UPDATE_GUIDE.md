# Component Update Guide - wasmflow:node@1.0.0 Migration

## Status Overview

### ✅ Complete and Building
- **string-concat** - Multi-input concatenation
- **string-trim** - Whitespace removal
- **string-length** - Unicode-aware character counting

### ⏳ Needs Update (Simple - No List Types)
- **string-case** - Case conversion
- **string-contains** - Substring search
- **string-substring** - Substring extraction

### ⚠️ Blocked - Requires List Type Support
- **string-split** - Returns list of strings (WIT doesn't support lists yet)

---

## Update Pattern

For each component, apply these changes to `src/lib.rs`:

### 1. Update Imports (Lines 1-10)

**Before:**
```rust
wit_bindgen::generate!({
    world: "component",
    path: "./wit",
});

use exports::metadata::Guest as MetadataGuest;
use exports::execution::Guest as ExecutionGuest;
use exports::{
    ComponentInfo, PortSpec, DataType, InputValue, OutputValue,
    ExecutionError, NodeValue,
};
```

**After:**
```rust
wit_bindgen::generate!({
    path: "./wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;
```

### 2. Update ComponentInfo - Add Category

**Before:**
```rust
ComponentInfo {
    name: "String Case".to_string(),
    description: "Converts string case...".to_string(),
    category: "Text".to_string(),
    version: "1.0.0".to_string(),
}
```

**After:**
```rust
ComponentInfo {
    name: "String Case".to_string(),
    version: "1.0.0".to_string(),
    description: "Converts string case...".to_string(),
    author: "WasmFlow Core Library".to_string(),
    category: Some("Core".to_string()),
}
```

### 3. Update DataType - Add -Type Suffix

**Before:**
```rust
data_type: DataType::String,
data_type: DataType::U32,
data_type: DataType::Bool,
```

**After:**
```rust
data_type: DataType::StringType,
data_type: DataType::U32Type,
data_type: DataType::BoolType,  // Note: Not used in current components
```

### 4. Update Execute Signature

**Before:**
```rust
fn execute(inputs: Vec<InputValue>) -> Result<Vec<OutputValue>, ExecutionError>
```

**After:**
```rust
fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError>
```

### 5. Update Input Extraction

**Before:**
```rust
let text = inputs.iter()
    .find(|i| i.name == "text")
    .and_then(|i| match &i.value {
        NodeValue::String(s) => Some(s.clone()),
        _ => None,
    })
    .ok_or_else(|| ExecutionError { ... })?;
```

**After:**
```rust
let text = inputs
    .iter()
    .find(|(n, _)| n == "text")
    .and_then(|(_, v)| {
        if let Value::StringVal(s) = v {
            Some(s.clone())
        } else {
            None
        }
    })
    .ok_or_else(|| ExecutionError {
        message: "Missing or invalid 'text' input".to_string(),
        input_name: Some("text".to_string()),
        recovery_hint: Some("Provide a string value".to_string()),
    })?;
```

### 6. Update Output Creation

**Before:**
```rust
Ok(vec![OutputValue {
    name: "result".to_string(),
    value: NodeValue::String(result),
}])
```

**After:**
```rust
Ok(vec![("result".to_string(), Value::StringVal(result))])
```

### 7. Add Logging (Optional but Recommended)

```rust
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "String Case executing");

        // ... rest of implementation
    }
}
```

### 8. Update Tests

**Before:**
```rust
let inputs = vec![
    InputValue {
        name: "text".to_string(),
        value: NodeValue::String("hello".to_string()),
    },
];

match &result[0].value {
    NodeValue::String(s) => assert_eq!(s, "hello"),
    _ => panic!("Expected string output"),
}
```

**After:**
```rust
let inputs = vec![
    ("text".to_string(), Value::StringVal("hello".to_string())),
];

match &result[0].1 {  // .1 gets the Value from the tuple
    Value::StringVal(s) => assert_eq!(s, "hello"),
    _ => panic!("Expected string output"),
}
```

---

## Component-Specific Updates

### string-case

**Inputs**: text (string), operation (string)
**Output**: result (string)
**Logic**: Match operation → uppercase/lowercase/titlecase

**Key Points**:
- Keep the `titlecase()` helper function
- Error on invalid operation
- Test all 3 operations + error case

### string-contains

**Inputs**: text (string), substring (string)
**Output**: result (bool - but wait, we don't have BoolType!)

**⚠️ ISSUE**: Component returns bool, but current Value variant doesn't have a bool type!

**Options**:
1. Return u32 (0 or 1)
2. Return string ("true" or "false")
3. Add BoolType to WIT (recommended)

### string-substring

**Inputs**: text (string), start (u32), length (u32, optional)
**Output**: result (string)
**Logic**: Unicode-aware character extraction

**Key Points**:
- Use `.chars().collect()` for Unicode safety
- Handle start >= text length → return empty string
- If no length provided, extract to end

---

## 🚨 Critical Issue: Missing Types in WIT

The current `wasmflow:node@1.0.0` WIT specification is **missing** support for:

### 1. List/Array Types
**Needed for**: string-split (returns list<string>)

**Current Value variant**:
```wit
variant value {
    u32-val(u32),
    i32-val(s32),
    f32-val(f32),
    string-val(string),
    binary-val(list<u8>),
}
```

**Missing**: Generic list type like `list-val(list<value>)`

**Workarounds**:
1. **JSON encoding**: Return list as JSON string `["a", "b", "c"]`
2. **Multiple outputs**: Create output ports `part1`, `part2`, etc. (not scalable)
3. **Binary encoding**: Encode as binary-val with custom serialization

**Recommended**: Add to WIT:
```wit
variant value {
    u32-val(u32),
    i32-val(s32),
    f32-val(f32),
    string-val(string),
    binary-val(list<u8>),
    bool-val(bool),              // Add bool support
    list-val(list<value>),       // Add recursive list support
}
```

### 2. Bool Type
**Needed for**: string-contains (returns bool)

**Current Issue**: No `bool-val` variant in Value

**Workarounds**:
1. Return u32 (0 = false, 1 = true)
2. Return string ("true" or "false")

**Recommended**: Add `bool-val(bool)` to Value variant

---

## Testing After Update

After updating each component:

```bash
cd components/core/string-<component-name>
just build
```

Expected output:
```
Building string-<name> component...
   Compiling example-string-<name> v1.0.0
    Finished `release` profile [optimized] target(s) in X.XXs
✓ Component built: target/wasm32-wasip2/release/example_string_<name>.wasm
```

If build succeeds, test installation:
```bash
just install
```

This copies the .wasm to `components/bin/`.

---

## Next Steps

1. **Add missing types to WIT** (bool-val, list-val)
2. **Update remaining 3 simple components** (case, contains, substring)
3. **Update string-split** once list support is added
4. **Update string-concat** to use the optional inputs properly
5. **Integration testing** with wasmflow UI

---

## Reference: Complete Example (string-length)

See `/home/user/wasmflow/components/core/string-length/src/lib.rs` for a complete, working example following all the patterns above.

---

**Last Updated**: 2025-10-23
**Status**: 3/7 components complete, 2 types missing from WIT (bool, list)
