# Data Model: Component-Driven Custom UI Views

**Feature**: Component-Driven Custom UI Views
**Branch**: `003-ui-customize-currently`
**Date**: 2025-10-15

## Overview

This feature introduces a trait-based interface for components to provide custom footer UI rendering. The data model extends the existing `ComponentSpec` structure to optionally hold a view implementation.

## Core Entities

### ComponentFooterView (NEW)

**Purpose**: Trait that components can implement to provide custom footer UI rendering.

**Trait Definition**:

```rust
pub trait ComponentFooterView: Send + Sync {
    /// Render custom footer content for this component
    ///
    /// # Parameters
    /// - `ui`: egui UI context for rendering widgets
    /// - `node`: The graph node being displayed (read-only access to state)
    ///
    /// # Returns
    /// - `Ok(())` if rendering succeeded
    /// - `Err(message)` if rendering failed (error will be displayed in footer)
    ///
    /// # Performance
    /// This method is called every frame while the node is selected.
    /// Keep rendering logic lightweight (<50ms target).
    fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String>;
}
```

**Properties**:
- **Send + Sync**: Required for thread safety (egui can be used in multi-threaded contexts)
- **Stateless**: View implementation should be stateless; state lives in GraphNode
- **Error Handling**: Returns Result to allow graceful error handling

**Lifecycle**:
1. Created when ComponentSpec is registered
2. Stored as `Arc<dyn ComponentFooterView>` in ComponentSpec
3. Retrieved by canvas when rendering footer
4. Called every frame while node is selected

**Relationships**:
- Implemented by component-specific view types (e.g., `HttpFetchFooterView`)
- Stored in `ComponentSpec.footer_view`
- Invoked by `CanvasViewer.show_footer()`

---

### ComponentSpec (MODIFIED)

**Purpose**: Specification of a component's behavior, interface, and UI rendering.

**New Field**:

```rust
pub struct ComponentSpec {
    // ... existing fields ...

    /// Optional custom footer view implementation
    pub footer_view: Option<Arc<dyn ComponentFooterView>>,
}
```

**Changes**:
- Added `footer_view` field to store optional view implementation
- Modified `new_builtin()` and `new_user_defined()` constructors to initialize `footer_view` as `None`
- Added `with_footer_view()` builder method to set custom view

**Builder Method**:

```rust
impl ComponentSpec {
    /// Set custom footer view for this component
    pub fn with_footer_view(mut self, view: Arc<dyn ComponentFooterView>) -> Self {
        self.footer_view = Some(view);
        self
    }

    /// Check if this component has a custom footer view
    pub fn has_footer_view(&self) -> bool {
        self.footer_view.is_some()
    }

    /// Get footer view if available
    pub fn get_footer_view(&self) -> Option<&Arc<dyn ComponentFooterView>> {
        self.footer_view.as_ref()
    }
}
```

**Validation Rules**:
- `footer_view` is always optional (Some or None)
- View implementation must be Send + Sync
- View must not hold mutable state (state lives in GraphNode)

**Serialization**:
- `footer_view` is not serialized (UI-only concern, not part of graph data)
- Add `#[serde(skip)]` attribute to footer_view field
- Components re-register their views when loaded from ComponentRegistry

---

### CanvasViewer (MODIFIED)

**Purpose**: Implements egui-snarl's SnarlViewer trait to render nodes and their custom views.

**Modified Methods**:

```rust
impl SnarlViewer for CanvasViewer {
    fn has_footer(&mut self, node: &SnarlNodeData) -> bool {
        // Check if component has custom footer view
        self.registry
            .get_by_id(&node.component_id)
            .and_then(|spec| spec.get_footer_view())
            .is_some()
    }

    fn show_footer(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<SnarlNodeData>,
    ) {
        if let Some(node_data) = snarl.get_node(node) {
            let node_uuid = node_data.uuid;

            // Get component spec and footer view
            if let Some(spec) = self.registry.get_by_id(&node_data.component_id) {
                if let Some(view) = spec.get_footer_view() {
                    // Render separator
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.set_max_width(250.0);
                        ui.add(egui::Separator::default().horizontal());
                    });
                    ui.add_space(6.0);

                    // Render custom view within scoped width
                    ui.scope(|ui| {
                        ui.set_max_width(250.0);

                        if let Some(graph_node) = self.graph.nodes.get(&node_uuid) {
                            // Call component's custom view
                            match view.render_footer(ui, graph_node) {
                                Ok(()) => { /* Success */ }
                                Err(err) => {
                                    // Display error in footer
                                    ui.colored_label(egui::Color32::RED, "⚠️ View render failed");
                                    ui.label(err);
                                }
                            }
                        }
                    });

                    ui.add_space(6.0);
                }
            }
        }
    }
}
```

**Changes**:
- `has_footer()`: Changed from hardcoded component_id check to trait-based check
- `show_footer()`: Changed from hardcoded rendering to trait-based dispatch
- Added error handling for view rendering failures

---

### HttpFetchFooterView (NEW)

**Purpose**: Example implementation of ComponentFooterView for the http_fetch component.

**Implementation**:

```rust
pub struct HttpFetchFooterView;

impl ComponentFooterView for HttpFetchFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
        egui::Grid::new(format!("http_footer_{}", node.id))
            .num_columns(2)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                // Display each output in a clean format
                for output_port in &node.outputs {
                    if let Some(value) = &output_port.current_value {
                        let value_str = value.format_display();

                        // Truncate long values for better display
                        let display_value = if value_str.len() > 100 {
                            format!("{}...", &value_str[..97])
                        } else {
                            value_str
                        };

                        ui.label(format!("{}:", output_port.name));
                        ui.label(display_value);
                        ui.end_row();
                    }
                }
            });

        Ok(())
    }
}
```

**Properties**:
- Zero-sized struct (no state)
- Reads node output ports to display HTTP response data
- Returns Result for error handling

---

## Entity Relationships

```
ComponentSpec
    ├── id: String
    ├── footer_view: Option<Arc<dyn ComponentFooterView>>  [NEW]
    └── ... other fields

ComponentFooterView (trait)
    └── render_footer(&self, ui, node) -> Result<(), String>

    Implementations:
        ├── HttpFetchFooterView
        └── ... (future custom views)

CanvasViewer
    ├── registry: &ComponentRegistry
    └── Methods:
        ├── has_footer() -> checks registry.get_by_id().has_footer_view()
        └── show_footer() -> calls view.render_footer()

GraphNode
    └── (unchanged - provides read-only state to views)
```

## Data Flow

### Initialization (Component Registration)

1. Create view implementation: `let view = Arc::new(HttpFetchFooterView);`
2. Create ComponentSpec with view: `spec.with_footer_view(view)`
3. Register in ComponentRegistry: `registry.register_builtin(spec)`

### Runtime (Footer Rendering)

1. User selects node on canvas
2. egui-snarl calls `CanvasViewer::has_footer(node_data)`
3. CanvasViewer checks: `registry.get_by_id(component_id)?.has_footer_view()`
4. If true, egui-snarl calls `CanvasViewer::show_footer()`
5. CanvasViewer retrieves view: `spec.get_footer_view()`
6. CanvasViewer calls: `view.render_footer(ui, graph_node)`
7. View renders UI widgets into egui context
8. If view returns Err, error message displayed in footer

## State Management

**Key Principle**: Views are stateless; all state lives in `GraphNode`.

- **Component State**: Stored in `GraphNode.outputs[].current_value`
- **View Logic**: Reads state from GraphNode parameter
- **Reactivity**: egui's immediate mode handles updates automatically (re-renders every frame)
- **No Persistence**: View implementations are not serialized (recreated on load)

## Validation Rules

1. **ComponentSpec Validation**:
   - `footer_view` field is optional (None is valid)
   - If Some, view must implement Send + Sync
   - View must not panic during render (return Err instead)

2. **View Rendering Validation**:
   - Render time should be <50ms (logged if exceeded)
   - View must not block UI thread (no network/disk I/O)
   - View must handle missing/invalid node state gracefully

3. **Error Handling**:
   - View errors (Result::Err) displayed to user, not logged only
   - View panics should not crash application (but not caught - rely on Result)
   - Missing node data returns early (no render, no error)

## Migration Impact

**Existing Code Changes**:

1. `src/graph/node.rs`:
   - Add `footer_view: Option<Arc<dyn ComponentFooterView>>` field to ComponentSpec
   - Add `#[serde(skip)]` attribute
   - Add `with_footer_view()`, `has_footer_view()`, `get_footer_view()` methods

2. `src/ui/canvas.rs`:
   - Remove hardcoded `component_id.contains("http_fetch")` checks (lines 604, 636)
   - Replace with `spec.has_footer_view()` and `view.render_footer()` calls
   - Add error handling for view rendering failures

3. `src/builtin/http_fetch.rs` (or wherever http_fetch is defined):
   - Create HttpFetchFooterView struct
   - Implement ComponentFooterView trait
   - Update component registration to include `.with_footer_view(Arc::new(HttpFetchFooterView))`

**No Breaking Changes**:
- Existing graphs serialize/deserialize without changes (footer_view skipped)
- Components without custom views continue to work (footer_view = None)
- ComponentSpec API remains backward compatible (with_footer_view is additive)
