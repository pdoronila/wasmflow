# WIT Bindings Integration Guide

This document explains how to replace the hardcoded component execution logic with proper WIT bindings that actually call the WASM component's code.

## Current Problem

Right now in `src/runtime/wasm_host.rs:398-413`, we have:

```rust
if component_id == "user:double_number" {
    let result = val * 2.0;  // HARDCODED - not calling the WASM component!
    ...
}
```

This means we're **not actually running the WASM code** - we're just duplicating its logic in Rust.

## Solution: Use WIT Bindings

We need to use `wasmtime::component::bindgen!` to generate Rust bindings from the WIT interface, then call the component's actual `execute()` function.

### Step 1: Add WIT Bindings Generation

Add this near the top of `src/runtime/wasm_host.rs`:

```rust
// Generate bindings from WIT files
wasmtime::component::bindgen!({
    path: "wit",
    world: "component",
    async: true,
});
```

This generates types and traits like:
- `Component` trait (different from wasmtime's Component type)
- `ComponentPre` for pre-instantiated components  
- `wasmflow::node::types::Value` for data
- `wasmflow::node::execution::Guest` trait that components implement

### Step 2: Implement Host Functions

Components import `wasmflow:node/host`, so we need to provide it:

```rust
impl wasmflow::node::host::Host for HostState {
    async fn log(&mut self, level: String, message: String) -> Result<()> {
        self.log(&level, &message);
        Ok(())
    }

    async fn get_temp_dir(&mut self) -> Result<Result<String, String>> {
        Ok(self.get_temp_dir())
    }
}
```

### Step 3: Update add_host_functions()

```rust
fn add_host_functions(linker: &mut Linker<HostState>) -> Result<()> {
    // Add WasmFlow host functions
    wasmflow::node::host::add_to_linker(linker, |state| state)?;
    Ok(())
}
```

### Step 4: Update execute_component() to Call WASM

Replace the hardcoded logic (lines 391-420) with:

```rust
pub async fn execute_component(
    &self,
    component_id: &str,
    inputs: &HashMap<String, NodeValue>,
    capabilities: CapabilitySet,
) -> Result<HashMap<String, NodeValue>, ComponentError> {
    // Get component
    let component = self.get_component(component_id)
        .ok_or_else(|| ComponentError::ExecutionError(format!(
            "Component not loaded: {}",
            component_id
        )))?;

    // Create host state
    let host_state = HostState::new(component_id.to_string(), capabilities)
        .map_err(|e| ComponentError::ExecutionError(format!("Failed to create host state: {}", e)))?;

    let mut store = Store::new(&self.engine, host_state);

    // Instantiate component using generated bindings
    let linker = self.linker.lock().unwrap();
    let instance = Component::instantiate_async(&mut store, &component, &linker)
        .await
        .map_err(|e| ComponentError::ExecutionError(format!("Failed to instantiate: {}", e)))?;

    // Convert inputs from NodeValue to WIT Value
    let wit_inputs: Vec<(String, wasmflow::node::types::Value)> = inputs.iter()
        .map(|(name, value)| (name.clone(), node_value_to_wit(value)))
        .collect();

    // Call the component's execute() function
    let result = instance.wasmflow_node_execution()
        .call_execute(&mut store, &wit_inputs)
        .await
        .map_err(|e| ComponentError::ExecutionError(format!("Execution failed: {}", e)))?;

    // Handle result
    match result {
        Ok(wit_outputs) => {
            // Convert WIT outputs back to NodeValue
            let outputs: HashMap<String, NodeValue> = wit_outputs.iter()
                .map(|(name, value)| (name.clone(), wit_to_node_value(value)))
                .collect();
            
            log::debug!("Component {} executed successfully", component_id);
            Ok(outputs)
        }
        Err(err) => {
            Err(ComponentError::ExecutionError(format!(
                "Component execution error: {}",
                err.message
            )))
        }
    }
}
```

### Step 5: Add Data Conversion Functions

```rust
/// Convert NodeValue to WIT Value
fn node_value_to_wit(value: &NodeValue) -> wasmflow::node::types::Value {
    use wasmflow::node::types::Value;
    match value {
        NodeValue::U32(v) => Value::U32Val(*v),
        NodeValue::I32(v) => Value::I32Val(*v),
        NodeValue::F32(v) => Value::F32Val(*v),
        NodeValue::String(s) => Value::StringVal(s.clone()),
        NodeValue::Binary(b) => Value::BinaryVal(b.clone()),
        NodeValue::List(items) => Value::ListVal(
            items.iter().map(node_value_to_wit).collect()
        ),
        NodeValue::Record(_) => {
            // Records need special handling - not implemented yet
            Value::StringVal("<record>".to_string())
        }
    }
}

/// Convert WIT Value to NodeValue
fn wit_to_node_value(value: &wasmflow::node::types::Value) -> NodeValue {
    use wasmflow::node::types::Value;
    match value {
        Value::U32Val(v) => NodeValue::U32(*v),
        Value::I32Val(v) => NodeValue::I32(*v),
        Value::F32Val(v) => NodeValue::F32(*v),
        Value::StringVal(s) => NodeValue::String(s.clone()),
        Value::BinaryVal(b) => NodeValue::Binary(b.clone()),
        Value::ListVal(items) => NodeValue::List(
            items.iter().map(wit_to_node_value).collect()
        ),
    }
}
```

## Testing

After making these changes:

1. Rebuild WasmFlow: `cargo build --release`
2. The double_number component will now **actually execute the WASM code**
3. Any component that implements the WIT interface will work automatically

## Benefits

✅ **Actually runs WASM code** - no more hardcoding logic  
✅ **Works with any component** - not limited to double_number  
✅ **Type-safe** - compile-time checking of WIT interfaces  
✅ **Dynamic metadata** - can call `get-info()`, `get-inputs()`, etc. from components  
✅ **Proper sandboxing** - WASM security guarantees apply  

## Current Status

- ✅ WIT interface defined (`wit/node.wit`)
- ⏳ WIT bindings generation (needs `bindgen!` macro)
- ⏳ Host trait implementation (needs `impl Host for HostState`)
- ⏳ Component execution (needs to call actual WASM functions)
- ⏳ Data conversion (needs `node_value_to_wit` / `wit_to_node_value`)

## Next Steps

1. Add the `bindgen!` macro to `src/runtime/wasm_host.rs`
2. Implement the `Host` trait
3. Replace hardcoded execution logic
4. Test with the double_number component
5. Remove the temporary hardcoded logic

Once complete, you'll be able to load ANY WASM component that implements the interface!
