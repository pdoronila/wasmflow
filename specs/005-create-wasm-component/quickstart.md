# Quickstart: WASM Component Creator Node

**Feature**: 005-create-wasm-component
**Audience**: Developers implementing this feature
**Time to Read**: 10 minutes

## Overview

This quickstart guide provides a practical introduction to implementing the WASM Component Creator feature. Follow these steps to understand the architecture and start development.

## Prerequisites

- Rust 1.75+ with wasm32-wasip2 target installed
- cargo-component 0.21+ installed (`cargo install cargo-component`)
- Familiarity with egui and wasmtime
- Understanding of WASI Component Model basics

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    WasmCreatorNode (UI)                      │
│  ┌──────────────────────┐  ┌──────────────────────────┐    │
│  │  Code Editor Widget  │  │  Component Name Input    │    │
│  │  (egui_code_editor)  │  │  + Save Code Checkbox    │    │
│  └──────────────────────┘  └──────────────────────────┘    │
│                            ┌──────────────────────────┐    │
│                            │    Execute Button        │    │
│                            └──────────────────────────┘    │
└───────────────────────────────┬─────────────────────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │  TemplateGenerator    │
                    │                       │
                    │  1. Parse @annotations│
                    │  2. Select template   │
                    │  3. Generate code     │
                    └───────────┬───────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │  ComponentCompiler    │
                    │                       │
                    │  1. Create workspace  │
                    │  2. Invoke cargo      │
                    │  3. Monitor progress  │
                    └───────────┬───────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │     WasmHost          │
                    │                       │
                    │  1. Load .wasm        │
                    │  2. Register component│
                    │  3. Add to palette    │
                    └───────────────────────┘
```

## 5-Minute Implementation Roadmap

### Phase 1: Core Infrastructure (Week 1)

**Goal**: Get basic compilation working

1. **Template Generator** (`src/runtime/template_generator.rs`)
   - Implement comment parser with regex
   - Create simple template with placeholders
   - Test: Parse inputs/outputs, generate code

2. **Component Compiler** (`src/runtime/compiler.rs`)
   - Create workspace in `/tmp`
   - Invoke `cargo component build`
   - Test: Compile double-number example

**Milestone**: Can generate and compile a simple component

### Phase 2: UI Integration (Week 2)

**Goal**: Create interactive creator node

3. **Code Editor Widget** (`src/ui/code_editor.rs`)
   - Wrap egui_code_editor with Rust syntax
   - Add line number display
   - Test: 500 lines at 60 FPS

4. **Creator Node UI** (`src/builtin/wasm_creator.rs`)
   - Add code editor + text input
   - Add execute button
   - Wire up to compiler
   - Test: Edit code, click execute, see result

**Milestone**: Can create components through UI

### Phase 3: Dynamic Loading (Week 3)

**Goal**: Load compiled components into palette

5. **Runtime Integration** (`src/runtime/wasm_host.rs`)
   - Add `register_dynamic_component()`
   - Modify palette to show user components
   - Add purple color theme
   - Test: Component appears in palette

6. **Persistence** (`src/graph/serialization.rs`)
   - Add optional code field to node
   - Implement save_code checkbox
   - Test: Save/load graph with creator nodes

**Milestone**: Full end-to-end workflow functional

### Phase 4: Polish & Error Handling (Week 4)

7. **Error Messages**
   - Parse cargo JSON output for line numbers
   - Display in node footer
   - Test: Syntax error shows correct line

8. **Validation & Limits**
   - Name format validation (PascalCase)
   - Code size limits (10K lines / 500KB)
   - Compilation timeout (120s)
   - Test: All edge cases

**Milestone**: Production-ready feature

## Key Implementation Details

### 1. Comment Parsing Pattern

```rust
// Regex for annotations
static ANNOTATION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^//\s*@(\w+)\s+(.+)$").unwrap()
});

// Parse single annotation
fn parse_annotation(line: &str) -> Option<Annotation> {
    let caps = ANNOTATION_REGEX.captures(line)?;
    let tag = caps.get(1)?.as_str();
    let content = caps.get(2)?.as_str();

    match tag {
        "input" => parse_port(content, PortKind::Input),
        "output" => parse_port(content, PortKind::Output),
        "description" => Some(Annotation::Description(content.to_string())),
        _ => None,
    }
}

// Port format: "name:Type description"
fn parse_port(content: &str, kind: PortKind) -> Option<Annotation> {
    let parts: Vec<&str> = content.splitn(2, ' ').collect();
    let name_type = parts[0];
    let description = parts.get(1).unwrap_or(&"").to_string();

    let (name, type_str) = name_type.split_once(':')?;

    let data_type = match type_str {
        "F32" => DataType::F32Type,
        "I32" => DataType::I32Type,
        "U32" => DataType::U32Type,
        "String" => DataType::StringType,
        "Boolean" => DataType::BooleanType,
        _ => return None,
    };

    Some(Annotation::Port(PortSpec {
        name: name.to_string(),
        data_type,
        description,
    }))
}
```

### 2. Template Substitution

```rust
// Simple template with {{placeholders}}
const SIMPLE_TEMPLATE: &str = r#"
wit_bindgen::generate!({ path: "wit", world: "component" });

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;
use wasmflow::node::host;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "{{COMPONENT_NAME}}".to_string(),
            version: "0.1.0".to_string(),
            description: "{{DESCRIPTION}}".to_string(),
            author: "User".to_string(),
            category: Some("User-Defined".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![{{INPUTS}}]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![{{OUTPUTS}}]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        {{CAPABILITIES}}
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        host::log("info", "{{COMPONENT_NAME}} executing");

        {{INPUT_EXTRACTION}}

        // USER CODE
        {{USER_CODE}}

        {{OUTPUT_CONSTRUCTION}}
    }
}

export!(Component);
"#;

// Generate code
fn generate(metadata: &ComponentMetadata, user_code: &str) -> String {
    SIMPLE_TEMPLATE
        .replace("{{COMPONENT_NAME}}", &metadata.name)
        .replace("{{DESCRIPTION}}", &metadata.description)
        .replace("{{INPUTS}}", &format_ports(&metadata.inputs))
        .replace("{{OUTPUTS}}", &format_ports(&metadata.outputs))
        .replace("{{CAPABILITIES}}", &format_capabilities(&metadata.capabilities))
        .replace("{{INPUT_EXTRACTION}}", &generate_input_extraction(&metadata.inputs))
        .replace("{{USER_CODE}}", user_code)
        .replace("{{OUTPUT_CONSTRUCTION}}", &generate_output_construction(&metadata.outputs))
}
```

### 3. Cargo Invocation

```rust
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn compile(workspace: &Path, timeout: Duration) -> CompilationResult {
    let mut child = Command::new("cargo")
        .args(&["component", "build", "--release", "--message-format=json"])
        .current_dir(workspace)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn cargo: {}", e))?;

    let start = Instant::now();

    // Poll for completion
    loop {
        if start.elapsed() > timeout {
            child.kill().ok();
            return CompilationResult::Timeout { elapsed: start.elapsed() };
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output()?;
                return if status.success() {
                    CompilationResult::Success {
                        wasm_path: workspace.join("target/wasm32-wasip2/release/component.wasm"),
                        build_time_ms: start.elapsed().as_millis() as u64,
                        output: String::from_utf8_lossy(&output.stdout).to_string(),
                    }
                } else {
                    CompilationResult::Failure {
                        error_message: parse_error(&output.stderr),
                        line_number: extract_line_number(&output.stderr),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    }
                };
            }
            Ok(None) => {
                // Still running, wait a bit
                std::thread::sleep(Duration::from_millis(500));
            }
            Err(e) => return Err(format!("Wait failed: {}", e)),
        }
    }
}
```

### 4. Dynamic Component Registration

```rust
impl ComponentRegistry {
    pub fn register_dynamic(&mut self,
        name: String,
        wasm_path: PathBuf,
        metadata: ComponentMetadata,
    ) -> Result<(), String> {
        // Check for duplicate
        if self.components.contains_key(&name) {
            eprintln!("Replacing existing component: {}", name);
            self.components.remove(&name);
        }

        // Load WASM
        let wasm_bytes = std::fs::read(&wasm_path)
            .map_err(|e| format!("Failed to read WASM: {}", e))?;

        // Create component spec
        let spec = ComponentSpec {
            id: name.clone(),
            name: metadata.name,
            category: Some(metadata.category),
            description: metadata.description,
            inputs: metadata.inputs,
            outputs: metadata.outputs,
            is_user_defined: true,  // NEW FIELD
            wasm_bytes,
        };

        self.components.insert(name, spec);
        Ok(())
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_input_annotation() {
        let line = "// @input value:F32 The input number";
        let ann = parse_annotation(line).unwrap();

        assert!(matches!(ann, Annotation::Port(p) if p.name == "value"));
    }

    #[test]
    fn generate_simple_component() {
        let metadata = ComponentMetadata {
            name: "Test".to_string(),
            inputs: vec![/* ... */],
            outputs: vec![/* ... */],
            // ...
        };

        let code = generate(&metadata, "let x = 42;");
        assert!(code.contains("struct Component"));
        assert!(code.contains("let x = 42;"));
    }
}
```

### Integration Tests

```rust
#[test]
fn end_to_end_compilation() {
    let compiler = ComponentCompiler::new(PathBuf::from("/tmp/test"));

    let config = CompilationConfig {
        component_name: "TestComponent".to_string(),
        source_code: "// @input x:F32\nlet result = x * 2.0;".to_string(),
        wit_definition: /* ... */,
        timeout: Duration::from_secs(60),
    };

    let result = compiler.compile(config).unwrap();
    assert!(matches!(result, CompilationResult::Success { .. }));
}
```

## Common Pitfalls

### 1. **Forgetting to clean up workspaces**
   - **Problem**: `/tmp` fills up with build artifacts
   - **Solution**: Always clean up in `Drop` impl

### 2. **Not handling timeouts**
   - **Problem**: UI freezes on infinite compilation
   - **Solution**: Kill process after 120s

### 3. **Hardcoding paths**
   - **Problem**: Tests fail on different machines
   - **Solution**: Use `env::temp_dir()` and relative paths

### 4. **Missing error context**
   - **Problem**: "Compilation failed" with no details
   - **Solution**: Parse cargo JSON for line numbers and messages

## Next Steps

1. **Read the full spec**: [spec.md](./spec.md)
2. **Review data model**: [data-model.md](./data-model.md)
3. **Check contracts**: [contracts/](./contracts/)
4. **Explore research**: [research.md](./research.md)
5. **Run `/speckit.tasks`**: Generate implementation task list

## Resources

- **egui_code_editor**: https://docs.rs/egui_code_editor/
- **cargo-component**: https://github.com/bytecodealliance/cargo-component
- **WASI Component Model**: https://component-model.bytecodealliance.org/
- **Example components**: `/examples/double-number/`, `/examples/example-http-fetch/`

## Questions?

For clarifications or design questions, refer to:
- **Constitution**: `.specify/memory/constitution.md` (principles and gates)
- **Existing code**: `src/runtime/wasm_host.rs`, `src/ui/palette.rs`
- **Spec clarifications**: Create issue in specs directory
