//! WIT UI Element Renderer
//!
//! This module renders declarative UI elements from WASM components (defined in WIT)
//! into egui UI. This allows WASM components to provide custom footer views without
//! knowing about egui implementation details.
//!
//! The WASM component exports a `get-footer-view` function that returns a declarative
//! description of UI elements, which this module renders.

use egui::Color32;

/// UI element types matching the WIT interface definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UiElement {
    Label(String),
    ColoredLabel(ColoredText),
    KeyValue(KeyValuePair),
    Horizontal(HorizontalLayout),
    Vertical(VerticalLayout),
    Separator,
}

/// Horizontal layout of UI elements
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HorizontalLayout {
    pub elements: Vec<UiElementItem>,
}

/// Vertical layout of UI elements
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerticalLayout {
    pub elements: Vec<UiElementItem>,
}

/// UI element item (non-recursive version for layouts)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UiElementItem {
    Label(String),
    ColoredLabel(ColoredText),
    KeyValue(KeyValuePair),
    Separator,
}

/// Colored text specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColoredText {
    pub text: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Key-value pair for grid display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

/// Footer view data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FooterView {
    pub elements: Vec<UiElement>,
}

/// Render a footer view into egui UI
///
/// This function takes a declarative UI description from a WASM component
/// and renders it using egui widgets.
///
/// Elements are laid out vertically (stacked) by default for readability.
///
/// # Example
///
/// ```rust,ignore
/// // WASM component returns:
/// let footer = FooterView {
///     elements: vec![
///         UiElement::KeyValue(KeyValuePair {
///             key: "Status".to_string(),
///             value: "Ready".to_string(),
///         }),
///         UiElement::Separator,
///         UiElement::ColoredLabel(ColoredText {
///             text: "Result: 42".to_string(),
///             r: 100, g: 255, b: 150,
///         }),
///     ],
/// };
///
/// // Host renders it:
/// render_footer_view(ui, &footer);
/// ```
pub fn render_footer_view(ui: &mut egui::Ui, view: &FooterView) -> Result<(), String> {
    // Wrap all elements in vertical layout so they stack vertically
    ui.vertical(|ui| {
        // Force the vertical layout to use full available width
        // Without this, the layout can shrink and cause narrow column rendering
        ui.set_min_width(ui.available_width());

        for element in &view.elements {
            render_element(ui, element)?;
        }
        Ok::<(), String>(())
    })
    .inner
}

/// Render a single UI element
fn render_element(ui: &mut egui::Ui, element: &UiElement) -> Result<(), String> {
    match element {
        UiElement::Label(text) => {
            ui.add_sized(
                egui::vec2(ui.available_width(), 0.0),
                egui::Label::new(text).wrap()
            );
        }
        UiElement::ColoredLabel(colored) => {
            let color = Color32::from_rgb(colored.r, colored.g, colored.b);
            ui.add_sized(
                egui::vec2(ui.available_width(), 0.0),
                egui::Label::new(egui::RichText::new(&colored.text).color(color)).wrap()
            );
        }
        UiElement::KeyValue(kv) => {
            // Render as vertical pair for full-width display
            ui.label(
                egui::RichText::new(format!("{}:", kv.key))
                    .color(egui::Color32::from_rgb(180, 180, 180))
            );
            // Use full available width for the value
            ui.add_sized(
                egui::vec2(ui.available_width(), 0.0),
                egui::Label::new(&kv.value).wrap()
            );
            ui.add_space(2.0);
        }
        UiElement::Horizontal(layout) => {
            ui.horizontal(|ui| {
                for elem in &layout.elements {
                    render_element_item(ui, elem)?;
                }
                Ok::<(), String>(())
            })
            .inner?;
        }
        UiElement::Vertical(layout) => {
            ui.vertical(|ui| {
                // Force the vertical layout to use full available width
                ui.set_min_width(ui.available_width());

                for elem in &layout.elements {
                    render_element_item(ui, elem)?;
                }
                Ok::<(), String>(())
            })
            .inner?;
        }
        UiElement::Separator => {
            ui.separator();
        }
    }
    Ok(())
}

/// Render a single UI element item (non-recursive)
fn render_element_item(ui: &mut egui::Ui, element: &UiElementItem) -> Result<(), String> {
    match element {
        UiElementItem::Label(text) => {
            // Don't use ui.available_width() here - this is used in horizontal layouts
            // where we want natural sizing, not full width
            ui.label(text);
        }
        UiElementItem::ColoredLabel(colored) => {
            let color = Color32::from_rgb(colored.r, colored.g, colored.b);
            // Don't use ui.available_width() here - this is used in horizontal layouts
            // where we want natural sizing, not full width
            ui.colored_label(color, &colored.text);
        }
        UiElementItem::KeyValue(kv) => {
            // Render as vertical pair for full-width display
            ui.label(
                egui::RichText::new(format!("{}:", kv.key))
                    .color(egui::Color32::from_rgb(180, 180, 180))
            );
            // Use full available width for the value
            ui.add_sized(
                egui::vec2(ui.available_width(), 0.0),
                egui::Label::new(&kv.value).wrap()
            );
            ui.add_space(2.0);
        }
        UiElementItem::Separator => {
            ui.separator();
        }
    }
    Ok(())
}

/// Adapter to make FooterView compatible with ComponentFooterView trait
///
/// This allows WASM component footer views to be used through the same
/// trait interface as built-in Rust component views.
///
/// The adapter reads the cached footer view from the GraphNode, which is
/// computed after each execution with the actual output values.
pub struct WitFooterViewAdapter {
    // No fields needed - we read from node.cached_footer_view at render time
}

impl WitFooterViewAdapter {
    /// Create a new adapter
    ///
    /// Note: This doesn't take a FooterView anymore because footer views
    /// are now dynamic and computed after each execution.
    pub fn new(_view: FooterView) -> Self {
        Self {}
    }
}

impl crate::ui::component_view::ComponentFooterView for WitFooterViewAdapter {
    fn render_footer(
        &self,
        ui: &mut egui::Ui,
        node: &mut crate::graph::node::GraphNode,
    ) -> Result<(), String> {
        // Check if node has a cached footer view
        if let Some(cached_view) = &node.cached_footer_view {
            // Render the cached view with actual output values
            render_footer_view(ui, cached_view)
        } else {
            // No cached view yet - node hasn't been executed
            ui.label("⏳ Execute node to see results");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_view_serialization() {
        let view = FooterView {
            elements: vec![
                UiElement::Label("Test".to_string()),
                UiElement::Separator,
                UiElement::ColoredLabel(ColoredText {
                    text: "Result".to_string(),
                    r: 100,
                    g: 200,
                    b: 255,
                }),
            ],
        };

        // Should serialize to JSON
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("Test"));

        // Should deserialize
        let deserialized: FooterView = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.elements.len(), 3);
    }

    #[test]
    fn test_key_value_pair() {
        let kv = KeyValuePair {
            key: "status".to_string(),
            value: "ready".to_string(),
        };

        assert_eq!(kv.key, "status");
        assert_eq!(kv.value, "ready");
    }

    #[test]
    fn test_colored_text() {
        let colored = ColoredText {
            text: "Hello".to_string(),
            r: 255,
            g: 128,
            b: 0,
        };

        assert_eq!(colored.r, 255);
        assert_eq!(colored.g, 128);
        assert_eq!(colored.b, 0);
    }
}
