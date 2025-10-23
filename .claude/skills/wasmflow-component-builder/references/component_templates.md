# WasmFlow Component Templates

Complete templates for all component types. Copy and customize for new components.

## Directory Structure Template

```
my-component/
├── Cargo.toml
├── build.rs
├── Justfile
├── src/
│   └── lib.rs
└── wit/
    └── node.wit    # Copy from parent wit/ directory
```

## build.rs

All components use the same build.rs:

```rust
fn main() {
    println!("cargo:rerun-if-changed=wit");
}
```

## Justfile

Generic Justfile that auto-detects component name from directory:

```just
# Generic WasmFlow Component Builder
# Auto-detects component name from directory

# Auto-detect component name from directory
component_name := replace(file_name(justfile_directory()), "-", "_")
component_display := file_name(justfile_directory())

# Build the component (default)
build: clean
    @echo "Building {{component_display}} component..."
    cargo build --target wasm32-wasip2 --release
    @echo "✓ Component built: target/wasm32-wasip2/release/{{component_name}}.wasm"

# Install the component to WasmFlow components directory
install: build
    @echo "Installing component to ../bin/"
    mkdir -p ../bin
    cp target/wasm32-wasip2/release/{{component_name}}.wasm \
       ../bin/{{component_name}}.wasm
    @echo "✓ Component installed"

# Clean build artifacts
clean:
    cargo clean

# Run tests
test:
    cargo test

# Check without building
check:
    cargo check --target wasm32-wasip2

# Install prerequisites
setup:
    @echo "Installing prerequisites..."
    rustup target add wasm32-wasip2
    @echo "✓ Prerequisites installed"

# Show component information
info:
    @echo "Component name: {{component_name}}"
    @echo "Display name: {{component_display}}"
    @echo "Directory: {{justfile_directory()}}"
```

## Template 1: Basic Component (Pure Computation)

### Cargo.toml

```toml
[package]
name = "example-component"
version = "1.0.0"
edition = "2021"
authors = ["Your Name"]

[workspace]
# This is a standalone crate, not part of the parent workspace

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.30"

[build-dependencies]
wit-component = "0.215"

[profile.release]
opt-level = "s"  # Optimize for size
lto = true
strip = true
```

### src/lib.rs

```rust
//! Component Description
//!
//! Detailed explanation of what this component does.

// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

// Implement the metadata interface
impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Component Display Name".to_string(),
            version: "1.0.0".to_string(),
            description: "Brief description".to_string(),
            author: "Your Name".to_string(),
            category: Some("Category".to_string()), // Math, Text, Data, Network, File I/O, Utility, Examples
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Input description".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "output".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Output description".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None // No special capabilities required
    }
}

// Implement the execution interface
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Component executing");

        // Extract input value
        let input_value = inputs
            .iter()
            .find(|(name, _)| name == "input")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'input' value".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect an F32 value to the input port".to_string()),
            })?;

        // Perform computation
        let result = input_value; // Replace with actual computation

        // Log result
        host::log("info", &format!("Computed result: {}", result));

        // Return outputs
        Ok(vec![("output".to_string(), Value::F32Val(result))])
    }
}

export!(Component);
```

## Template 2: Component with Custom UI

### Cargo.toml

```toml
[package]
name = "ui-component"
version = "1.0.0"
edition = "2021"
authors = ["Your Name"]

[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
# Note: Different wit-bindgen version and features for UI support
wit-bindgen = { version = "0.33.0", default-features = false, features = ["macros", "realloc"] }

[build-dependencies]
wit-component = "0.215"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

### src/lib.rs

```rust
//! Component with Custom Footer View
//!
//! This component provides a custom UI footer view.

// Generate bindings from WIT files (note: component-with-ui world)
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});

use exports::wasmflow::node::{
    metadata::{ComponentInfo, Guest as MetadataGuest, PortSpec},
    execution::{ExecutionError, Guest as ExecutionGuest, Value},
    ui::{ColoredText, FooterView, Guest as UiGuest, HorizontalLayout,
         KeyValuePair, UiElement, UiElementItem, VerticalLayout},
};
use wasmflow::node::types::DataType;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "UI Component".to_string(),
            version: "1.0.0".to_string(),
            description: "Component with custom footer".to_string(),
            author: "Your Name".to_string(),
            category: Some("Examples".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Input value".to_string(),
            }
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "output".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Output value".to_string(),
            }
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Executing UI component");

        // Extract input
        let input = inputs
            .iter()
            .find(|(name, _)| name == "input")
            .and_then(|(_, val)| match val {
                Value::F32Val(v) => Some(*v),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'input' value".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect an F32 value".to_string()),
            })?;

        // Compute output
        let output = input; // Replace with computation

        // Return outputs
        Ok(vec![
            ("output".to_string(), Value::F32Val(output)),
        ])
    }
}

/// Custom UI implementation
impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        let mut elements = Vec::new();

        // Header with color
        elements.push(UiElement::ColoredLabel(ColoredText {
            text: "📊 Component Output".to_string(),
            r: 100, g: 200, b: 255,
        }));

        elements.push(UiElement::Separator);

        // Display outputs as key-value pairs
        for (name, value) in outputs {
            let value_str = match value {
                Value::F32Val(v) => format!("{:.2}", v),
                Value::I32Val(v) => format!("{}", v),
                Value::U32Val(v) => format!("{}", v),
                Value::StringVal(s) => s,
                Value::BinaryVal(_) => "<binary data>".to_string(),
                Value::ListVal(items) => format!("[{} items]", items.len()),
            };

            elements.push(UiElement::KeyValue(KeyValuePair {
                key: name,
                value: value_str,
            }));
        }

        // Status indicator with horizontal layout
        elements.push(UiElement::Separator);
        elements.push(UiElement::Horizontal(HorizontalLayout {
            elements: vec![
                UiElementItem::Label("Status:".to_string()),
                UiElementItem::ColoredLabel(ColoredText {
                    text: "✓ Complete".to_string(),
                    r: 100, g: 255, b: 150,
                }),
            ],
        }));

        Some(FooterView { elements })
    }
}

export!(Component);
```

## Template 3: Network Component (HTTP)

### Cargo.toml

```toml
[package]
name = "network-component"
version = "1.0.0"
edition = "2021"
authors = ["Your Name"]

[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = { version = "0.33.0", default-features = false, features = ["macros", "realloc"] }

[build-dependencies]
wit-component = "0.215"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

### src/lib.rs

```rust
//! Network Component Example
//!
//! Demonstrates HTTP requests with capabilities.

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

use exports::wasmflow::node::{
    metadata::{ComponentInfo, Guest as MetadataGuest, PortSpec},
    execution::{ExecutionError, Guest as ExecutionGuest, Value},
};
use wasmflow::node::types::DataType;
use wasmflow::node::host;

use wasi::http::types::{Fields, Method, OutgoingRequest, Scheme};
use wasi::http::outgoing_handler;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "HTTP Fetch".to_string(),
            version: "1.0.0".to_string(),
            description: "Fetches data from HTTP endpoints".to_string(),
            author: "Your Name".to_string(),
            category: Some("Network".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "url".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "URL to fetch".to_string(),
            }
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "data".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Fetched data".to_string(),
            }
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        // Declare required network access
        Some(vec![
            "network:api.example.com".to_string(),
            "network:httpbin.org".to_string(),
        ])
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Executing HTTP fetch");

        // Extract URL
        let url = inputs
            .iter()
            .find(|(name, _)| name == "url")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'url' input".to_string(),
                input_name: Some("url".to_string()),
                recovery_hint: Some("Provide a valid URL string".to_string()),
            })?;

        // Parse URL (simplified - production code should use proper URL parsing)
        // For this example, assume URL format: https://hostname/path

        // Create HTTP request
        let request = OutgoingRequest::new(Fields::new());
        request.set_method(&Method::Get).map_err(|_| ExecutionError {
            message: "Failed to set HTTP method".to_string(),
            input_name: None,
            recovery_hint: None,
        })?;
        request.set_scheme(Some(&Scheme::Https)).map_err(|_| ExecutionError {
            message: "Failed to set scheme".to_string(),
            input_name: None,
            recovery_hint: None,
        })?;
        request.set_authority(Some("api.example.com")).map_err(|_| ExecutionError {
            message: "Failed to set authority".to_string(),
            input_name: None,
            recovery_hint: None,
        })?;
        request.set_path_with_query(Some("/data")).map_err(|_| ExecutionError {
            message: "Failed to set path".to_string(),
            input_name: None,
            recovery_hint: None,
        })?;

        // Send request
        let future = outgoing_handler::handle(request, None).map_err(|e| ExecutionError {
            message: format!("HTTP request failed: {:?}", e),
            input_name: None,
            recovery_hint: Some("Check network permissions and URL".to_string()),
        })?;

        // Get response (simplified - production code should handle response properly)
        // Note: This is a simplified example. Actual implementation would need to:
        // 1. Wait for response
        // 2. Read response body
        // 3. Handle errors
        let data = "Response data".to_string();

        host::log("info", "HTTP fetch completed");

        Ok(vec![
            ("data".to_string(), Value::StringVal(data)),
        ])
    }
}

export!(Component);
```

## Template 4: Multi-Input Component

### src/lib.rs snippet (handling multiple inputs)

```rust
impl MetadataGuest for Component {
    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "required".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Required input".to_string(),
            },
            PortSpec {
                name: "optional".to_string(),
                data_type: DataType::F32Type,
                optional: true,
                description: "Optional input".to_string(),
            },
            PortSpec {
                name: "text".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Text input".to_string(),
            },
        ]
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Required input - error if missing
        let required = inputs
            .iter()
            .find(|(name, _)| name == "required")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'required' input".to_string(),
                input_name: Some("required".to_string()),
                recovery_hint: Some("Connect an F32 value to 'required' port".to_string()),
            })?;

        // Optional input - use default if missing
        let optional = inputs
            .iter()
            .find(|(name, _)| name == "optional")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(1.0); // Default value

        // String input
        let text = inputs
            .iter()
            .find(|(name, _)| name == "text")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'text' input".to_string(),
                input_name: Some("text".to_string()),
                recovery_hint: Some("Connect a string value to 'text' port".to_string()),
            })?;

        // ... computation using all inputs

        Ok(outputs)
    }
}
```

## Template 5: List Processing Component

### src/lib.rs snippet (handling lists)

```rust
impl MetadataGuest for Component {
    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "numbers".to_string(),
                data_type: DataType::ListType(Box::new(DataType::F32Type)),
                optional: false,
                description: "List of numbers to process".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "result".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Aggregated result".to_string(),
            },
        ]
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract list of numbers
        let numbers = inputs
            .iter()
            .find(|(name, _)| name == "numbers")
            .and_then(|(_, val)| match val {
                Value::ListVal(items) => {
                    // Convert list items to f32
                    let nums: Vec<f32> = items
                        .iter()
                        .filter_map(|item| match item {
                            Value::F32Val(f) => Some(*f),
                            _ => None,
                        })
                        .collect();
                    Some(nums)
                }
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'numbers' input".to_string(),
                input_name: Some("numbers".to_string()),
                recovery_hint: Some("Connect a list of F32 values".to_string()),
            })?;

        // Process list (example: sum)
        let sum: f32 = numbers.iter().sum();

        Ok(vec![
            ("result".to_string(), Value::F32Val(sum)),
        ])
    }
}
```

## Quick Reference: Data Types

```rust
// Port specifications
DataType::U32Type        -> Value::U32Val(u32)
DataType::I32Type        -> Value::I32Val(i32)
DataType::F32Type        -> Value::F32Val(f32)
DataType::StringType     -> Value::StringVal(String)
DataType::BinaryType     -> Value::BinaryVal(Vec<u8>)
DataType::ListType(T)    -> Value::ListVal(Vec<Value>)
DataType::AnyType        -> Any Value variant

// Common categories
"Math"      - Mathematical operations
"Text"      - String manipulation
"Data"      - Data processing (JSON, CSV, etc.)
"Network"   - HTTP, API clients
"File I/O"  - File operations
"Utility"   - General utilities
"Examples"  - Tutorial components
```

## Common Patterns

### Error Handling

```rust
// Input validation error
.ok_or_else(|| ExecutionError {
    message: "Missing or invalid input".to_string(),
    input_name: Some("input_name".to_string()),
    recovery_hint: Some("Connect a valid value".to_string()),
})?

// Operation error
.map_err(|e| ExecutionError {
    message: format!("Operation failed: {}", e),
    input_name: None,
    recovery_hint: Some("Check input values".to_string()),
})?
```

### Logging

```rust
use wasmflow::node::host;

host::log("debug", "Debug message");
host::log("info", &format!("Info: {}", value));
host::log("warn", "Warning message");
host::log("error", "Error message");
```

### Multiple Outputs

```rust
Ok(vec![
    ("output1".to_string(), Value::F32Val(value1)),
    ("output2".to_string(), Value::StringVal(value2)),
    ("status".to_string(), Value::StringVal("Success".to_string())),
])
```
