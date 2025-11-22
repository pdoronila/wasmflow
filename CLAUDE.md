<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# wasmflow_cc Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-21

## Active Technologies
- Rust 1.75+ (stable channel with wasm32-wasip2 target) + egui 0.29 (UI), eframe 0.29 (app framework), egui-snarl 0.3 (node editor), wasmtime 27.0 with component-model (WASM runtime), petgraph 0.6 (graph algorithms), serde/bincode (serialization with BTreeMap for deterministic order), crc (CRC64 checksums) (001-webassembly-based-node)
- Rust 1.75+ (stable channel with wasm32-wasip2 target) + wasmtime 27.0 (component-model, async), wasmtime-wasi-http 27.0 (WASI HTTP Preview support), tokio (async runtime) (002-lets-focus-on)
- N/A (no persistent storage for this feature) (002-lets-focus-on)
- Rust 1.75+ (stable channel with wasm32-wasip2 target) + egui 0.29 (UI framework), eframe 0.29 (app framework), egui-snarl 0.3 (node editor), wasmtime 27.0 (WASM runtime) (003-ui-customize-currently)
- N/A (UI architecture refactoring only) (003-ui-customize-currently)
- Graph serialization via serde + bincode (BTreeMap for deterministic order) (004-node-input-update)
- File system (temporary build artifacts in temp directory, optional code persistence in graph JSON via BTreeMap) (005-create-wasm-component)
- Rust 1.75+ (stable channel with wasm32-wasip2 target) + egui 0.29 (UI), eframe 0.29 (app framework), egui-snarl 0.3 (node editor), wasmtime 27.0 with component-model (WASM runtime), tokio (async runtime for continuous execution) (006-continuous-node-can)
- Graph serialization via serde + bincode (BTreeMap for deterministic order), persistence of execution state in node metadata (006-continuous-node-can)
- Rust 1.75+ (stable channel with wasm32-wasip2 target) + egui 0.33 (UI framework), eframe 0.33 (app framework), egui-snarl (node editor), wasmtime 27.0 (WASM runtime with component-model), petgraph 0.6 (graph algorithms), serde/bincode (serialization), WAC CLI (WebAssembly Composition) (007-rectangle-selection-tool)
- Graph serialization via serde + bincode (BTreeMap for deterministic order), composite node internal structure persisted in graph JSON (007-rectangle-selection-tool)
- Rust 1.75+ (stable channel with wasm32-wasip2 target) + serde_json (JSON parsing), wasmtime 27.0 (component-model runtime), wit-bindgen (WIT interface generation) (008-json-parser-a)
- N/A (stateless component - processes inputs to outputs) (008-json-parser-a)
- File system (components directory structure) (009-reorginize-components-currently)
- Rust 1.75+ (stable channel with wasm32-wasip2 target) + wit-bindgen 0.30, serde (for list/data serialization), standard library (no external crates for core operations) (010-wasm-components-core)
- N/A (stateless components - all data flows through inputs/outputs) (010-wasm-components-core)

## Project Structure
```
src/
tests/
```

## Commands
cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style
Rust 1.75+ (stable channel with wasm32-wasip2 target): Follow standard conventions

## Data Structure Guidelines
- **Use BTreeMap for all serialized data structures** (e.g., NodeGraph.nodes, NodeValue::Record) to ensure deterministic serialization and enable CRC64 checksum validation
- **Use HashMap for runtime-only structures** (marked with #[serde(skip)]) where non-deterministic ordering is acceptable
- Performance difference is negligible for <1000 nodes

## Continuous Execution Guidelines (006-continuous-node-can)
- **Runtime State**: Use `ContinuousNodeConfig` with `runtime_state` marked `#[serde(skip)]` to prevent persistence
- **State Transitions**: Follow the state machine: Idle → Starting → Running → Stopping → Stopped/Error
- **Shutdown**: Implement 3-phase shutdown: 1.5s graceful wait + 0.5s forced abort + cleanup
- **Input Resolution**: Continuous nodes must resolve inputs by following graph connections, not just reading port values
- **Logging**: Add comprehensive logging for lifecycle events (start, stop, iterations, errors)
- **Visual Feedback**: Use state colors (green pulsing for running, red for error, gray for idle) and iteration counters
- **Example Nodes**: See `src/builtin/continuous_example.rs` for timer and combiner examples

## Clone Selection Guidelines (011-clone-selection)

**Location**: `src/ui/app/duplication.rs`

Users can duplicate selected nodes using the Clone button or Ctrl+D keyboard shortcut.

### Implementation Details

- **Offset**: Clones are placed at `(+50px, +50px)` from original position
- **Naming**: Appends " (Clone)" to display name
- **Connections**: Only internal connections (both nodes selected) are cloned
- **External connections**: NOT cloned (ambiguous behavior)
- **Selection**: Cloned nodes are selected, originals deselected
- **Undo/Redo**: Supported via `Command::CloneNodes`

### Constants

```rust
const CLONE_OFFSET: egui::Vec2 = egui::Vec2::new(50.0, 50.0);
const CLONE_NAME_SUFFIX: &str = " (Clone)";
```

### Keyboard Shortcut

- **Ctrl+D**: Clone selected nodes (available in both Normal and Selection modes)

### Special Node Handling

- **WASM Creator nodes**: `creator_data` is cloned (code preserved)
- **Composite nodes**: `composition_data` is cloned (binary blob preserved)
- **Continuous nodes**: `continuous_config` is cloned, but `runtime_state` is reset (not running)
- **Constant nodes**: Port values are cloned

### Command Pattern

The clone operation uses the `Command::CloneNodes` variant for undo/redo support:

```rust
Command::CloneNodes {
    cloned_nodes: Vec<GraphNode>,
    cloned_connections: Vec<Connection>,
}
```

**Execute**: Adds cloned nodes and connections to graph
**Undo**: Removes cloned nodes and connections from graph

## Composite Node Naming Guidelines

**Location**: `src/ui/dialogs.rs`, `src/ui/app/composition.rs`, `src/graph/command.rs`

Users can name composite nodes at creation time and rename them afterward via context menu.

### Name Entry at Creation

When composing multiple nodes into a composite:
1. After validation passes, a modal dialog appears
2. Default name is "Composite Node"
3. User can accept or modify the name
4. Empty names are rejected with inline validation
5. Any non-empty string is accepted (Unicode, special characters allowed)
6. Cancel aborts the composition operation

### Rename via Context Menu

To rename an existing composite node:
1. Right-click on the composite node
2. Select "✏ Rename" from the context menu (appears only for composite nodes)
3. Dialog opens pre-filled with current name
4. Modify the name and click "Rename"
5. Same validation as creation (non-empty strings only)

### Undo/Redo Support

Rename operations are fully reversible:
- **Command**: `Command::RenameNode { node_id, old_name, new_name }`
- **Execute**: Updates `display_name` and `composition_data.name`
- **Undo**: Restores both fields to `old_name`
- **Redo**: Re-applies `new_name`
- Accessible via Ctrl+Z (undo) and Ctrl+Y (redo)

### Name Propagation

Composite node names appear consistently across the UI:
- **Canvas**: Node title shows `display_name`
- **Drill-down breadcrumb**: Shows custom name (not "Composite Node")
- **Status messages**: "Viewing internal structure of 'Custom Name'"
- **Port mappings**: External ports use composite name (e.g., "Data Processor.input")
- **Serialization**: Both `display_name` and `composition_data.name` persist in graph JSON

### Data Model

Two fields must be kept in sync during rename:
- `GraphNode.display_name`: String - shown in UI
- `CompositionData.name`: String - stored in composition metadata

### Implementation Details

**Dialog**: `CompositeNameDialog` in `src/ui/dialogs.rs`
- Mode enum: `Create` or `Rename`
- Validation: trims whitespace, rejects empty strings
- Keyboard support: Enter to confirm, Escape to cancel

**Composition workflow**: `src/ui/app/composition.rs`
- `handle_compose_action()`: Opens naming dialog after validation
- `handle_composite_name_confirmed()`: Continues composition with provided name

**Context menu**: `src/ui/canvas/viewer.rs`
- "Rename" button appears only for composite nodes (`node.is_composite`)
- Sets `pending_rename` field for app to handle

**Command system**: `src/graph/command.rs`
- `RenameNode` variant with `node_id`, `old_name`, `new_name`
- Updates both `display_name` and `composition_data.name` in execute/undo

## Recent Changes
- 010-wasm-components-core: Added Rust 1.75+ (stable channel with wasm32-wasip2 target) + wit-bindgen 0.30, serde (for list/data serialization), standard library (no external crates for core operations)
- 009-reorginize-components-currently: Added Rust 1.75+ (stable channel with wasm32-wasip2 target)
- 009-reorginize-components-currently: Added Rust 1.75+ (stable channel with wasm32-wasip2 target)

<!-- MANUAL ADDITIONS START -->

## Node Layout and Size Constraints

### Critical: Preventing Infinite Node Growth
**Location**: `src/ui/canvas.rs` in `show_footer()` method

Nodes in egui-snarl will grow infinitely if not properly constrained. The footer rendering MUST have both width and height constraints:

```rust
// In show_footer() for non-resizable nodes:
ui.scope(|ui| {
    ui.set_max_width(300.0);  // Prevent horizontal growth
    ui.set_max_height(200.0); // Prevent vertical growth
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .auto_shrink([false, true])  // Don't shrink horizontally, allow vertical shrinking
        .show(ui, |ui| {
            // Footer content here
        });
});

// For resizable WASM Creator nodes:
let min_width = 600.0;
let max_width = 1800.0;
let current_width = custom_width.unwrap_or(975.0);
```

**Why All Constraints Are Needed**:
- `ui.set_max_width()` / `ui.set_max_height()` - Tell the layout system the maximum size the UI will report
- `ScrollArea::max_height()` - Provides actual scrolling when content exceeds limits
- `ScrollArea::auto_shrink([false, true])` - Prevents horizontal shrinking (fixes narrow column bug in WASM component footers)
- `wrap_mode` - Enables text wrapping within width constraints

**Critical: auto_shrink Fix**:
Without `.auto_shrink([false, true])`, the ScrollArea will auto-shrink horizontally, causing `ui.available_width()` inside to return tiny values. This manifests as text appearing in extremely narrow columns (one character per line) in WASM component footers that use the WIT renderer.

**Critical: WIT Renderer Vertical Layout Fix** (`src/ui/wit_ui_renderer.rs`):
Vertical layouts inside the WIT renderer MUST set minimum width to prevent shrinking:
```rust
pub fn render_footer_view(ui: &mut egui::Ui, view: &FooterView) -> Result<(), String> {
    ui.vertical(|ui| {
        // Force the vertical layout to use full available width
        // Without this, the layout can shrink and cause narrow column rendering
        ui.set_min_width(ui.available_width());

        for element in &view.elements {
            render_element(ui, element)?;
        }
        Ok::<(), String>(())
    })
    .inner
}
```
Without `ui.set_min_width(ui.available_width())`, even with the ScrollArea fix, text will still render in narrow columns because `ui.vertical()` creates a new layout context that can shrink to fit content.

**Critical: Horizontal vs Vertical Element Sizing** (`src/ui/wit_ui_renderer.rs`):
- **UiElement items** (used in `render_element()` for top-level or vertical layouts): Use `ui.add_sized(egui::vec2(ui.available_width(), 0.0), ...)` for full-width labels
- **UiElementItem items** (used in `render_element_item()` for horizontal layouts): Use `ui.label()` or `ui.colored_label()` with natural sizing

```rust
// Top-level labels (UiElement) - use full width
UiElement::Label(text) => {
    ui.add_sized(
        egui::vec2(ui.available_width(), 0.0),
        egui::Label::new(text).wrap()
    );
}

// Labels inside horizontal layouts (UiElementItem) - use natural sizing
UiElementItem::Label(text) => {
    ui.label(text);  // NOT ui.add_sized() - would break horizontal layout
}
```

If you use `ui.available_width()` for labels inside horizontal layouts, each label will take the full width and push other elements to new lines, breaking the horizontal layout.

**Common Mistakes That Don't Work**:
- ❌ Using only `ScrollArea` without `ui.set_max_*()` - Node still grows
- ❌ Using `TextEdit::desired_rows()` alone - Doesn't prevent layout growth
- ❌ Using `ui.allocate_exact_size()` - Still allows layout to grow around it
- ❌ Constraining inside component views (e.g., ConstantNodeFooterView) - Too late, layout already calculated

**Why This Is At Canvas Level**:
The constraint must be applied in `show_footer()` where snarl calculates node dimensions, NOT inside individual component footer views. By the time a component's `render_footer()` is called, the layout system has already committed to a size.

### Footer Content Layout Guidelines
**All footer content should use vertical layouts with full width**:

```rust
// ✅ CORRECT - Vertical layout with full width labels
ui.vertical(|ui| {
    ui.label("Field name:");
    ui.add_sized(
        egui::vec2(ui.available_width(), 0.0),
        egui::Label::new(value).wrap()
    );
});

// ❌ INCORRECT - Grid layout splits width
egui::Grid::new("id")
    .num_columns(2)  // Splits available width in half!
    .show(ui, |ui| {
        ui.label("Field:");
        ui.label(value);
    });
```

**Files Using This Pattern**:
- `src/ui/canvas.rs` - Main constraint enforcement
- `src/ui/wit_ui_renderer.rs` - WASM component footer rendering
- `src/builtin/views.rs` - Builtin node footer views (ConstantNodeFooterView, MathNodeFooterView)

## Component World Selection Guidelines

### Critical: Choosing the Correct Component World

**Location**: `components/.templates/` contains two WIT templates

When creating new WASM components, you MUST choose the correct world type. Using the wrong template will cause component loading failures or missing functionality.

### Available Templates

**1. Standard Component World** (`components/.templates/node.wit`)
```wit
world component {
    import host;
    export metadata;
    export execution;
}
```

**2. Component with UI World** (`components/.templates/node-with-ui.wit`)
```wit
world component-with-ui {
    import host;
    export metadata;
    export execution;
    export ui;  // Additional UI interface for custom footer rendering
}
```

### Decision Criteria

**Use `component-with-ui` world when:**
- ✅ Component needs to display custom formatted output in the footer (colors, layouts, key-value pairs)
- ✅ Component processes data that benefits from visual presentation (HTTP responses, JSON parsing results, formatted data)
- ✅ Component implements the `ui::Guest` trait with `get_footer_view()` method

**Use standard `component` world when:**
- ✅ Component performs pure computation (math, string operations, type conversions)
- ✅ Component's outputs are simple values that don't need custom rendering
- ✅ Component doesn't need visual feedback beyond the default port value display

### Component Categories by World Type

**Standard `component` world:**
- Math operations: `adder`, `double-number`, `multiplier`, `divider`, etc.
- String operations: `string-concat`, `string-trim`, `string-length`, `string-case`, etc.
- Type conversions: `convert-f32-to-u32`, `convert-u32-to-f32`, etc.
- Simple I/O: `echo`, `file-reader`
- Collections: `list-filter`, `list-map`, `list-reduce`, etc.
- Data transformations without UI needs

**`component-with-ui` world:**
- `json-parser` - Displays extracted JSON values with formatting
- `http-fetch` - Shows HTTP status, headers, and response body with color coding
- `footer-view` - Example component demonstrating custom UI rendering
- Any component that needs rich output visualization

### Special Cases

**HTTP/Network Components:**
Components that need WASI HTTP imports (like `http-fetch`) require a custom WIT file that includes BOTH:
- The `component-with-ui` world (for UI rendering)
- WASI imports (for network functionality)

See `components/http-fetch/wit/node.wit` for the pattern.

### Common Mistakes to Avoid

**❌ Batch Updating WIT Files Without Checking World Type**

**Problem:** During version updates or migrations, blindly copying the standard template to all components will break UI components.

**Example of what went wrong:**
```bash
# This breaks json-parser, footer-view, and http-fetch:
for component in components/*/; do
    cp components/.templates/node.wit "$component/wit/node.wit"
done
```

**✅ Correct approach:**
```bash
# Check if component has ui::Guest implementation first
if grep -q "impl UiGuest" "$component/src/lib.rs"; then
    cp components/.templates/node-with-ui.wit "$component/wit/node.wit"
else
    cp components/.templates/node.wit "$component/wit/node.wit"
fi
```

**❌ Using standard `component` world for components with `impl UiGuest`**

**Symptom:**
```
error: no world named `component-with-ui` in package
```

**Solution:** Copy `node-with-ui.wit` template instead.

**❌ Using `component-with-ui` world for simple components**

**Problem:** Adds unnecessary complexity and requires implementing unused `ui::Guest` trait.

**Solution:** Use standard `node.wit` template.

### How to Identify What a Component Needs

**Check the component's source code** (`src/lib.rs`):

```rust
// Standard component - uses only these traits:
impl MetadataGuest for Component { ... }
impl ExecutionGuest for Component { ... }

// Component with UI - adds this trait:
impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        // Custom UI rendering logic
    }
}
```

**Check the wit_bindgen configuration:**
```rust
// Standard component:
wit_bindgen::generate!({
    path: "wit",
    world: "component",  // ← Look here
});

// Component with UI:
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",  // ← Look here
});
```

### Verification Checklist

Before building a new component category (math, collections, etc.):

- [ ] Determine if components need custom UI rendering
- [ ] Choose appropriate template (`node.wit` or `node-with-ui.wit`)
- [ ] Copy template to `components/<name>/wit/node.wit`
- [ ] Update component code to match world type
- [ ] Verify `wit_bindgen::generate!` world matches WIT file
- [ ] Build and test component loads in UI

### Files to Reference

- **Standard template:** `components/.templates/component.wit`
- **UI template:** `components/.templates/component-with-ui.wit`
- **Standard example:** `components/math/math-adder/` (simple math operation)
- **UI example:** `components/data/json-parser/` (formatted output)
- **Special case:** `components/html/http-fetch/` (UI + WASI imports)

## Shared WIT Package Architecture

**Updated**: 2025-11-08 (Shared Package Migration)

**Location**: `/wit/wasmflow-node.wit` (shared package), component-specific files in `components/*/wit/node.wit`

### Overview

All WasmFlow components now use a **shared WIT package** (`wasmflow:node@1.1.0`) instead of duplicating interface definitions. This provides:
- **Single source of truth** for all component interfaces
- **86% reduction in WIT code** (~8,200 lines → ~1,171 lines)
- **Easier maintenance** - update interfaces once, all components benefit
- **Consistent versioning** across all components

### Architecture

**Shared Package** (`wit/wasmflow-node.wit`):
- Defines all common interfaces: `types`, `host`, `metadata`, `execution`, `ui`
- 156 lines, shared across 83+ components
- Version-controlled as `wasmflow:node@1.1.0`

**Component WIT Files** (`components/*/wit/node.wit`):
- Minimal (12-15 lines per component, down from ~99 lines)
- Declares component package (e.g., `package wasmflow:math-adder@1.0.0`)
- Defines world with imports/exports from shared package
- Can add custom interfaces for special needs (e.g., WASI imports)

**Dependency Resolution**:
Components access the shared package through:
1. **Local deps directory**: `wit/deps/wasmflow-node/node.wit` (copy of shared package)
2. **Cargo.toml metadata**: `[package.metadata.component.target.dependencies]`
3. **wit-bindgen configuration**: `with:` mappings in `src/lib.rs`

### Component Structure

**Standard Component** (components/.templates/component.wit):
```wit
package wasmflow:COMPONENT_NAME@1.0.0;

world component {
    import wasmflow:node/host@1.1.0;
    export wasmflow:node/metadata@1.1.0;
    export wasmflow:node/execution@1.1.0;
}
```

**UI Component** (components/.templates/component-with-ui.wit):
```wit
package wasmflow:COMPONENT_NAME@1.0.0;

world component-with-ui {
    import wasmflow:node/host@1.1.0;
    export wasmflow:node/metadata@1.1.0;
    export wasmflow:node/execution@1.1.0;
    export wasmflow:node/ui@1.1.0;
}
```

**Cargo.toml Configuration**:
```toml
[package.metadata.component.target.dependencies]
"wasmflow:node" = { path = "../../../wit" }
```

**Rust wit-bindgen Configuration** (src/lib.rs):
```rust
wit_bindgen::generate!({
    path: "./wit",
    world: "component",  // or "component-with-ui"
    with: {
        "wasmflow:node/types@1.1.0": generate,
        "wasmflow:node/host@1.1.0": generate,
        "wasmflow:node/metadata@1.1.0": generate,
        "wasmflow:node/execution@1.1.0": generate,
        // Add "wasmflow:node/ui@1.1.0": generate, for UI components
    },
});
```

### Creating New Components

1. **Copy appropriate template**:
   ```bash
   cp components/.templates/component.wit components/my-component/wit/node.wit
   # Or component-with-ui.wit for UI components
   ```

2. **Replace placeholder**:
   ```bash
   sed -i 's/COMPONENT_NAME/my-component/g' components/my-component/wit/node.wit
   ```

3. **Set up dependencies**:
   ```bash
   mkdir -p components/my-component/wit/deps/wasmflow-node
   cp wit/wasmflow-node.wit components/my-component/wit/deps/wasmflow-node/node.wit
   ```

4. **Add Cargo.toml metadata** (if not already present):
   ```toml
   [package.metadata.component.target.dependencies]
   "wasmflow:node" = { path = "../../../wit" }
   ```

5. **Configure wit-bindgen** in src/lib.rs with `with:` mappings (see above)

### Special Cases

**Components with Additional Imports** (e.g., http-fetch with WASI):
```wit
package wasmflow:http-fetch@1.0.0;

world component-with-ui {
    import wasmflow:node/host@1.1.0;

    // Additional WASI imports
    import wasi:http/types@0.2.0;
    import wasi:http/outgoing-handler@0.2.0;
    import wasi:io/streams@0.2.0;
    import wasi:io/poll@0.2.0;
    import wasi:io/error@0.2.0;

    export wasmflow:node/metadata@1.1.0;
    export wasmflow:node/execution@1.1.0;
    export wasmflow:node/ui@1.1.0;
}
```

For components with WASI or other external dependencies:
1. Keep WIT files for external packages in `wit/deps/` (e.g., `wit/deps/wasi-http/`)
2. Add corresponding `with:` mappings in wit-bindgen configuration
3. Ensure all imported interfaces are listed

### Migration Notes

**Migration completed**: 2025-11-08
- **82 components migrated successfully**
- **1 component already migrated** (math-adder pilot)
- **1 component with special needs** (http-fetch requires manual WASI setup)

**Migration script**: `migrate-to-shared-wit.sh` (for reference or future components)

### Building Components

**Use `just build` (NOT `cargo component build` - deprecated):**

```bash
# Build single component
cd components/math/math-adder
just build              # Builds to target/wasm32-wasip2/release/
just install            # Builds and copies to ../../bin/
just test               # Runs component tests

# Build all components in a category
cd components/math
just build              # Builds all math components in parallel

# Build all components (from project root)
cd components
just build              # Builds all component categories in parallel
just build math         # Build specific category
just build core/string-concat  # Build specific component by path
```

**Build System Details:**
- Uses `cargo build --target wasm32-wasip2 --release` (not cargo-component)
- Parallel builds supported with `threads` parameter: `just build "" 4`
- Automatically detects component names from directory structure
- Category-level Justfiles orchestrate builds across multiple components
- Requires nushell (`nu`) for parallel execution scripts

### Troubleshooting

**Error: "package 'wasmflow:node@1.1.0' not found"**
- Ensure `wit/deps/wasmflow-node/node.wit` exists and contains shared package
- Check `Cargo.toml` has `[package.metadata.component.target.dependencies]`
- Verify path in Cargo.toml points to correct wit/ directory

**Error: "missing `with` mapping for the key 'wasmflow:node/...'\"**
- Add all required wasmflow:node interfaces to `with:` block in wit-bindgen
- Standard components need: types, host, metadata, execution
- UI components also need: ui

**Build succeeds but component doesn't load**
- Verify world name matches wit-bindgen configuration
- Check that all exported interfaces are implemented in Rust code
- Ensure WIT file package name matches component name convention

### Updating Shared Interfaces

To add or modify shared interfaces:

1. **Edit** `wit/wasmflow-node.wit` with new/changed interface
2. **Update version** if making breaking changes (e.g., 1.1.0 → 1.2.0)
3. **Propagate to all components**:
   ```bash
   find components -path "*/wit/deps/wasmflow-node/node.wit" -exec cp wit/wasmflow-node.wit {} \;
   ```
4. **Update component WIT files** to reference new version if changed
5. **Rebuild affected components**

**Breaking vs Non-Breaking Changes**:
- **Breaking**: Changing function signatures, removing interfaces, renaming types
- **Non-breaking**: Adding new optional interfaces, adding new variants, documentation

## Core Component Library Development Patterns

**Added**: 2025-10-23 (Phase 8 - Polish & Integration)

### Overview

The core library implementation (34 components across 5 categories) established proven patterns for WASM component development. This section documents best practices discovered during implementation.

### Standard Component Structure

All core library components follow this battle-tested structure:

```rust
wit_bindgen::generate!({
    path: "wit",
    world: "component",  // or "component-with-ui" for custom rendering
});

use exports::wasmflow::node::metadata::Guest as MetadataGuest;
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
use wasmflow::node::types::*;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Component Name".to_string(),
            version: "1.0.0".to_string(),
            description: "Clear, concise description".to_string(),
            author: "WasmFlow Core Library".to_string(),
            category: Some("Category".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> { /* ... */ }
    fn get_outputs() -> Vec<PortSpec> { /* ... */ }
    fn get_capabilities() -> Option<Vec<String>> { None }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Implementation
    }
}

export!(Component);  // REQUIRED

#[cfg(test)]
mod tests { /* 3-9 tests per component */ }
```

### Input Extraction Patterns

#### Pattern 1: Required Input with Type Validation

```rust
// Extract and validate required input
let input = inputs
    .iter()
    .find(|(name, _)| name == "input_name")
    .ok_or_else(|| ExecutionError {
        message: "Missing required input: input_name".to_string(),
        input_name: Some("input_name".to_string()),
        recovery_hint: Some("Connect a value to this input".to_string()),
    })?;

// Type-safe value extraction
let value = match &input.1 {
    Value::StringVal(s) => s,
    _ => {
        return Err(ExecutionError {
            message: format!("Expected string for 'input_name', got {:?}", input.1),
            input_name: Some("input_name".to_string()),
            recovery_hint: Some("Provide a string value".to_string()),
        });
    }
};
```

**Used in**: All text, data, and logic components

#### Pattern 2: Optional Input with Default Value

```rust
// Handle optional input with graceful fallback
let optional_value = if let Some(input) = inputs.iter().find(|(name, _)| name == "optional") {
    match &input.1 {
        Value::U32Val(n) => *n as usize,
        _ => {
            return Err(ExecutionError {
                message: format!("Expected u32 for 'optional', got {:?}", input.1),
                input_name: Some("optional".to_string()),
                recovery_hint: Some("Provide a positive integer".to_string()),
            });
        }
    }
} else {
    default_value  // Use sensible default
};
```

**Used in**: list-slice (optional `end`), string-substring (optional `length`)

#### Pattern 3: Multi-Input Collection

```rust
// Collect all inputs (for variable-arity operations)
let mut values = Vec::new();
for input in &inputs {
    match &input.1 {
        Value::BoolVal(b) => values.push(*b),
        _ => {
            return Err(ExecutionError {
                message: format!("Expected boolean for '{}', got {:?}", input.0, input.1),
                input_name: Some(input.0.clone()),
                recovery_hint: Some("All inputs must be boolean values".to_string()),
            });
        }
    }
}

// Use collected values
let result = values.iter().all(|&x| x);  // AND operation
```

**Used in**: boolean-and, boolean-or, min, max

### Error Handling Patterns

#### Pattern 4: Parse Errors with Context

```rust
// Provide detailed context for parse failures
let number = text.trim().parse::<f32>().map_err(|e| ExecutionError {
    message: format!("Failed to parse '{}' as a number: {}", text, e),
    input_name: Some("text".to_string()),
    recovery_hint: Some("Provide a valid number string (e.g., '42', '3.14', '1.5e2')".to_string()),
})?;
```

**Used in**: parse-number

**Key Insight**: Always include the invalid value in error message and provide concrete examples in recovery hints.

#### Pattern 5: Bounds Checking with Helpful Messages

```rust
// Validate array/list access with clear error messages
if index >= list_values.len() {
    return Err(ExecutionError {
        message: format!(
            "Index {} out of bounds for list of length {}",
            index,
            list_values.len()
        ),
        input_name: Some("index".to_string()),
        recovery_hint: Some(format!(
            "Provide an index between 0 and {}",
            list_values.len().saturating_sub(1)
        )),
    });
}
```

**Used in**: list-get, list-slice

**Key Insight**: Include both the problematic value AND the valid range in error messages.

### Type System Patterns

#### Pattern 6: Working with StringListVal

```rust
// CORRECT: StringListVal contains Vec<String>, not Vec<Value>
let list_values = match &list.1 {
    Value::StringListVal(items) => items,  // items is &Vec<String>
    _ => return Err(/* error */),
};

// Direct iteration - strings are already unwrapped
for item in list_values.iter() {
    // item is &String, NOT &Value
    println!("{}", item);  // Direct use, no pattern matching needed
}

// INCORRECT: Trying to pattern match
for item in list_values.iter() {
    match item {  // WRONG! item is &String, not &Value
        Value::StringVal(s) => ...,  // This doesn't compile
    }
}
```

**Critical Learning**: StringListVal, U32ListVal, and F32ListVal contain primitive Rust types, not Value enums. This caught us in list-join implementation (commit 12ed6d9).

**Used in**: All list components

#### Pattern 7: Type Conversion Chain

```rust
// Convert between types with validation at each step
let text = match &value.1 {
    Value::U32Val(n) => n.to_string(),
    Value::I32Val(n) => n.to_string(),
    Value::F32Val(n) => n.to_string(),
    Value::StringVal(s) => s.clone(),
    Value::BoolVal(b) => b.to_string(),
    Value::BinaryVal(_) | Value::StringListVal(_) | ... => {
        return Err(ExecutionError {
            message: "Cannot convert ... to string".to_string(),
            recovery_hint: Some(
                "Use a primitive value (number, boolean, or string). \
                 For complex types, use json-stringify or list-join."
            .to_string()),
        });
    }
};
```

**Used in**: to-string

**Key Insight**: Explicitly handle ALL Value variants. Provide alternative solutions for unsupported types in recovery hints.

### String Operation Patterns

#### Pattern 8: Unicode-Aware String Operations

```rust
// CORRECT: Unicode-aware length
let length = text.chars().count() as u32;

// WRONG: Byte length (incorrect for non-ASCII)
let length = text.len() as u32;  // Don't use this!
```

**Used in**: string-length

**Key Insight**: Always use `.chars()` for Unicode correctness when counting or iterating characters.

#### Pattern 9: Immutable String Transformations

```rust
// Create new strings, don't mutate
let trimmed = text.trim().to_string();  // New string
let uppercase = text.to_uppercase();     // New string
let result = format!("{}{}", str1, str2); // New string

// This ensures:
// 1. No side effects
// 2. Predictable behavior
// 3. Thread safety
```

**Used in**: All text components

### Build and Deployment Patterns

#### Pattern 10: Standard Cargo.toml Configuration

```toml
[package]
name = "component-name"
version = "1.0.0"
edition = "2021"

[workspace]  # IMPORTANT: Prevents dependency conflicts

[lib]
crate-type = ["cdylib"]  # Required for WASM components

[dependencies]
wit-bindgen = "0.30"
# Add others as needed (e.g., serde_json)

[profile.release]
opt-level = "s"    # Optimize for size
lto = true         # Link-time optimization
strip = true       # Strip symbols
```

**Result**: Components are 50-150KB (json-stringify with serde_json is ~150KB, others are ~100KB)

### Testing Patterns

#### Pattern 11: Comprehensive Test Coverage

Each component should have minimum 3 tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typical_usage() {
        // Test common use case
        let inputs = vec![
            ("input".to_string(), Value::StringVal("hello".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        assert_eq!(result[0].0, "output");
    }

    #[test]
    fn test_edge_cases() {
        // Test boundaries: empty, zero, max values
        let inputs = vec![
            ("input".to_string(), Value::StringVal("".to_string())),
        ];
        let result = Component::execute(inputs).unwrap();
        // Validate expected behavior
    }

    #[test]
    fn test_error_handling() {
        // Test invalid inputs
        let inputs = vec![
            ("input".to_string(), Value::U32Val(42)),  // Wrong type
        ];
        let result = Component::execute(inputs);
        assert!(result.is_err());
    }
}
```

**Actual test coverage** (Phase 7 complete):
- Text: 21+ tests across 7 components
- Logic: 21+ tests across 7 components
- Math: 27+ tests across 9 components
- Collections: 21+ tests across 7 components
- Data: 32 tests across 4 components
- **Total: 122+ tests**

### Common Pitfalls (Learned from Implementation)

**1. Wrong WIT Import Paths** (Commits 00b1de9, b688840)

```rust
// ❌ WRONG
use exports::execution::Guest as ExecutionGuest;

// ✅ CORRECT
use exports::wasmflow::node::execution::Guest as ExecutionGuest;
```

**2. Missing export! Macro** (Commit 4b9600d)

```rust
impl ExecutionGuest for Component { ... }

export!(Component);  // ← REQUIRED! Forgot this in first iteration
```

**3. Generic ListVal Doesn't Exist** (Commit b688840)

```rust
// ❌ WRONG
Value::ListVal(items)

// ✅ CORRECT
Value::StringListVal(items)  // or U32ListVal, F32ListVal
```

**4. Pattern Matching Inside Lists** (Commit 12ed6d9)

```rust
// ❌ WRONG - list_values is Vec<String>, not Vec<Value>
for value in list_values.iter() {
    match value {
        Value::StringVal(s) => ...,  // Doesn't compile!
    }
}

// ✅ CORRECT
for value in list_values.iter() {
    // value is already &String
    result.push(value.clone());
}
```

**5. ComponentInfo Field Order** (Commit 00b1de9)

```rust
// ✅ CORRECT order and types
ComponentInfo {
    name: "Name".to_string(),
    version: "1.0.0".to_string(),        // Before description!
    description: "Description".to_string(),
    author: "Author".to_string(),
    category: Some("Category".to_string()),  // Option<String>!
}
```

### Performance Characteristics

Based on implementation experience:

- **Binary sizes**: 50-150KB (with LTO and strip)
- **Execution time**: <10ms for typical operations
- **Memory**: Stack-allocated, no heap allocations in hot paths
- **Compilation**: ~5-10 seconds per component in release mode

### Component Development Workflow

The proven workflow from Phase 3-7 implementation:

1. **Create structure** from template
2. **Implement metadata** (name, ports, category)
3. **Write execution logic** following patterns above
4. **Add 3+ unit tests** (typical, edge, error)
5. **Build and fix** import/export errors
6. **Test edge cases** discovered during development
7. **Add to category Justfile**
8. **Document** in phase documentation

**Time per component**: 15-30 minutes after learning patterns

### Documentation

**Per-Phase Documentation**:
- `specs/010-wasm-components-core/PHASE3_STRING_COMPONENTS.md` - Text (7)
- `specs/010-wasm-components-core/PHASE4_LOGIC_COMPONENTS.md` - Logic (7)
- `specs/010-wasm-components-core/PHASE5_MATH_COMPONENTS.md` - Math (9)
- `specs/010-wasm-components-core/PHASE6_LIST_COMPONENTS.md` - Collections (7)
- `specs/010-wasm-components-core/PHASE7_DATA_COMPONENTS.md` - Data (4)

**Library Documentation**:
- `components/LIBRARY.md` - Comprehensive API reference and developer guide
- `components/README.md` - User-focused usage guide

**Integration Tests**:
- `tests/component_tests/string_processing.json`
- `tests/component_tests/data_validation.json`
- `tests/component_tests/math_operations.json`
- `tests/component_tests/list_manipulation.json`
- `tests/component_tests/data_transformation.json`
- `tests/component_tests/comprehensive_workflow.json` - All categories

### Key Takeaways

1. **Consistency matters**: Using the same structure across all components made development predictable
2. **Error messages are UX**: Detailed errors with recovery hints saved debugging time
3. **Test early**: Unit tests caught 90% of issues before WASM build
4. **Templates accelerate**: Having working templates reduced copy-paste errors
5. **Documentation prevents rework**: Phase docs captured decisions and prevented backtracking

**Total implementation time**: ~4 days for all 34 components (including bug fixes and testing)

### Future Component Development

When adding new components to the library:

1. **Choose correct template**: `node.wit` for computation, `node-with-ui.wit` for custom rendering
2. **Follow naming**: `kebab-case` directories, `snake_case` Rust code
3. **Use proven patterns**: See examples above
4. **Write tests first**: TDD caught type system issues early
5. **Check existing components**: Similar components provide good templates
6. **Update Justfiles**: Add to category build/test/install targets

## Graphics Components and GLSL Shader Nodes

**Added**: 2025-11-20 (Phase 1: Core Graphics Nodes)

**Location**: `components/graphics/`, `src/builtin/shader_preview.rs`

Phase 1 provides foundational components for building 3D graphics pipelines with placeholder shader preview.

### Architecture Conventions

**Coordinate System:**
- Right-handed coordinate system (consistent with OpenGL/GLSL)
- +Y is up, -Z is forward (camera looks toward -Z)
- All rotations follow right-hand rule

**Matrix Format:**
- Column-major order (GLSL/OpenGL compatible)
- 4×4 matrices represented as 16-element f32 lists
- Matrix multiplication: `result = A × B` (A applied first, then B)

**Data Representations:**
- `vec3`: 3-element f32 list [x, y, z]
- `mat4`: 16-element f32 list (column-major)
- `color`: 3-element f32 list [r, g, b] (clamped to [0.0, 1.0])
- `UV coordinates`: (f32, f32) tuple

### Component Categories

**Vector Math** (7 components):
- `vec3-construct`: Build vector from x, y, z
- `vec3-add`, `vec3-subtract`: Basic arithmetic
- `vec3-scale`: Multiply by scalar
- `vec3-normalize`: Convert to unit length (returns length + normalized vector)
- `vec3-dot`: Scalar product (projection)
- `vec3-cross`: Vector product (perpendicular vector)

**Matrix Operations** (2 components):
- `mat4-construct`: Build from 16 components or 4 column vectors
- `mat4-multiply`: Standard matrix multiplication for transforms

**Color Utilities** (1 component):
- `color-rgb`: Create RGB color with automatic clamping

**Geometry Primitives** (3 components):
- `primitive-sphere`: UV sphere (parametric latitude/longitude)
- `primitive-cube`: Box with 24 vertices (4 per face for proper normals)
- `primitive-plane`: Subdivided XZ plane facing +Y

All geometry components output: positions, normals, UVs, indices (triangle list)

**Camera** (1 component):
- `perspective-camera`: Look-at view matrix + perspective projection
  - Inputs: position, target, up, fov, aspect_ratio, near, far
  - Outputs: view_matrix, projection_matrix, camera_position, view_direction

**Render Target** (1 component):
- `render-target`: Configure render target parameters
  - Formats: rgba8, rgba16-float, rgba32-float, rgb8, r8
  - MSAA: 1, 2, 4, or 8 samples
  - Outputs JSON configuration string

### Built-in Shader Preview Node

**Component ID**: `builtin:graphics:shader-preview`

**Phase 1 Status**: Placeholder mode - displays UI but no actual GPU rendering

**Implementation**:
- Located at: `src/builtin/shader_preview.rs`
- Node data: `ShaderPreviewNodeData` in `src/graph/node.rs`
- Registered in: `src/builtin/mod.rs`, `src/ui/app.rs`

**Node Data Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPreviewNodeData {
    pub preview_size: (u32, u32),      // Display dimensions
    pub auto_refresh: bool,             // Auto-refresh toggle
    pub refresh_rate: f32,              // Hz (1-60)
    pub zoom: f32,                      // Display zoom (0.1-10.0)
    #[serde(skip)]
    pub last_texture_size: Option<(u32, u32)>,  // Runtime only
    #[serde(skip)]
    pub last_update: Option<std::time::Instant>, // Runtime only
}
```

**Footer View Features**:
- Preview area with placeholder icon and status
- Size presets: Small (400×300), Medium (600×450), Large (800×600)
- Zoom slider (0.1× to 10×)
- Auto-refresh controls (on/off, rate 1-60 Hz)
- Stats display (last texture size, update time)

**Cloning Behavior**:
- `shader_preview_data` is cloned along with node
- Runtime state (`last_texture_size`, `last_update`) is reset to None

### Special Node Cloning

When cloning nodes with `shader_preview_data`:
```rust
// In src/ui/app/duplication.rs
shader_preview_data: original.shader_preview_data.clone(), // Clone shader preview data
```

Runtime-only fields (marked `#[serde(skip)]`) are automatically reset.

### Integration Testing

Test graphs in `tests/component_tests/`:

**graphics_geometry.json**:
- Tests primitive generation (sphere, cube, plane)
- Validates vertex counts and mesh structure

**graphics_camera.json**:
- Tests camera setup and matrix operations
- Validates MVP (Model-View-Projection) calculation

**graphics_shader_pipeline.json**:
- End-to-end pipeline from geometry to preview
- Demonstrates complete workflow with all component categories

### Build Requirements

**Dependencies**:
- `glam = "0.25"` - Vector and matrix math (all components)
- `serde = "1.0"`, `serde_json = "1.0"` - Serialization (render-target)
- `wit-bindgen = "0.30"` - WASM interface generation

**Binary Sizes**: 100-150 KB per component (with glam)

**Build Commands**:
```bash
cd components/graphics
just build        # Build all graphics components
just install      # Copy to components/bin/
```

### Phase Roadmap

**Phase 1: Core Components** (Complete ✓):
- Vector math operations
- Matrix operations
- Geometry primitives
- Camera system
- Render target configuration
- Shader preview placeholder

**Phase 2: GPU Integration & Lighting** (Complete ✓):
- WebGPU integration (wgpu 22.0 + naga)
- GLSL shader compilation (GLSL → WGSL)
- GPU buffer management (vertex, index, uniform)
- Multi-light support (up to 8 lights)
- Basic lighting components (directional, point, Phong)
- Shader program linker built-in node
- Example GLSL shaders (diffuse, Phong, multi-light)

**Phase 3: Advanced Features** (Future):
- PBR materials and BRDF calculations
- Shadow mapping and deferred rendering
- Post-processing effects (bloom, tone mapping, SSAO)
- Compute shaders
- Ray tracing utilities

### Documentation

**Component README**: `components/graphics/README.md`
- Complete API reference for all 15 components
- Usage examples and test descriptions
- Architecture notes and conventions

**Integration Tests**: `tests/component_tests/graphics_*.json`
- Demonstrates usage patterns
- Validates component behavior

### Common Patterns

**Vector Construction**:
```
vec3-construct(x: 1.0, y: 2.0, z: 3.0) → vec3: [1.0, 2.0, 3.0]
```

**Matrix Multiplication for MVP**:
```
model_matrix → mat4-multiply ← view_matrix
                     ↓
                projection_matrix → mvp_matrix
```

**Geometry Generation**:
```
primitive-sphere(radius: 1.0, segments: 16, rings: 8)
  → positions: [(segments+1)×(rings+1) vertices]
  → normals: [normalized per vertex]
  → uvs: [(u, v) per vertex]
  → indices: [triangle list]
```

**Camera Setup**:
```
camera_position, camera_target, camera_up
  → perspective-camera(fov: 60, aspect: 16:9, near: 0.1, far: 100)
  → view_matrix, projection_matrix
```

## Phase 2: GPU Integration and Lighting System

**Added**: 2025-11-22 (Phase 2: Steps 8-10)

**Location**: `src/gpu/`, `components/graphics/light-*`, `src/builtin/shader_program_linker.rs`, `examples/shaders/lighting/`

Phase 2 extends the graphics system with GPU integration, shader compilation, and lighting support.

### GPU Architecture

**WebGPU Integration** (`src/gpu/context.rs`):
- wgpu 22.0 for WebGPU implementation
- Async initialization with device/queue management
- Surface creation for rendering output

**Shader Compilation Pipeline** (`src/gpu/shader.rs`):
```
GLSL Source → naga Parser → naga IR → naga Validator → WGSL → wgpu::ShaderModule
```

**Key Types**:
```rust
pub struct CompiledShader {
    pub id: Uuid,
    pub source: String,
    pub module: wgpu::ShaderModule,
    pub stage: ShaderStage,  // Vertex or Fragment
    pub entry_point: String,
}

pub enum ShaderCompilationError {
    ParseError(String),
    ValidationError(String),
    SpirVGenerationError(String),
    InvalidStage,
    EntryPointNotFound(String),
}
```

**Compilation API**:
```rust
let shader = CompiledShader::from_glsl(
    &device,
    glsl_source,
    ShaderStage::Vertex,  // or ShaderStage::Fragment
    Some("main"),         // Entry point
)?;
```

### GPU Buffer System

**Location**: `src/gpu/buffer.rs`

**Vertex Buffer Layout**:
```rust
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],  // 12 bytes
    pub normal: [f32; 3],    // 12 bytes
    pub uv: [f32; 2],        // 8 bytes
}
// Total: 32 bytes per vertex
```

**GLSL Attributes**:
```glsl
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
```

**Uniform Buffer Layouts**:

**Camera Uniforms**:
```rust
#[repr(C)]
pub struct CameraUniforms {
    pub view_matrix: [f32; 16],       // 64 bytes
    pub projection_matrix: [f32; 16], // 64 bytes
    pub camera_position: [f32; 3],    // 12 bytes
    pub _padding: f32,                // 4 bytes
}
```

**Multi-Light Uniforms** (up to 8 lights):
```rust
pub const MAX_LIGHTS: usize = 8;

#[repr(C)]
pub struct LightData {
    pub position_or_direction: [f32; 3],  // 12 bytes
    pub light_type: u32,                  // 4 bytes (0=directional, 1=point)
    pub color: [f32; 3],                  // 12 bytes
    pub intensity: f32,                   // 4 bytes
    pub radius: f32,                      // 4 bytes (point lights)
    pub _padding: [f32; 3],               // 12 bytes
}

#[repr(C)]
pub struct MultiLightUniforms {
    pub lights: [LightData; MAX_LIGHTS],  // 384 bytes
    pub light_count: u32,                 // 4 bytes
    pub _padding: [f32; 3],               // 12 bytes
}
```

**GLSL Binding**:
```glsl
layout(set = 0, binding = 1) uniform Lights {
    LightData lights[8];
    uint lightCount;
} u_lights;
```

### Lighting Components

**light-directional** (`components/graphics/light-directional/`):
- Sun-like directional lighting with parallel rays
- Inputs: direction (vec3), color (vec3), intensity (f32)
- Output: light_data (JSON string for GPU uniforms)
- Features: Automatic direction normalization, color clamping

**light-point** (`components/graphics/light-point/`):
- Omni-directional point light with radius-based attenuation
- Inputs: position (vec3), color (vec3), intensity (f32), radius (f32)
- Output: light_data (JSON string for GPU uniforms)
- Attenuation: `1 / (1 + (distance² / radius²))`

**lighting-phong** (`components/graphics/lighting-phong/`):
- CPU-side Phong lighting calculation (diffuse + specular)
- Inputs: normal, light_dir, view_dir, surface_color, light_color, shininess
- Output: lit_color (vec3)
- Formula: `Diffuse = max(N·L, 0) * colors`, `Specular = (R·V)^shininess`

**JSON Light Data Format**:
```json
{
  "light_type": "directional",
  "direction": [0.0, -1.0, 0.0],
  "color": [1.0, 1.0, 1.0],
  "intensity": 1.0
}
```

### Shader Program Linker

**Component ID**: `builtin:graphics:shader-program-linker`

**Location**: `src/builtin/shader_program_linker.rs`

Links vertex and fragment shaders into an executable GPU program.

**Node Data**:
```rust
pub struct LinkedProgram {
    pub id: Uuid,
    pub vertex_shader_source: String,
    pub fragment_shader_source: String,
    pub compilation_status: ProgramStatus,
    pub error_message: Option<String>,
}

pub enum ProgramStatus {
    Idle,       // Not compiled
    Compiling,  // In progress
    Success,    // ✓ Linked successfully
    Failed,     // ✗ Error
}
```

**Linking Process**:
1. Validates vertex shader (GLSL → WGSL)
2. Validates fragment shader (GLSL → WGSL)
3. Creates wgpu::ShaderModule for both stages
4. TODO: Interface validation (vertex outputs match fragment inputs)
5. Generates unique program ID on success

**Footer UI Features**:
- Color-coded status indicator (gray/yellow/green/red)
- Program ID display (UUID)
- Scrollable error details with compilation messages
- Shader source line counts
- Manual link button (when idle or failed)

**Cloning Behavior**:
```rust
// In src/ui/app/duplication.rs
linked_program: original.linked_program.clone(),
```

### Example GLSL Shaders

**Location**: `examples/shaders/lighting/`

**basic_diffuse.vert.glsl / .frag.glsl**:
- Simple Lambert diffuse lighting
- Single directional light
- No specular highlights

**phong.vert.glsl / .frag.glsl**:
- Phong lighting model (diffuse + specular)
- Shininess parameter for highlight control
- View direction for specular calculation

**multi_light.vert.glsl / .frag.glsl**:
- Supports up to 8 mixed light types
- Directional and point lights in same shader
- Per-light attenuation for point lights
- Ambient term (10% of albedo)

**Shader Usage Pattern**:
```glsl
// Fragment shader with multi-light support
for (uint i = 0u; i < u_lights.lightCount && i < MAX_LIGHTS; i++) {
    LightData light = u_lights.lights[i];

    if (light.lightType == LIGHT_TYPE_DIRECTIONAL) {
        // Directional lighting
        vec3 lightDir = normalize(light.positionOrDirection);
        float diffuse = max(dot(normal, lightDir), 0.0);
        // ... specular calculation

    } else if (light.lightType == LIGHT_TYPE_POINT) {
        // Point light with attenuation
        vec3 lightVec = light.positionOrDirection - fragPosition;
        float distance = length(lightVec);
        float attenuation = 1.0 / (1.0 + (distance * distance) / (light.radius * light.radius));
        // ... lighting calculation
    }
}
```

### Integration Testing

**graphics_lighting.json**:
- Directional light creation
- Point light creation
- Phong lighting calculations (aligned, perpendicular, colored)
- Multi-component workflow (directional + point)

**graphics_complete_workflow.json**:
- Comprehensive 16-node pipeline
- Geometry → Camera → Lighting → Shader Authoring → Linking → Preview
- Demonstrates complete workflow from primitives to rendering

### Phase 2 Build Requirements

**Additional Dependencies**:
```toml
wgpu = "22.0"        # WebGPU implementation
naga = "22.0"        # Shader translation
bytemuck = "1.16"    # Pod/Zeroable for GPU buffers
```

**Build Commands**:
```bash
cd components/graphics
just build light-directional
just build light-point
just build lighting-phong
just install  # Copy all to components/bin/
```

### Critical Implementation Details

**Buffer Alignment**:
- All uniform buffers must use `#[repr(C)]`
- Add padding for 16-byte alignment requirements
- Use `bytemuck::Pod` and `bytemuck::Zeroable` derives

**Shader Compilation**:
- Always validate both stages before linking
- Provide detailed error messages with line numbers
- Cache compiled shaders by source hash (future optimization)

**Light Data JSON Parsing** (`src/gpu/buffer.rs`):
```rust
impl LightData {
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        // Parse JSON from WASM components
        // Convert to GPU-compatible struct
        // Validate light_type and parameters
    }
}
```

**Common Patterns**:

**Loading Light Data**:
```
light-directional → light_data (JSON)
                         ↓
                GPU Buffer (parse JSON)
                         ↓
                    Shader Uniform
```

**Shader Pipeline**:
```
Vertex GLSL → shader-program-linker ← Fragment GLSL
                      ↓
              LinkedProgram (UUID)
                      ↓
         (Future: Render Pipeline Creation)
```

### Documentation

**GPU Integration Guide**: `docs/GPU_INTEGRATION.md`
- Complete shader compilation pipeline documentation
- Buffer layout specifications
- Lighting system reference
- Example shader walkthroughs
- Troubleshooting guide

**Component README**: `components/graphics/README.md`
- Updated with Phase 2 lighting components
- Shader program linker documentation
- Multi-light uniform buffer specs

**Shader Examples**: `examples/shaders/lighting/README.md`
- Buffer layout diagrams
- Shader usage examples
- Performance notes

## Phase 3: PBR Materials and Texture System

**Added**: 2025-11-22 (Phase 3: Step 1 + Steps 3-7 complete)

**Location**: `src/gpu/texture.rs`, `src/builtin/texture_loader.rs`, `components/graphics/pbr-*`, `components/graphics/normal-map`, `examples/shaders/pbr/`

Phase 3 implements physically-based rendering (PBR) with texture loading, Cook-Torrance BRDF, and normal mapping.

### Texture System (Step 1 Complete ✓)

**GPU Texture Management** (`src/gpu/texture.rs`):
- `GpuTexture` struct: Wraps wgpu::Texture, view, and sampler
- `from_rgba8()`: Create texture from RGBA8 pixel data
- `from_rgb8()`: Create texture from RGB8 data (adds alpha = 255)
- `create_render_target()`: Create offscreen render target with MSAA
- `create_depth_texture()`: Create depth buffer for depth testing
- Procedural generators: `generate_solid_color()`, `generate_checker()`, `generate_gradient()`

**Texture Loader Built-in Node** (`src/builtin/texture_loader.rs`):
- Component ID: `builtin:graphics:texture-loader`
- File picker UI for PNG, JPG, BMP, GIF images
- Loads image via `image` crate, converts to RGBA8
- Outputs: `texture` (TextureData), `width` (u32), `height` (u32)
- Footer view features:
  - Thumbnail preview (max 256×256, aspect-ratio preserved)
  - File path and dimensions display
  - Memory usage statistics
  - Error message display
  - Status indicator (loaded/no file)

**Texture Sampler Component** (existing):
- CPU-side bilinear texture sampling
- UV wrapping modes: repeat, clamp, mirror
- Used for CPU workflows and testing

**Integration:**
```rust
// In node graph
TextureData { width, height, data: Vec<u8>, format: TextureFormat::Rgba8 }

// Upload to GPU
let gpu_texture = GpuTexture::from_rgba8(&device, &queue, width, height, &data, Some("label"))?;
```

### PBR BRDF Components (Steps 3-7 Complete ✓)

**Cook-Torrance Microfacet BRDF:**

Components implement the formula:
```
f(l, v) = k_d * base_color / π + k_s * (D * F * G) / (4 * (n·v) * (n·l))

Where:
  D = GGX normal distribution function
  F = Fresnel-Schlick approximation
  G = Smith geometry function
  k_d = (1 - F) * (1 - metallic)  // Energy conservation
  k_s = F
```

**Components:**

1. **pbr-fresnel** (`components/graphics/pbr-fresnel/`):
   - Inputs: `f0` (vec3), `view_dir` (vec3), `half_vector` (vec3)
   - Output: `fresnel` (vec3)
   - Formula: `F = F0 + (1 - F0) * (1 - cos_theta)^5`
   - 5 unit tests

2. **pbr-ggx-distribution** (`components/graphics/pbr-ggx-distribution/`):
   - Inputs: `normal` (vec3), `half_vector` (vec3), `roughness` (f32)
   - Output: `distribution` (f32)
   - Formula: `D = α² / (π * ((n·h)² * (α² - 1) + 1)²)` where α = roughness²
   - 6 unit tests

3. **pbr-smith-geometry** (`components/graphics/pbr-smith-geometry/`):
   - Inputs: `normal`, `view_dir`, `light_dir`, `roughness`
   - Output: `geometry` (f32)
   - Formula: `G = G1(v) * G1(l)` with GGX variant
   - 7 unit tests

4. **pbr-material** (`components/graphics/pbr-material/`):
   - Inputs: `base_color` (vec3), `metallic` (f32), `roughness` (f32), `ao` (f32, optional)
   - Outputs: `f0`, `roughness`, `ao`, `base_color`
   - F0 calculation: `lerp(vec3(0.04), base_color, metallic)`
   - 9 unit tests

5. **pbr-brdf** (`components/graphics/pbr-brdf/`):
   - Complete Cook-Torrance BRDF implementation
   - Inputs: `normal`, `view_dir`, `light_dir`, `f0`, `roughness`, `base_color`, `metallic`
   - Outputs: `diffuse` (vec3), `specular` (vec3), `total_brdf` (vec3)
   - Energy conservation verified in tests
   - 8 unit tests

**Advanced Lighting:**

6. **light-spot** (`components/graphics/light-spot/`):
   - Spot light with cone-shaped emission
   - Inputs: `position`, `direction`, `color`, `intensity`, `inner_angle`, `outer_angle`, `radius`
   - Output: `light_data` (JSON string for GPU uniforms)
   - Cone falloff: `smoothstep(outer_cos, inner_cos, cos_angle)`
   - Distance attenuation: `1 / (1 + (d² / r²))`
   - 9 unit tests

**Normal Mapping:**

7. **normal-map** (`components/graphics/normal-map/`):
   - Tangent-space to world-space normal transformation
   - Inputs: `tangent_normal` (vec3), `normal` (vec3), `tangent` (vec3), `bitangent` (vec3, optional)
   - Output: `world_normal` (vec3)
   - TBN matrix construction: `[T B N]` (column vectors)
   - Conversion: `[0,1] → * 2.0 - 1.0 → [-1,1]` tangent space
   - Auto-calculates bitangent if not provided: `B = N × T`
   - 8 unit tests

### Example PBR Shaders

**Location**: `examples/shaders/pbr/`

1. **pbr_single_light.vert/frag.glsl**:
   - Single directional light PBR
   - Complete Cook-Torrance BRDF in fragment shader
   - Tone mapping (Reinhard) + gamma correction
   - ~50-100 ALU per fragment

2. **pbr_multi_light.vert/frag.glsl**:
   - Up to 8 mixed lights (directional, point, spot)
   - Multi-light accumulation loop
   - Per-light attenuation and cone falloff
   - ~50-100 ALU per light per fragment

3. **pbr_normal_mapped.vert/frag.glsl**:
   - Full PBR with normal mapping
   - TBN matrix construction in vertex shader
   - Normal map sampling and transformation
   - `normal_strength` parameter for blending
   - +10-15 ALU for TBN transformation

### Integration Tests

**graphics_texture_sampling.json** (10 tests):
- Texture sampler component tests
- UV coordinates (center, corners, edges)
- Wrapping modes (repeat, clamp, mirror)
- Bilinear interpolation verification
- Negative UV handling
- Complete texture sampling workflow

**graphics_pbr_workflow.json** (12 tests):
- Individual PBR component tests
- Complete PBR pipeline workflow
- Multi-material comparison (plastic vs gold)
- Energy conservation validation

**graphics_pbr_multi_light.json** (13 tests):
- Spot light configuration tests
- Multi-light scene creation (directional + point + spot)
- Roughness variation tests
- Complete multi-light PBR scene

**graphics_normal_mapping.json** (9 tests):
- Basic normal mapping tests
- Complete normal-mapped PBR pipeline
- Effect comparison (flat vs bumped)
- TBN orthogonality verification

### Material Presets

**Metals:**
- Gold: `{base_color: [1.0, 0.71, 0.29], metallic: 1.0, roughness: 0.2}`
- Copper: `{base_color: [0.95, 0.64, 0.54], metallic: 1.0, roughness: 0.3}`
- Brushed Aluminum: `{base_color: [0.9, 0.9, 0.9], metallic: 1.0, roughness: 0.6}`

**Dielectrics:**
- Red Plastic: `{base_color: [0.8, 0.1, 0.1], metallic: 0.0, roughness: 0.5}`
- Polished Stone: `{base_color: [0.3, 0.3, 0.35], metallic: 0.0, roughness: 0.2, ao: 0.9}`
- Rough Fabric: `{base_color: [0.6, 0.1, 0.1], metallic: 0.0, roughness: 0.7}`

### Documentation

**PBR Implementation Guide**: `docs/PHASE3_PBR_COMPLETE.md` (472 lines)
- Complete Phase 3 documentation
- All component details with formulas
- Material property guidelines
- Example configurations
- Performance characteristics
- Physical accuracy principles

**Graphics Pipeline Summary**: `docs/GRAPHICS_PIPELINE_SUMMARY.md` (650+ lines)
- Complete architecture overview
- Component reference table
- Shader reference table
- Integration test summary
- Example workflows
- Material presets
- Performance characteristics
- Usage examples (GLSL and WASM)

**PBR Shader README**: `examples/shaders/pbr/README.md` (398 lines)
- Buffer layout specifications
- Normal mapping integration
- Material workflow documentation
- Troubleshooting guide

### Performance Characteristics

**WASM Components:**
- Binary sizes: 100-120 KB per component (with glam + LTO)
- Execution: <1ms per BRDF calculation
- Memory: Stack-allocated, no heap allocations

**GLSL Shaders:**
- Single light: ~50-100 ALU per fragment
- Multi-light: ~50-100 ALU per light per fragment
- Normal mapping: +10-15 ALU for TBN
- GPU throughput: Millions of fragments per second

**Texture System:**
- Image loading: `image` crate with PNG, JPG, BMP, GIF support
- Texture upload: <100ms for 1024×1024 RGBA8
- GPU formats: Rgba8UnormSrgb (base color), Rgba8Unorm (normal maps)

### Future Work (Phase 4)

- **Texture Maps**: Albedo, normal, metallic/roughness, AO, emissive texture support
- **IBL**: Image-based lighting with split-sum approximation
- **Advanced PBR**: Clear coat, subsurface scattering, anisotropic reflections
- **Shadow Mapping**: Directional, point, spot shadows with PCF/VSM
- **Post-Processing**: Bloom, tone mapping, SSAO, DOF, motion blur
- **Deferred Rendering**: Support for many lights (>8)
- **Performance**: Compute shaders, light culling, LOD systems

<!-- MANUAL ADDITIONS END -->
