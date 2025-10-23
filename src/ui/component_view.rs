// Component Footer View Trait
//
// This module defines the trait interface for components to provide custom
// footer UI rendering. Components can optionally implement this trait to
// display custom content in the canvas footer when a node is selected.

use crate::graph::node::GraphNode;

/// Trait for components to provide custom footer UI rendering.
///
/// Components can implement this trait to display custom content in the
/// canvas footer when a node is selected. This enables colocation of
/// component logic with its UI presentation.
///
/// # Example
///
/// ```rust,ignore
/// use wasmflow::ui::component_view::ComponentFooterView;
/// use wasmflow::graph::node::GraphNode;
/// use std::sync::Arc;
///
/// struct MyComponentFooterView;
///
/// impl ComponentFooterView for MyComponentFooterView {
///     fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
///         // Display component-specific information
///         ui.label("Custom footer content");
///
///         // Access node outputs to show results
///         for output in &node.outputs {
///             if let Some(value) = &output.current_value {
///                 ui.label(format!("{}: {}", output.name, value.format_display()));
///             }
///         }
///
///         Ok(())
///     }
/// }
///
/// // Register the view with your component
/// let spec = ComponentSpec::new_builtin(...)
///     .with_footer_view(Arc::new(MyComponentFooterView));
/// ```
///
/// # Performance
///
/// The `render_footer` method is called every frame while the node is selected.
/// Keep rendering logic lightweight (target <50ms) to maintain responsive UI:
///
/// - Avoid expensive operations like network requests or disk I/O
/// - Cache computed values if possible
/// - Limit the amount of data displayed (truncate long strings/lists)
/// - Use simple UI widgets (avoid complex layouts)
///
/// Performance is automatically monitored - warnings are logged if rendering
/// exceeds 50ms.
///
/// # Reactivity (T042)
///
/// Views automatically reflect component state changes thanks to egui's
/// immediate mode rendering paradigm:
///
/// - The view receives a fresh `&GraphNode` reference each frame
/// - Any changes to node outputs are immediately visible
/// - No explicit update notifications or callbacks needed
/// - The canvas re-renders on every frame, calling your view each time
///
/// This means your view implementation can be stateless - just read from
/// the provided `GraphNode` and render accordingly.
///
/// # Error Handling
///
/// Return `Err(message)` if rendering fails. The error will be displayed
/// in the footer to help users diagnose issues:
///
/// ```rust,ignore
/// fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
///     let output = node.outputs.first()
///         .ok_or_else(|| "No outputs available".to_string())?;
///
///     // ... render using output
///     Ok(())
/// }
/// ```
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` since views may be accessed from
/// multiple threads in egui's rendering pipeline.
pub trait ComponentFooterView: Send + Sync {
    /// Render custom footer content for this component.
    ///
    /// # Parameters
    ///
    /// - `ui`: egui UI context for rendering widgets
    /// - `node`: The graph node being displayed (mutable access for editing)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if rendering succeeded
    /// - `Err(message)` if rendering failed (error will be displayed in footer)
    ///
    /// # Performance
    ///
    /// This method is called every frame while the node is selected.
    /// Keep rendering logic lightweight (<50ms target).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
    ///     // Display output values in a grid
    ///     egui::Grid::new(format!("footer_{}", node.id))
    ///         .num_columns(2)
    ///         .show(ui, |ui| {
    ///             for output in &mut node.outputs {
    ///                 if let Some(value) = &mut output.current_value {
    ///                     ui.label(&output.name);
    ///                     ui.label(value.format_display());
    ///                     ui.end_row();
    ///                 }
    ///             }
    ///         });
    ///     Ok(())
    /// }
    /// ```
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String>;
}
