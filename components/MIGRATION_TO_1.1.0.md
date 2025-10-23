# Migration Guide: wasmflow:node@1.0.0 → 1.1.0

## Overview

The WIT specification was updated from version 1.0.0 to 1.1.0 to support boolean values and typed lists. All existing components need to be rebuilt to work with the updated runtime.

## What Changed

### New Types in wasmflow:node@1.1.0

**DataType enum:**
- Added: `bool-type`

**Value variant:**
- Added: `bool-val(bool)`
- Added: `string-list-val(list<string>)`
- Added: `u32-list-val(list<u32>)`
- Added: `f32-list-val(list<f32>)`
- Removed: `list-val(list<value>)` (WIT doesn't support recursive types)

## Components Status

### ✅ Already Updated (v1.1.0)
These components were built with the new specification:
- string-concat
- string-trim
- string-length
- string-case
- string-substring
- string-contains
- string-split

### 🔄 Need Rebuilding (v1.0.0 → v1.1.0)
These components have updated WIT specs but need rebuilding:
- echo (code updated for new Value variants)
- adder
- double-number
- http-fetch
- json-parser
- file-reader
- footer-view (uses `component-with-ui` world, not standard `component` world)

## How to Rebuild Old Components

### Option 1: Use the Helper Script (Recommended)

```bash
cd components
./rebuild-old-components.sh
```

The script will:
- Build each old component with the updated WIT specification
- Copy binaries to the `components/bin/` directory
- Report success/failure for each component

### Option 2: Manual Build

For each component:

```bash
cd components/<component-name>
cargo build --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/<component_name>.wasm ../bin/
```

**Note:** Component binary names use underscores:
- `echo` → `echo.wasm`
- `http-fetch` → `http_fetch.wasm`
- `double-number` → `double_number.wasm`
- `adder` → `example_adder.wasm`
- etc.

### Option 3: Use Nushell + Just

If you have `just` and `nushell` installed:

```bash
cd components
just install-all
```

## Code Changes Required

### For Components with Exhaustive Pattern Matches

If your component has exhaustive pattern matching on `Value` variants (no catch-all `_` pattern), you must add the new variants:

**Example (echo component):**

```rust
// BEFORE (won't compile with v1.1.0)
let value_type = match &value {
    Value::U32Val(_) => "u32",
    Value::I32Val(_) => "i32",
    Value::F32Val(_) => "f32",
    Value::StringVal(_) => "string",
    Value::BinaryVal(_) => "binary",
};

// AFTER (v1.1.0 compatible)
let value_type = match &value {
    Value::U32Val(_) => "u32",
    Value::I32Val(_) => "i32",
    Value::F32Val(_) => "f32",
    Value::StringVal(_) => "string",
    Value::BoolVal(_) => "bool",              // NEW
    Value::BinaryVal(_) => "binary",
    Value::StringListVal(_) => "string-list", // NEW
    Value::U32ListVal(_) => "u32-list",       // NEW
    Value::F32ListVal(_) => "f32-list",       // NEW
};
```

### For Components with Catch-All Patterns

If your component uses a catch-all pattern (`_ => ...`), no code changes are needed:

```rust
// This pattern works with both v1.0.0 and v1.1.0
.and_then(|(_, val)| match val {
    Value::F32Val(f) => Some(*f),
    _ => None,  // Handles all other variants gracefully
})
```

Most old components (adder, double-number, http-fetch, json-parser, file-reader, footer-view) already use catch-all patterns and should build without code changes.

## Verification

After rebuilding:

1. **Check binary existence:**
   ```bash
   ls -lh components/bin/*.wasm
   ```

2. **Run wasmflow:**
   ```bash
   cargo run
   ```

3. **Verify component loading:**
   - Check that all components appear in the palette
   - String components and old components should both load
   - Status message should show "Loaded X components" with no errors

## Why This Migration Is Necessary

The WebAssembly Component Model requires exact version matching for imports. When the runtime was updated to `wasmflow:node@1.1.0`, it can no longer load components compiled with `wasmflow:node@1.0.0` because:

1. **Package version mismatch** - WIT package versions must match exactly
2. **Interface changes** - New types in the interface require updated bindings
3. **Metadata extraction** - Component instantiation fails when versions don't match

## Troubleshooting

### "Failed to load component" errors

**Symptom:** Components in `bin/` directory don't appear in the UI palette.

**Cause:** Components are still compiled with v1.0.0 specification.

**Solution:** Rebuild the component with the updated WIT specification.

### Compilation errors about non-exhaustive patterns

**Symptom:**
```
error[E0004]: non-exhaustive patterns: `&types::Value::BoolVal(_)`,
  `&types::Value::StringListVal(_)`, ...
```

**Cause:** Component code uses exhaustive pattern matching without handling new variants.

**Solution:** Add match arms for the new Value variants (see "Code Changes Required" above).

### Network errors during build

**Symptom:**
```
error: failed to get `wit-bindgen` as a dependency
  failed to get successful HTTP response from crates.io
```

**Cause:** Temporary network issues preventing cargo from accessing crates.io.

**Solution:** Wait and retry the build. Dependencies should already be cached after the first successful build.

## Files Changed

- ✅ `wit/node.wit` - Root WIT specification (1.0.0 → 1.1.0)
- ✅ `components/.templates/node.wit` - Template for standard components
- ✅ `components/.templates/node-with-ui.wit` - Template for UI components (NEW)
- ✅ All old component WIT files (`components/*/wit/node.wit`)
- ✅ `components/footer-view/wit/node.wit` - Restored component-with-ui world
- ✅ `components/echo/src/lib.rs` - Code updated for new Value variants
- ✅ Runtime type system (`src/graph/node.rs`, `src/runtime/wasm_host.rs`, etc.)

## Component Worlds

There are two WIT world types:

### Standard Component World
Most components use the standard `component` world:
```wit
world component {
    import host;
    export metadata;
    export execution;
}
```
**Template:** `components/.templates/node.wit`

### Component with UI World
Components that provide custom footer rendering use `component-with-ui`:
```wit
world component-with-ui {
    import host;
    export metadata;
    export execution;
    export ui;  // Additional UI interface
}
```
**Template:** `components/.templates/node-with-ui.wit`
**Example:** `components/footer-view`

## Additional Resources

- WIT Specification: `wit/node.wit`
- Component Development Guide: `COMPONENT_DEVELOPMENT.md`
- String Components Guide: `components/core/COMPONENT_UPDATE_GUIDE.md`
