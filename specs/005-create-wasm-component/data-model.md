# Data Model: WASM Component Creator Node

**Feature**: 005-create-wasm-component
**Date**: 2025-10-18
**References**: [spec.md](./spec.md), [research.md](./research.md)

## Overview

This document defines the data structures, state transitions, and validation rules for the WASM Component Creator feature. The data model supports in-editor component creation, compilation, and dynamic loading.

## Core Entities

### 1. WasmCreatorNode

**Purpose**: Represents the creator node instance in the graph with editable code and compilation state.

**Fields**:
```rust
struct WasmCreatorNode {
    // Standard node fields
    id: NodeId,                          // Unique node identifier
    position: Pos2,                       // Canvas position

    // Creator-specific fields
    component_name: String,               // User-specified component name
    source_code: String,                  // User's Rust code
    save_code: bool,                      // Checkbox: persist code in graph

    // Compilation state
    compilation_state: CompilationState,  // Current compilation status
    last_error: Option<String>,           // Most recent error message
    generated_component_id: Option<String>, // ID if successfully compiled
}
```

**Validation Rules**:
- `component_name`: Must match regex `^[A-Z][a-zA-Z0-9_]*$` (PascalCase identifier)
- `component_name`: Length 3-50 characters
- `source_code`: Maximum 10,000 lines or 500KB (whichever smaller)
- `save_code`: Defaults to `true` for data loss prevention

**State Transitions**: See `CompilationState` below

**Relationships**:
- Has zero or one `GeneratedComponent` (after successful compilation)
- Stored in `NodeGraph.nodes` BTreeMap
- Serialized to graph JSON when graph is saved

**Serialization**:
```rust
// In graph JSON
{
  "id": "node_123",
  "type": "WasmCreator",
  "component_name": "TripleNumber",
  "source_code": "// Only if save_code=true",
  "save_code": true,
  "position": [100.0, 200.0]
}
```

---

### 2. CompilationState

**Purpose**: Tracks the current state of the compilation process for a creator node.

**Variants**:
```rust
enum CompilationState {
    Idle,                              // No compilation attempted
    Compiling {                        // Compilation in progress
        started_at: Instant,           // Timeout tracking
        pid: Option<u32>,              // Process ID for cancellation
    },
    Success {                          // Compilation succeeded
        compiled_at: Instant,
        component_path: PathBuf,       // Path to .wasm file
        build_time_ms: u64,
    },
    Failed {                           // Compilation failed
        error_message: String,         // Formatted error from cargo
        line_number: Option<usize>,    // Error location if available
        failed_at: Instant,
    },
}
```

**State Transition Diagram**:
```
Idle ──[Execute clicked]──> Compiling
  ▲                            │
  │                            ├──[Success]──> Success
  │                            │                  │
  │                            └──[Failure]──> Failed
  │                                              │
  └────────────[Edit code]──────────────────────┘
```

**Transition Rules**:
- `Idle → Compiling`: Only if `component_name` is valid and `source_code` is non-empty
- `Compiling → Success`: When cargo-component exits with code 0 and .wasm exists
- `Compiling → Failed`: When cargo-component exits non-zero or timeout (120s)
- `{Success, Failed} → Idle`: When user edits `source_code` or `component_name`
- `Compiling → Idle`: When user explicitly cancels (future enhancement)

**Timeout Handling**:
- If `Instant::now() - started_at > 120s`: Kill process, transition to `Failed`

---

### 3. GeneratedComponent

**Purpose**: Represents a dynamically loaded user-defined WASM component.

**Fields**:
```rust
struct GeneratedComponent {
    // Component metadata (from parsed annotations)
    name: String,                      // From component_name
    version: String,                   // Default "0.1.0"
    description: String,               // From // @description comment
    category: String,                  // "User-Defined" or from // @category

    // Port specifications (parsed from comments)
    inputs: Vec<PortSpec>,             // From // @input annotations
    outputs: Vec<PortSpec>,            // From // @output annotations

    // Capabilities (parsed from comments)
    capabilities: Vec<String>,         // From // @capability annotations

    // Runtime data
    wasm_path: PathBuf,                // Path to compiled .wasm file
    loaded_at: Instant,                // When registered
    source_creator_node: NodeId,       // Which creator node generated this
}
```

**Validation Rules**:
- `name`: Must match `component_name` from creator node
- `inputs`/`outputs`: Each port must have valid type from `{F32, I32, U32, String, Boolean}`
- `capabilities`: Must match allowed patterns (e.g., `network:domain.com`)
- `wasm_path`: File must exist and be valid WASM component

**Relationships**:
- Owned by `ComponentRegistry`
- Referenced by `WasmCreatorNode.generated_component_id`
- Visual color: Purple (RGB: 180, 100, 220) in palette

**Lifecycle**:
1. Created when compilation succeeds
2. Registered in `ComponentRegistry`
3. Appears in palette under "User-Defined" category
4. Replaced when same `name` recompiled
5. Persists until app restart (not saved to disk permanently)

---

### 4. ComponentTemplate

**Purpose**: Base template for generating complete WASM component Rust code.

**Fields**:
```rust
struct ComponentTemplate {
    name: &'static str,                // Template identifier
    description: &'static str,         // Template purpose
    template_source: &'static str,     // Rust code with {{placeholders}}
    required_capabilities: Vec<String>, // What this template needs
}
```

**Available Templates**:

1. **Simple Template** (`"simple"`):
   - Pure computation, no I/O
   - Placeholders: `{{COMPONENT_NAME}}`, `{{DESCRIPTION}}`, `{{INPUTS}}`, `{{OUTPUTS}}`, `{{USER_CODE}}`
   - Example: Arithmetic operations, data transformations

2. **HTTP Template** (`"http"`):
   - Network-enabled with WASI HTTP
   - Placeholders: Same as simple + `{{CAPABILITIES}}`
   - Example: API calls, webhooks

**Template Selection Logic**:
- If user code contains `// @capability network:`: Use HTTP template
- Otherwise: Use Simple template

**Placeholder Substitution**:
```rust
// Example input
component_name: "TripleNumber"
description: "Multiplies by 3"
inputs: [("value", F32)]
outputs: [("result", F32)]
user_code: "let result = input * 3.0;"

// Generates
impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<...> {
        // ... input extraction ...
        let result = input * 3.0;
        Ok(vec![("result".to_string(), Value::F32Val(result))])
    }
}
```

---

### 5. CommentAnnotation

**Purpose**: Parsed structured comment from user code.

**Fields**:
```rust
struct CommentAnnotation {
    annotation_type: AnnotationType,   // @input, @output, etc.
    content: String,                   // Everything after the @tag
    line_number: usize,                // Source location
}

enum AnnotationType {
    Input,         // @input name:type description
    Output,        // @output name:type description
    Description,   // @description text
    Category,      // @category name
    Capability,    // @capability pattern
}
```

**Parsing Rules**:
```rust
// Format: // @tag content
Regex: r"^//\s*@(\w+)\s+(.+)$"

// @input examples
"// @input value:F32 The input number"
  -> Input { name: "value", type: F32, desc: "The input number" }

"// @output result:String The result text"
  -> Output { name: "result", type: String, desc: "The result text" }

"// @description Multiplies input by 3"
  -> Description("Multiplies input by 3")

"// @capability network:api.example.com"
  -> Capability("network:api.example.com")
```

**Validation**:
- Type must be one of: `F32`, `I32`, `U32`, `String`, `Boolean`
- Name must be valid Rust identifier (lowercase_snake_case)
- Capability must match pattern: `(network|file-read|file-write):value`

**Defaults**:
- No `@input`: Single F32 input named "input"
- No `@output`: Single F32 output named "output"
- No `@description`: Use component name
- No `@category`: "User-Defined"
- No `@capability`: Empty capabilities list

---

### 6. CompilationWorkspace

**Purpose**: Temporary directory structure for cargo-component build.

**Structure**:
```
/tmp/wasmflow-build-{uuid}/
├── Cargo.toml           # Generated project manifest
├── src/
│   └── lib.rs           # Generated component code
├── wit/
│   └── world.wit        # WIT interface definition
└── target/              # Build artifacts (created by cargo)
    └── wasm32-wasip2/
        └── release/
            └── component.wasm  # Output file
```

**Lifecycle**:
1. Created in temp dir when compilation starts
2. `Cargo.toml`, `src/lib.rs`, `wit/world.wit` written
3. `cargo component build --release` invoked
4. On success: Copy `component.wasm` to permanent location
5. On completion: Delete entire workspace (cleanup)

**Cargo.toml Generation**:
```toml
[package]
name = "{{component_name_snake_case}}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.33.0"

[profile.release]
opt-level = "z"
lto = true
```

**Permanent Component Location**:
```
~/.wasmflow/user_components/{component_name}.wasm
```

---

## Data Flow

### Component Creation Flow

```
User Input (UI)
    │
    ├─> component_name ──┐
    └─> source_code ─────┤
                         │
                         ▼
              [Parse Annotations] ──> CommentAnnotation[]
                         │
                         ▼
              [Select Template] ──> ComponentTemplate
                         │
                         ▼
              [Generate Code] ──> Complete Rust source
                         │
                         ▼
              [Create Workspace] ──> CompilationWorkspace
                         │
                         ▼
              [Invoke cargo-component]
                         │
                    ┌────┴────┐
                    │         │
                Success    Failure
                    │         │
                    ▼         ▼
          [Load Component]  [Show Error]
                    │         │
                    ▼         ▼
          GeneratedComponent  CompilationState::Failed
                    │
                    ▼
          [Register in Palette]
```

### Recompilation Flow

```
User edits code
    │
    ▼
CompilationState → Idle
    │
    ▼
[Execute clicked again]
    │
    ▼
Check if GeneratedComponent exists with same name
    │
    ├─> Yes: Unregister old component
    └─> No: Continue
    │
    ▼
[Follow Creation Flow]
    │
    ▼
Replace GeneratedComponent
```

---

## Validation Rules Summary

### Component Name
- **Format**: PascalCase identifier
- **Regex**: `^[A-Z][a-zA-Z0-9_]*$`
- **Length**: 3-50 characters
- **Examples**: ✅ `TripleNumber`, `HTTPFetcher` | ❌ `triple_number`, `a`, `123Start`

### Source Code
- **Max Size**: 10,000 lines OR 500KB (whichever smaller)
- **Encoding**: UTF-8
- **Required**: Non-empty when Execute clicked

### Port Specifications
- **Name**: Valid Rust identifier (snake_case)
- **Type**: One of `{F32, I32, U32, String, Boolean}`
- **Format**: `@input name:Type description` or `@output name:Type description`

### Capabilities
- **Pattern**: `category:value`
- **Allowed Categories**: `network`, `file-read`, `file-write`
- **Network Example**: `network:api.example.com` (domain only, no paths)
- **File Example**: `file-read:/data/input` (absolute path)

### Compilation Constraints
- **Timeout**: 120 seconds
- **Concurrent Builds**: 1 per creator node (global queue possible future enhancement)
- **Workspace Cleanup**: Always performed, even on failure

---

## Persistence Model

### Graph Serialization

**When `save_code = true`**:
```json
{
  "nodes": {
    "node_123": {
      "type": "WasmCreator",
      "component_name": "TripleNumber",
      "source_code": "// @input value:F32\nlet result = value * 3.0;",
      "save_code": true,
      "position": [100.0, 200.0]
    }
  }
}
```

**When `save_code = false`**:
```json
{
  "nodes": {
    "node_123": {
      "type": "WasmCreator",
      "component_name": "TripleNumber",
      "save_code": false,
      "position": [100.0, 200.0]
      // source_code omitted
    }
  }
}
```

**Compilation State**: Never persisted (always `Idle` on load)
**Generated Components**: Not persisted (must recompile after app restart)

---

## Error Handling

### Validation Errors
- **Empty component name**: "Component name cannot be empty"
- **Invalid name format**: "Component name must be PascalCase (e.g., MyComponent)"
- **Code too large**: "Code exceeds maximum size (10,000 lines or 500KB)"
- **Invalid port type**: "Port type must be one of: F32, I32, U32, String, Boolean"

### Compilation Errors
- **Cargo not found**: "cargo-component not found. Please install: cargo install cargo-component"
- **Timeout**: "Compilation timed out after 120 seconds"
- **Syntax error**: Parse cargo output for error location and message
- **Workspace creation failure**: "Failed to create build workspace: {reason}"

### Runtime Errors
- **Component load failure**: "Failed to load compiled component: {wasm_error}"
- **Duplicate name**: "A component named '{name}' already exists. It will be replaced."
- **Disk space**: "Insufficient disk space for compilation"

---

## Performance Considerations

### Memory
- Each `WasmCreatorNode`: ~1KB + source_code size
- `CompilationWorkspace`: 5-20MB during build (temporary)
- `GeneratedComponent`: ~100KB-5MB (WASM binary)
- Target: Support 20 creator nodes = ~100MB total

### Compilation
- Small component: 3-6 seconds
- Medium component: 8-15 seconds
- Large component: 15-25 seconds
- Timeout: 120 seconds (safety margin)

### UI Responsiveness
- Code editor: 60 FPS up to 1000 lines
- Compilation runs in background (doesn't block UI)
- Progress indicator updates every 500ms

---

## Future Enhancements

### Planned
- Export user components to standalone `.wasm` files
- Import external `.wasm` files into creator nodes
- Shared component library across projects
- Build artifact caching (skip rebuild if code unchanged)

### Under Consideration
- Multiple template types (File I/O, Database, etc.)
- Visual port designer (drag-and-drop instead of comments)
- Inline error highlighting (red squiggles in editor)
- Component versioning and dependency management
