# Data Model: Four-Section Node Layout

**Feature**: `004-node-input-update`
**Date**: 2025-10-16
**Phase**: 1 (Design & Contracts)

## Overview

This feature primarily extends the **UI presentation layer** without modifying the core graph data model. The existing `GraphNode`, `Port`, and `ComponentSpec` structures remain unchanged. New entities are introduced in the **UI domain** for view customization.

---

## Core Entities (Unchanged)

### GraphNode
**Location**: `src/graph/node.rs:155-225`
**Purpose**: Represents a computational unit in the visual programming graph

```rust
pub struct GraphNode {
    pub id: Uuid,
    pub component_id: String,
    pub display_name: String,
    pub position: egui::Pos2,
    pub inputs: Vec<Port>,          // → Populates connections + default body
    pub outputs: Vec<Port>,         // → Populates connections + default footer
    pub metadata: NodeMetadata,
    pub capabilities: CapabilitySet,
    pub execution_state: ExecutionState,
    pub dirty: bool,
    pub cached_footer_view: Option<FooterView>,
}
```

**Changes**: None (existing structure sufficient)

**Four-Section Mapping**:
- **Header**: `display_name`
- **Connections**: Derived from `inputs` (left) and `outputs` (right)
- **Body**: Default generated from `inputs`, OR custom view if `ComponentSpec.body_view` exists
- **Footer**: Default generated from `outputs`, OR custom view if `ComponentSpec.footer_view` exists

---

### Port
**Location**: `src/graph/node.rs:92-126`
**Purpose**: Connection point for data flow (input or output)

```rust
pub struct Port {
    pub id: Uuid,
    pub name: String,               // Displayed in connections section
    pub data_type: DataType,        // Displayed with type color in connections
    pub direction: PortDirection,   // Input (left) or Output (right)
    pub optional: bool,
    pub current_value: Option<NodeValue>,  // Displayed in default footer
}
```

**Changes**: None

**Four-Section Usage**:
- **Connections Section**: Shows `name` + `data_type` with color-coded pins
- **Default Body**: Input ports → generate input widgets based on `data_type`
- **Default Footer**: Output ports → show `current_value` if present

---

### ComponentSpec
**Location**: `src/graph/node.rs:266-418`
**Purpose**: Specification of a node's behavior, interface, and custom views

```rust
pub struct ComponentSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub component_type: ComponentType,
    pub input_spec: Vec<PortSpec>,
    pub output_spec: Vec<PortSpec>,
    pub required_capabilities: Vec<String>,
    pub category: Option<String>,

    #[serde(skip)]
    pub footer_view: Option<Arc<dyn ComponentFooterView>>,  // EXISTING

    // NEW: Optional custom body view
    #[serde(skip)]
    pub body_view: Option<Arc<dyn ComponentBodyView>>,      // NEW
}
```

**Changes**:
- **Add** `body_view` field (optional custom body view)
- **Add** `with_body_view()` method (builder pattern)
- **Add** `has_body_view()` method (check if custom body exists)
- **Add** `get_body_view()` method (retrieve custom body view)

**Rationale**: Mirrors existing `footer_view` pattern for consistency

---

## New Entities (UI Domain)

### ComponentBodyView (Trait)
**Location**: `src/ui/component_view.rs` (NEW section)
**Purpose**: Interface for components to provide custom body UI rendering

```rust
/// Trait for components to provide custom body UI rendering.
///
/// Components can implement this trait to display custom interactive content
/// in the body section (between connections and footer). If not provided,
/// a default body view is generated with input fields for all node parameters.
pub trait ComponentBodyView: Send + Sync {
    /// Render custom body content for this component.
    ///
    /// # Parameters
    /// - `ui`: egui UI context for rendering widgets
    /// - `node`: The graph node being displayed (mutable for interactive widgets)
    ///
    /// # Returns
    /// - `Ok(())` if rendering succeeded
    /// - `Err(message)` if rendering failed (error displayed in body)
    ///
    /// # Performance
    /// This method is called every frame (60 FPS). Keep rendering <50ms.
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String>;
}
```

**Responsibilities**:
- Render custom interactive controls (e.g., specialized parameter editors)
- Update `node.inputs` port values directly when user interacts
- Handle validation and error states

**Usage Example** (for a color picker component):
```rust
struct ColorPickerBodyView;

impl ComponentBodyView for ColorPickerBodyView {
    fn render_body(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        if let Some(input) = node.get_input_mut("color") {
            if let Some(NodeValue::U32(ref mut color_u32)) = input.current_value {
                let mut color = Color32::from_rgb(
                    (*color_u32 >> 16) as u8,
                    (*color_u32 >> 8) as u8,
                    *color_u32 as u8
                );
                if ui.color_edit_button_srgba(&mut color).changed() {
                    *color_u32 = ((color.r() as u32) << 16)
                               | ((color.g() as u32) << 8)
                               | (color.b() as u32);
                }
            }
        }
        Ok(())
    }
}
```

---

### DefaultBodyView (Struct)
**Location**: `src/ui/canvas.rs` (NEW helper)
**Purpose**: Auto-generate input fields for nodes without custom body views

```rust
/// Helper to render default body view with input fields for all parameters
struct DefaultBodyView;

impl DefaultBodyView {
    /// Generate input widgets based on port data types
    fn render_for_node(ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        for input_port in &mut node.inputs {
            ui.horizontal(|ui| {
                ui.label(&input_port.name);

                match input_port.data_type {
                    DataType::U32 => { /* DragValue widget */ }
                    DataType::I32 => { /* DragValue widget */ }
                    DataType::F32 => { /* DragValue widget */ }
                    DataType::String => { /* TextEdit widget */ }
                    DataType::List(_) | DataType::Record(_) => {
                        ui.label("(complex type - use custom view)");
                    }
                    _ => { ui.label("(unsupported)"); }
                }
            });
        }
        Ok(())
    }
}
```

**Responsibilities**:
- Match on `input_port.data_type` to emit appropriate widget
- Handle simple types (U32, I32, F32, String) with standard egui widgets
- Show placeholder for complex types (List, Record, Binary)
- Limit to 20 fields max to stay under performance budget

---

### DefaultFooterView (Struct)
**Location**: `src/ui/canvas.rs` (NEW helper)
**Purpose**: Auto-generate status display for nodes without custom footer views

```rust
/// Helper to render default footer view with output value status
struct DefaultFooterView;

impl DefaultFooterView {
    /// Generate status labels based on output port current values
    fn render_for_node(ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
        for output_port in &node.outputs {
            if let Some(value) = &output_port.current_value {
                ui.horizontal(|ui| {
                    ui.label(format!("Current {}: {}",
                        output_port.name,
                        value.format_display()));
                });
            }
        }

        // Show placeholder if no outputs computed yet
        if node.outputs.iter().all(|p| p.current_value.is_none()) {
            ui.label("(no values computed yet)");
        }

        Ok(())
    }
}
```

**Responsibilities**:
- Iterate `output_port.current_value` and format for display
- Use `NodeValue::format_display()` for type-appropriate formatting
- Show "(no values computed yet)" if all outputs are None
- Limit to 10 outputs max for performance

---

## Data Flow

### Rendering Pipeline (Four Sections)

```
GraphNode (src/graph/node.rs)
    ↓
SnarlViewer::show_header() → Display node.display_name + controls
    ↓
SnarlViewer::show_input() → For each node.inputs[i]:
SnarlViewer::show_output() →   Display port.name + port.data_type (with color)
    ↓                          [This is the "connections section"]
SnarlViewer::show_body() → Check ComponentSpec.has_body_view():
    |                          - YES: Call body_view.render_body(ui, node)
    |                          - NO: Call DefaultBodyView::render_for_node(ui, node)
    ↓
SnarlViewer::show_footer() → Check ComponentSpec.has_footer_view():
                               - YES: Call footer_view.render_footer(ui, node)
                               - NO: Call DefaultFooterView::render_for_node(ui, node)
```

### User Interaction Flow (Body Section)

```
User edits input field in default body
    ↓
egui widget detects change
    ↓
Update node.inputs[i].current_value (if input port has editable value)
    OR
Update component-specific state (if custom body view)
    ↓
Mark node.dirty = true (trigger re-execution)
    ↓
Execution engine re-runs node (separate from UI)
    ↓
Output values update → node.outputs[i].current_value
    ↓
Default footer shows new values (next frame)
```

---

## Validation Rules

### Port Display (Connections Section)
- Input ports MUST be shown on the left with `PortDirection::Input`
- Output ports MUST be shown on the right with `PortDirection::Output`
- Port type information MUST be visible (via color + label)
- Maximum 50 input ports + 50 output ports (performance limit)

### Default Body View Generation
- Only generate for ports with `direction == Input`
- Skip ports with complex types (List, Record) or show placeholder
- Limit to 20 input fields per node (edge case from spec)
- Must handle missing `current_value` gracefully (empty/default widgets)

### Default Footer View Generation
- Only show outputs with `current_value.is_some()`
- Limit to 10 output fields per node (edge case from spec)
- Use `NodeValue::format_display()` for consistent formatting
- Show placeholder text if no outputs computed

### Custom View Selection
- If `ComponentSpec.has_body_view() == true`, skip default body generation
- If `ComponentSpec.has_footer_view() == true`, skip default footer generation
- Custom views MUST handle their own error states (return `Err(String)`)
- Custom views MUST complete render in <50ms (performance budget)

---

## State Transitions

### Node Execution State (Unchanged)
The four-section layout displays execution state visually but does not modify state transitions:

```
ExecutionState::Idle
    ↓ (user connects inputs)
ExecutionState::Running
    ↓ (execution completes)
ExecutionState::Completed → outputs populated → default footer shows values
    OR
ExecutionState::Failed → default footer shows error placeholder
```

### View Customization State (New)

```
Component registered without custom views
    ↓
has_body_view() == false, has_footer_view() == false
    ↓
Default views used for body + footer
    ---
Component updated with .with_body_view(Arc::new(CustomView))
    ↓
has_body_view() == true
    ↓
Custom body shown, default footer still used (if footer_view not set)
    ---
Component updated with .with_footer_view(Arc::new(CustomView))
    ↓
has_footer_view() == true
    ↓
Custom footer shown, default body still used (if body_view not set)
```

**Note**: Body and footer customization are **independent** - can mix default and custom

---

## Migration Notes

### Existing Data Compatibility
- **GraphNode serialization format**: Unchanged (backward compatible)
- **Port structure**: Unchanged
- **ComponentSpec**: New `body_view` field is `#[serde(skip)]` (not serialized)

### Existing Custom Footer Views
Components with existing `ComponentFooterView` implementations:
- Continue working without changes
- Default footer skipped when `has_footer_view() == true`
- Example: HTTP Fetch component (src/examples/example-http-fetch/src/lib.rs)

### Constant Nodes Migration
Current constant nodes (src/builtin/constants.rs) with inline editing:
- Move editing logic to `ComponentBodyView` implementation
- Keep existing behavior: edit values in body section
- Gain automatic footer: "Current value: X" shown by default footer

---

## Summary

### No Core Data Model Changes
- `GraphNode`, `Port`, `ComponentSpec`: Structure unchanged (only `ComponentSpec` gains optional `body_view` field)
- Graph serialization format: Unchanged (backward compatible)

### New UI Entities
- `ComponentBodyView` trait: Interface for custom body rendering
- `DefaultBodyView` struct: Auto-generate input fields
- `DefaultFooterView` struct: Auto-generate status display

### Relationships
```
ComponentSpec (1) ----optional----> (0..1) ComponentBodyView
ComponentSpec (1) ----optional----> (0..1) ComponentFooterView
GraphNode (1) --------has--------> (N) Port (inputs)
GraphNode (1) --------has--------> (M) Port (outputs)
Port.data_type -------determines-> Widget type in DefaultBodyView
Port.current_value ---displayed--> DefaultFooterView status
```

### Next Steps
Phase 1 continues with contracts/ documentation for ComponentBodyView trait API.
