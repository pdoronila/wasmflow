# Rust Module Interfaces

**Contract Stability**: EVOLVING (v0.1.0)
**Last Updated**: 2025-10-18

## Overview

This document defines the internal Rust module interfaces for the WASM Component Creator feature. These are internal APIs and may evolve in minor versions.

---

## Module: `src/runtime/compiler.rs`

### Purpose
Handles cargo-component invocation and build orchestration.

### Public Interface

```rust
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Configuration for component compilation
pub struct CompilationConfig {
    /// Component name (PascalCase)
    pub component_name: String,
    /// Generated Rust source code
    pub source_code: String,
    /// WIT interface definition
    pub wit_definition: String,
    /// Maximum build time before timeout
    pub timeout: Duration,
}

/// Result of compilation attempt
pub enum CompilationResult {
    Success {
        /// Path to compiled .wasm file
        wasm_path: PathBuf,
        /// Build duration in milliseconds
        build_time_ms: u64,
        /// Compiler output (stdout)
        output: String,
    },
    Failure {
        /// Error message from compiler
        error_message: String,
        /// Line number if available
        line_number: Option<usize>,
        /// Full compiler output (stderr)
        stderr: String,
    },
    Timeout {
        /// How long it ran before timeout
        elapsed: Duration,
    },
}

/// Main compilation service
pub struct ComponentCompiler {
    /// Base directory for build workspaces
    workspace_root: PathBuf,
}

impl ComponentCompiler {
    /// Create new compiler with specified workspace root
    pub fn new(workspace_root: PathBuf) -> Self;

    /// Compile a component from config
    ///
    /// # Returns
    /// - Ok(CompilationResult): Compilation completed (success or failure)
    /// - Err(String): System error (workspace creation failed, etc.)
    pub fn compile(&self, config: CompilationConfig) -> Result<CompilationResult, String>;

    /// Cancel ongoing compilation (future enhancement)
    pub fn cancel(&self, component_name: &str) -> Result<(), String>;

    /// Clean up workspace for a component
    pub fn cleanup_workspace(&self, component_name: &str) -> Result<(), String>;
}
```

### Usage Example

```rust
let compiler = ComponentCompiler::new(PathBuf::from("/tmp/wasmflow-builds"));

let config = CompilationConfig {
    component_name: "TripleNumber".to_string(),
    source_code: generated_code,
    wit_definition: generated_wit,
    timeout: Duration::from_secs(120),
};

match compiler.compile(config)? {
    CompilationResult::Success { wasm_path, build_time_ms, .. } => {
        println!("Built in {}ms: {:?}", build_time_ms, wasm_path);
    }
    CompilationResult::Failure { error_message, line_number, .. } => {
        eprintln!("Build failed at line {:?}: {}", line_number, error_message);
    }
    CompilationResult::Timeout { elapsed } => {
        eprintln!("Build timed out after {:?}", elapsed);
    }
}
```

### Contract Guarantees

- **Thread Safety**: `ComponentCompiler` is `Send + Sync` (future: parallel builds)
- **Cleanup**: Workspace always deleted, even on panic (use `Drop` impl)
- **Timeout**: Strictly enforced via process kill
- **Error Parsing**: Line numbers extracted from cargo JSON output when available

---

## Module: `src/runtime/template_generator.rs`

### Purpose
Generates complete Rust source code from user code and annotations.

### Public Interface

```rust
use crate::graph::node::DataType;

/// Parsed port specification from annotations
#[derive(Debug, Clone)]
pub struct PortSpec {
    pub name: String,
    pub data_type: DataType,
    pub optional: bool,
    pub description: String,
}

/// Parsed component metadata
#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    pub name: String,
    pub description: String,
    pub category: String,
    pub inputs: Vec<PortSpec>,
    pub outputs: Vec<PortSpec>,
    pub capabilities: Vec<String>,
}

/// Template selection
#[derive(Debug, Clone, Copy)]
pub enum TemplateType {
    Simple,   // Pure computation
    Http,     // Network-enabled
}

/// Main template generator
pub struct TemplateGenerator;

impl TemplateGenerator {
    /// Parse structured comments from user code
    ///
    /// # Format
    /// - `// @input name:Type description`
    /// - `// @output name:Type description`
    /// - `// @description text`
    /// - `// @category name`
    /// - `// @capability pattern`
    ///
    /// # Returns
    /// - Ok(ComponentMetadata): Successfully parsed
    /// - Err(String): Parse error with line number
    pub fn parse_annotations(
        component_name: &str,
        source_code: &str,
    ) -> Result<ComponentMetadata, String>;

    /// Select appropriate template based on metadata
    pub fn select_template(metadata: &ComponentMetadata) -> TemplateType;

    /// Generate complete Rust source code
    ///
    /// # Arguments
    /// - `metadata`: Component metadata from parse_annotations
    /// - `user_code`: User's execute function body
    /// - `template_type`: Which template to use
    ///
    /// # Returns
    /// Complete Rust source code ready for compilation
    pub fn generate_component_code(
        metadata: &ComponentMetadata,
        user_code: &str,
        template_type: TemplateType,
    ) -> String;

    /// Generate WIT interface definition
    pub fn generate_wit(metadata: &ComponentMetadata) -> String;

    /// Generate Cargo.toml
    pub fn generate_cargo_toml(component_name: &str) -> String;
}
```

### Usage Example

```rust
let user_code = r#"
// @input value:F32 The input number
// @output result:F32 The tripled value
// @description Multiplies input by 3

let result = value * 3.0;
"#;

let metadata = TemplateGenerator::parse_annotations("TripleNumber", user_code)?;
let template = TemplateGenerator::select_template(&metadata);
let source = TemplateGenerator::generate_component_code(&metadata, user_code, template);
let wit = TemplateGenerator::generate_wit(&metadata);
let cargo_toml = TemplateGenerator::generate_cargo_toml("TripleNumber");

// Now ready for compilation
```

### Contract Guarantees

- **Default Values**: Missing annotations use sensible defaults (see data-model.md)
- **Type Validation**: Invalid types rejected at parse time
- **Name Validation**: Port names must be valid Rust identifiers
- **Idempotent**: Same input always produces same output

---

## Module: `src/builtin/wasm_creator.rs`

### Purpose
Implements the creator node UI and orchestration logic.

### Public Interface

```rust
use crate::graph::node::{GraphNode, NodeType};
use crate::runtime::compiler::{ComponentCompiler, CompilationResult};

/// Creator node state
pub struct WasmCreatorNode {
    // Fields from data-model.md
    id: NodeId,
    component_name: String,
    source_code: String,
    save_code: bool,
    compilation_state: CompilationState,
    last_error: Option<String>,
}

impl WasmCreatorNode {
    /// Create new creator node with defaults
    pub fn new(id: NodeId) -> Self;

    /// Render the node UI (code editor, name input, button)
    pub fn render_ui(&mut self, ui: &mut egui::Ui, compiler: &ComponentCompiler);

    /// Handle execute button click
    pub fn on_execute_clicked(&mut self, compiler: &ComponentCompiler);

    /// Handle code editor changes
    pub fn on_code_changed(&mut self, new_code: String);

    /// Handle component name changes
    pub fn on_name_changed(&mut self, new_name: String);

    /// Validate component name format
    pub fn validate_name(&self) -> Result<(), String>;

    /// Validate code size
    pub fn validate_code(&self) -> Result<(), String>;
}

/// Implement GraphNode trait
impl GraphNode for WasmCreatorNode {
    fn node_type(&self) -> NodeType {
        NodeType::WasmCreator
    }

    fn execute(&mut self) -> Result<(), String> {
        // Creator nodes don't "execute" in graph sense
        // Execution is compilation, triggered by UI button
        Ok(())
    }

    // ... other trait methods
}
```

### Contract Guarantees

- **UI State**: All edits local until Execute clicked
- **Validation**: Name and code validated before compilation starts
- **Non-Blocking**: Compilation runs in background (async/threaded)
- **Error Display**: Errors shown in node footer UI

---

## Module: `src/ui/code_editor.rs`

### Purpose
Wrapper around egui_code_editor with Rust syntax highlighting.

### Public Interface

```rust
use egui_code_editor::{CodeEditor as BaseEditor, ColorTheme, Syntax};

/// Configured code editor for Rust
pub struct RustCodeEditor {
    /// Underlying editor widget
    editor: BaseEditor,
    /// Whether content has changed
    dirty: bool,
}

impl RustCodeEditor {
    /// Create new Rust code editor
    pub fn new() -> Self;

    /// Render the editor and return true if content changed
    pub fn show(&mut self, ui: &mut egui::Ui, code: &mut String) -> bool;

    /// Set theme (light/dark)
    pub fn set_theme(&mut self, theme: ColorTheme);

    /// Get line count
    pub fn line_count(&self, code: &str) -> usize;

    /// Scroll to line (for error highlighting)
    pub fn scroll_to_line(&mut self, line: usize);
}
```

### Usage Example

```rust
let mut editor = RustCodeEditor::new();
let mut code = String::from("fn main() {}");

ui.vertical(|ui| {
    if editor.show(ui, &mut code) {
        println!("Code changed!");
    }
});
```

### Contract Guarantees

- **Performance**: 60 FPS up to 1000 lines
- **Syntax Highlighting**: Rust keywords, strings, comments
- **Line Numbers**: Always visible
- **Scrolling**: Keyboard and mouse navigation

---

## Module: `src/runtime/wasm_host.rs` (Modifications)

### Purpose
Extend existing WASM host to support dynamic component registration.

### New Interface

```rust
impl WasmHost {
    /// Dynamically load and register a component
    ///
    /// # Arguments
    /// - `name`: Component name (must be unique)
    /// - `wasm_path`: Path to compiled .wasm file
    /// - `metadata`: Component metadata
    ///
    /// # Returns
    /// - Ok(()): Component loaded and registered
    /// - Err(String): Load or registration failure
    pub fn register_dynamic_component(
        &mut self,
        name: String,
        wasm_path: PathBuf,
        metadata: ComponentMetadata,
    ) -> Result<(), String>;

    /// Unregister a component (for replacement)
    pub fn unregister_component(&mut self, name: &str) -> Result<(), String>;

    /// Check if component is registered
    pub fn has_component(&self, name: &str) -> bool;
}
```

### Contract Guarantees

- **Replacement**: Unregister before re-registering same name
- **Validation**: WASM component validated before registration
- **Capability Enforcement**: Metadata capabilities respected at runtime

---

## Error Handling Conventions

### Error Types

All modules use `Result<T, String>` for simplicity in this feature. Future enhancement: custom error types.

### Error Message Format

```rust
// Component name: Context: Details
"TripleNumber: Compilation failed: missing semicolon at line 42"

// System errors
"Failed to create workspace: Permission denied: /tmp/wasmflow-builds"
```

### Line Number Extraction

From cargo JSON output:
```json
{
  "reason": "compiler-message",
  "message": {
    "spans": [{"line_start": 42, ...}],
    "message": "expected `;`"
  }
}
```

Extract: `line_number = 42`, `message = "expected ';'"`

---

## Threading Model

### Current (MVP)
- **UI Thread**: All UI rendering, event handling
- **Background Thread**: Compilation (via `std::process::Command`)
- **Synchronization**: Poll `child.try_wait()` every 500ms from UI thread

### Future Enhancement
- **Thread Pool**: Multiple concurrent compilations
- **Async Runtime**: tokio for process management
- **Channels**: mpsc for progress updates

---

## Testing Strategy

### Unit Tests
- `template_generator_test.rs`: Parse annotations, generate code
- `comment_parser_test.rs`: Validate regex and defaults
- Name validation, code size validation

### Integration Tests
- `compilation_workflow_test.rs`: End-to-end compile + load
- Requires cargo-component installed
- Creates real temp workspaces

### Contract Tests
- `generated_component_test.rs`: Validate WIT compliance
- Load generated .wasm and verify metadata

---

## Versioning

Module interfaces follow semver:
- **Breaking**: Change function signatures, remove pub items
- **Non-Breaking**: Add new functions, add optional parameters
- **Compatible**: Bug fixes, documentation

Current version: **0.1.0** (initial implementation)
