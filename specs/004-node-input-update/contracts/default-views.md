# Contract: Default Views (Body & Footer)

**Feature**: `004-node-input-update`
**Date**: 2025-10-16
**Type**: Auto-Generated UI Specification

## Overview

Default views are auto-generated UI sections that appear when components do not provide custom `ComponentBodyView` or `ComponentFooterView` implementations. They provide immediate usability for all nodes without requiring component developers to write custom UI code.

---

## Default Body View

### Purpose
Generate interactive input fields for all node parameters (input ports) based on their data types.

### Trigger Condition
```rust
if !component_spec.has_body_view() {
    // Render default body view
    DefaultBodyView::render_for_node(ui, node)?;
}
```

### Rendering Rules

#### 1. Input Port Iteration
```rust
for input_port in &mut node.inputs {
    ui.horizontal(|ui| {
        // Label: Port name
        ui.label(&input_port.name);

        // Widget: Based on data type
        match input_port.data_type {
            DataType::U32 => { render_u32_input(ui, input_port); }
            DataType::I32 => { render_i32_input(ui, input_port); }
            DataType::F32 => { render_f32_input(ui, input_port); }
            DataType::String => { render_string_input(ui, input_port); }
            DataType::List(_) | DataType::Record(_) | DataType::Binary => {
                ui.label("(complex type - use custom view)");
            }
            DataType::Any => {
                ui.label("(any type - no default editor)");
            }
        }
    });
}
```

#### 2. Widget Specifications

##### U32 (Unsigned 32-bit Integer)
```rust
fn render_u32_input(ui: &mut egui::Ui, port: &mut Port) {
    if let Some(NodeValue::U32(ref mut value)) = port.current_value {
        if ui.add(egui::DragValue::new(value).speed(1.0)).changed() {
            // Mark node dirty for re-execution
            // (handled at node level, not in this function)
        }
    } else {
        // No current value - initialize with default
        port.current_value = Some(NodeValue::U32(0));
    }
}
```

**Widget**: `egui::DragValue`
**Constraints**: `0..=u32::MAX`
**Default Value**: `0`
**Speed**: `1.0` (integer increments)

##### I32 (Signed 32-bit Integer)
```rust
fn render_i32_input(ui: &mut egui::Ui, port: &mut Port) {
    if let Some(NodeValue::I32(ref mut value)) = port.current_value {
        if ui.add(egui::DragValue::new(value).speed(1.0)).changed() {
            // Mark dirty
        }
    } else {
        port.current_value = Some(NodeValue::I32(0));
    }
}
```

**Widget**: `egui::DragValue`
**Constraints**: `i32::MIN..=i32::MAX`
**Default Value**: `0`
**Speed**: `1.0`

##### F32 (32-bit Float)
```rust
fn render_f32_input(ui: &mut egui::Ui, port: &mut Port) {
    if let Some(NodeValue::F32(ref mut value)) = port.current_value {
        if ui.add(egui::DragValue::new(value).speed(0.1)).changed() {
            // Mark dirty
        }
    } else {
        port.current_value = Some(NodeValue::F32(0.0));
    }
}
```

**Widget**: `egui::DragValue`
**Constraints**: `f32::MIN..=f32::MAX`
**Default Value**: `0.0`
**Speed**: `0.1` (decimal increments)

##### String
```rust
fn render_string_input(ui: &mut egui::Ui, port: &mut Port) {
    if let Some(NodeValue::String(ref mut value)) = port.current_value {
        if ui.add(egui::TextEdit::singleline(value).desired_width(150.0)).changed() {
            // Mark dirty
        }
    } else {
        port.current_value = Some(NodeValue::String(String::new()));
    }
}
```

**Widget**: `egui::TextEdit::singleline`
**Constraints**: None (arbitrary UTF-8)
**Default Value**: `""` (empty string)
**Width**: `150.0` px

##### Complex Types (List, Record, Binary)
```rust
// Placeholder text only - no interactive editor
ui.label("(complex type - use custom view)");
ui.small("Lists, records, and binary data require custom editors");
```

**Rationale**: Complex types need structured editors (array grids, key-value tables, hex viewers) that are component-specific. Default view would be too generic to be useful.

##### Any Type
```rust
ui.label("(any type - no default editor)");
```

**Rationale**: "Any" type is used for passthrough/logger nodes where input editing doesn't make sense (values come from connections only).

---

### Layout Constraints

```rust
ui.scope(|ui| {
    ui.set_max_width(250.0); // Match node width
    ui.style_mut().spacing.item_spacing.y = 4.0; // Compact vertical spacing

    // Render input fields (max 20 to stay under performance budget)
    let field_limit = 20;
    for (idx, input_port) in node.inputs.iter_mut().enumerate() {
        if idx >= field_limit {
            ui.label(format!("... and {} more fields", node.inputs.len() - field_limit));
            ui.small("(too many parameters - consider custom view)");
            break;
        }

        // Render field
        render_input_field(ui, input_port);
    }
});
```

**Max Width**: 250px (snarl node constraint)
**Max Fields**: 20 (performance limit from spec edge cases)
**Vertical Spacing**: 4px between fields
**Overflow Behavior**: Show "... and N more" message

---

### Dirty Flag Handling

When user edits a field:
```rust
if widget.changed() {
    // Get mutable reference to node (passed from show_body caller)
    node.dirty = true;

    // Update port value (already done via &mut binding)
    // input_port.current_value is automatically updated by widget

    // Execution engine will detect dirty=true and re-run node
}
```

**Trigger**: Widget returns `changed()` response
**Effect**: Set `node.dirty = true` to queue re-execution
**Value Update**: Widget modifies `input_port.current_value` directly (mutable reference)

---

### Empty State

If node has **no input ports**:
```rust
fn render_for_node(ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
    if node.inputs.is_empty() {
        ui.label("(no parameters)");
        ui.small("This node has no configurable inputs");
        return Ok(());
    }

    // ... render input fields ...
}
```

**Display**: Gray placeholder text
**Vertical Space**: Minimal (single line)

---

## Default Footer View

### Purpose
Display current output values for all node outputs based on execution results.

### Trigger Condition
```rust
if !component_spec.has_footer_view() {
    // Render default footer view
    DefaultFooterView::render_for_node(ui, node)?;
}
```

### Rendering Rules

#### 1. Output Port Iteration
```rust
for output_port in &node.outputs {
    if let Some(value) = &output_port.current_value {
        ui.horizontal(|ui| {
            ui.label(format!("Current {}: {}",
                output_port.name,
                value.format_display()));
        });
    }
}
```

**Filter**: Only show outputs with `current_value.is_some()`
**Format**: `"Current {port_name}: {formatted_value}"`
**Formatter**: Use `NodeValue::format_display()` for type-appropriate formatting

#### 2. Value Formatting (NodeValue::format_display)

```rust
impl NodeValue {
    pub fn format_display(&self) -> String {
        match self {
            NodeValue::U32(v) => format!("{}", v),          // "42"
            NodeValue::I32(v) => format!("{}", v),          // "-123"
            NodeValue::F32(v) => format!("{:.2}", v),       // "3.14"
            NodeValue::String(s) => format!("\"{}\"", s),   // "\"hello\""
            NodeValue::Binary(b) => format!("<{} bytes>", b.len()), // "<1024 bytes>"
            NodeValue::List(items) => format!("[{} items]", items.len()), // "[5 items]"
            NodeValue::Record(fields) => format!("{{{} fields}}", fields.len()), // "{3 fields}"
        }
    }
}
```

**U32/I32**: Plain integer (`42`, `-123`)
**F32**: Two decimal places (`3.14`, `0.00`)
**String**: Quoted (`"hello world"`)
**Binary**: Byte count (`<1024 bytes>`)
**List**: Item count (`[5 items]`)
**Record**: Field count (`{3 fields}`)

---

### Layout Constraints

```rust
ui.scope(|ui| {
    ui.set_max_width(250.0); // Match node width
    ui.style_mut().spacing.item_spacing.y = 4.0;

    // Render output values (max 10 to stay under performance budget)
    let value_limit = 10;
    let mut shown_count = 0;

    for output_port in &node.outputs {
        if let Some(value) = &output_port.current_value {
            if shown_count >= value_limit {
                ui.label(format!("... and {} more outputs", node.outputs.len() - shown_count));
                break;
            }

            ui.horizontal(|ui| {
                ui.label(format!("Current {}: {}", output_port.name, value.format_display()));
            });

            shown_count += 1;
        }
    }

    // Show placeholder if no outputs computed yet
    if shown_count == 0 {
        ui.label("(no values computed yet)");
        ui.small("Run execution to see output values");
    }
});
```

**Max Width**: 250px
**Max Outputs**: 10 (performance limit from spec edge cases)
**Vertical Spacing**: 4px
**Empty State**: Show "(no values computed yet)" if all outputs are None

---

### Execution State Awareness

```rust
match node.execution_state {
    ExecutionState::Idle => {
        ui.label("(awaiting execution)");
    }
    ExecutionState::Running => {
        ui.label("⏳ Computing...");
    }
    ExecutionState::Completed => {
        // Show output values (normal rendering)
        for output in &node.outputs { /* ... */ }
    }
    ExecutionState::Failed => {
        ui.colored_label(egui::Color32::RED, "❌ Execution failed");
        ui.small("Check node connections and inputs");
    }
}
```

**Idle**: Show "(awaiting execution)"
**Running**: Show "⏳ Computing..." spinner
**Completed**: Show output values (main path)
**Failed**: Show red error message

---

### Empty State

If node has **no output ports**:
```rust
if node.outputs.is_empty() {
    ui.label("(no outputs)");
    ui.small("This node does not produce output values");
    return Ok(());
}
```

**Display**: Gray placeholder
**Use Case**: Sink nodes (e.g., file writers, loggers)

---

## Performance Specifications

### Render Time Budget

| Section | Target | Limit | Measurement |
|---------|--------|-------|-------------|
| Default Body (per node) | <10ms | <50ms | 20 input fields × 0.5ms/widget |
| Default Footer (per node) | <5ms | <50ms | 10 output labels × 0.5ms/label |
| Combined (worst case) | <15ms | <50ms | Body + Footer sequential |

### Field Limits (From Spec Edge Cases)

| Limit | Value | Spec Reference |
|-------|-------|----------------|
| Max default body fields | 20 | Spec edge case: "many parameters" |
| Max default footer outputs | 10 | Spec edge case: "many parameters" |
| Total widgets per node | 30 | 20 body + 10 footer |

### Measurement Implementation

```rust
fn render_for_node(ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
    let start_time = std::time::Instant::now();

    // ... render fields ...

    let elapsed = start_time.elapsed();
    if elapsed.as_millis() > 50 {
        log::warn!(
            "Slow default body view for '{}': {}ms (target: <10ms, limit: <50ms)",
            node.display_name,
            elapsed.as_millis()
        );
    } else {
        log::trace!(
            "Default body rendered for '{}' in {}ms",
            node.display_name,
            elapsed.as_millis()
        );
    }

    Ok(())
}
```

**Trace Log**: <50ms (expected case)
**Warning Log**: ≥50ms (performance issue)

---

## Accessibility & UX

### Visual Hierarchy

```
┌─────────────────────────┐
│   Node Name        [✖]  │ ← Header (unchanged)
├─────────────────────────┤
│o port1:u32       out1:u32 o │ ← Connections (pin area)
│o port2:f32              │
├─────────────────────────┤
│port1: [DragValue: 42]▼  │ ← Default Body (this spec)
│port2: [DragValue: 3.14]▼│   White background, interactive
├─────────────────────────┤
│Current out1: 45.14      │ ← Default Footer (this spec)
└─────────────────────────┘   Gray background, read-only
```

**Body Section**: White background, interactive widgets
**Footer Section**: Light gray background, read-only labels
**Separation**: Horizontal separator line (egui::Separator)

### Color Coding

- **Labels**: Default text color (white/black based on theme)
- **Editable Values**: Highlighted on hover (egui default)
- **Read-Only Values**: Muted gray (footer outputs)
- **Error States**: Red text (`egui::Color32::RED`)
- **Placeholders**: Light gray (`ui.small()` style)

### Tooltips

```rust
ui.horizontal(|ui| {
    ui.label(&input_port.name)
        .on_hover_text(format!("Type: {}", input_port.data_type.name()));

    // Widget...
});
```

**Hover Target**: Port name label
**Content**: Data type name (e.g., "Type: u32")

---

## Testing Specifications

### Unit Tests

```rust
#[test]
fn test_default_body_renders_all_simple_types() {
    let mut node = create_test_node_with_inputs(&[
        ("u32_param", DataType::U32),
        ("i32_param", DataType::I32),
        ("f32_param", DataType::F32),
        ("str_param", DataType::String),
    ]);

    let result = DefaultBodyView::render_for_node(&mut ui, &mut node);

    assert!(result.is_ok());
    // Verify all 4 widgets rendered (requires egui test harness)
}

#[test]
fn test_default_body_limits_field_count() {
    let inputs: Vec<_> = (0..30)
        .map(|i| (format!("param_{}", i), DataType::U32))
        .collect();

    let mut node = create_test_node_with_inputs(&inputs);

    let result = DefaultBodyView::render_for_node(&mut ui, &mut node);

    assert!(result.is_ok());
    // Verify only 20 widgets + overflow message shown
}

#[test]
fn test_default_footer_shows_computed_values() {
    let mut node = create_test_node_with_outputs(&[
        ("out1", DataType::U32, Some(NodeValue::U32(42))),
        ("out2", DataType::F32, Some(NodeValue::F32(3.14))),
    ]);

    let result = DefaultFooterView::render_for_node(&mut ui, &node);

    assert!(result.is_ok());
    // Verify "Current out1: 42" and "Current out2: 3.14" shown
}

#[test]
fn test_default_footer_shows_placeholder_when_no_values() {
    let mut node = create_test_node_with_outputs(&[
        ("out1", DataType::U32, None), // No current value
    ]);

    let result = DefaultFooterView::render_for_node(&mut ui, &node);

    assert!(result.is_ok());
    // Verify "(no values computed yet)" shown
}
```

### Integration Tests

```rust
#[test]
fn test_four_section_layout_with_defaults() {
    let mut graph = NodeGraph::new();
    let registry = create_registry_without_custom_views();

    // Add node with inputs and outputs
    let node_id = graph.add_node(/* ... */);

    // Render canvas
    let mut canvas = NodeCanvas::new();
    canvas.sync_with_graph(&graph);

    // Verify:
    // 1. Header shows node name
    // 2. Connections show ports
    // 3. Body shows default input fields
    // 4. Footer shows "(no values computed yet)"

    // Execute graph
    execute_graph(&mut graph);

    // Verify:
    // 5. Footer now shows "Current out: X"
}
```

---

## Migration & Backward Compatibility

### Existing Nodes Without Custom Views

**Before** (three-section):
- Header: Node name
- Connections: Inline with pins
- Body: Empty OR constant editing (hardcoded)
- Footer: Empty OR custom view

**After** (four-section with defaults):
- Header: Node name (unchanged)
- Connections: Dedicated section with pins
- Body: **Default input fields** (NEW - automatic improvement)
- Footer: **Default output values** (NEW) OR custom view (unchanged)

**Impact**: Existing nodes become more usable without code changes

### Constant Nodes Migration

**Before** (hardcoded in `show_body`):
```rust
fn show_body(...) {
    if node.component_id.starts_with("builtin:constant:") {
        // Hardcoded value editing
        match value {
            F32(v) => { ui.add(TextEdit::singleline(&mut text)); }
            // ...
        }
    }
}
```

**After** (via ComponentBodyView):
```rust
struct ConstantBodyView;
impl ComponentBodyView for ConstantBodyView {
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Same editing logic, now in trait implementation
        // ...
    }
}

// Register
ComponentSpec::new_builtin("builtin:constant:f32", ...)
    .with_body_view(Arc::new(ConstantBodyView));
```

**Migration Path**:
1. Move constant editing from hardcoded `show_body` to `ComponentBodyView`
2. Register view with `with_body_view()`
3. Remove hardcoded check in `show_body`

**Backward Compatibility**: Existing graphs load and work identically

---

## Summary

### Default Body View

**Purpose**: Auto-generate input fields for node parameters
**Trigger**: `!component_spec.has_body_view()`
**Supported Types**: U32, I32, F32, String (via DragValue/TextEdit)
**Unsupported Types**: List, Record, Binary (show placeholder)
**Performance**: <10ms for 20 fields
**Layout**: 250px width, 4px vertical spacing, max 20 fields

### Default Footer View

**Purpose**: Display current output values
**Trigger**: `!component_spec.has_footer_view()`
**Display Format**: `"Current {port_name}: {value}"`
**Performance**: <5ms for 10 outputs
**Layout**: 250px width, 4px vertical spacing, max 10 outputs
**Empty State**: "(no values computed yet)"

### Design Principles

1. **Immediate Usability**: All nodes are interactive by default
2. **Type Safety**: Widget selection based on DataType
3. **Performance Bounds**: Hard limits on field count (20/10)
4. **Graceful Degradation**: Complex types show placeholders
5. **Opt-Out Pattern**: Custom views override defaults completely

### Files Affected

- `src/ui/canvas.rs`: Implement `DefaultBodyView` and `DefaultFooterView` structs
- `src/ui/component_view.rs`: Document default view behavior in trait docs
- `tests/unit/component_view_tests.rs`: Add tests for default view rendering

### Next Steps

See `quickstart.md` for developer guide on using default views vs custom views.
