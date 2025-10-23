# Contract: ComponentFooterView Trait

**Feature**: Component-Driven Custom UI Views
**Contract Type**: Rust Trait Interface
**Date**: 2025-10-15

## Overview

The `ComponentFooterView` trait defines the interface that components implement to provide custom footer UI rendering in the node canvas. This contract ensures consistent behavior across all custom view implementations.

## Trait Definition

```rust
/// Trait for components to provide custom footer rendering
///
/// Components that implement this trait can display custom UI in the node footer
/// area when the node is selected on the canvas. The footer is rendered using
/// egui widgets within a constrained width area (250px max).
pub trait ComponentFooterView: Send + Sync {
    /// Render custom footer content for this component
    ///
    /// This method is called every frame while the node is selected. Implementations
    /// should be lightweight and avoid blocking operations.
    ///
    /// # Parameters
    ///
    /// * `ui` - egui UI context for rendering widgets. The UI is pre-scoped to 250px max width.
    /// * `node` - The graph node being displayed. Use this to access component state via
    ///            `node.outputs[].current_value` or other node properties.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Rendering succeeded
    /// * `Err(message)` - Rendering failed. The error message will be displayed in the footer
    ///                    with a warning icon.
    ///
    /// # Performance Requirements
    ///
    /// - Target render time: <50ms per frame
    /// - Must not block the UI thread (no network/disk I/O)
    /// - Should handle rapid re-renders (called every frame, ~60 FPS)
    ///
    /// # Error Handling
    ///
    /// - Return `Err` for any rendering failures (missing data, invalid state, etc.)
    /// - Do NOT panic - panics will crash the application
    /// - Provide user-friendly error messages (displayed to end users)
    ///
    /// # Thread Safety
    ///
    /// - Must be Send + Sync (can be shared across threads)
    /// - Implementations should be stateless (state lives in GraphNode)
    ///
    /// # Example Implementation
    ///
    /// ```rust
    /// pub struct MyComponentFooterView;
    ///
    /// impl ComponentFooterView for MyComponentFooterView {
    ///     fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
    ///         // Display node outputs
    ///         for output in &node.outputs {
    ///             if let Some(value) = &output.current_value {
    ///                 ui.label(format!("{}: {}", output.name, value.format_display()));
    ///             }
    ///         }
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String>;
}
```

## Contract Requirements

### Functional Requirements

1. **FR-VIEW-001**: Implementations MUST be Send + Sync
   - **Rationale**: egui can be used in multi-threaded contexts
   - **Verification**: Rust compiler enforces trait bounds

2. **FR-VIEW-002**: Implementations MUST NOT panic during render_footer()
   - **Rationale**: Panics crash the application
   - **Verification**: Return Err instead of panicking; code review
   - **Example**: Instead of `node.outputs[0]` (panics if empty), use `node.outputs.get(0).ok_or("Missing output")?`

3. **FR-VIEW-003**: Implementations MUST complete rendering in <50ms
   - **Rationale**: Maintains 60 FPS UI performance (16ms per frame budget)
   - **Verification**: Performance logging (debug mode), manual testing
   - **Note**: 50ms threshold allows for complex views while staying responsive

4. **FR-VIEW-004**: Implementations MUST NOT perform blocking I/O
   - **Rationale**: Blocks UI thread, freezes application
   - **Verification**: Code review, no use of std::fs, network calls
   - **Alternative**: If I/O needed, cache results and display "Loading..." state

5. **FR-VIEW-005**: Error messages MUST be user-friendly
   - **Rationale**: Errors displayed directly to end users in footer
   - **Verification**: Code review, UX review
   - **Example**: `Err("Failed to render: missing response data")` not `Err("outputs[2] is None")`

6. **FR-VIEW-006**: Implementations MUST be stateless
   - **Rationale**: State lives in GraphNode for proper serialization/undo
   - **Verification**: Code review - no mutable fields in view struct
   - **Pattern**: Use zero-sized structs (e.g., `pub struct MyView;`)

### Non-Functional Requirements

1. **NFR-VIEW-001**: Memory Efficiency
   - View structs should be small (prefer zero-sized structs)
   - Stored as Arc<dyn ComponentFooterView> - minimal overhead

2. **NFR-VIEW-002**: Testability
   - Must be testable without full egui context
   - Can create mock GraphNode for unit testing

3. **NFR-VIEW-003**: Extensibility
   - Trait designed to allow future additions (new methods with default implementations)
   - Example: `fn render_tooltip(&self, ui, node) -> Result<(), String> { Ok(()) }`

## Usage Contract

### Component Registration

```rust
// Create view implementation
let view = Arc::new(MyComponentFooterView);

// Attach to component spec
let spec = ComponentSpec::new_builtin(
    "my:component".to_string(),
    "My Component".to_string(),
    "Description".to_string(),
    Some("Category".to_string()),
)
.with_footer_view(view);

// Register with registry
registry.register_builtin(spec);
```

### Canvas Integration

Canvas implementation MUST:

1. Check `ComponentSpec::has_footer_view()` before rendering footer
2. Call `view.render_footer(ui, node)` with valid parameters
3. Handle `Err` result by displaying error message to user
4. Provide UI context with max width constraint (250px)

```rust
// In CanvasViewer::show_footer()
if let Some(view) = spec.get_footer_view() {
    match view.render_footer(ui, graph_node) {
        Ok(()) => { /* Success */ }
        Err(err) => {
            ui.colored_label(egui::Color32::RED, "⚠️ View render failed");
            ui.label(err);
        }
    }
}
```

## Breaking Change Policy

This trait is part of the public API (for builtin components). Changes must follow semantic versioning:

- **Major Version**: Adding required methods (breaks existing implementations)
- **Minor Version**: Adding optional methods with default implementations
- **Patch Version**: Documentation improvements, bug fixes

## Testing Contract

### Unit Testing

```rust
#[test]
fn test_view_renders_without_error() {
    let view = MyComponentFooterView;
    let node = create_test_node();

    // Create minimal egui context for testing
    let ctx = egui::Context::default();
    ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let result = view.render_footer(ui, &node);
            assert!(result.is_ok());
        });
    });
}

#[test]
fn test_view_handles_missing_data() {
    let view = MyComponentFooterView;
    let node = create_empty_node(); // Node with no outputs

    let ctx = egui::Context::default();
    ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let result = view.render_footer(ui, &node);
            // Should either render empty or return error (both valid)
            // Should NOT panic
            match result {
                Ok(()) => { /* Handles gracefully */ }
                Err(msg) => { assert!(!msg.is_empty()); /* Has error message */ }
            }
        });
    });
}
```

### Integration Testing

```rust
#[test]
fn test_canvas_calls_view_for_selected_node() {
    let mut registry = ComponentRegistry::new();
    let view = Arc::new(TestFooterView);
    let spec = ComponentSpec::new_builtin(...)
        .with_footer_view(view.clone());
    registry.register_builtin(spec);

    // Create canvas with test node
    let mut canvas = NodeCanvas::new();
    let mut graph = NodeGraph::new();
    let node_id = graph.add_node(spec.create_node(Pos2::ZERO));

    // Verify canvas detects footer
    canvas.sync_with_graph(&graph);
    // ... test rendering logic
}
```

## Error Scenarios

| Scenario | Expected Behavior | Contract Requirement |
|----------|-------------------|----------------------|
| Missing output data | Return `Err("Missing output data")` | FR-VIEW-002, FR-VIEW-005 |
| Invalid node state | Return `Err` with user-friendly message | FR-VIEW-002, FR-VIEW-005 |
| Slow rendering (>50ms) | Log warning (debug), allow completion | FR-VIEW-003 |
| Panic during render | Application crash (NOT ALLOWED) | FR-VIEW-002 |
| Empty node (no outputs) | Render nothing OR Err | FR-VIEW-002 |

## Compliance Verification

### Code Review Checklist

- [ ] View struct is Send + Sync (compiler enforced)
- [ ] View struct has no mutable state fields
- [ ] render_footer() returns Result, not panics
- [ ] No blocking I/O (no std::fs, network calls)
- [ ] Error messages are user-friendly
- [ ] No performance issues (benchmarked if complex)

### Automated Checks

- Compiler verifies Send + Sync bounds
- Unit tests verify no panics
- Integration tests verify canvas integration
- Performance tests measure render time

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-10-15 | Initial trait definition |

## References

- Feature Spec: [spec.md](../spec.md)
- Data Model: [data-model.md](../data-model.md)
- Research: [research.md](../research.md)
