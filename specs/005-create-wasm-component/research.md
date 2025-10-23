# Research Findings: WASM Component Creator Feature

## 1. cargo-component Version and Integration

### Decision
- **Version**: Use cargo-component 0.21.1 (latest stable as of October 2025)
- **Integration Method**: Invoke via `std::process::Command` with timeout control

### Rationale
- **Stable Version**: cargo-component 0.21.1 is the current stable release available on crates.io
- **Process-based Invocation**: While cargo-component has a library API (cargo-component-core), it's designed primarily as a CLI tool. Using `std::process::Command` provides:
  - Simpler integration without complex dependency management
  - Better isolation between the host app and compilation process
  - Easier timeout handling for compilation safety
  - Standard cargo output parsing (JSON format available via `--message-format json`)
  - More stable API surface (CLI is more stable than internal library APIs)

### Alternatives Considered
1. **cargo-component-core library**: Internal library used by cargo-component
   - Pros: More direct Rust API
   - Cons: Internal API may change between versions, more complex dependency tree, harder to implement timeouts

2. **Direct wasm32-wasip2 compilation**: Using `cargo build --target wasm32-wasip2` + manual component conversion
   - Pros: No external tool dependency
   - Cons: Requires implementing WIT binding generation manually, more complex, error-prone

### Command-Line Usage
```rust
use std::process::Command;
use std::time::Duration;

// Basic build command
Command::new("cargo")
    .arg("component")
    .arg("build")
    .arg("--release")
    .arg("--message-format=json")  // For structured error parsing
    .current_dir(component_dir)
    .timeout(Duration::from_secs(120))  // 2 minute timeout
    .output()
```

### Output Format
- Use `--message-format=json` for structured output parsing
- Parse compiler errors, warnings, and build artifacts from JSON
- Standard output includes: artifact paths, error messages, build status

---

## 2. Syntax Highlighting Library

### Decision
- **Library**: Use egui_code_editor 0.2.20
- **Features**: Enable default features (includes Rust syntax highlighting)

### Rationale
- **egui Integration**: Purpose-built for egui with native widget support
  - No integration glue code needed
  - Follows egui's immediate-mode pattern
  - Consistent look and feel with rest of the application

- **Rust Support**: Built-in Rust syntax highlighting
  - Simple keyword-based highlighting (sufficient for code editor use case)
  - Multiple color themes available (Ayu, GitHub, Gruvbox, Sonokai)

- **Lightweight**: Minimal dependencies and small binary size
  - Simple lexer-based approach (not full parser like syntect)
  - ~10KB additional binary size

- **Editor Features**: Includes essential code editing features
  - Line numbers
  - Auto-completion from syntax dictionary
  - Keyboard navigation (Tab, arrows, etc.)

- **Performance**: Fast rendering for typical component code sizes
  - Immediate-mode rendering fits egui model
  - No syntax tree building overhead

### Alternatives Considered

1. **syntect 5.3.0**: Full-featured syntax highlighting library
   - Pros:
     - High-quality highlighting using Sublime Text grammars
     - Very accurate syntax parsing
     - Wide language support
   - Cons:
     - Large binary size increase (~1-2MB with syntax definitions)
     - Requires custom egui integration code
     - Slower than keyword-based highlighting
     - Overkill for basic code editing

2. **Custom implementation**: Build simple keyword highlighter
   - Pros: Complete control, minimal dependencies
   - Cons: Time-consuming, error-prone, reinventing the wheel

3. **egui_litecode 0.1.2**: Another egui code editor
   - Pros: Also built for egui
   - Cons: Less maintained (last update 1+ year ago), fewer features than egui_code_editor

### Usage Example
```rust
use egui_code_editor::{CodeEditor, ColorTheme};

CodeEditor::default()
    .with_rows(20)
    .with_numlines(true)
    .with_theme(ColorTheme::GITHUB_DARK)
    .with_syntax(egui_code_editor::Syntax::rust())
    .show(ui, &mut code_string);
```

---

## 3. Code Size and Timeout Limits

### Decision
- **Maximum Code Size**: 10,000 lines or 500KB (whichever is smaller)
- **Compilation Timeout**: 120 seconds (2 minutes)

### Rationale

#### Code Size Limit
Based on analysis of existing components:
- **double-number**: 83 lines, 75KB compiled WASM
- **example-http-fetch**: 1,225 lines, 133KB compiled WASM
- **Minimal test component**: 15 lines, 13KB compiled WASM

Rationale for 10,000 lines / 500KB:
- **Realistic Upper Bound**: Most components should be 100-500 lines
  - 10,000 lines is 10-100x larger than typical components
  - Prevents abuse while allowing complex components

- **Memory Efficiency**: 500KB of source code is reasonable for in-memory editing
  - ~500KB text = ~8,000-10,000 lines of Rust
  - Prevents UI freezing during syntax highlighting

- **User Experience**: Editor remains responsive with files up to this size
  - egui_code_editor can handle this size efficiently
  - Syntax highlighting remains fast

- **WASM Output Size**: Even large components produce reasonably sized WASM
  - 1,000 line component ≈ 100-150KB WASM (with optimizations)
  - 10,000 line component would be ~500KB-1MB WASM (still acceptable)

#### Compilation Timeout
Based on build time measurements:
- **Minimal component** (15 lines): 3-5 seconds (debug), 5-6 seconds (release)
  - Clean build with dependency downloads

- **double-number** (83 lines): ~8-12 seconds (estimated, release)

- **example-http-fetch** (1,225 lines): ~15-20 seconds (estimated, release)

Rationale for 120 seconds:
- **Safety Margin**: 2 minutes is 6-10x typical build time
  - Accounts for slower systems
  - Handles dependency compilation on first build
  - Includes WIT binding generation time

- **User Experience**: Long enough to be patient, short enough to prevent hung UI
  - Users expect compilation to take time
  - But not so long they assume it's frozen

- **Resource Protection**: Prevents runaway builds from consuming server resources
  - Malicious or buggy macro code
  - Infinite compilation loops

- **Progressive Feedback**: 120 seconds allows for streaming compiler output
  - Show progress messages during build
  - Display warnings and errors as they occur

### Implementation Notes
```rust
const MAX_CODE_LINES: usize = 10_000;
const MAX_CODE_BYTES: usize = 500 * 1024;  // 500KB
const COMPILE_TIMEOUT_SECS: u64 = 120;

// Validation before compilation
fn validate_code_size(code: &str) -> Result<(), String> {
    let line_count = code.lines().count();
    let byte_count = code.len();

    if line_count > MAX_CODE_LINES {
        return Err(format!("Code exceeds maximum {} lines", MAX_CODE_LINES));
    }

    if byte_count > MAX_CODE_BYTES {
        return Err(format!("Code exceeds maximum {}KB", MAX_CODE_BYTES / 1024));
    }

    Ok(())
}
```

### Alternatives Considered

1. **Smaller limits** (1,000 lines / 100KB):
   - Pros: Faster compilation, less resource usage
   - Cons: Too restrictive for legitimate complex components (like HTTP fetch example)

2. **Larger limits** (50,000 lines / 5MB):
   - Pros: More flexibility
   - Cons: Editor becomes sluggish, compilation too slow, higher abuse potential

3. **No limits**:
   - Pros: Maximum flexibility
   - Cons: DoS attack vector, poor UX for large files, resource exhaustion

---

## 4. Capability Declaration Syntax

### Decision
Capabilities are declared via the `get_capabilities()` function in Rust code, not in WIT files. Use domain-based network access patterns.

### Rationale

**WIT Does Not Define Capabilities**:
- WIT (WebAssembly Interface Types) defines component interfaces and type signatures
- It does NOT provide a standard syntax for declaring runtime capabilities/permissions
- Capabilities are a runtime enforcement concern, not an interface concern

**Component-Level Declaration**:
- Capabilities are declared in the component's Rust implementation
- The `get_capabilities()` function returns capability strings
- Host runtime (WasmFlow) validates and enforces these capabilities

**Domain-Based Network Access**:
- Network capabilities use domain patterns: `"network:domain.com"`
- Allows fine-grained control over which domains a component can access
- Subdomain matching supported: `"network:api.example.com"` allows `v2.api.example.com`

### Capability Format

```rust
impl MetadataGuest for Component {
    fn get_capabilities() -> Option<Vec<String>> {
        Some(vec![
            // Network access to specific domains
            "network:httpbin.org".to_string(),
            "network:api.example.com".to_string(),

            // File system access (future)
            "fs:read:/tmp".to_string(),
            "fs:write:/tmp".to_string(),
        ])
    }
}
```

### Standard Capability Patterns

1. **Network Access**: `network:<domain>`
   - Examples:
     - `"network:httpbin.org"` - Access to httpbin.org and subdomains
     - `"network:api.github.com"` - Access to GitHub API
   - Validation: Host checks URL domain against declared capabilities

2. **File System Access** (future):
   - Read: `fs:read:<path>`
   - Write: `fs:write:<path>`
   - Examples:
     - `"fs:read:/tmp"` - Read files in /tmp
     - `"fs:write:/home/user/data"` - Write to specific directory

3. **No Capabilities**: `None`
   - Pure computation components (like double-number)
   - No external resource access

### WIT Interface (for reference)
```wit
interface metadata {
    use types.{component-info, port-spec};

    // ...

    /// Get required capabilities
    /// Returns None if no special capabilities needed
    /// Returns Some(list) with capability strings
    get-capabilities: func() -> option<list<string>>;
}
```

### Alternatives Considered

1. **WIT-level capability declaration**: Add capability syntax to WIT
   - Pros: Declarative, compile-time checked
   - Cons: WIT spec doesn't support this, would require custom extensions

2. **WASI Preview 2 imports**: Use standard WASI imports in WIT world
   - Pros: Standards-compliant
   - Cons: Too coarse-grained (all-or-nothing), harder to implement fine-grained control
   - Example: Importing `wasi:http` gives access to ALL network, not specific domains

3. **Separate manifest file**: JSON/TOML capability manifest
   - Pros: Separate concerns, easier to parse
   - Cons: Extra file to manage, component metadata split across files

### Security Model
- **Declaration**: Component declares what it needs
- **Validation**: Host validates all resource access against declared capabilities
- **Enforcement**: Runtime blocks access to undeclared resources
- **Principle of Least Privilege**: Components should declare minimal required capabilities

---

## 5. Template Structure

### Decision
Provide two base templates: **Simple Component** (no external capabilities) and **HTTP Component** (with network access).

### Rationale
- **Two Common Patterns**: Analysis shows components fall into two categories:
  1. Pure computation (double-number, adder)
  2. Network-enabled (HTTP fetch)

- **Learning Curve**: Templates help users understand the structure
  - Clear examples of metadata, execution, and capability patterns
  - Copy-paste starting point for new components

- **Boilerplate Reduction**: ~30 lines of common code in every component
  - WIT binding imports
  - Struct definitions
  - Export macros

### Template 1: Simple Component (Pure Computation)

**Use Case**: Math operations, data transformations, pure functions

**Dependencies** (Cargo.toml):
```toml
[package]
name = "{{component-name}}"
version = "1.0.0"
edition = "2021"

[workspace]  # Required for standalone component

[lib]
crate-type = ["cdylib"]

[package.metadata.component]
package = "wasmflow:node"

[package.metadata.component.target]
path = "wit"

[dependencies]
cargo-component-bindings = "0.6"
wit-bindgen-rt = "0.44"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

**WIT File** (wit/world.wit):
```wit
package wasmflow:node@1.0.0;

interface types {
    variant data-type {
        u32-type,
        i32-type,
        f32-type,
        string-type,
        binary-type,
        list-type,
        any-type,
    }

    variant value {
        u32-val(u32),
        i32-val(s32),
        f32-val(f32),
        string-val(string),
        binary-val(list<u8>),
    }

    record port-spec {
        name: string,
        data-type: data-type,
        optional: bool,
        description: string,
    }

    record component-info {
        name: string,
        version: string,
        description: string,
        author: string,
        category: option<string>,
    }

    record execution-error {
        message: string,
        input-name: option<string>,
        recovery-hint: option<string>,
    }
}

interface host {
    log: func(level: string, message: string);
    get-temp-dir: func() -> result<string, string>;
}

interface metadata {
    use types.{component-info, port-spec};

    get-info: func() -> component-info;
    get-inputs: func() -> list<port-spec>;
    get-outputs: func() -> list<port-spec>;
    get-capabilities: func() -> option<list<string>>;
}

interface execution {
    use types.{value, execution-error};

    execute: func(inputs: list<tuple<string, value>>) -> result<list<tuple<string, value>>, execution-error>;
}

world component {
    import host;
    export metadata;
    export execution;
}
```

**Rust Template** (src/lib.rs):
```rust
//! {{COMPONENT_NAME}} - WasmFlow Component
//!
//! {{DESCRIPTION}}

#[allow(warnings)]
mod bindings;

use bindings::exports::wasmflow::node::metadata::Guest as MetadataGuest;
use bindings::exports::wasmflow::node::execution::Guest as ExecutionGuest;
use bindings::wasmflow::node::types::*;
use bindings::wasmflow::node::host;

struct Component;

// ============================================================================
// METADATA INTERFACE
// ============================================================================

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "{{COMPONENT_NAME}}".to_string(),
            version: "1.0.0".to_string(),
            description: "{{DESCRIPTION}}".to_string(),
            author: "{{AUTHOR}}".to_string(),
            category: Some("{{CATEGORY}}".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "input".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Input value".to_string(),
            },
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "output".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Output value".to_string(),
            },
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None  // No special capabilities required
    }
}

// ============================================================================
// EXECUTION INTERFACE
// ============================================================================

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("info", "{{COMPONENT_NAME}} executing");

        // Extract input
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
                recovery_hint: Some("Connect a value to the input port".to_string()),
            })?;

        // TODO: Implement your logic here
        let result = input_value;

        // Return output
        Ok(vec![("output".to_string(), Value::F32Val(result))])
    }
}

bindings::export!(Component with_types_in bindings);
```

### Template 2: HTTP Component (Network-Enabled)

**Use Case**: API calls, web scraping, HTTP requests

**Additional Dependencies** (Cargo.toml):
```toml
[dependencies]
wit-bindgen = { version = "0.33.0", default-features = false, features = ["macros", "realloc"] }

[build-dependencies]
wit-bindgen = "0.33.0"
```

**Extended WIT** (wit/node.wit) - adds WASI HTTP imports:
```wit
// ... (same types, host, metadata, execution interfaces)

world component-with-ui {
    import host;

    // WASI HTTP imports for network functionality
    import wasi:http/types@0.2.0;
    import wasi:http/outgoing-handler@0.2.0;

    // WASI IO imports (required by HTTP)
    import wasi:io/streams@0.2.0;
    import wasi:io/poll@0.2.0;
    import wasi:io/error@0.2.0;

    export metadata;
    export execution;
}
```

**Rust Template** (src/lib.rs) - key differences:
```rust
// Generate bindings with WASI HTTP support
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

// ... (same imports)

// WASI HTTP imports
use wasi::http::types::{Fields, Method, OutgoingRequest, Scheme};
use wasi::http::outgoing_handler;

// Metadata: Declare network capabilities
impl MetadataGuest for Component {
    // ...

    fn get_capabilities() -> Option<Vec<String>> {
        Some(vec![
            "network:api.example.com".to_string(),
        ])
    }
}

// Execution: Make HTTP request
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Extract URL input
        let url = extract_string(&inputs, "url")?;

        // Validate URL against capabilities
        validate_url(&url, &["api.example.com"])?;

        // Make HTTP request using WASI HTTP
        let response = perform_http_request(&url)?;

        Ok(vec![
            ("body".to_string(), Value::StringVal(response)),
        ])
    }
}
```

### Template Selection UI

Provide dropdown in component creator:
- **Simple Component** (default): Pure computation, no capabilities
- **HTTP Component**: Network access with WASI HTTP
- **Custom**: Start from blank template

### Key Template Elements

1. **Required Imports**:
   - `#[allow(warnings)] mod bindings;` - Generated bindings
   - Guest trait imports for metadata and execution
   - Type imports from bindings
   - Host function imports

2. **Trait Implementations**:
   - `MetadataGuest`: Component info, inputs, outputs, capabilities
   - `ExecutionGuest`: Main execution logic

3. **Export Macro**:
   - Simple: `bindings::export!(Component with_types_in bindings);`
   - HTTP: `export!(Component);` (when using wit-bindgen::generate!)

4. **Error Handling Pattern**:
   ```rust
   .ok_or_else(|| ExecutionError {
       message: "Error description".to_string(),
       input_name: Some("input_name".to_string()),
       recovery_hint: Some("How to fix".to_string()),
   })
   ```

5. **Logging Pattern**:
   ```rust
   host::log("info", "Message");
   host::log("debug", &format!("Value: {}", value));
   ```

### Alternatives Considered

1. **Single universal template**: One template for all use cases
   - Pros: Simpler to maintain
   - Cons: Confusing for beginners (too many options), includes unused imports

2. **Many specialized templates**: Templates for every capability type
   - Pros: Very specific starting points
   - Cons: Too many choices, maintenance burden, most would be rarely used

3. **Wizard-based generation**: Step-by-step template customization
   - Pros: Interactive, educational
   - Cons: Complex UI, over-engineered for simple task

4. **No templates**: Users start from scratch
   - Pros: Maximum flexibility
   - Cons: High barrier to entry, steep learning curve, error-prone

---

## Summary Recommendations

### Immediate Implementation Priorities

1. **cargo-component Integration** (High Priority):
   - Implement `std::process::Command` wrapper with timeout
   - Parse JSON output for error messages
   - Test on various component types

2. **Code Editor** (High Priority):
   - Integrate egui_code_editor 0.2.20
   - Configure with Rust syntax highlighting
   - Set up line numbers and basic auto-completion

3. **Templates** (Medium Priority):
   - Create Simple and HTTP component templates
   - Template variable substitution system
   - Template selection UI

4. **Validation** (Medium Priority):
   - Code size limits (10k lines / 500KB)
   - Compilation timeout (120s)
   - Capability declaration validation

5. **Documentation** (Low Priority):
   - Template usage guide
   - Capability patterns reference
   - Compilation error troubleshooting

### Future Considerations

1. **Additional Templates**:
   - File I/O component template (when WASI filesystem support added)
   - Timer/scheduled component template
   - Multi-output component template

2. **Enhanced Editor Features**:
   - Error highlighting in editor (parse compiler errors, show inline)
   - Auto-completion from WasmFlow types
   - Code formatting (rustfmt integration)

3. **Build Optimization**:
   - Incremental compilation support
   - Dependency caching between builds
   - Build artifact reuse

4. **Advanced Capabilities**:
   - Resource limits (memory, CPU time)
   - Capability runtime parameter (e.g., `network:*` with user approval)
   - Capability scoping (time-based, usage-limited)

---

## References

- cargo-component: https://github.com/bytecodealliance/cargo-component
- egui_code_editor: https://github.com/p4ymak/egui_code_editor
- syntect: https://github.com/trishume/syntect
- WIT specification: https://component-model.bytecodealliance.org/design/wit.html
- WASI Preview 2: https://github.com/WebAssembly/WASI
- Example components: `/Users/doronila/git/wasmflow_cc/examples/`

---

*Research completed: 2025-10-18*
*Research by: Claude (Anthropic)*
*For: WasmFlow Component Creator Feature (005-create-wasm-component)*
