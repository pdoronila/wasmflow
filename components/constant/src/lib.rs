//! Constant Component - Outputs a configurable constant value
//!
//! This component provides a constant output with the value configured
//! through the node's output port. The type and value can be changed via the UI.

wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});

use exports::wasmflow::node::{
    metadata::{ComponentInfo, Guest as MetadataGuest, PortSpec},
    execution::{ExecutionError, Guest as ExecutionGuest, Value},
    ui::{ColoredText, FooterView, Guest as UiGuest, KeyValuePair, UiElement},
};
use wasmflow::node::types::DataType;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Constant".to_string(),
            version: "0.1.0".to_string(),
            description: "Outputs a constant value with configurable type (U32, I32, F32, String, Binary)".to_string(),
            author: "WasmFlow".to_string(),
            category: Some("Utility".to_string()),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        // No inputs - this is a source component
        vec![]
    }

    fn get_outputs() -> Vec<PortSpec> {
        vec![PortSpec {
            name: "value".to_string(),
            data_type: DataType::AnyType,
            optional: false,
            description: "The constant output value".to_string(),
        }]
    }

    fn get_capabilities() -> Option<Vec<String>> {
        None
    }
}

impl ExecutionGuest for Component {
    fn execute(
        _inputs: Vec<(String, Value)>,
    ) -> Result<Vec<(String, Value)>, ExecutionError> {
        // The constant value is stored in the node's output port
        // and doesn't need to be computed. The app handles setting
        // the output value directly from the UI configuration.
        //
        // We return an empty result to indicate no dynamic computation.
        Ok(vec![])
    }
}

impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<FooterView> {
        let mut elements = Vec::new();

        // Header
        elements.push(UiElement::ColoredLabel(ColoredText {
            text: "⚙️ Constant Value".to_string(),
            r: 100,
            g: 200,
            b: 255,
        }));

        elements.push(UiElement::Separator);

        // Display the constant value with type information
        if let Some((_, value)) = outputs.iter().find(|(name, _)| name == "value") {
            // Show type
            let (type_name, type_color) = match value {
                Value::U32Val(_) => ("Type: U32 (Unsigned Integer)", (100, 200, 255)),
                Value::I32Val(_) => ("Type: I32 (Signed Integer)", (150, 180, 255)),
                Value::F32Val(_) => ("Type: F32 (Float)", (200, 150, 255)),
                Value::StringVal(_) => ("Type: String", (255, 200, 100)),
                Value::BinaryVal(_) => ("Type: Binary", (200, 100, 255)),
            };

            elements.push(UiElement::ColoredLabel(ColoredText {
                text: type_name.to_string(),
                r: type_color.0,
                g: type_color.1,
                b: type_color.2,
            }));

            // Show value
            let value_str = match value {
                Value::U32Val(v) => format!("{}", v),
                Value::I32Val(v) => format!("{}", v),
                Value::F32Val(v) => format!("{}", v),
                Value::StringVal(v) => {
                    if v.len() > 50 {
                        format!("{}... ({} chars)", &v[..50], v.len())
                    } else {
                        v.clone()
                    }
                }
                Value::BinaryVal(v) => format!("{} bytes", v.len()),
            };

            elements.push(UiElement::KeyValue(KeyValuePair {
                key: "Value".to_string(),
                value: value_str,
            }));
        } else {
            elements.push(UiElement::ColoredLabel(ColoredText {
                text: "⚠ No value configured".to_string(),
                r: 255,
                g: 200,
                b: 100,
            }));
        }

        elements.push(UiElement::Separator);

        // Status
        elements.push(UiElement::ColoredLabel(ColoredText {
            text: "✓ Ready".to_string(),
            r: 100,
            g: 255,
            b: 150,
        }));

        Some(FooterView { elements })
    }
}

export!(Component);
