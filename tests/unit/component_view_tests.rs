//! Unit tests for ComponentFooterView trait
//!
//! Tests the trait interface, registration, and detection logic.

use std::sync::Arc;
use wasmflow::graph::node::{ComponentSpec, GraphNode};
use wasmflow::ui::component_view::ComponentFooterView;

/// T010: Mock test footer view implementation
struct TestFooterView {
    message: String,
}

impl TestFooterView {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ComponentFooterView for TestFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, _node: &mut GraphNode) -> Result<(), String> {
        ui.label(&self.message);
        Ok(())
    }
}

/// T010: Test that we can create a mock ComponentFooterView implementation
#[test]
fn test_create_mock_footer_view() {
    let view = TestFooterView::new("Test footer".to_string());
    assert_eq!(view.message, "Test footer");
}

/// T011: Test that has_footer_view returns true when view is set
#[test]
fn test_has_footer_view_returns_true_when_set() {
    let view = Arc::new(TestFooterView::new("Test".to_string()));

    let spec = ComponentSpec::new_builtin(
        "test:component".to_string(),
        "Test Component".to_string(),
        "A test component".to_string(),
        Some("Test".to_string()),
    )
    .with_footer_view(view);

    assert!(spec.has_footer_view(), "has_footer_view should return true when view is set");
}

/// T011: Test that has_footer_view returns false when view is not set
#[test]
fn test_has_footer_view_returns_false_when_not_set() {
    let spec = ComponentSpec::new_builtin(
        "test:component".to_string(),
        "Test Component".to_string(),
        "A test component".to_string(),
        Some("Test".to_string()),
    );

    assert!(!spec.has_footer_view(), "has_footer_view should return false when view is not set");
}

/// T012: Test that get_footer_view returns Some when view is set
#[test]
fn test_get_footer_view_returns_some_when_set() {
    let view = Arc::new(TestFooterView::new("Test".to_string()));

    let spec = ComponentSpec::new_builtin(
        "test:component".to_string(),
        "Test Component".to_string(),
        "A test component".to_string(),
        Some("Test".to_string()),
    )
    .with_footer_view(view);

    assert!(spec.get_footer_view().is_some(), "get_footer_view should return Some when view is set");
}

/// T012: Test that get_footer_view returns None when view is not set
#[test]
fn test_get_footer_view_returns_none_when_not_set() {
    let spec = ComponentSpec::new_builtin(
        "test:component".to_string(),
        "Test Component".to_string(),
        "A test component".to_string(),
        Some("Test".to_string()),
    );

    assert!(spec.get_footer_view().is_none(), "get_footer_view should return None when view is not set");
}

/// T013: Test that trait can be stored as Arc<dyn ComponentFooterView>
#[test]
fn test_trait_can_be_stored_as_arc() {
    let view: Arc<dyn ComponentFooterView> = Arc::new(TestFooterView::new("Test".to_string()));

    let spec = ComponentSpec::new_builtin(
        "test:component".to_string(),
        "Test Component".to_string(),
        "A test component".to_string(),
        Some("Test".to_string()),
    )
    .with_footer_view(view);

    assert!(spec.has_footer_view(), "Arc<dyn ComponentFooterView> should be storable");
}

/// T013: Test that multiple different view types can be stored
#[test]
fn test_multiple_view_types() {
    struct ViewA;
    impl ComponentFooterView for ViewA {
        fn render_footer(&self, ui: &mut egui::Ui, _node: &mut GraphNode) -> Result<(), String> {
            ui.label("View A");
            Ok(())
        }
    }

    struct ViewB;
    impl ComponentFooterView for ViewB {
        fn render_footer(&self, ui: &mut egui::Ui, _node: &mut GraphNode) -> Result<(), String> {
            ui.label("View B");
            Ok(())
        }
    }

    let spec_a = ComponentSpec::new_builtin(
        "test:a".to_string(),
        "Component A".to_string(),
        "Component with view A".to_string(),
        None,
    )
    .with_footer_view(Arc::new(ViewA));

    let spec_b = ComponentSpec::new_builtin(
        "test:b".to_string(),
        "Component B".to_string(),
        "Component with view B".to_string(),
        None,
    )
    .with_footer_view(Arc::new(ViewB));

    assert!(spec_a.has_footer_view(), "Component A should have footer view");
    assert!(spec_b.has_footer_view(), "Component B should have footer view");
}

/// Test that builder pattern works correctly
#[test]
fn test_builder_pattern() {
    let view = Arc::new(TestFooterView::new("Test".to_string()));

    let spec = ComponentSpec::new_builtin(
        "test:component".to_string(),
        "Test Component".to_string(),
        "A test component".to_string(),
        Some("Test".to_string()),
    )
    .with_input("input".to_string(), wasmflow::graph::node::DataType::F32, "Test input".to_string())
    .with_output("output".to_string(), wasmflow::graph::node::DataType::F32, "Test output".to_string())
    .with_footer_view(view);

    assert!(spec.has_footer_view(), "Builder pattern should preserve footer view");
    assert_eq!(spec.input_spec.len(), 1, "Builder should preserve input specs");
    assert_eq!(spec.output_spec.len(), 1, "Builder should preserve output specs");
}

/// Test that view is properly skipped in serialization
#[test]
fn test_view_not_serialized() {
    let view = Arc::new(TestFooterView::new("Test".to_string()));

    let spec = ComponentSpec::new_builtin(
        "test:component".to_string(),
        "Test Component".to_string(),
        "A test component".to_string(),
        Some("Test".to_string()),
    )
    .with_footer_view(view);

    // Serialize and deserialize
    let serialized = serde_json::to_string(&spec).expect("Should serialize");
    let deserialized: ComponentSpec = serde_json::from_str(&serialized).expect("Should deserialize");

    // View should not be present after deserialization
    assert!(!deserialized.has_footer_view(), "View should not be serialized/deserialized");
}
