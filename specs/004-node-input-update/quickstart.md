# Quickstart: Four-Section Node Layout

**Feature**: `004-node-input-update`
**Date**: 2025-10-16
**Audience**: Component developers and WasmFlow contributors

## Overview

This guide shows how to create and customize nodes with the new four-section layout. You'll learn:
1. How default views work automatically
2. When and how to create custom body views
3. How to migrate existing components
4. Performance best practices

---

## Quick Start: Using Default Views (Zero Code)

### Example: Simple Math Component

The easiest way to use the four-section layout is to **do nothing** - default views are automatic!

```rust
use wasmflow::graph::node::{ComponentSpec, DataType};

// Create a component spec (no custom views needed)
let spec = ComponentSpec::new_builtin(
    "builtin:math:add".to_string(),
    "Add".to_string(),
    "Add two numbers".to_string(),
    Some("Math".to_string())
)
.with_input("a", DataType::F32, "First number".to_string())
.with_input("b", DataType::F32, "Second number".to_string())
.with_output("sum", DataType::F32, "a + b".to_string());

// Register with registry
registry.register_builtin(spec);
```

**Result**: Node automatically gets:
- ✅ **Header**: "Add" with delete button
- ✅ **Connections**: Inputs (a, b) on left, output (sum) on right with type colors
- ✅ **Body**: Default input fields for a and b (DragValue widgets)
- ✅ **Footer**: "Current sum: 5.0" when executed

**Rendered UI**:
```
┌──────────────────┐
│   Add       [✖]  │
├──────────────────┤
│o a:f32    sum:f32 o │
│o b:f32           │
├──────────────────┤
│a: [2.0]▼         │ ← Default body
│b: [3.0]▼         │
├──────────────────┤
│Current sum: 5.0  │ ← Default footer
└──────────────────┘
```

---

## Custom Body View: Interactive Controls

### When to Use Custom Body Views

Create a custom body view when:
- You need specialized widgets (color pickers, file selectors, custom editors)
- You want to validate input before accepting it
- You need layout control (multi-column, collapsible sections)
- Default widgets don't match your UX needs

**DON'T create custom views for**:
- Simple numeric inputs (default DragValue works great)
- Basic string inputs (default TextEdit is fine)
- Just to change labels (use descriptive port names instead)

### Example: Custom Slider Range

```rust
use std::sync::Arc;
use wasmflow::ui::component_view::ComponentBodyView;
use wasmflow::graph::node::{GraphNode, NodeValue};

// Step 1: Implement ComponentBodyView trait
struct RangeSliderView {
    min: f32,
    max: f32,
}

impl ComponentBodyView for RangeSliderView {
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Get the 'value' input port
        if let Some(input) = node.get_input_mut("value") {
            if let Some(NodeValue::F32(ref mut value)) = input.current_value {
                ui.label("Value:");

                // Custom slider widget with range constraints
                if ui.add(egui::Slider::new(value, self.min..=self.max)
                    .text("units"))
                    .changed()
                {
                    node.dirty = true; // Trigger re-execution
                }
            }
        }

        Ok(())
    }
}

// Step 2: Create component spec with custom view
let spec = ComponentSpec::new_builtin(
    "builtin:ranged:value".to_string(),
    "Ranged Value".to_string(),
    "Value with slider constraints".to_string(),
    Some("Input".to_string())
)
.with_input("value", DataType::F32, "Input value".to_string())
.with_output("output", DataType::F32, "Constrained value".to_string())
.with_body_view(Arc::new(RangeSliderView { min: 0.0, max: 100.0 })); // ← Register custom view

// Step 3: Register
registry.register_builtin(spec);
```

**Result**:
```
┌──────────────────────────┐
│   Ranged Value      [✖]  │
├──────────────────────────┤
│o value:f32    output:f32 o│
├──────────────────────────┤
│Value:                    │ ← Custom body
│[━━━━━━━━●━━━] 75 units   │ (Slider instead of DragValue)
├──────────────────────────┤
│Current output: 75.00     │ ← Default footer (still automatic)
└──────────────────────────┘
```

---

## Custom Footer View: Rich Status Display

### When to Use Custom Footer Views

Create a custom footer when:
- You need complex visualizations (charts, previews, multi-line output)
- You want to format output in a specific way
- You need to show derived/computed information beyond output values

### Example: HTTP Request Status

```rust
use wasmflow::ui::component_view::ComponentFooterView;

struct HttpFetchFooterView;

impl ComponentFooterView for HttpFetchFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
        // Get the 'response' output
        if let Some(response_output) = node.get_output("response") {
            if let Some(NodeValue::Record(fields)) = &response_output.current_value {
                // Extract status code and body from record
                if let Some(NodeValue::U32(status)) = fields.get("status") {
                    let color = if *status < 400 {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    };

                    ui.colored_label(color, format!("Status: {}", status));
                }

                if let Some(NodeValue::String(body)) = fields.get("body") {
                    ui.label("Response:");
                    ui.add(egui::TextEdit::multiline(&mut body.as_str())
                        .desired_width(230.0)
                        .desired_rows(3));
                }

                return Ok(());
            }
        }

        // Fallback if no response yet
        ui.label("(no response yet)");
        Ok(())
    }
}

// Register with component
let spec = ComponentSpec::new_builtin(
    "builtin:http:fetch".to_string(),
    "HTTP Fetch".to_string(),
    "Make HTTP request".to_string(),
    Some("Network".to_string())
)
.with_input("url", DataType::String, "Request URL".to_string())
.with_output("response", DataType::Record(vec![
    ("status".to_string(), DataType::U32),
    ("body".to_string(), DataType::String),
]), "HTTP response".to_string())
// No custom body view - get default URL input field automatically
.with_footer_view(Arc::new(HttpFetchFooterView)); // ← Custom footer only

registry.register_builtin(spec);
```

**Result**:
```
┌────────────────────────────┐
│   HTTP Fetch          [✖]  │
├────────────────────────────┤
│o url:string  response:rec o│
├────────────────────────────┤
│url: [https://api...]       │ ← Default body (automatic)
├────────────────────────────┤
│Status: 200                 │ ← Custom footer
│Response:                   │
│{                           │
│  "data": "example"         │
│}                           │
└────────────────────────────┘
```

**Note**: This example mixes default body (automatic URL input) with custom footer (rich status display).

---

## Combining Custom Body + Custom Footer

You can provide **both** custom views for full control:

```rust
let spec = ComponentSpec::new_builtin(...)
    .with_body_view(Arc::new(MyBodyView))    // Custom body
    .with_footer_view(Arc::new(MyFooterView)) // Custom footer
    // Both default views are skipped
    .build();
```

---

## Migration Guide: Existing Components

### Scenario 1: Component with No Custom Views

**Before** (three-section):
```rust
let spec = ComponentSpec::new_builtin(...)
    .with_input("x", DataType::F32, "Input".to_string())
    .with_output("y", DataType::F32, "Output".to_string());
```

**After** (four-section):
```rust
// No changes needed!
let spec = ComponentSpec::new_builtin(...)
    .with_input("x", DataType::F32, "Input".to_string())
    .with_output("y", DataType::F32, "Output".to_string());
// Automatically gets default body + default footer
```

**Impact**: Node gains interactive input field for 'x' and "Current y: ..." footer automatically.

---

### Scenario 2: Component with Existing Custom Footer

**Before** (three-section):
```rust
let spec = ComponentSpec::new_builtin(...)
    .with_footer_view(Arc::new(MyCustomFooterView));
```

**After** (four-section):
```rust
// No changes needed!
let spec = ComponentSpec::new_builtin(...)
    .with_footer_view(Arc::new(MyCustomFooterView));
// Custom footer still shown, body gets default input fields automatically
```

**Impact**: Custom footer unchanged, body section gains default input fields (new feature).

---

### Scenario 3: Constant Nodes (Hardcoded Editing)

**Before** (hardcoded in `canvas.rs::show_body`):
```rust
fn show_body(...) {
    if node.component_id.starts_with("builtin:constant:") {
        // Hardcoded constant editing
        match value {
            NodeValue::F32(v) => {
                let mut text = v.to_string();
                ui.add(egui::TextEdit::singleline(&mut text));
                // ...
            }
        }
    }
}
```

**After** (move to ComponentBodyView):
```rust
// Step 1: Create view struct
struct ConstantF32BodyView;

impl ComponentBodyView for ConstantF32BodyView {
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        if let Some(output) = node.get_output_mut("value") {
            if let Some(NodeValue::F32(ref mut v)) = output.current_value {
                let mut text = v.to_string();
                if ui.add(egui::TextEdit::singleline(&mut text).desired_width(60.0)).changed() {
                    if let Ok(new_val) = text.parse::<f32>() {
                        *v = new_val;
                        node.dirty = true;
                    }
                }
            }
        }
        Ok(())
    }
}

// Step 2: Register with component spec
let spec = ComponentSpec::new_builtin(
    "builtin:constant:f32".to_string(),
    "Constant F32".to_string(),
    "Constant floating-point value".to_string(),
    Some("Constants".to_string())
)
.with_output("value", DataType::F32, "Constant value".to_string())
.with_body_view(Arc::new(ConstantF32BodyView)); // ← Moved from hardcoded show_body

// Step 3: Remove hardcoded check from canvas.rs
// Delete the `if node.component_id.starts_with("builtin:constant:")` block
```

**Migration checklist**:
1. ✅ Extract editing logic to `ComponentBodyView` implementation
2. ✅ Register view with `with_body_view()`
3. ✅ Remove hardcoded check from `canvas.rs::show_body`
4. ✅ Test that editing still works

---

## Performance Best Practices

### ✅ DO: Keep Render Time Under 50ms

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

### ✅ DO: Limit Widget Count

```rust
// For nodes with many parameters, use collapsible sections
if node.inputs.len() > 10 {
    ui.collapsing("Parameters", |ui| {
        for input in &mut node.inputs {
            // Render fields
        }
    });
} else {
    // Render inline
}
```

### ❌ DON'T: Do Expensive Operations in Render Loop

```rust
// ❌ BAD: Network request every frame (60 FPS!)
fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
    let data = reqwest::blocking::get("https://api.example.com/data")?; // SLOW!
    ui.label(format!("Data: {}", data));
    Ok(())
}

// ✅ GOOD: Cache data in node state, fetch only when dirty
fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
    // Assume cached_data stored in node.inputs[0].current_value
    if let Some(NodeValue::String(cached)) = node.get_input("cached_data").and_then(|p| p.current_value.as_ref()) {
        ui.label(format!("Data: {}", cached));
    }
    Ok(())
}
```

### ✅ DO: Use Efficient Widgets

```rust
// ✅ GOOD: DragValue is faster than TextEdit for numbers
if let Some(NodeValue::F32(ref mut v)) = port.current_value {
    ui.add(egui::DragValue::new(v).speed(0.1)); // Fast
}

// ❌ SLOWER: TextEdit requires parsing
let mut text = v.to_string();
ui.add(egui::TextEdit::singleline(&mut text)); // Slower (string allocation + parse)
```

---

## Testing Your Custom Views

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_body_view_renders() {
        let view = MyCustomBodyView;
        let mut node = GraphNode::new(
            "test:component".to_string(),
            "Test Node".to_string(),
            egui::Pos2::ZERO
        );

        // Add test inputs
        node.inputs.push(Port::new(
            "test_input".to_string(),
            DataType::F32,
            PortDirection::Input,
            false
        ));
        node.inputs[0].current_value = Some(NodeValue::F32(42.0));

        // Render (requires egui context)
        let ctx = egui::Context::default();
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let result = view.render_body(ui, &mut node);
                assert!(result.is_ok());
            });
        });
    }

    #[test]
    fn test_custom_body_handles_missing_input() {
        let view = MyCustomBodyView;
        let mut node = GraphNode::new(/*...*/); // No inputs

        let ctx = egui::Context::default();
        ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let result = view.render_body(ui, &mut node);

                // Should return error, not panic
                assert!(result.is_err());
                assert!(result.unwrap_err().contains("Missing input"));
            });
        });
    }
}
```

### Integration Test: Four-Section Rendering

```rust
#[test]
fn test_node_renders_with_four_sections() {
    let mut graph = NodeGraph::new();
    let registry = create_test_registry();

    // Add node
    let spec = registry.get_by_id("builtin:math:add").unwrap();
    let node = spec.create_node(egui::Pos2::new(100.0, 100.0));
    graph.add_node_instance(node);

    // Render canvas
    let mut canvas = NodeCanvas::new();
    canvas.sync_with_graph(&graph);

    // Verify sections rendered (requires screenshot comparison or UI testing framework)
    // 1. Header: Node name
    // 2. Connections: Pins visible
    // 3. Body: Input fields for a, b
    // 4. Footer: "(no values computed yet)"
}
```

---

## Troubleshooting

### Issue: "Default body view not showing input fields"

**Possible Causes**:
1. Component has custom `body_view` registered (check with `has_body_view()`)
2. Input ports have complex types (List, Record) - default view shows placeholder
3. Node has no input ports (`node.inputs.is_empty()`)

**Solution**: Add debug logging to see which path is taken:

```rust
fn show_body(...) {
    if let Some(spec) = registry.get_by_id(&node.component_id) {
        if spec.has_body_view() {
            log::debug!("Using custom body view for {}", node.component_id);
        } else {
            log::debug!("Using default body view for {} ({} inputs)",
                node.component_id, node.inputs.len());
        }
    }
}
```

### Issue: "Custom body view not rendering"

**Checklist**:
- ✅ Implemented `ComponentBodyView` trait?
- ✅ Registered with `.with_body_view(Arc::new(MyView))`?
- ✅ `has_body_view()` returns `true`?
- ✅ No errors returned from `render_body()`?

**Debug**:
```rust
// Check registration
if let Some(spec) = registry.get_by_id("my:component") {
    println!("Has body view: {}", spec.has_body_view()); // Should be true
    if let Some(view) = spec.get_body_view() {
        println!("View registered: yes");
    }
}
```

### Issue: "Performance warning: Slow body view rendering"

**Diagnosis**: Render time exceeding 50ms

**Solutions**:
1. Reduce widget count (use collapsible sections)
2. Cache expensive computations (don't recalculate every frame)
3. Profile with `cargo flamegraph` to find hotspots
4. Consider pagination for large parameter lists

---

## Summary

### Key Takeaways

1. **Default views are automatic** - No code needed for basic components
2. **Custom body views** - Use for specialized widgets or validation
3. **Custom footer views** - Use for rich status displays
4. **Mix and match** - Can use default body with custom footer (or vice versa)
5. **Performance matters** - Keep render time under 50ms

### Next Steps

- **Read contracts**: See `contracts/component-body-view.md` for full API reference
- **See examples**: Check `src/builtin/views.rs` for real implementations
- **Migrate constants**: Update existing constant nodes to use `ComponentBodyView`
- **Write tests**: Add tests for your custom views

### Quick Reference

| Feature | File | Key Type |
|---------|------|----------|
| Custom body view trait | `src/ui/component_view.rs` | `ComponentBodyView` |
| Custom footer view trait | `src/ui/component_view.rs` | `ComponentFooterView` |
| Default body rendering | `src/ui/canvas.rs` | `DefaultBodyView` |
| Default footer rendering | `src/ui/canvas.rs` | `DefaultFooterView` |
| Component registration | `src/graph/node.rs` | `ComponentSpec::with_body_view()` |

---

**Questions?** See `research.md` for design decisions and `data-model.md` for architecture details.
