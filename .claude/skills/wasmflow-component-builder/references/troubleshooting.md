# WasmFlow Component Troubleshooting Guide

Common problems and solutions when building WASM components.

## Build Errors

### Error: "crate-type must be cdylib"

**Symptom:**
```
error: cannot produce cdylib for `component` as the target `wasm32-wasip2` does not support these crate types
```

**Solution:**
Ensure `Cargo.toml` has correct `crate-type`:
```toml
[lib]
crate-type = ["cdylib"]
```

---

### Error: "target wasm32-wasip2 not installed"

**Symptom:**
```
error: target 'wasm32-wasip2' not found
```

**Solution:**
Install the WASM target:
```bash
rustup target add wasm32-wasip2
```

Or use the setup command:
```bash
just setup
```

---

### Error: "package has a workspace field but is not a member"

**Symptom:**
```
error: package has a workspace field but is not a member of the workspace
```

**Solution:**
Add workspace declaration to `Cargo.toml`:
```toml
[workspace]
# This is a standalone crate, not part of the parent workspace
```

---

### Error: "unresolved import wit_bindgen"

**Symptom:**
```
error[E0432]: unresolved import `wit_bindgen`
```

**Solution:**
Add `wit-bindgen` to dependencies in `Cargo.toml`:
```toml
[dependencies]
wit-bindgen = "0.30"
```

For UI components:
```toml
[dependencies]
wit-bindgen = { version = "0.33.0", default-features = false, features = ["macros", "realloc"] }
```

---

### Error: "world `component-with-ui` not found"

**Symptom:**
```
error: world `component-with-ui` not found in package
```

**Solution:**
Ensure `wit/node.wit` is copied from parent directory:
```bash
mkdir wit
cp ../wit/node.wit wit/
```

---

### Error: Large WASM file size

**Symptom:**
Component `.wasm` file is > 1MB

**Solution:**
Verify release profile settings in `Cargo.toml`:
```toml
[profile.release]
opt-level = "s"  # Optimize for size
lto = true       # Link-time optimization
strip = true     # Strip debug symbols
```

Build in release mode:
```bash
cargo build --target wasm32-wasip2 --release
```

---

## Runtime Errors

### Error: "Component failed to load"

**Symptom:**
Component doesn't appear in palette after File → Reload Components

**Diagnosis:**
1. Check `components/bin/` for `.wasm` file:
   ```bash
   ls -lh components/bin/
   ```

2. Verify build completed successfully:
   ```bash
   cargo check --target wasm32-wasip2
   ```

3. Check wasmflow logs for error messages

**Solution:**
- Rebuild component: `just build && just install`
- Verify WIT interface is correct
- Check component exports all required interfaces

---

### Error: "Missing or invalid input"

**Symptom:**
Component execution fails with `ExecutionError`

**Diagnosis:**
Check input extraction logic:
```rust
let value = inputs.iter()
    .find(|(name, _)| name == "port_name")
    .and_then(|(_, val)| match val {
        Value::F32Val(f) => Some(*f),
        _ => None,
    })
```

**Common Issues:**
1. **Port name mismatch** - `find()` name doesn't match `get_inputs()` name
2. **Type mismatch** - Value variant doesn't match DataType
3. **Missing optional check** - Required input not connected

**Solution:**
```rust
// Verify port names match exactly
fn get_inputs() -> Vec<PortSpec> {
    vec![PortSpec {
        name: "input".to_string(),  // Must match exactly
        // ...
    }]
}

fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
    let value = inputs.iter()
        .find(|(name, _)| name == "input")  // Same name
        // ...
}
```

---

### Error: Type mismatch during execution

**Symptom:**
```
ExecutionError { message: "Missing or invalid 'input' value" }
```

**Diagnosis:**
Port data type doesn't match value variant:

**Problem:**
```rust
// Port spec says F32Type
PortSpec {
    data_type: DataType::F32Type,
    // ...
}

// But extracting I32Val
match val {
    Value::I32Val(i) => Some(*i),  // Wrong type!
    _ => None,
}
```

**Solution:**
Match port spec data type to value variant:
```rust
DataType::U32Type  -> Value::U32Val(u32)
DataType::I32Type  -> Value::I32Val(i32)
DataType::F32Type  -> Value::F32Val(f32)
DataType::StringType -> Value::StringVal(String)
DataType::BinaryType -> Value::BinaryVal(Vec<u8>)
```

---

### Error: "Network access denied"

**Symptom:**
HTTP component fails with permission error

**Diagnosis:**
Capabilities not declared or incorrect format

**Solution:**
Declare capabilities in `get_capabilities()`:
```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "network:api.example.com".to_string(),
    ])
}
```

**Capability format:**
- Network: `network:<hostname>` (no protocol, no path)
- Correct: `network:api.github.com`
- Wrong: `https://api.github.com`, `network:api.github.com/path`

---

## UI Rendering Errors

### Error: Custom footer not rendering

**Symptom:**
Component has `impl UiGuest` but footer shows default view

**Diagnosis:**
1. Check using correct world in `wit_bindgen::generate!`:
   ```rust
   wit_bindgen::generate!({
       path: "wit",
       world: "component-with-ui",  // Must be this
   });
   ```

2. Check `Cargo.toml` has correct wit-bindgen version:
   ```toml
   wit-bindgen = { version = "0.33.0", default-features = false, features = ["macros", "realloc"] }
   ```

3. Verify `export!(Component)` is present

**Solution:**
Complete UI component template:
```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});

use exports::wasmflow::node::ui::Guest as UiGuest;

impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        Some(FooterView { elements: vec![/* ... */] })
    }
}

export!(Component);
```

---

### Error: "Horizontal layout appears vertical"

**Symptom:**
Elements in `HorizontalLayout` render vertically

**Diagnosis:**
Using `UiElement` instead of `UiElementItem` inside layout

**Problem:**
```rust
UiElement::Horizontal(HorizontalLayout {
    elements: vec![
        UiElement::Label("Wrong!".to_string()),  // Wrong type!
    ],
})
```

**Solution:**
Use `UiElementItem` inside layouts:
```rust
use exports::wasmflow::node::ui::{UiElement, UiElementItem, HorizontalLayout};

UiElement::Horizontal(HorizontalLayout {
    elements: vec![
        UiElementItem::Label("Correct".to_string()),
        UiElementItem::ColoredLabel(/* ... */),
    ],
})
```

**Rule:**
- `UiElement` for top-level elements
- `UiElementItem` for elements inside layouts

---

### Error: Footer text wrapping incorrectly

**Symptom:**
Text appears in narrow columns or wraps strangely

**Diagnosis:**
This is a host rendering issue, not a component issue.

**Note:**
The wasmflow UI renderer handles layout constraints. Component only provides declarative UI elements. If layout looks wrong, the issue is in `src/ui/wit_ui_renderer.rs` or `src/ui/canvas.rs`, not the component.

**Workaround:**
Keep labels concise to avoid wrapping issues.

---

## Integration Issues

### Error: Component not appearing in palette

**Symptom:**
Component builds successfully but doesn't appear in palette

**Diagnosis:**
1. Check .wasm file installed:
   ```bash
   ls components/bin/*.wasm
   ```

2. Check file size (should be < 50MB):
   ```bash
   ls -lh components/bin/component.wasm
   ```

3. Reload components in wasmflow:
   - File → Reload Components

**Solution:**
```bash
# Rebuild and install
just build
just install

# In wasmflow application
File → Reload Components
```

---

### Error: Component loads but palette shows wrong category

**Symptom:**
Component appears in wrong category or no category

**Diagnosis:**
Check `get_info()` category field:
```rust
fn get_info() -> ComponentInfo {
    ComponentInfo {
        category: Some("Math".to_string()),  // Must be valid category
        // ...
    }
}
```

**Valid Categories:**
- `"Math"`
- `"Text"`
- `"Data"`
- `"Network"`
- `"File I/O"`
- `"Utility"`
- `"Examples"`

**Solution:**
Use exact category name (case-sensitive) or `None` for uncategorized.

---

### Error: Graph won't save with component

**Symptom:**
Saved graph fails to load when component is used

**Diagnosis:**
Component ID or version changed

**Solution:**
- Keep component ID stable (based on package name)
- Use semantic versioning
- Don't change component name in `ComponentInfo`

---

### Error: Component works in dev but not in release build

**Symptom:**
Component functions correctly with `cargo build` but fails with `--release`

**Diagnosis:**
Optimization breaking assumptions (rare with WASM)

**Solution:**
1. Check for undefined behavior
2. Disable specific optimizations:
   ```toml
   [profile.release]
   opt-level = 2  # Instead of "s"
   lto = false
   ```

3. Add debug logging to identify where failure occurs

---

## Testing Issues

### Error: "Cargo test fails to build"

**Symptom:**
```
cargo test
error: cannot compile for `wasm32-wasip2` in test mode
```

**Diagnosis:**
Tests can't run on WASM target

**Solution:**
Use conditional compilation for tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic() {
        // Test non-WASM logic
    }
}
```

Or run tests on host:
```bash
cargo test  # Runs on host, not wasm32-wasip2
```

---

## Performance Issues

### Error: Component execution is slow

**Diagnosis:**
1. Check execution logic for inefficiencies
2. Verify using release build
3. Check input/output data sizes

**Solution:**

**Optimize computation:**
```rust
// Avoid repeated allocations
let mut result = Vec::with_capacity(expected_size);

// Use iterators instead of loops
items.iter()
    .map(|x| x * 2.0)
    .collect()
```

**Ensure release build:**
```bash
cargo build --target wasm32-wasip2 --release
```

**Profile with logging:**
```rust
use wasmflow::node::host;

let start = std::time::Instant::now();
// ... computation
let elapsed = start.elapsed();
host::log("debug", &format!("Computation took {:?}", elapsed));
```

---

### Error: Graph execution locks up

**Symptom:**
Graph stops responding during execution

**Diagnosis:**
1. Infinite loop in component
2. Deadlock in async code
3. Very large data processing

**Solution:**

**Add timeout logging:**
```rust
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
        host::log("debug", "Starting execution");

        // ... computation

        host::log("debug", "Execution complete");
        Ok(outputs)
    }
}
```

**Avoid infinite loops:**
```rust
// Add iteration limit
const MAX_ITERATIONS: usize = 10000;
for i in 0..MAX_ITERATIONS {
    // ...
    if condition { break; }
}
```

---

## WASI-Specific Issues

### Error: "WASI interface not found"

**Symptom:**
```
error: interface `wasi:http/types@0.2.0` not found
```

**Diagnosis:**
WASI imports not configured in `wit_bindgen::generate!`

**Solution:**
Add WASI interfaces to `with` block:
```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
    with: {
        "wasi:io/error@0.2.0": generate,
        "wasi:io/poll@0.2.0": generate,
        "wasi:io/streams@0.2.0": generate,
        "wasi:http/types@0.2.0": generate,
        "wasi:http/outgoing-handler@0.2.0": generate,
    },
});
```

---

### Error: "File system access denied"

**Symptom:**
Component can't read/write files even with capabilities

**Diagnosis:**
1. Capabilities not declared correctly
2. Path outside permitted directories
3. Using absolute paths instead of relative

**Solution:**

**Declare file capabilities:**
```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "file:read:/path/to/dir".to_string(),
    ])
}
```

**Use temp directory:**
```rust
let temp_dir = host::get_temp_dir()?;
let file_path = format!("{}/file.txt", temp_dir);
```

---

## Debugging Strategies

### Enable Debug Logging

Add comprehensive logging:
```rust
use wasmflow::node::host;

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
        host::log("debug", "=== Execution Start ===");
        host::log("debug", &format!("Inputs: {} ports", inputs.len()));

        for (name, value) in &inputs {
            host::log("debug", &format!("  {}: {:?}", name, value));
        }

        // ... computation

        host::log("debug", "=== Execution Complete ===");
        Ok(outputs)
    }
}
```

### Validate Inputs Early

```rust
fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
    // Log all inputs first
    host::log("debug", &format!("Received {} inputs", inputs.len()));

    // Validate each input individually
    let input_a = inputs.iter()
        .find(|(name, _)| name == "a")
        .ok_or_else(|| {
            host::log("error", "Input 'a' not found");
            ExecutionError { /* ... */ }
        })?;

    // ... continue
}
```

### Test Incrementally

Build complexity gradually:
1. Start with simple passthrough
2. Add input extraction
3. Add computation
4. Add error handling
5. Add custom UI

### Use Minimal Reproducible Example

Create simple test component:
```rust
// Simplest possible component
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
        host::log("info", "Component executed");
        Ok(vec![("output".to_string(), Value::F32Val(42.0))])
    }
}
```

If this works, add complexity step by step.

---

## Getting Help

### Information to Include

When reporting issues, provide:

1. **Component code** (especially `get_inputs()`, `get_outputs()`, `execute()`)
2. **Cargo.toml** dependencies
3. **Build output** (full error message)
4. **Runtime logs** from wasmflow console
5. **Steps to reproduce**

### Verification Checklist

Before reporting:

- [ ] `cargo check --target wasm32-wasip2` succeeds
- [ ] `just build` completes without errors
- [ ] `.wasm` file exists in `components/bin/`
- [ ] File → Reload Components executed
- [ ] Component appears in palette
- [ ] Input/output types match between spec and execution
- [ ] Capabilities declared if needed
- [ ] Using correct world (`component` vs `component-with-ui`)
- [ ] `wit/node.wit` copied from parent directory
