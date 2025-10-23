# Contract: ComponentBodyView Trait

**Feature**: `004-node-input-update`
**Date**: 2025-10-16
**Type**: Rust Trait Interface

## Overview

`ComponentBodyView` is a trait interface enabling component developers to provide custom interactive UI rendering in the body section of nodes. If not implemented, nodes automatically receive a default body view with input fields for all parameters.

---

## Trait Definition

```rust
/// Location: src/ui/component_view.rs

pub trait ComponentBodyView: Send + Sync {
    /// Render custom body content for this component.
    ///
    /// # Parameters
    /// - `ui`: egui UI context for rendering widgets
    /// - `node`: The graph node being displayed (mutable for interactive updates)
    ///
    /// # Returns
    /// - `Ok(())` if rendering succeeded
    /// - `Err(message)` if rendering failed (error will be displayed in body section)
    ///
    /// # Performance Requirements
    /// - MUST complete in <50ms to maintain 60 FPS rendering
    /// - Called every frame while node is visible
    /// - Avoid expensive operations (network, disk I/O, heavy computation)
    ///
    /// # Thread Safety
    /// - Implementations MUST be Send + Sync
    /// - May be accessed from multiple threads in egui's rendering pipeline
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String>;
}
```

---

## Contract Obligations

### Implementers MUST

1. **Complete rendering in <50ms**
   - Performance is monitored (warnings logged if exceeded)
   - Keep UI simple: avoid complex layouts, limit widget count
   - Cache expensive computations if needed

2. **Handle missing/invalid state gracefully**
   - Check for `node.inputs` existence before accessing
   - Validate data types before casting
   - Return `Err(message)` for unrecoverable errors (displayed to user)

3. **Update node state correctly**
   - Modify `node.inputs[i].current_value` when user edits parameters
   - Mark `node.dirty = true` if changes require re-execution
   - Do NOT modify `node.outputs` (read-only in body view)

4. **Be stateless**
   - egui immediate-mode means `render_body()` called every frame
   - Do NOT rely on instance variables for UI state
   - Read all state from `node` parameter
   - Any persistent state must be stored in `node.inputs` or component-specific data

5. **Respect the body section context**
   - Body section is between connections (above) and footer (below)
   - Available width: ~250px (constrained by snarl node layout)
   - Do NOT attempt to render outside body bounds (egui clips automatically)

### Implementers SHOULD

1. **Follow egui widget conventions**
   - Use standard widgets when possible (DragValue, TextEdit, Slider, etc.)
   - Maintain consistent spacing with `ui.add_space()`
   - Use `ui.horizontal()` / `ui.vertical()` for layout

2. **Provide clear labels**
   - Show parameter names next to input widgets
   - Use tooltips (`.on_hover_text()`) for complex parameters
   - Display units if applicable (e.g., "Speed: [input] m/s")

3. **Validate user input**
   - Clamp numeric values to valid ranges
   - Show error messages for invalid input (e.g., red text)
   - Prevent setting invalid values in `node.inputs`

4. **Limit interactive widgets**
   - Target: <20 widgets per body view (performance bound)
   - Use collapsible sections (egui::CollapsingHeader) for large parameter sets
   - Prioritize most-used parameters in default view

### Users (CanvasViewer) MUST

1. **Call `render_body()` only when needed**
   - Check `ComponentSpec.has_body_view()` first
   - Fall back to `DefaultBodyView` if returns false

2. **Handle errors appropriately**
   - Display `Err(message)` in body section with red text
   - Log errors for debugging
   - Continue rendering other sections even if body fails

3. **Provide mutable node reference**
   - `render_body()` requires `&mut GraphNode` for interactive updates
   - Ensure node reference is valid and not aliased elsewhere

4. **Monitor performance**
   - Measure render time using `std::time::Instant`
   - Log warnings if >50ms (see example in show_footer implementation)

---

## Usage Examples

### Example 1: Simple Numeric Parameter Editor

```rust
struct MathOperatorBodyView;

impl ComponentBodyView for MathOperatorBodyView {
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Get the 'multiplier' input port
        if let Some(input) = node.get_input_mut("multiplier") {
            if let Some(NodeValue::F32(ref mut value)) = input.current_value {
                ui.horizontal(|ui| {
                    ui.label("Multiplier:");
                    if ui.add(egui::DragValue::new(value).speed(0.1)).changed() {
                        node.dirty = true; // Mark for re-execution
                    }
                });
            }
        }

        Ok(())
    }
}

// Register with component spec
let spec = ComponentSpec::new_builtin(
    "builtin:math:multiply".to_string(),
    "Multiply".to_string(),
    "Multiply input by constant".to_string(),
    Some("Math".to_string())
)
.with_input("value", DataType::F32, "Input value".to_string())
.with_input("multiplier", DataType::F32, "Multiplier constant".to_string())
.with_output("result", DataType::F32, "value * multiplier".to_string())
.with_body_view(Arc::new(MathOperatorBodyView));
```

**Rendered UI**:
```
┌────────────────────┐
│   Multiply    [✖]  │ ← Header
├────────────────────┤
│o value:f32         │ ← Connections (inputs)
│o multiplier:f32    │
│        result:f32 o│ ← Connections (outputs)
├────────────────────┤
│Multiplier: [2.5]▼  │ ← Custom Body (this view)
├────────────────────┤
│Current result: 5.0 │ ← Default Footer
└────────────────────┘
```

---

### Example 2: String Input with Validation

```rust
struct TextFilterBodyView;

impl ComponentBodyView for TextFilterBodyView {
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        if let Some(pattern_input) = node.get_input_mut("pattern") {
            if let Some(NodeValue::String(ref mut pattern)) = pattern_input.current_value {
                ui.label("Filter pattern:");

                // Text input for pattern
                let response = ui.add(
                    egui::TextEdit::singleline(pattern)
                        .hint_text("Enter regex pattern")
                        .desired_width(200.0)
                );

                // Validate regex on change
                if response.changed() {
                    match regex::Regex::new(pattern) {
                        Ok(_) => {
                            node.dirty = true; // Valid pattern, trigger re-execution
                        }
                        Err(e) => {
                            // Show error without blocking input
                            ui.colored_label(egui::Color32::RED, format!("Invalid regex: {}", e));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
```

**Rendered UI**:
```
┌──────────────────────────┐
│   Text Filter      [✖]   │
├──────────────────────────┤
│o text:string             │
│o pattern:string          │
│             filtered:str o│
├──────────────────────────┤
│Filter pattern:           │ ← Custom Body
│[Enter regex pattern...]  │
│Invalid regex: unclosed ( │ ← Validation error
├──────────────────────────┤
│Current filtered: (empty) │
└──────────────────────────┘
```

---

### Example 3: Complex Type with Fallback

```rust
struct DataProcessorBodyView;

impl ComponentBodyView for DataProcessorBodyView {
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Handle simple config parameters
        if let Some(threshold) = node.get_input_mut("threshold") {
            if let Some(NodeValue::F32(ref mut val)) = threshold.current_value {
                ui.horizontal(|ui| {
                    ui.label("Threshold:");
                    ui.add(egui::Slider::new(val, 0.0..=1.0));
                });
            }
        }

        // Show placeholder for complex config (not editable in body)
        if let Some(config) = node.get_input("config") {
            if matches!(config.data_type, DataType::Record(_)) {
                ui.label("Config: (use JSON editor)");
                ui.small("Complex types require custom editor");
            }
        }

        Ok(())
    }
}
```

---

## Error Handling

### Recoverable Errors (Return Err)

```rust
fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
    // Check preconditions
    let input = node.get_input("required_param")
        .ok_or_else(|| "Missing required input 'required_param'".to_string())?;

    let value = input.current_value.as_ref()
        .ok_or_else(|| "Input value not set".to_string())?;

    // Validate type
    if !matches!(value, NodeValue::F32(_)) {
        return Err(format!("Expected F32, got {}", value.type_name()));
    }

    // ... render UI ...
    Ok(())
}
```

**User Experience**:
- Error message shown in red text in body section
- Other sections (header, connections, footer) still render
- User can fix issue by reconnecting inputs or editing values

### Unrecoverable Errors (Panic - AVOID)

```rust
// ❌ BAD: Don't panic, return Err instead
fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
    let input = node.get_input("param").unwrap(); // PANICS if missing!
    // ...
}

// ✅ GOOD: Handle gracefully
fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
    let input = node.get_input("param")
        .ok_or("Missing parameter 'param'")?;
    // ...
}
```

---

## Performance Guidelines

### Measuring Render Time

```rust
impl ComponentBodyView for MyView {
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        let start = std::time::Instant::now();

        // ... render UI ...

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 50 {
            log::warn!("Slow body view: {}ms", elapsed.as_millis());
        }

        Ok(())
    }
}
```

### Optimization Tips

1. **Avoid allocations in hot path**
   ```rust
   // ❌ Slow: Allocates string every frame
   ui.label(format!("Value: {}", value));

   // ✅ Fast: Use formatting only when needed
   ui.label("Value:");
   ui.label(value.to_string());
   ```

2. **Limit widget count**
   ```rust
   // ❌ Slow: 100 widgets
   for i in 0..100 {
       ui.label(format!("Param {}: [input]", i));
   }

   // ✅ Fast: Collapsible sections
   ui.collapsing("Parameters (100)", |ui| {
       for i in 0..100 {
           ui.label(format!("Param {}", i));
       }
   });
   ```

3. **Cache expensive computations**
   ```rust
   // Store computed values in node.inputs current_value
   // Don't recompute on every frame unless dirty
   ```

---

## Integration with ComponentSpec

### Registration

```rust
let spec = ComponentSpec::new_builtin(
    "builtin:example".to_string(),
    "Example".to_string(),
    "Example component".to_string(),
    None
)
.with_input("param", DataType::F32, "Parameter".to_string())
.with_output("result", DataType::F32, "Result".to_string())
.with_body_view(Arc::new(MyBodyView)); // ← Register custom body view

registry.register_builtin(spec);
```

### Checking for Custom View

```rust
// In CanvasViewer::show_body()
if let Some(spec) = registry.get_by_id(&node.component_id) {
    if let Some(view) = spec.get_body_view() {
        // Use custom view
        match view.render_body(ui, node) {
            Ok(()) => { /* Success */ }
            Err(e) => { /* Show error */ }
        }
    } else {
        // Use default body view
        DefaultBodyView::render_for_node(ui, node)?;
    }
}
```

---

## Backward Compatibility

### Existing Components Without Custom Body

- **Before**: No body section shown (or minimal constant editing)
- **After**: Default body view auto-generates input fields
- **Migration**: Zero changes required, automatic upgrade

### Components with Existing Custom Footer

- **Before**: Custom footer shown in footer section
- **After**: Custom footer still shown, default body auto-generated
- **Migration**: Optionally add `with_body_view()` to customize body too

### Constant Nodes (Special Case)

- **Before**: Inline value editing in body via hardcoded `show_body()`
- **After**: Move to `ComponentBodyView` implementation for consistency
- **Migration**: Refactor constant editing into custom body view

---

## Testing Requirements

### Unit Tests (src/tests/unit/component_view_tests.rs)

```rust
#[test]
fn test_body_view_renders_without_error() {
    let view = MyBodyView;
    let mut node = create_test_node();
    let mut ctx = egui::Context::default();

    ctx.run(Default::default(), |ctx| {
        let mut ui = ctx.debug_painter();
        let result = view.render_body(&mut ui, &mut node);
        assert!(result.is_ok());
    });
}

#[test]
fn test_body_view_handles_missing_input() {
    let view = MyBodyView;
    let mut node = GraphNode::new(/*...*/); // No inputs
    let result = view.render_body(&mut ui, &mut node);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing"));
}

#[test]
fn test_body_view_updates_dirty_flag() {
    let view = MyBodyView;
    let mut node = create_test_node_with_inputs();

    // Simulate user edit (requires egui interaction, may need mock)
    // Assert node.dirty == true after edit
}
```

### Integration Tests (src/tests/integration/canvas_view_tests.rs)

```rust
#[test]
fn test_four_section_rendering_with_custom_body() {
    let mut canvas = NodeCanvas::new();
    let registry = create_test_registry_with_custom_views();
    let graph = create_test_graph();

    // Render canvas
    canvas.show(&mut ui, &mut graph, &registry);

    // Verify custom body view was called
    // Verify default footer was shown (if no custom footer)
}
```

---

## Summary

### Trait Purpose
Enable component developers to provide custom interactive UI in the body section, with automatic fallback to default input fields.

### Key Constraints
- Performance: <50ms render time
- Thread safety: Send + Sync
- Stateless: Re-rendered every frame
- Error handling: Return Err, never panic

### Integration Points
- `ComponentSpec.with_body_view(Arc::new(View))`
- `CanvasViewer.show_body()` checks `has_body_view()` before rendering
- `DefaultBodyView` used as fallback for all components

### Next Contract
See `component-footer-view.md` for parallel footer view interface (existing, no changes).
