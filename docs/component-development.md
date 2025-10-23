# Component Development Guide

**A practical, step-by-step tutorial for creating WasmFlow components**

This guide walks you through the complete workflow of developing custom components for WasmFlow, from initial setup to deployment and testing.

## Table of Contents

1. [Environment Setup](#environment-setup)
2. [Tutorial: Your First Component](#tutorial-your-first-component)
3. [Understanding the WIT Interface](#understanding-the-wit-interface)
4. [Testing Workflow](#testing-workflow)
5. [Working with Capabilities](#working-with-capabilities)
6. [Advanced Patterns](#advanced-patterns)
7. [Deployment and Distribution](#deployment-and-distribution)

---

## Environment Setup

### Initial Installation

Set up your development environment once:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add WebAssembly target
rustup target add wasm32-wasip2

# Install component build tool
cargo install cargo-component

# Verify installation
cargo component --version
```

### Project Structure

We'll use this workspace structure:

```
my-components/               # Your components workspace
├── components/              # Built .wasm files go here
├── example-adder/          # Individual component projects
│   ├── Cargo.toml
│   ├── wit/
│   │   └── node.wit
│   └── src/
│       └── lib.rs
├── example-filter/
│   └── ...
└── README.md
```

---

## Tutorial: Your First Component

Let's build a simple "Add Numbers" component from scratch.

### Step 1: Create the Project

```bash
# Create workspace directory
mkdir my-components
cd my-components

# Create the component
cargo component new example-adder --lib
cd example-adder
```

### Step 2: Configure Cargo.toml

Edit `Cargo.toml`:

```toml
[package]
name = "example-adder"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]

[lib]
crate-type = ["cdylib"]

# Configure cargo-component to use wasm32-wasip2
[package.metadata.component]
package = "wasmflow:node"

[package.metadata.component.target]
path = "wit"

[dependencies]
# Component model bindings
cargo-component-bindings = "0.6"
wit-bindgen-rt = "0.44"

[profile.release]
opt-level = "s"  # Optimize for size
lto = true       # Link-time optimization
strip = true     # Strip debug symbols
```

### Step 3: Set Up WIT Interface

Copy the WasmFlow WIT interface to your project:

```bash
# If you have access to WasmFlow source:
cp -r /path/to/wasmflow/wit ./

# Or create wit/node.wit manually (see WIT Interface section below)
```

### Step 4: Implement the Component

Edit `src/lib.rs`:

```rust
//! Example Adder Component
//!
//! Adds two numbers together - demonstrates basic component structure

#[allow(warnings)]
mod bindings;

use bindings::exports::wasmflow::node::metadata::Guest as MetadataGuest;
use bindings::exports::wasmflow::node::execution::Guest as ExecutionGuest;
use bindings::wasmflow::node::types::*;
use bindings::wasmflow::node::host;

struct Component;

// Step 4a: Implement metadata interface
impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Add Numbers".to_string(),
            version: "1.0.0".to_string(),
            description: "Adds two numbers together".to_string(),
            author: "Your Name".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "a".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "First number".to_string(),
            },
            PortSpec {
                name: "b".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Second number".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "sum".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Sum of a and b".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None // No special capabilities needed
    }
}

// Step 4b: Implement execution interface
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Log execution start
        host::log("debug", "Add Numbers component executing");

        // Extract input 'a'
        let a = inputs
            .iter()
            .find(|(name, _)| name == "a")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'a' value".to_string(),
                input_name: Some("a".to_string()),
                recovery_hint: Some("Connect an F32 value to the 'a' input port".to_string()),
            })?;

        // Extract input 'b'
        let b = inputs
            .iter()
            .find(|(name, _)| name == "b")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'b' value".to_string(),
                input_name: Some("b".to_string()),
                recovery_hint: Some("Connect an F32 value to the 'b' input port".to_string()),
            })?;

        // Perform the computation
        let sum = a + b;

        // Log the result
        let log_msg = format!("{} + {} = {}", a, b, sum);
        host::log("info", &log_msg);

        // Return the output
        Ok(vec![("sum".to_string(), Value::F32Val(sum))])
    }
}

// Step 4c: Export the component
bindings::export!(Component with_types_in bindings);
```

### Step 5: Build the Component

```bash
# Build for development
cargo component build --release

# Find your built component
ls -lh target/wasm32-wasip1/release/example_adder.wasm
```

**Expected output**: A `.wasm` file around 50-100KB

### Step 6: Load into WasmFlow

```bash
# Copy to WasmFlow components directory
cp target/wasm32-wasip1/release/example_adder.wasm \
   /path/to/wasmflow/components/

# Or if running from WasmFlow repo:
cp target/wasm32-wasip1/release/example_adder.wasm \
   ../../components/
```

Then in WasmFlow:
1. **File → Reload Components**
2. Find "Add Numbers" in the **Math** category of the palette
3. Drag it onto the canvas

### Step 7: Test in WasmFlow

Create a simple test graph:

1. Add two **Constant** nodes (set values to 5.0 and 3.0)
2. Add your **Add Numbers** component
3. Connect:
   - Constant(5.0) → a
   - Constant(3.0) → b
4. Click **Execute**
5. Verify output: `sum = 8.0`

**Congratulations!** You've built your first WasmFlow component!

---

## Understanding the WIT Interface

The WIT (WebAssembly Interface Types) interface defines the contract between WasmFlow and your component.

### The wasmflow:node Interface

All components must implement:

```wit
// Located in wit/node.wit
package wasmflow:node@1.0.0;

// Metadata: Component information
interface metadata {
    get-info: func() -> component-info;
    get-inputs: func() -> list<port-spec>;
    get-outputs: func() -> list<port-spec>;
    get-capabilities: func() -> option<list<string>>;
}

// Execution: Computational logic
interface execution {
    execute: func(inputs: list<tuple<string, value>>)
        -> result<list<tuple<string, value>>, execution-error>;
}

// Host: Functions provided by WasmFlow
interface host {
    log: func(level: string, message: string);
    get-temp-dir: func() -> result<string, string>;
}
```

### Data Types

WasmFlow supports these data types:

| WIT Type | Rust Type | Use Case |
|----------|-----------|----------|
| `u32-type` | `u32` | Positive integers, counts |
| `i32-type` | `i32` | Signed integers |
| `f32-type` | `f32` | Floating point numbers |
| `string-type` | `String` | Text data |
| `binary-type` | `Vec<u8>` | Raw bytes, files |
| `list-type` | (not yet supported) | Arrays of values |
| `any-type` | (any) | Generic/polymorphic nodes |

### Port Specifications

Each port needs:

```rust
PortSpec {
    name: "my_input".to_string(),      // Unique identifier
    data_type: DataType::F32Type,       // Type constraint
    optional: false,                    // Required vs optional
    description: "Clear description".to_string(),  // Help text
}
```

**Best practices**:
- Use lowercase with underscores for port names: `input_value`, `file_path`
- Provide clear, actionable descriptions
- Mark ports as `optional: true` only if the component has sensible defaults

---

## Testing Workflow

### Local Unit Tests

Create `tests/component_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_numbers() {
        let inputs = vec![
            ("a".to_string(), Value::F32Val(5.0)),
            ("b".to_string(), Value::F32Val(3.0)),
        ];

        let result = Component::execute(inputs).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "sum");
        match result[0].1 {
            Value::F32Val(v) => assert_eq!(v, 8.0),
            _ => panic!("Wrong output type"),
        }
    }

    #[test]
    fn test_missing_input() {
        let inputs = vec![
            ("a".to_string(), Value::F32Val(5.0)),
            // Missing 'b'
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.input_name, Some("b".to_string()));
    }

    #[test]
    fn test_wrong_type() {
        let inputs = vec![
            ("a".to_string(), Value::StringVal("not a number".to_string())),
            ("b".to_string(), Value::F32Val(3.0)),
        ];

        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
```

Run tests:

```bash
cargo test
```

### Integration Testing in WasmFlow

1. **Manual testing**:
   - Create test graphs
   - Verify output values
   - Test error conditions (missing inputs, wrong types)

2. **Automated testing** (advanced):
   - Create `.wasmflow` graph files with test cases
   - Run via CLI: `wasmflow --graph test.wasmflow --execute`
   - Parse output for validation

### Debugging Tips

**Enable debug logging**:

```rust
// In your component
host::log("debug", &format!("Input value: {:?}", input));
host::log("info", "Starting computation...");
```

Then run WasmFlow with:

```bash
RUST_LOG=debug cargo run
```

**Common issues**:

1. **Component doesn't appear in palette**
   - Check file has `.wasm` extension
   - Verify it's in `components/` directory
   - Look for errors in WasmFlow console

2. **Execution fails silently**
   - Add logging to your `execute()` function
   - Check for panics (use `Result` everywhere)
   - Verify input/output types match

3. **Build errors**
   - Ensure WIT files are up to date
   - Check Cargo.toml has correct `crate-type`
   - Verify target is `wasm32-wasip2`

---

## Working with Capabilities

Components that need system access must declare capabilities.

### File System Access

**Example: File Reader Component**

```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec!["file-read:/tmp".to_string()])
}

fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
    // Extract file path
    let path = extract_string_input(&inputs, "path")?;

    // Validate path is within allowed scope
    if !path.starts_with("/tmp") {
        return Err(ExecutionError {
            message: "Access denied: path outside allowed scope".to_string(),
            input_name: Some("path".to_string()),
            recovery_hint: Some("Only /tmp directory is accessible".to_string()),
        });
    }

    // Read file
    use std::fs;
    let content = fs::read_to_string(&path)
        .map_err(|e| ExecutionError {
            message: format!("Failed to read file: {}", e),
            input_name: Some("path".to_string()),
            recovery_hint: Some("Check file exists and is readable".to_string()),
        })?;

    Ok(vec![("content".to_string(), Value::StringVal(content))])
}
```

### Network Access

**Example: HTTP Fetch Component**

```rust
fn get_capabilities() -> Option<Vec<String>> {
    Some(vec![
        "network:api.example.com".to_string(),
        "network:httpbin.org".to_string(),
    ])
}

fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
    let url = extract_string_input(&inputs, "url")?;

    // Validate URL is in allowed list
    let allowed_hosts = ["api.example.com", "httpbin.org"];
    let is_allowed = allowed_hosts.iter().any(|host| url.contains(host));

    if !is_allowed {
        return Err(ExecutionError {
            message: "Access denied: host not in allowed list".to_string(),
            input_name: Some("url".to_string()),
            recovery_hint: Some(
                "Only api.example.com and httpbin.org are accessible".to_string()
            ),
        });
    }

    // TODO: Actual HTTP implementation would go here
    // (requires WASI HTTP bindings)

    Ok(vec![
        ("body".to_string(), Value::StringVal("response".to_string())),
        ("status".to_string(), Value::U32Val(200)),
    ])
}
```

### Capability Types

| Capability | Format | Risk Level | Use Case |
|------------|--------|------------|----------|
| File Read | `file-read:/path` | Medium | Load config, read data |
| File Write | `file-write:/path` | High | Save output, cache |
| Network | `network:host.com` | Medium | API calls, web scraping |
| Process | `process` | High | Run external tools |
| Environment | `env` | Medium | Read env vars |
| Time | `time` | Low | Timestamps, scheduling |

**Security best practices**:
- Request **minimum necessary** capabilities
- Validate all inputs before system access
- Use specific paths, not wildcards (`/tmp/myapp` not `/`)
- Specify exact hosts (`api.example.com` not `*`)

---

## Advanced Patterns

### Helper Functions for Input Extraction

Create reusable extractors:

```rust
fn extract_f32_input(
    inputs: &[(String, Value)],
    name: &str,
) -> Result<f32, ExecutionError> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, val)| match val {
            Value::F32Val(f) => Some(*f),
            _ => None,
        })
        .ok_or_else(|| ExecutionError {
            message: format!("Missing or invalid '{}' value", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(format!(
                "Connect an F32 value to the '{}' input port",
                name
            )),
        })
}

fn extract_string_input(
    inputs: &[(String, Value)],
    name: &str,
) -> Result<String, ExecutionError> {
    inputs
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, val)| match val {
            Value::StringVal(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| ExecutionError {
            message: format!("Missing or invalid '{}' value", name),
            input_name: Some(name.to_string()),
            recovery_hint: Some(format!(
                "Connect a String value to the '{}' input port",
                name
            )),
        })
}
```

### Optional Inputs with Defaults

```rust
fn get_inputs() -> Vec<PortSpec> {
    vec![
        PortSpec {
            name: "value".to_string(),
            data_type: DataType::F32Type,
            optional: false,  // Required
            description: "Input value".to_string(),
        },
        PortSpec {
            name: "multiplier".to_string(),
            data_type: DataType::F32Type,
            optional: true,   // Optional
            description: "Multiplier (default: 2.0)".to_string(),
        },
    ]
}

fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
    let value = extract_f32_input(&inputs, "value")?;

    // Optional input with default
    let multiplier = extract_f32_input(&inputs, "multiplier")
        .unwrap_or(2.0);

    let result = value * multiplier;
    Ok(vec![("output".to_string(), Value::F32Val(result))])
}
```

### Multiple Outputs

```rust
fn get_outputs() -> Vec<PortSpec> {
    vec![
        PortSpec {
            name: "quotient".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Division result".to_string(),
        },
        PortSpec {
            name: "remainder".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Division remainder".to_string(),
        },
    ]
}

fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
    let dividend = extract_f32_input(&inputs, "dividend")?;
    let divisor = extract_f32_input(&inputs, "divisor")?;

    if divisor == 0.0 {
        return Err(ExecutionError {
            message: "Division by zero".to_string(),
            input_name: Some("divisor".to_string()),
            recovery_hint: Some("Ensure divisor is not zero".to_string()),
        });
    }

    let quotient = (dividend / divisor).floor();
    let remainder = dividend % divisor;

    Ok(vec![
        ("quotient".to_string(), Value::F32Val(quotient)),
        ("remainder".to_string(), Value::F32Val(remainder)),
    ])
}
```

---

## Deployment and Distribution

### Sharing Your Component

**Option 1: Direct distribution**

```bash
# Package component with README
tar -czf my-component-v1.0.0.tar.gz \
    target/wasm32-wasip1/release/my_component.wasm \
    README.md

# Users extract and copy to their components/ directory
```

**Option 2: Git repository**

```bash
# Your repo structure
my-component/
├── Cargo.toml
├── README.md
├── wit/
├── src/
└── examples/

# Users clone and build:
git clone https://github.com/you/my-component.git
cd my-component
cargo component build --release
cp target/wasm32-wasip1/release/*.wasm /path/to/wasmflow/components/
```

### Versioning

Follow semantic versioning:

```rust
fn get_info() -> ComponentInfo {
    ComponentInfo {
        name: "My Component".to_string(),
        version: "1.2.3".to_string(),  // MAJOR.MINOR.PATCH
        // ...
    }
}
```

- **MAJOR**: Breaking changes (incompatible port changes)
- **MINOR**: New features (new optional ports)
- **PATCH**: Bug fixes

### Documentation

Include in your repository:

1. **README.md**: Overview, installation, usage examples
2. **CHANGELOG.md**: Version history
3. **LICENSE**: Open source license (MIT, Apache, etc.)
4. **examples/**: Sample graphs demonstrating usage

---

## Next Steps

Now that you understand component development:

1. **Explore examples**: Check out `examples/` in the WasmFlow repository
2. **Build your own**: Create components for your specific use cases
3. **Read the reference**: See [BUILDING_COMPONENTS.md](./BUILDING_COMPONENTS.md) for detailed API docs
4. **Join the community**: Share your components and get help

Happy building! 🚀
