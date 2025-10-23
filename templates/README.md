# Component Templates

This directory contains templates for generating WASM components dynamically from user code.

## Requirements

### cargo-component 0.21+

The WASM Component Creator feature requires `cargo-component` to compile user-defined components.

**Installation**:
```bash
cargo install cargo-component --version "^0.21"
```

**Verification**:
```bash
cargo component --version
# Should output: cargo-component-component 0.21.x
```

## Templates

- `component_template.rs.tmpl` - Simple component template for pure computation
- `http_component_template.rs.tmpl` - HTTP-enabled component template with WASI HTTP support

## Template Placeholders

Templates use `{{PLACEHOLDER}}` syntax for variable substitution:

- `{{COMPONENT_NAME}}` - Component struct name (PascalCase)
- `{{DESCRIPTION}}` - Component description text
- `{{INPUTS}}` - Generated PortSpec array for inputs
- `{{OUTPUTS}}` - Generated PortSpec array for outputs
- `{{CAPABILITIES}}` - Generated capabilities vector or None
- `{{INPUT_EXTRACTION}}` - Generated code to extract inputs from Vec<(String, Value)>
- `{{USER_CODE}}` - User's execute function body
- `{{OUTPUT_CONSTRUCTION}}` - Generated code to build output Vec<(String, Value)>

## Usage

Templates are processed by `src/runtime/template_generator.rs` to generate complete WASM component source code from user input.
