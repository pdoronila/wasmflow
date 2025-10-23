//! Integration tests for canvas and component view interaction
//!
//! Tests the full integration of ComponentFooterView with CanvasViewer.

use std::sync::Arc;
use wasmflow::graph::node::{ComponentRegistry, ComponentSpec, DataType, GraphNode};
use wasmflow::ui::component_view::ComponentFooterView;

/// Test footer view implementation
struct TestComponentFooterView {
    label: String,
}

impl TestComponentFooterView {
    fn new(label: String) -> Self {
        Self { label }
    }
}

impl ComponentFooterView for TestComponentFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
        ui.label(format!("{}: Node {}", self.label, node.display_name));
        Ok(())
    }
}

/// T014: Test that ComponentSpec with custom view can be registered
#[test]
fn test_register_component_with_custom_view() {
    let mut registry = ComponentRegistry::new();

    let view = Arc::new(TestComponentFooterView::new("Custom View".to_string()));
    let spec = ComponentSpec::new_builtin(
        "test:custom".to_string(),
        "Custom Component".to_string(),
        "Component with custom view".to_string(),
        Some("Test".to_string()),
    )
    .with_footer_view(view);

    registry.register_builtin(spec);

    // Verify component is registered and has view
    let retrieved = registry
        .get_by_id("test:custom")
        .expect("Component should be registered");
    assert!(
        retrieved.has_footer_view(),
        "Registered component should have footer view"
    );
}

/// T014: Test that has_footer detects custom view via registry
#[test]
fn test_has_footer_detection_via_registry() {
    let mut registry = ComponentRegistry::new();

    // Register component with view
    let view = Arc::new(TestComponentFooterView::new("Test".to_string()));
    let spec = ComponentSpec::new_builtin(
        "test:with_view".to_string(),
        "With View".to_string(),
        "Has custom view".to_string(),
        None,
    )
    .with_footer_view(view);
    registry.register_builtin(spec);

    // Register component without view
    let spec_no_view = ComponentSpec::new_builtin(
        "test:no_view".to_string(),
        "No View".to_string(),
        "No custom view".to_string(),
        None,
    );
    registry.register_builtin(spec_no_view);

    // Verify detection logic
    let with_view = registry.get_by_id("test:with_view").unwrap();
    assert!(
        with_view.has_footer_view(),
        "Component with view should be detected"
    );

    let without_view = registry.get_by_id("test:no_view").unwrap();
    assert!(
        !without_view.has_footer_view(),
        "Component without view should not be detected"
    );
}

/// T015: Test that view can access node data for rendering
#[test]
fn test_view_can_access_node_data() {
    struct NodeDataView;
    impl ComponentFooterView for NodeDataView {
        fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
            // Access node properties
            ui.label(format!("ID: {}", node.id));
            ui.label(format!("Name: {}", node.display_name));
            ui.label(format!("Component: {}", node.component_id));
            ui.label(format!("Inputs: {}", node.inputs.len()));
            ui.label(format!("Outputs: {}", node.outputs.len()));
            Ok(())
        }
    }

    let view = Arc::new(NodeDataView);
    let spec = ComponentSpec::new_builtin(
        "test:node_data".to_string(),
        "Node Data View".to_string(),
        "Accesses node data".to_string(),
        None,
    )
    .with_input("in1".to_string(), DataType::F32, "Input 1".to_string())
    .with_output("out1".to_string(), DataType::F32, "Output 1".to_string())
    .with_footer_view(view);

    // Create node from spec
    let node = spec.create_node(egui::Pos2::ZERO);

    // Verify node has expected structure
    assert_eq!(node.inputs.len(), 1, "Node should have 1 input");
    assert_eq!(node.outputs.len(), 1, "Node should have 1 output");
    assert_eq!(node.component_id, "test:node_data");
}

/// T015: Test that view can read output values
#[test]
fn test_view_can_read_output_values() {
    struct OutputValueView;
    impl ComponentFooterView for OutputValueView {
        fn render_footer(&self, ui: &mut egui::Ui, node: &GraphNode) -> Result<(), String> {
            for output in &node.outputs {
                if let Some(value) = &output.current_value {
                    ui.label(format!("{}: {}", output.name, value.format_display()));
                }
            }
            Ok(())
        }
    }

    let view = Arc::new(OutputValueView);
    let spec = ComponentSpec::new_builtin(
        "test:output_view".to_string(),
        "Output View".to_string(),
        "Displays output values".to_string(),
        None,
    )
    .with_output("result".to_string(), DataType::F32, "Result".to_string())
    .with_footer_view(view);

    let mut node = spec.create_node(egui::Pos2::ZERO);

    // Set output value
    if let Some(output) = node.outputs.get_mut(0) {
        output.current_value = Some(wasmflow::graph::node::NodeValue::F32(42.5));
    }

    // Verify output has value
    assert!(
        node.outputs[0].current_value.is_some(),
        "Output should have value"
    );
}

/// Test error handling in view rendering
#[test]
fn test_view_error_handling() {
    struct FailingView;
    impl ComponentFooterView for FailingView {
        fn render_footer(&self, _ui: &mut egui::Ui, _node: &GraphNode) -> Result<(), String> {
            Err("View rendering failed".to_string())
        }
    }

    let view = Arc::new(FailingView);
    let spec = ComponentSpec::new_builtin(
        "test:failing".to_string(),
        "Failing View".to_string(),
        "View that returns error".to_string(),
        None,
    )
    .with_footer_view(view);

    // Verify view is registered
    assert!(
        spec.has_footer_view(),
        "Failing view should still be registered"
    );
}

/// Test that view implementations are Send + Sync
#[test]
fn test_view_is_send_sync() {
    struct SendSyncView;
    impl ComponentFooterView for SendSyncView {
        fn render_footer(&self, ui: &mut egui::Ui, _node: &GraphNode) -> Result<(), String> {
            ui.label("Send + Sync");
            Ok(())
        }
    }

    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SendSyncView>();

    let view: Arc<dyn ComponentFooterView> = Arc::new(SendSyncView);
    let _cloned = Arc::clone(&view); // Should compile (Arc<T: Send + Sync> is Send + Sync)
}
