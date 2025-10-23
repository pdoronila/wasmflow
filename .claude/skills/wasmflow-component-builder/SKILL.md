---
name: wasmflow-component-builder
description: Build WebAssembly components for the wasmflow visual programming application. Use when creating new WASM components with WIT interfaces, implementing custom footer views, adding network/file I/O capabilities, or integrating components into the wasmflow node graph system. Covers component structure, build system, WIT bindings, and UI integration.
---

# WasmFlow Component Builder

## Overview

Build WebAssembly components that integrate with the wasmflow visual programming application. Components are WASM modules with WIT (WebAssembly Interface Types) interfaces that expose computation nodes in the visual graph editor. This skill covers the complete workflow from component creation to integration, including custom UI rendering, capability management, and build automation.

## Core Concepts

### Component Architecture

**Component Types:**
- **Basic Components** - Pure computation (math, text processing)
- **UI Components** - Include custom footer views for rich output display
- **Capability Components** - Require file I/O or network access

**WIT Interface Contract:**
Every component implements the WIT interface defined in `wit/node.wit`:
- **metadata** - Component info, input/output ports, capabilities
- **execution** - Main computation logic
- **ui** (optional) - Custom footer view rendering

**Data Flow:**
1. Host application converts NodeValue → WIT Value
2. Component executes with WIT-typed inputs
3. Component returns WIT-typed outputs
4. Host converts WIT Value → NodeValue
5. Optional: Component provides FooterView for UI rendering

### Project Structure

```
wasmflow/
├── components/                    # Component source directory
│   ├── bin/                      # Compiled .wasm files
│   ├── Justfile                  # Top-level build automation
│   └── <component-name>/         # Individual component directory
│       ├── Cargo.toml
│       ├── Justfile             # Component build script
│       ├── build.rs
│       ├── src/lib.rs
│       └── wit/node.wit         # Copy of main WIT interface
├── wit/node.wit                  # Main WIT interface definition
└── src/
    ├── runtime/wasm_host.rs     # Component manager and execution
    ├── graph/node.rs            # Component registry
    └── ui/wit_ui_renderer.rs    # UI element rendering
```

## Quick Start Guide

### Creating a New Component

**1. Navigate to components directory:**
```bash
cd components/
```

**2. Create component structure:**
```bash
mkdir my-component && cd my-component
```

**3. Copy template files:**
Use the templates in `references/component_templates.md`:
- `Cargo.toml` - Package configuration
- `build.rs` - Build script
- `Justfile` - Build automation
- `wit/node.wit` - WIT interface (copy from parent `wit/` directory)
- `src/lib.rs` - Component implementation

**4. Build and install:**
```bash
just build    # Compile to WASM
just install  # Copy to bin/ directory
```

**5. Load in wasmflow:**
- Run wasmflow application
- File → Reload Components
- Find component in palette under specified category

## Component Implementation Patterns

### Pattern 1: Basic Component (Pure Computation)

**Use when:** Component performs pure computation without UI or I/O

**Example:** Double Number component (multiplies input by 2)

**Implementation steps:**

1. **Configure Cargo.toml:**
```toml
[package]
name = "double-number"
version = "1.0.0"
edition = "2021"

[workspace]  # Standalone crate

[lib]
crate-type = ["cdylib"]  # Critical: Must be cdylib

[dependencies]
wit-bindgen = "0.30"

[build-dependencies]
wit-component = "0.215"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

2. **Implement in src/lib.rs:**
```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component",  // Basic world
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Double Number".to_string(),
            version: "1.0.0".to_string(),
            description: "Multiplies input by 2".to_string(),
            author: "Your Name".to_string(),
            category: Some("Math".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "input".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Number to double".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "output".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Doubled number".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None  // Pure computation, no capabilities needed
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Executing double-number");

        // Extract input value
        let input_value = inputs.iter()
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
        let result = input_value * 2.0;

        // Return output
        Ok(vec![("output".to_string(), Value::F32Val(result))])
    }
}

export!(Component);
```

3. **Build and test:**
```bash
just build
just install
# Load in wasmflow and test
```

### Pattern 2: Component with Custom UI

**Use when:** Component needs rich visual output display (charts, formatted data, status indicators)

**Key differences from basic components:**
- Use `component-with-ui` world instead of `component`
- Implement `ui::Guest` trait
- Return `FooterView` with declarative UI elements
- Use `wit-bindgen` version 0.33.0+ with specific features

**Implementation steps:**

1. **Update Cargo.toml:**
```toml
[dependencies]
wit-bindgen = { version = "0.33.0", default-features = false, features = ["macros", "realloc"] }
```

2. **Generate bindings for UI world:**
```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",  // UI-enabled world
});

use exports::wasmflow::node::ui::{
    ColoredText, FooterView, Guest as UiGuest,
    HorizontalLayout, KeyValuePair, UiElement, UiElementItem
};
```

3. **Implement UI trait:**
```rust
impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        let mut elements = Vec::new();

        // Header with color
        elements.push(UiElement::ColoredLabel(ColoredText {
            text: "📊 Results".to_string(),
            r: 100, g: 200, b: 255,
        }));

        elements.push(UiElement::Separator);

        // Display outputs as key-value pairs
        for (name, value) in outputs {
            let value_str = match value {
                Value::F32Val(v) => format!("{:.2}", v),
                Value::StringVal(s) => s,
                Value::U32Val(v) => v.to_string(),
                Value::I32Val(v) => v.to_string(),
                Value::BinaryVal(_) => "<binary data>".to_string(),
                Value::ListVal(items) => format!("[{} items]", items.len()),
            };

            elements.push(UiElement::KeyValue(KeyValuePair {
                key: name,
                value: value_str,
            }));
        }

        // Status indicator with horizontal layout
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
```

**Available UI Elements:**
- `UiElement::Label(String)` - Plain text
- `UiElement::ColoredLabel(ColoredText)` - Colored text (RGB)
- `UiElement::KeyValue(KeyValuePair)` - Key-value display
- `UiElement::Horizontal(HorizontalLayout)` - Horizontal layout
- `UiElement::Vertical(VerticalLayout)` - Vertical layout
- `UiElement::Separator` - Visual separator line

**UI Layout Guidelines:**
- Use full-width labels for main content: `UiElement::Label`
- Use horizontal layouts for inline items: `UiElement::Horizontal`
- Use key-value pairs for structured data: `UiElement::KeyValue`
- Use colored labels for status/highlights: `UiElement::ColoredLabel`
- Add separators between logical sections: `UiElement::Separator`

**Critical:** For horizontal layouts, use `UiElementItem` variants (not `UiElement`) to prevent layout breaking.

### Pattern 3: Component with Capabilities (Network/File I/O)

**Use when:** Component needs to access network resources or file system

**Key requirements:**
- Declare capabilities in `get_capabilities()`
- Use WASI interfaces (wasi:http, wasi:filesystem)
- Configure wit-bindgen with WASI imports

**Example: HTTP Fetch Component**

1. **Configure wit-bindgen with WASI:**
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

use wasi::http::types::{Fields, Method, OutgoingRequest, Scheme};
use wasi::http::outgoing_handler;
```

2. **Declare capabilities:**
```rust
impl MetadataGuest for Component {
    fn get_capabilities() -> Option<Vec<String>> {
        Some(vec![
            "network:httpbin.org".to_string(),
            "network:api.example.com".to_string(),
        ])
    }
}
```

3. **Use WASI APIs in execution:**
```rust
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract URL from inputs
        let url = /* extract URL string */;

        // Create HTTP request
        let request = OutgoingRequest::new(Fields::new());
        request.set_method(&Method::Get).unwrap();
        request.set_scheme(Some(&Scheme::Https)).unwrap();
        request.set_authority(Some("api.example.com")).unwrap();
        request.set_path_with_query(Some("/data")).unwrap();

        // Send request
        let response = outgoing_handler::handle(request, None)
            .map_err(|e| ExecutionError {
                message: format!("HTTP request failed: {:?}", e),
                input_name: None,
                recovery_hint: Some("Check network permissions".to_string()),
            })?;

        // Process response...

        Ok(outputs)
    }
}
```

**Capability Formats:**
- Network: `network:<hostname>` (e.g., `network:api.github.com`)
- File read: `file:read:<path>`
- File write: `file:write:<path>`

**Security Notes:**
- Capabilities are enforced by the host runtime
- Components cannot access resources outside declared capabilities
- Network access limited to specified hosts
- File access limited to specified paths

## Build System Reference

### Justfile Commands

**Top-level (components/Justfile):**
```bash
just build <component>      # Build single component
just install <component>    # Build and install to bin/
just build-all             # Build all components
just install-all           # Install all components
```

**Component-level (each component's Justfile):**
```bash
just build      # Compile to WASM
just install    # Build and copy to bin/
just clean      # Clean build artifacts
just check      # Check without building
just setup      # Install wasm32-wasip2 target
```

### Manual Build Commands

```bash
# Setup (first time only)
rustup target add wasm32-wasip2

# Build
cargo build --target wasm32-wasip2 --release

# Install
cp target/wasm32-wasip2/release/component.wasm ../bin/

# Check without building
cargo check --target wasm32-wasip2

# Clean
cargo clean
```

## Data Types and Conversion

### WIT Data Types

Defined in `wit/node.wit`:

```wit
variant data-type {
    u32-type,
    i32-type,
    f32-type,
    string-type,
    binary-type,
    list-type(data-type),
    record-type(list<tuple<string, data-type>>),
    any-type,
}

variant value {
    u32-val(u32),
    i32-val(s32),
    f32-val(float32),
    string-val(string),
    binary-val(list<u8>),
    list-val(list<value>),
}
```

### Type Matching

When defining ports, match `PortSpec::data_type` to expected `Value` variant:

```rust
// Input port expecting F32
PortSpec {
    name: "input".to_string(),
    data_type: DataType::F32Type,  // Expects Value::F32Val
    optional: false,
    description: "Float input".to_string(),
}

// Extract value in execute()
let value = match inputs.get("input") {
    Some(Value::F32Val(f)) => *f,
    _ => return Err(/* type mismatch error */),
};
```

### Handling Optional Inputs

```rust
impl MetadataGuest for Component {
    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "required".to_string(),
                data_type: DataType::StringType,
                optional: false,  // Must be connected
                description: "Required input".to_string(),
            },
            PortSpec {
                name: "optional".to_string(),
                data_type: DataType::F32Type,
                optional: true,  // May be unconnected
                description: "Optional threshold".to_string(),
            },
        ]
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Required input - return error if missing
        let required = inputs.iter()
            .find(|(name, _)| name == "required")
            .ok_or_else(|| ExecutionError {
                message: "Missing required input".to_string(),
                input_name: Some("required".to_string()),
                recovery_hint: Some("Connect a value to 'required' port".to_string()),
            })?;

        // Optional input - use default if missing
        let threshold = inputs.iter()
            .find(|(name, _)| name == "optional")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(0.5);  // Default value

        // ... execution logic
    }
}
```

## Error Handling Best Practices

### Structured Error Returns

```rust
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Input validation
        let url = inputs.iter()
            .find(|(name, _)| name == "url")
            .and_then(|(_, val)| match val {
                Value::StringVal(s) => Some(s.clone()),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Invalid or missing 'url' input".to_string(),
                input_name: Some("url".to_string()),
                recovery_hint: Some("Connect a string value containing a valid URL".to_string()),
            })?;

        // Operation error
        let data = fetch_data(&url).map_err(|e| ExecutionError {
            message: format!("Failed to fetch data: {}", e),
            input_name: None,
            recovery_hint: Some("Check URL and network permissions".to_string()),
        })?;

        // Parsing error
        let parsed = parse_json(&data).map_err(|e| ExecutionError {
            message: format!("Failed to parse JSON: {}", e),
            input_name: None,
            recovery_hint: Some("Ensure URL returns valid JSON".to_string()),
        })?;

        Ok(outputs)
    }
}
```

### Logging

Use host logging for debugging:

```rust
use wasmflow::node::host;

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Starting execution");
        host::log("info", &format!("Processing {} inputs", inputs.len()));

        // ... execution logic

        host::log("debug", "Execution complete");
        Ok(outputs)
    }
}
```

**Log levels:** `debug`, `info`, `warn`, `error`

## Integration and Testing

### Component Loading

Components are automatically discovered from `components/bin/` directory:

1. Application scans `components/bin/*.wasm`
2. Each WASM module is loaded and metadata extracted
3. Components registered in ComponentRegistry
4. Available in palette under specified category

**Manual reload:** File → Reload Components

### Testing Workflow

**1. Unit testing (Rust tests):**
```bash
cargo test
```

**2. Build verification:**
```bash
cargo check --target wasm32-wasip2
just build
```

**3. Integration testing in wasmflow:**
- Create test graph: Constant → YourComponent → Display
- Verify inputs/outputs match specification
- Test error handling (missing inputs, invalid types)
- Verify custom footer view (if applicable)
- Test capabilities (network/file access)

**4. Graph persistence testing:**
- Save graph with component
- Close and reopen graph
- Verify component loads and executes correctly

### Common Issues and Solutions

**Issue:** Component not appearing in palette
- **Solution:** Check `components/bin/` for .wasm file, File → Reload Components

**Issue:** "Component failed to load" error
- **Solution:** Run `cargo check --target wasm32-wasip2` to verify build, check wasmflow logs

**Issue:** Type mismatch errors during execution
- **Solution:** Verify PortSpec data types match Value variants in execute()

**Issue:** Custom footer not rendering
- **Solution:** Ensure using `component-with-ui` world, implement `ui::Guest` trait

**Issue:** Network/file access denied
- **Solution:** Declare capabilities in `get_capabilities()`, verify format

**Issue:** WASM module too large
- **Solution:** Verify `profile.release` settings in Cargo.toml (opt-level, lto, strip)

## Component Categories

Organize components by category for palette grouping:

- **Math** - Arithmetic, trigonometry, statistics
- **Text** - String manipulation, formatting, parsing
- **Data** - JSON, CSV, serialization, deserialization
- **Network** - HTTP requests, API clients
- **File I/O** - File reading, writing, processing
- **Utility** - General-purpose utilities
- **Examples** - Tutorial and example components

Specify in `ComponentInfo`:
```rust
ComponentInfo {
    // ...
    category: Some("Math".to_string()),
}
```

## Advanced Topics

### Complex Data Structures

**Lists:**
```rust
// Define list port
PortSpec {
    name: "numbers".to_string(),
    data_type: DataType::ListType(Box::new(DataType::F32Type)),
    optional: false,
    description: "List of numbers".to_string(),
}

// Extract list in execute()
let numbers = inputs.iter()
    .find(|(name, _)| name == "numbers")
    .and_then(|(_, val)| match val {
        Value::ListVal(items) => Some(
            items.iter()
                .filter_map(|v| match v {
                    Value::F32Val(f) => Some(*f),
                    _ => None,
                })
                .collect::<Vec<f32>>()
        ),
        _ => None,
    })
    .ok_or_else(|| /* error */)?;
```

**Records (experimental):**
Records are defined in WIT but not fully supported in current runtime. Use workarounds:
- Serialize as JSON string
- Use multiple ports for struct fields
- Use list of tuples

### Temporary File Access

```rust
use wasmflow::node::host;

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Get temp directory from host
        let temp_dir = host::get_temp_dir()
            .map_err(|e| ExecutionError {
                message: format!("Failed to get temp dir: {}", e),
                input_name: None,
                recovery_hint: None,
            })?;

        // Use temp directory for intermediate files
        let temp_file = format!("{}/intermediate.dat", temp_dir);

        // ... file operations

        Ok(outputs)
    }
}
```

### Component Composition (WAC)

Components can be composed using WebAssembly Composition (WAC):

**Future feature:** Combine multiple components into composite nodes
- Select components in graph
- Create composite with internal graph
- Expose selected inputs/outputs
- Package as single component

## Resources

### scripts/

**new_component.sh** - Interactive component scaffolding script
- Prompts for component name, category, description
- Generates Cargo.toml, src/lib.rs, Justfile from templates
- Creates wit/ directory with node.wit copy
- Initializes build.rs

### references/

**component_templates.md** - Complete file templates for all component types
- Basic component template
- UI component template
- Network component template
- File I/O component template
- Cargo.toml variants
- Justfile template

**wit_interface_reference.md** - Complete WIT interface documentation
- Full wit/node.wit specification
- All data types and variants
- Interface contracts
- Example usage patterns

**troubleshooting.md** - Common problems and solutions
- Build errors and fixes
- Runtime errors and debugging
- Integration issues
- Performance optimization

### assets/

No assets included - components are code-only.

## Version Requirements

- **Rust:** 1.75+ (stable channel)
- **Target:** wasm32-wasip2 (WASI Preview 2)
- **wit-bindgen:** 0.30+ (basic), 0.33.0+ (with UI)
- **wit-component:** 0.215+
- **wasmtime:** 27.0+ (runtime, not in component)

## See Also

- `/Users/doronila/git/wasmflow/wit/node.wit` - Main WIT interface
- `/Users/doronila/git/wasmflow/docs/BUILDING_COMPONENTS.md` - Extended documentation
- `/Users/doronila/git/wasmflow/components/README.md` - Component directory guide
- `/Users/doronila/git/wasmflow/CLAUDE.md` - Project development guidelines
