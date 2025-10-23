# WIT Interface Reference

Complete reference for the WasmFlow WIT interface (`wit/node.wit`).

## Package Declaration

```wit
package wasmflow:node@1.0.0;
```

All components must use this package namespace.

## Interfaces

### 1. types - Core Data Types

Defines all data types and structures used throughout the system.

#### data-type (variant)

Port type specifications:

```wit
variant data-type {
    u32-type,          // Unsigned 32-bit integer
    i32-type,          // Signed 32-bit integer
    f32-type,          // 32-bit floating point
    string-type,       // UTF-8 string
    binary-type,       // Binary data (Vec<u8>)
    list-type,         // List of values
    any-type,          // Any value type (generic)
}
```

**Usage in Rust:**
```rust
use wasmflow::node::types::DataType;

DataType::U32Type
DataType::I32Type
DataType::F32Type
DataType::StringType
DataType::BinaryType
DataType::ListType
DataType::AnyType
```

#### value (variant)

Runtime values that flow through connections:

```wit
variant value {
    u32-val(u32),           // Unsigned 32-bit integer value
    i32-val(s32),           // Signed 32-bit integer value
    f32-val(f32),           // 32-bit float value
    string-val(string),     // String value
    binary-val(list<u8>),   // Binary data
}
```

**Note:** List values are not directly supported in the WIT definition but are handled by the runtime through recursive wrapping.

**Usage in Rust:**
```rust
use wasmflow::node::types::Value;

Value::U32Val(42)
Value::I32Val(-10)
Value::F32Val(3.14)
Value::StringVal("hello".to_string())
Value::BinaryVal(vec![0x00, 0xFF])
```

**Type Matching:**
```rust
match value {
    Value::U32Val(v) => /* handle u32 */,
    Value::I32Val(v) => /* handle i32 */,
    Value::F32Val(v) => /* handle f32 */,
    Value::StringVal(s) => /* handle string */,
    Value::BinaryVal(b) => /* handle binary */,
}
```

#### port-spec (record)

Defines a component's input or output port:

```wit
record port-spec {
    name: string,           // Port identifier (e.g., "input", "value")
    data-type: data-type,   // Expected data type
    optional: bool,         // Whether connection is optional
    description: string,    // Human-readable description
}
```

**Usage in Rust:**
```rust
use wasmflow::node::types::{PortSpec, DataType};

PortSpec {
    name: "input".to_string(),
    data_type: DataType::F32Type,
    optional: false,
    description: "Input value".to_string(),
}
```

**Optional vs Required Ports:**
- `optional: false` - Port must be connected, execution fails if missing
- `optional: true` - Port may be unconnected, component provides default

#### component-info (record)

Component metadata:

```wit
record component-info {
    name: string,              // Display name (e.g., "Add Numbers")
    version: string,           // Semantic version (e.g., "1.0.0")
    description: string,       // Brief description
    author: string,            // Author name
    category: option<string>,  // Palette category (e.g., "Math", "Text")
}
```

**Usage in Rust:**
```rust
use wasmflow::node::types::ComponentInfo;

ComponentInfo {
    name: "Double Number".to_string(),
    version: "1.0.0".to_string(),
    description: "Multiplies input by 2".to_string(),
    author: "Your Name".to_string(),
    category: Some("Math".to_string()),
}
```

**Standard Categories:**
- `"Math"` - Mathematical operations
- `"Text"` - String manipulation
- `"Data"` - Data processing
- `"Network"` - HTTP, API clients
- `"File I/O"` - File operations
- `"Utility"` - General utilities
- `"Examples"` - Tutorial components

#### execution-error (record)

Structured error for execution failures:

```wit
record execution-error {
    message: string,              // Error message
    input-name: option<string>,   // Which input caused error (if applicable)
    recovery-hint: option<string>, // How to fix the error
}
```

**Usage in Rust:**
```rust
use wasmflow::node::types::ExecutionError;

// Input validation error
ExecutionError {
    message: "Missing or invalid 'input' value".to_string(),
    input_name: Some("input".to_string()),
    recovery_hint: Some("Connect an F32 value to the input port".to_string()),
}

// General execution error
ExecutionError {
    message: "Computation failed: division by zero".to_string(),
    input_name: None,
    recovery_hint: Some("Ensure denominator is not zero".to_string()),
}
```

### 2. host - Runtime Functions

Host functions provided by WasmFlow runtime (imported by components).

#### log

Log messages to the console:

```wit
log: func(level: string, message: string);
```

**Parameters:**
- `level`: Log level - `"debug"`, `"info"`, `"warn"`, `"error"`
- `message`: Log message

**Usage in Rust:**
```rust
use wasmflow::node::host;

host::log("debug", "Starting computation");
host::log("info", &format!("Processing {} items", count));
host::log("warn", "Input value near overflow");
host::log("error", "Critical failure");
```

**Log Output:**
Logs appear in the WasmFlow application console, useful for debugging component execution.

#### get-temp-dir

Get temporary directory path:

```wit
get-temp-dir: func() -> result<string, string>;
```

**Returns:**
- `Ok(path)`: Temporary directory path
- `Err(message)`: Error message

**Usage in Rust:**
```rust
use wasmflow::node::host;

let temp_dir = host::get_temp_dir()
    .map_err(|e| ExecutionError {
        message: format!("Failed to get temp dir: {}", e),
        input_name: None,
        recovery_hint: None,
    })?;

let temp_file = format!("{}/intermediate.dat", temp_dir);
```

**Use Cases:**
- Intermediate file storage
- Temporary output before copying
- Cache files during processing

### 3. metadata - Component Information

Component metadata interface (exported by components).

#### get-info

Get component metadata:

```wit
get-info: func() -> component-info;
```

**Implementation:**
```rust
use exports::wasmflow::node::metadata::Guest as MetadataGuest;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Component Name".to_string(),
            version: "1.0.0".to_string(),
            description: "Component description".to_string(),
            author: "Your Name".to_string(),
            category: Some("Category".to_string()),
        }
    }
}
```

#### get-inputs

Get input port specifications:

```wit
get-inputs: func() -> list<port-spec>;
```

**Implementation:**
```rust
impl MetadataGuest for Component {
    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input1".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "First input".to_string(),
            },
            PortSpec {
                name: "input2".to_string(),
                data_type: DataType::StringType,
                optional: true,
                description: "Optional second input".to_string(),
            },
        ]
    }
}
```

#### get-outputs

Get output port specifications:

```wit
get-outputs: func() -> list<port-spec>;
```

**Implementation:**
```rust
impl MetadataGuest for Component {
    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "output".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Result value".to_string(),
            },
        ]
    }
}
```

#### get-capabilities

Get required capabilities:

```wit
get-capabilities: func() -> option<list<string>>;
```

**Returns:**
- `None` - No special capabilities required (pure computation)
- `Some(list)` - List of required capabilities

**Implementation:**
```rust
impl MetadataGuest for Component {
    fn get_capabilities() -> Option<Vec<String>> {
        // Pure computation - no capabilities
        None

        // OR with network access
        Some(vec![
            "network:api.example.com".to_string(),
        ])

        // OR with file access
        Some(vec![
            "file:read:/path/to/dir".to_string(),
            "file:write:/path/to/output".to_string(),
        ])
    }
}
```

**Capability Formats:**
- Network: `network:<hostname>` (e.g., `network:api.github.com`)
- File read: `file:read:<path>`
- File write: `file:write:<path>`
- File read/write: `file:rw:<path>`

### 4. execution - Component Logic

Execution interface (exported by components).

#### execute

Execute component with inputs:

```wit
execute: func(inputs: list<tuple<string, value>>) -> result<list<tuple<string, value>>, execution-error>;
```

**Parameters:**
- `inputs`: List of (name, value) pairs for connected input ports

**Returns:**
- `Ok(outputs)`: List of (name, value) pairs for output ports
- `Err(error)`: Execution error

**Implementation:**
```rust
use exports::wasmflow::node::execution::Guest as ExecutionGuest;

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // 1. Extract inputs
        let input_val = inputs
            .iter()
            .find(|(name, _)| name == "input")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'input' value".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect an F32 value".to_string()),
            })?;

        // 2. Perform computation
        let result = input_val * 2.0;

        // 3. Return outputs
        Ok(vec![
            ("output".to_string(), Value::F32Val(result)),
        ])
    }
}
```

**Input Extraction Pattern:**
```rust
// Required input with type checking
let value = inputs.iter()
    .find(|(name, _)| name == "port_name")
    .and_then(|(_, val)| match val {
        Value::F32Val(f) => Some(*f),
        _ => None,
    })
    .ok_or_else(|| ExecutionError { /* ... */ })?;

// Optional input with default
let value = inputs.iter()
    .find(|(name, _)| name == "port_name")
    .and_then(|(_, val)| match val {
        Value::F32Val(f) => Some(*f),
        _ => None,
    })
    .unwrap_or(default_value);
```

### 5. ui - Custom Footer View (Optional)

UI interface for custom footer rendering (exported by components using `component-with-ui` world).

#### UI Element Types

##### ui-element (variant)

Top-level UI elements:

```wit
variant ui-element {
    label(string),                      // Plain text
    colored-label(colored-text),        // Colored text
    key-value(key-value-pair),         // Key-value grid
    horizontal(horizontal-layout),      // Horizontal container
    vertical(vertical-layout),          // Vertical container
    separator,                          // Separator line
}
```

**Usage in Rust:**
```rust
use exports::wasmflow::node::ui::{UiElement, ColoredText, KeyValuePair};

// Plain label
UiElement::Label("Text".to_string())

// Colored label
UiElement::ColoredLabel(ColoredText {
    text: "Status: Ready".to_string(),
    r: 100, g: 255, b: 150,
})

// Key-value pair
UiElement::KeyValue(KeyValuePair {
    key: "Result".to_string(),
    value: "42.0".to_string(),
})

// Separator
UiElement::Separator
```

##### ui-element-item (variant)

Elements for use inside layouts (non-recursive):

```wit
variant ui-element-item {
    label(string),
    colored-label(colored-text),
    key-value(key-value-pair),
    separator,
}
```

**Critical:** Use `UiElementItem` inside `horizontal` and `vertical` layouts, not `UiElement`.

**Usage in Rust:**
```rust
use exports::wasmflow::node::ui::{UiElement, UiElementItem, HorizontalLayout, ColoredText};

UiElement::Horizontal(HorizontalLayout {
    elements: vec![
        UiElementItem::Label("Status:".to_string()),
        UiElementItem::ColoredLabel(ColoredText {
            text: "✓ Ready".to_string(),
            r: 100, g: 255, b: 150,
        }),
    ],
})
```

##### colored-text (record)

Text with RGB color:

```wit
record colored-text {
    text: string,
    r: u8,      // Red (0-255)
    g: u8,      // Green (0-255)
    b: u8,      // Blue (0-255)
}
```

**Usage:**
```rust
ColoredText {
    text: "Success".to_string(),
    r: 100, g: 255, b: 150,  // Light green
}
```

**Common Colors:**
- Green (success): `r: 100, g: 255, b: 150`
- Red (error): `r: 255, g: 100, b: 100`
- Blue (info): `r: 100, g: 200, b: 255`
- Yellow (warning): `r: 255, g: 255, b: 100`

##### key-value-pair (record)

Key-value display:

```wit
record key-value-pair {
    key: string,
    value: string,
}
```

**Usage:**
```rust
KeyValuePair {
    key: "Temperature".to_string(),
    value: "72.5°F".to_string(),
}
```

##### horizontal-layout (record)

Horizontal layout container:

```wit
record horizontal-layout {
    elements: list<ui-element-item>,
}
```

**Usage:**
```rust
use exports::wasmflow::node::ui::{HorizontalLayout, UiElementItem};

HorizontalLayout {
    elements: vec![
        UiElementItem::Label("Count:".to_string()),
        UiElementItem::Label("42".to_string()),
    ],
}
```

##### vertical-layout (record)

Vertical layout container:

```wit
record vertical-layout {
    elements: list<ui-element-item>,
}
```

**Usage:**
```rust
use exports::wasmflow::node::ui::{VerticalLayout, UiElementItem};

VerticalLayout {
    elements: vec![
        UiElementItem::Label("Line 1".to_string()),
        UiElementItem::Label("Line 2".to_string()),
    ],
}
```

##### footer-view (record)

Complete footer view:

```wit
record footer-view {
    elements: list<ui-element>,
}
```

**Usage:**
```rust
use exports::wasmflow::node::ui::FooterView;

FooterView {
    elements: vec![
        UiElement::Label("Header".to_string()),
        UiElement::Separator,
        UiElement::KeyValue(/* ... */),
    ],
}
```

#### get-footer-view

Get custom footer view:

```wit
get-footer-view: func(outputs: list<tuple<string, value>>) -> option<footer-view>;
```

**Parameters:**
- `outputs`: Current output values

**Returns:**
- `None`: Use default footer rendering
- `Some(view)`: Custom footer view

**Implementation:**
```rust
use exports::wasmflow::node::ui::Guest as UiGuest;

impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        let mut elements = Vec::new();

        // Header
        elements.push(UiElement::ColoredLabel(ColoredText {
            text: "📊 Results".to_string(),
            r: 100, g: 200, b: 255,
        }));

        elements.push(UiElement::Separator);

        // Display outputs
        for (name, value) in outputs {
            let value_str = match value {
                Value::F32Val(v) => format!("{:.2}", v),
                Value::I32Val(v) => format!("{}", v),
                Value::U32Val(v) => format!("{}", v),
                Value::StringVal(s) => s,
                Value::BinaryVal(_) => "<binary>".to_string(),
            };

            elements.push(UiElement::KeyValue(KeyValuePair {
                key: name,
                value: value_str,
            }));
        }

        // Status
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
```

## Worlds

### component (basic world)

For components without custom UI:

```wit
world component {
    import host;        // Use host functions
    export metadata;    // Provide metadata
    export execution;   // Provide execution
}
```

**Rust binding generation:**
```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component",
});
```

### component-with-ui (extended world)

For components with custom UI:

```wit
world component-with-ui {
    import host;        // Use host functions
    export metadata;    // Provide metadata
    export execution;   // Provide execution
    export ui;          // Provide custom UI
}
```

**Rust binding generation:**
```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});
```

**Note:** Requires `wit-bindgen` version 0.33.0+ with specific features:
```toml
wit-bindgen = { version = "0.33.0", default-features = false, features = ["macros", "realloc"] }
```

## Complete Implementation Example

### Basic Component

```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component",
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Example".to_string(),
            version: "1.0.0".to_string(),
            description: "Example component".to_string(),
            author: "Author".to_string(),
            category: Some("Examples".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "input".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Input value".to_string(),
        }]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "output".to_string(),
            data_type: DataType::F32Type,
            optional: false,
            description: "Output value".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("debug", "Executing");

        let input = inputs.iter()
            .find(|(name, _)| name == "input")
            .and_then(|(_, val)| match val {
                Value::F32Val(f) => Some(*f),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Invalid input".to_string(),
                input_name: Some("input".to_string()),
                recovery_hint: Some("Connect F32 value".to_string()),
            })?;

        let output = input * 2.0;

        Ok(vec![("output".to_string(), Value::F32Val(output))])
    }
}

export!(Component);
```

### UI Component

```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});

use exports::wasmflow::node::{
    metadata::{ComponentInfo, Guest as MetadataGuest, PortSpec},
    execution::{ExecutionError, Guest as ExecutionGuest, Value},
    ui::{ColoredText, FooterView, Guest as UiGuest, UiElement},
};
use wasmflow::node::types::DataType;

struct Component;

// ... metadata and execution implementations ...

impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        Some(FooterView {
            elements: vec![
                UiElement::ColoredLabel(ColoredText {
                    text: "Custom UI".to_string(),
                    r: 100, g: 200, b: 255,
                }),
            ],
        })
    }
}

export!(Component);
```
