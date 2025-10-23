//! Example WasmFlow component with custom footer view
//!
//! This component demonstrates how WASM components can provide custom UI
//! through the WIT ui interface, without knowing anything about egui.

// Generate bindings from WIT files
wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});

use exports::wasmflow::node::{
    metadata::{ComponentInfo, Guest as MetadataGuest, PortSpec},
    execution::{ExecutionError, Guest as ExecutionGuest, Value},
    ui::{ColoredText, FooterView, Guest as UiGuest, HorizontalLayout, KeyValuePair, UiElement, UiElementItem},
};
use wasmflow::node::types::DataType;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Footer View Example".to_string(),
            version: "1.0.0".to_string(),
            description: "Demonstrates custom footer view via WIT".to_string(),
            author: "WasmFlow Examples".to_string(),
            category: Some("Examples".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "value".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Input value to process".to_string(),
            }
        ]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![
            PortSpec {
                name: "result".to_string(),
                data_type: DataType::F32Type,
                optional: false,
                description: "Processed result".to_string(),
            },
            PortSpec {
                name: "status".to_string(),
                data_type: DataType::StringType,
                optional: false,
                description: "Operation status".to_string(),
            }
        ]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(inputs: Vec<(String, Value)>) -> Result<Vec<(String, Value)>, ExecutionError> {
        // Get input value
        let value = inputs
            .iter()
            .find(|(name, _)| name == "value")
            .and_then(|(_, val)| match val {
                Value::F32Val(v) => Some(*v),
                _ => None,
            })
            .ok_or_else(|| ExecutionError {
                message: "Missing or invalid 'value' input".to_string(),
                input_name: Some("value".to_string()),
                recovery_hint: Some("Provide a valid F32 value".to_string()),
            })?;

        // Simple computation: multiply by 2
        let result = value * 2.0;

        // Return outputs
        Ok(vec![
            ("result".to_string(), Value::F32Val(result)),
            ("status".to_string(), Value::StringVal("Success".to_string())),
        ])
    }
}

/// This is the key implementation - custom footer view
impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        // Build a custom footer view declaratively
        let mut elements = Vec::new();

        // Title
        elements.push(UiElement::ColoredLabel(ColoredText {
            text: "📊 Custom Footer View".to_string(),
            r: 100,
            g: 200,
            b: 255,
        }));

        elements.push(UiElement::Separator);

        // Show outputs in a nice format
        for (name, value) in outputs {
            let value_str = match value {
                Value::F32Val(v) => format!("{:.2}", v),
                Value::I32Val(v) => format!("{}", v),
                Value::U32Val(v) => format!("{}", v),
                Value::StringVal(s) => s,
                Value::BinaryVal(_) => "<binary data>".to_string(),
            };

            elements.push(UiElement::KeyValue(KeyValuePair {
                key: name,
                value: value_str,
            }));
        }

        // Add a custom status indicator
        elements.push(UiElement::Separator);
        elements.push(UiElement::Horizontal(HorizontalLayout {
            elements: vec![
                UiElementItem::Label("Status:".to_string()),
                UiElementItem::ColoredLabel(ColoredText {
                    text: "✓ Ready".to_string(),
                    r: 100,
                    g: 255,
                    b: 150,
                }),
            ],
        }));

        Some(FooterView { elements })
    }
}

export!(Component);
