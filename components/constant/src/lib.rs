wit_bindgen::generate!({
    path: "wit",
    world: "component-with-ui",
});

use exports::execution::Guest as ExecutionGuest;
use exports::metadata::Guest as MetadataGuest;
use exports::ui::Guest as UiGuest;

use types::*;

struct Component;

impl MetadataGuest for Component {
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "Constant".to_string(),
            version: "0.1.0".to_string(),
            description: "Outputs a constant value. The value and type can be configured.".to_string(),
            author: "WasmFlow".to_string(),
            category: "Utility".to_string(),
        }
    }

    fn get_inputs() -> Vec<PortSpec> {
        // No inputs - this component outputs a constant value
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
        // This component doesn't actually compute the value here.
        // The value is configured in the node metadata and stored in the graph.
        // The execution just passes through whatever was configured.
        //
        // In practice, the app will set the output port value directly from the configuration,
        // so we return an empty result to indicate "no dynamic computation needed".
        Ok(vec![])
    }
}

impl UiGuest for Component {
    fn get_footer_view(outputs: Vec<(String, Value)>) -> Option<exports::ui::FooterView> {
        use exports::ui::{Color, FooterView, UiElement};

        let mut elements = Vec::new();

        // Display the constant value
        if let Some((_, value)) = outputs.iter().find(|(name, _)| name == "value") {
            let type_label = match value {
                Value::U32Val(_) => ("Type: U32", Color::Cyan),
                Value::I32Val(_) => ("Type: I32", Color::Cyan),
                Value::F32Val(_) => ("Type: F32", Color::Cyan),
                Value::StringVal(_) => ("Type: String", Color::Cyan),
                Value::BinaryVal(_) => ("Type: Binary", Color::Cyan),
            };

            elements.push(UiElement::ColoredLabel(type_label));

            let value_str = match value {
                Value::U32Val(v) => format!("Value: {}", v),
                Value::I32Val(v) => format!("Value: {}", v),
                Value::F32Val(v) => format!("Value: {}", v),
                Value::StringVal(v) => format!("Value: {}", v),
                Value::BinaryVal(v) => format!("Value: {} bytes", v.len()),
            };

            elements.push(UiElement::Label(value_str));
        } else {
            elements.push(UiElement::ColoredLabel((
                "No value configured".to_string(),
                Color::Yellow,
            )));
        }

        Some(FooterView { elements })
    }
}

export!(Component with_types_in types);
