# Quickstart: Component-Driven Custom UI Views

**Feature**: Component-Driven Custom UI Views
**Branch**: `003-ui-customize-currently`
**Date**: 2025-10-15

## Overview

This guide shows you how to add custom footer views to your components using the new `ComponentFooterView` trait. After this refactoring, components can provide their own UI rendering logic instead of hardcoding it in the canvas.

## For Component Developers: Adding a Custom Footer View

### Step 1: Define Your View Struct

Create a zero-sized struct that will implement the view logic:

```rust
// In src/builtin/my_component.rs (or wherever your component lives)

use crate::ui::component_view::ComponentFooterView;
use crate::graph::node::GraphNode;

pub struct MyComponentFooterView;
```

### Step 2: Implement the ComponentFooterView Trait

```rust
impl ComponentFooterView for MyComponentFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
        // Example: Display all output values in a grid
        egui::Grid::new(format!("footer_{}", node.id))
            .num_columns(2)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for output in &node.outputs {
                    if let Some(value) = &output.current_value {
                        ui.label(format!("{}:", output.name));
                        ui.label(value.format_display());
                        ui.end_row();
                    }
                }
            });

        Ok(())
    }
}
```

**Key Points**:
- Access node state via `node.outputs`, `node.inputs`, etc.
- Use egui widgets for rendering: `ui.label()`, `ui.button()`, etc.
- Return `Ok(())` on success, `Err("message")` on failure
- Keep it lightweight - this runs every frame!

### Step 3: Register Your Component with the View

```rust
// In your component registration code (e.g., src/builtin/mod.rs)

use std::sync::Arc;

pub fn register_my_component(registry: &mut ComponentRegistry) {
    // Create the view
    let view = Arc::new(MyComponentFooterView);

    // Create component spec with the view
    let spec = ComponentSpec::new_builtin(
        "builtin:my_component".to_string(),
        "My Component".to_string(),
        "A component with a custom footer view".to_string(),
        Some("Utilities".to_string()),
    )
    .with_input("input".to_string(), DataType::String, "Input data".to_string())
    .with_output("output".to_string(), DataType::String, "Output data".to_string())
    .with_footer_view(view);  // <-- Attach the view here

    registry.register_builtin(spec);
}
```

### Step 4: Test It

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_view_renders() {
        let view = MyComponentFooterView;
        let mut node = GraphNode::new(
            "builtin:my_component".to_string(),
            "Test Node".to_string(),
            egui::Pos2::ZERO,
        );

        // Add some test data
        node.outputs.push(Port::new(
            "output".to_string(),
            DataType::String,
            PortDirection::Output,
            false,
        ));
        node.outputs[0].current_value = Some(NodeValue::String("Hello".to_string()));

        // Create minimal egui context
        let ctx = egui::Context::default();
        ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let result = view.render_footer(ui, &node);
                assert!(result.is_ok());
            });
        });
    }
}
```

## Example: HTTP Fetch Component (Migration)

Here's how the existing `http_fetch` component is migrated to use the new pattern:

### Before (Hardcoded in canvas.rs)

```rust
// In CanvasViewer::has_footer()
fn has_footer(&mut self, node: &SnarlNodeData) -> bool {
    node.component_id.contains("http_fetch")  // <-- Hardcoded!
}

// In CanvasViewer::show_footer()
fn show_footer(...) {
    if node_data.component_id.contains("http_fetch") {  // <-- Hardcoded!
        // ... render HTTP response details ...
    }
}
```

### After (Trait-Based)

```rust
// In src/builtin/http_fetch.rs

pub struct HttpFetchFooterView;

impl ComponentFooterView for HttpFetchFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
        egui::Grid::new(format!("http_footer_{}", node.id))
            .num_columns(2)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for output in &node.outputs {
                    if let Some(value) = &output.current_value {
                        let value_str = value.format_display();

                        // Truncate long values
                        let display_value = if value_str.len() > 100 {
                            format!("{}...", &value_str[..97])
                        } else {
                            value_str
                        };

                        ui.label(format!("{}:", output.name));
                        ui.label(display_value);
                        ui.end_row();
                    }
                }
            });

        Ok(())
    }
}

// In registration
pub fn register_http_fetch(registry: &mut ComponentRegistry) {
    let view = Arc::new(HttpFetchFooterView);

    let spec = ComponentSpec::new_builtin(...)
        .with_footer_view(view);

    registry.register_builtin(spec);
}
```

```rust
// In canvas.rs - now generic!

fn has_footer(&mut self, node: &SnarlNodeData) -> bool {
    self.registry
        .get_by_id(&node.component_id)
        .and_then(|spec| spec.get_footer_view())
        .is_some()  // <-- Generic check!
}

fn show_footer(...) {
    if let Some(spec) = self.registry.get_by_id(&node_data.component_id) {
        if let Some(view) = spec.get_footer_view() {
            match view.render_footer(ui, graph_node) {  // <-- Generic dispatch!
                Ok(()) => { /* Success */ }
                Err(err) => {
                    ui.colored_label(egui::Color32::RED, "⚠️ View render failed");
                    ui.label(err);
                }
            }
        }
    }
}
```

## Advanced: Interactive Controls

You can add interactive controls (buttons, inputs) in your footer view:

```rust
impl ComponentFooterView for InteractiveFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
        ui.label("Current value:");

        // Display output value
        if let Some(output) = node.outputs.get(0) {
            if let Some(value) = &output.current_value {
                ui.label(value.format_display());
            }
        }

        // NOTE: To make controls truly interactive, you need mutable access to GraphNode
        // Current design gives read-only access. For mutation, you'd need to:
        // 1. Change trait signature to accept &mut GraphNode, OR
        // 2. Use event callbacks that canvas handles
        //
        // For P1 (this feature), we focus on read-only display views.

        Ok(())
    }
}
```

## Error Handling Best Practices

### Good: Return Err with User-Friendly Message

```rust
fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
    let output = node.outputs.get(0)
        .ok_or("Component has no outputs to display")?;

    let value = output.current_value.as_ref()
        .ok_or("No data available yet - execute the node first")?;

    ui.label(value.format_display());
    Ok(())
}
```

### Bad: Panic

```rust
fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
    let output = &node.outputs[0];  // ❌ PANICS if no outputs!
    ui.label(output.current_value.as_ref().unwrap().format_display());  // ❌ PANICS if None!
    Ok(())
}
```

## Performance Tips

### DO: Keep Views Lightweight

```rust
fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
    // ✅ Read from in-memory state
    for output in &node.outputs {
        ui.label(output.name.clone());
    }
    Ok(())
}
```

### DON'T: Perform Heavy Computation or I/O

```rust
fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
    // ❌ BLOCKS UI THREAD - Don't do this!
    let data = std::fs::read_to_string("/path/to/file")?;
    ui.label(data);

    // ❌ SLOW - Don't do this every frame!
    let expensive_result = some_heavy_computation();
    ui.label(expensive_result);

    Ok(())
}
```

**Alternative**: Cache expensive results in `GraphNode.outputs[].current_value` during node execution, then just display them in the view.

## Troubleshooting

### Footer Not Showing Up

**Problem**: You implemented the trait but footer doesn't appear.

**Solution Checklist**:
1. Did you call `.with_footer_view(view)` when creating the ComponentSpec?
2. Did you wrap the view in `Arc::new()`?
3. Did you register the component with the registry?
4. Is the node selected on the canvas? (Footer only shows when selected)

### Error: "View render failed"

**Problem**: Red warning message appears in footer.

**Debugging**:
1. Check the error message displayed below the warning
2. Verify node has the expected outputs/inputs
3. Check for None values when you expect Some
4. Add more defensive checks (use `ok_or()` instead of `unwrap()`)

### Footer Is Slow / Laggy

**Problem**: UI feels sluggish when node is selected.

**Solution**:
1. Profile the `render_footer` method - should be <50ms
2. Remove any I/O operations (file reads, network calls)
3. Cache computed values in node.outputs instead of recomputing
4. Simplify UI (fewer widgets, less text)

## Next Steps

- See [data-model.md](data-model.md) for detailed trait and struct definitions
- See [contracts/component_footer_view_trait.md](contracts/component_footer_view_trait.md) for the full contract
- See [research.md](research.md) for architectural decisions and alternatives

## Summary

**To add a custom footer view**:

1. Create a zero-sized struct (e.g., `pub struct MyFooterView;`)
2. Implement `ComponentFooterView` trait with `render_footer()` method
3. Use `.with_footer_view(Arc::new(MyFooterView))` when creating ComponentSpec
4. Register the component - footer will automatically appear when node is selected!

The canvas handles everything else - no need to modify canvas code for each new component type!
