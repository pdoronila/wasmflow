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

<!-- MANUAL ADDITIONS END -->
