# egui-snarl - Node Graph Library

egui-snarl is a customizable node-graph library for egui. It provides typed data-only nodes with beautiful wire connections, context menus, UI scaling, and serialization support.

## Overview

egui-snarl is designed for building node-based visual programming interfaces, shader editors, audio processing graphs, and other node-graph applications. It handles the graph visualization and interaction while letting you define the semantics.

## Core Concepts

### Snarl Structure

The `Snarl<T>` struct is the main container that holds:
- Positioned nodes (each containing data of type `T`)
- Wires connecting node pins
- UI state (collapsed nodes, selection, etc.)

### SnarlViewer Trait

Implement this trait to define how your nodes look and behave:

```rust
pub trait SnarlViewer<T> {
    // Required methods
    fn title(&mut self, node: &T) -> String;
    fn inputs(&mut self, node: &T) -> usize;
    fn outputs(&mut self, node: &T) -> usize;
    fn show_input(&mut self, pin: &InPin, ui: &mut Ui, scale: f32, 
                  snarl: &mut Snarl<T>) -> PinInfo;
    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, scale: f32,
                   snarl: &mut Snarl<T>) -> PinInfo;
    
    // Optional methods for customization
    fn has_body(&mut self, node: &T) -> bool { false }
    fn show_body(&mut self, node: NodeId, inputs: &[InPin], outputs: &[OutPin],
                 ui: &mut Ui, scale: f32, snarl: &mut Snarl<T>) { }
    fn has_header(&mut self, node: &T) -> bool { true }
    fn show_header(&mut self, node: NodeId, inputs: &[InPin], outputs: &[OutPin],
                   ui: &mut Ui, scale: f32, snarl: &mut Snarl<T>) { }
    fn has_footer(&mut self, node: &T) -> bool { false }
    fn show_footer(&mut self, node: NodeId, inputs: &[InPin], outputs: &[OutPin],
                   ui: &mut Ui, scale: f32, snarl: &mut Snarl<T>) { }
    
    // Connection handling
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<T>) { }
    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<T>) { }
    
    // Context menus
    fn node_context_menu(&mut self, node: NodeId, ui: &mut Ui, 
                        scale: f32, snarl: &mut Snarl<T>) { }
    fn graph_menu(&mut self, pos: Pos2, ui: &mut Ui, 
                  scale: f32, snarl: &mut Snarl<T>) { }
}
```

## Complete Implementation Example

### Node Data Definition

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
enum NodeData {
    // Input nodes
    FloatInput { value: f32 },
    Vec3Input { x: f32, y: f32, z: f32 },
    
    // Math operations
    Add,
    Multiply,
    Normalize,
    
    // Output
    Output,
}

impl NodeData {
    fn inputs(&self) -> usize {
        match self {
            NodeData::FloatInput { .. } => 0,
            NodeData::Vec3Input { .. } => 0,
            NodeData::Add => 2,
            NodeData::Multiply => 2,
            NodeData::Normalize => 1,
            NodeData::Output => 1,
        }
    }
    
    fn outputs(&self) -> usize {
        match self {
            NodeData::FloatInput { .. } => 1,
            NodeData::Vec3Input { .. } => 1,
            NodeData::Add => 1,
            NodeData::Multiply => 1,
            NodeData::Normalize => 1,
            NodeData::Output => 0,
        }
    }
}
```

### Viewer Implementation

```rust
use egui_snarl::{
    InPin, OutPin, Snarl, SnarlViewer,
    ui::{PinInfo, SnarlStyle}
};

struct GraphViewer;

impl SnarlViewer<NodeData> for GraphViewer {
    fn title(&mut self, node: &NodeData) -> String {
        match node {
            NodeData::FloatInput { .. } => "Float".into(),
            NodeData::Vec3Input { .. } => "Vector3".into(),
            NodeData::Add => "Add".into(),
            NodeData::Multiply => "Multiply".into(),
            NodeData::Normalize => "Normalize".into(),
            NodeData::Output => "Output".into(),
        }
    }
    
    fn inputs(&mut self, node: &NodeData) -> usize {
        node.inputs()
    }
    
    fn outputs(&mut self, node: &NodeData) -> usize {
        node.outputs()
    }
    
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<NodeData>,
    ) -> PinInfo {
        let node = &snarl[pin.id.node];
        
        match node {
            NodeData::Add | NodeData::Multiply => {
                if pin.id.input == 0 {
                    ui.label("A");
                } else {
                    ui.label("B");
                }
            }
            NodeData::Normalize => {
                ui.label("In");
            }
            NodeData::Output => {
                ui.label("Value");
            }
            _ => {}
        }
        
        // Return pin shape and color
        PinInfo::circle().with_fill(egui::Color32::from_rgb(100, 150, 200))
    }
    
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut egui::Ui,
        _scale: f32,
        _snarl: &mut Snarl<NodeData>,
    ) -> PinInfo {
        ui.label("Out");
        PinInfo::circle().with_fill(egui::Color32::from_rgb(200, 150, 100))
    }
    
    // Optional: Add node body for editable values
    fn has_body(&mut self, node: &NodeData) -> bool {
        matches!(node, NodeData::FloatInput { .. } | NodeData::Vec3Input { .. })
    }
    
    fn show_body(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<NodeData>,
    ) {
        match &mut snarl[node] {
            NodeData::FloatInput { value } => {
                ui.add(egui::DragValue::new(value).speed(0.1));
            }
            NodeData::Vec3Input { x, y, z } => {
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(x).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(y).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Z:");
                    ui.add(egui::DragValue::new(z).speed(0.1));
                });
            }
            _ => {}
        }
    }
    
    // Handle connections
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<NodeData>) {
        // Validate connection and create wire
        snarl.connect(from.id, to.id);
    }
    
    fn disconnect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<NodeData>) {
        snarl.disconnect(from.id, to.id);
    }
    
    // Context menu for nodes
    fn node_context_menu(
        &mut self,
        node: egui_snarl::NodeId,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<NodeData>,
    ) {
        if ui.button("Delete Node").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
        
        if ui.button("Duplicate").clicked() {
            let data = snarl[node].clone();
            let pos = snarl.get_node_pos(node) + egui::vec2(50.0, 50.0);
            snarl.insert_node(pos, data);
            ui.close_menu();
        }
    }
    
    // Context menu for background
    fn graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<NodeData>,
    ) {
        ui.label("Add Node:");
        ui.separator();
        
        if ui.button("Float Input").clicked() {
            snarl.insert_node(pos, NodeData::FloatInput { value: 0.0 });
            ui.close_menu();
        }
        
        if ui.button("Vector3 Input").clicked() {
            snarl.insert_node(pos, NodeData::Vec3Input { 
                x: 0.0, y: 0.0, z: 0.0 
            });
            ui.close_menu();
        }
        
        if ui.button("Add").clicked() {
            snarl.insert_node(pos, NodeData::Add);
            ui.close_menu();
        }
        
        if ui.button("Multiply").clicked() {
            snarl.insert_node(pos, NodeData::Multiply);
            ui.close_menu();
        }
        
        if ui.button("Normalize").clicked() {
            snarl.insert_node(pos, NodeData::Normalize);
            ui.close_menu();
        }
        
        if ui.button("Output").clicked() {
            snarl.insert_node(pos, NodeData::Output);
            ui.close_menu();
        }
    }
}
```

### Application Integration

```rust
use eframe::egui;
use egui_snarl::{Snarl, ui::SnarlStyle};

struct MyApp {
    snarl: Snarl<NodeData>,
    viewer: GraphViewer,
    style: SnarlStyle,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut snarl = Snarl::new();
        
        // Create some initial nodes
        let input = snarl.insert_node(
            egui::pos2(100.0, 100.0),
            NodeData::FloatInput { value: 1.0 }
        );
        let multiply = snarl.insert_node(
            egui::pos2(300.0, 100.0),
            NodeData::Multiply
        );
        let output = snarl.insert_node(
            egui::pos2(500.0, 100.0),
            NodeData::Output
        );
        
        Self {
            snarl,
            viewer: GraphViewer,
            style: SnarlStyle::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.snarl.show(
                &mut self.viewer,
                &self.style,
                "node_graph",
                ui,
            );
        });
    }
}
```

## Pin Customization

### Pin Shapes

```rust
fn show_output(&mut self, pin: &OutPin, ui: &mut egui::Ui,
               _scale: f32, snarl: &mut Snarl<NodeData>) -> PinInfo {
    ui.label("Out");
    
    // Different shapes based on data type
    match get_pin_type(pin, snarl) {
        PinType::Float => PinInfo::circle(),
        PinType::Vector => PinInfo::square(),
        PinType::Bool => PinInfo::triangle(),
    }
}
```

### Pin Colors and Styling

```rust
PinInfo::circle()
    .with_fill(egui::Color32::from_rgb(100, 150, 200))
    .with_stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
```

## Wire Styling

### Custom Wire Colors

```rust
fn show_output(&mut self, pin: &OutPin, ui: &mut egui::Ui,
               _scale: f32, _snarl: &mut Snarl<NodeData>) -> PinInfo {
    ui.label("Out");
    
    PinInfo::circle().with_wire_style(
        egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 100, 50))
    )
}
```

### Background Patterns

```rust
use egui_snarl::ui::{BackgroundPattern, SnarlStyle};

let mut style = SnarlStyle::default();

// Grid pattern
style.background_pattern = Some(BackgroundPattern::Grid);

// Custom grid spacing
style.background_pattern_stroke = egui::Stroke::new(
    1.0, 
    egui::Color32::from_rgba_premultiplied(100, 100, 100, 50)
);
```

## Node Layout Customization

### Node Structure

Nodes have five layout areas:
1. **Header** - Top section with title and collapse button
2. **Inputs** - Left side input pins
3. **Body** - Center content area
4. **Outputs** - Right side output pins
5. **Footer** - Bottom section

```rust
fn has_header(&mut self, node: &NodeData) -> bool {
    true // Show header with title
}

fn has_body(&mut self, node: &NodeData) -> bool {
    // Show body only for nodes with editable content
    matches!(node, NodeData::Input(_) | NodeData::Constant(_))
}

fn has_footer(&mut self, node: &NodeData) -> bool {
    false // No footer by default
}
```

### Custom Header

```rust
fn show_header(
    &mut self,
    node: egui_snarl::NodeId,
    _inputs: &[InPin],
    _outputs: &[OutPin],
    ui: &mut egui::Ui,
    _scale: f32,
    snarl: &mut Snarl<NodeData>,
) {
    ui.horizontal(|ui| {
        // Custom icon
        ui.label("🔧");
        
        // Node title
        ui.heading(self.title(&snarl[node]));
        
        // Status indicator
        if is_node_active(node, snarl) {
            ui.colored_label(egui::Color32::GREEN, "●");
        }
    });
}
```

## Serialization

Enable with `serde` feature:

```toml
[dependencies]
egui-snarl = { version = "0.8", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct GraphState {
    snarl: Snarl<NodeData>,
}

impl GraphState {
    fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.snarl)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let snarl = serde_json::from_str(&json)?;
        Ok(Self { snarl })
    }
}
```

## Advanced Patterns

### Type Checking Connections

```rust
fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<NodeData>) {
    // Check if connection is valid
    let from_type = get_output_type(&snarl[from.id.node], from.id.output);
    let to_type = get_input_type(&snarl[to.id.node], to.id.input);
    
    if types_compatible(from_type, to_type) {
        snarl.connect(from.id, to.id);
    } else {
        eprintln!("Cannot connect incompatible types");
    }
}
```

### Dynamic Pin Counts

```rust
impl SnarlViewer<NodeData> for GraphViewer {
    fn inputs(&mut self, node: &NodeData) -> usize {
        match node {
            NodeData::Merge { input_count } => *input_count,
            _ => node.default_inputs(),
        }
    }
    
    fn show_body(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<NodeData>,
    ) {
        if let NodeData::Merge { input_count } = &mut snarl[node] {
            ui.horizontal(|ui| {
                ui.label("Inputs:");
                if ui.button("-").clicked() && *input_count > 1 {
                    *input_count -= 1;
                }
                ui.label(input_count.to_string());
                if ui.button("+").clicked() {
                    *input_count += 1;
                }
            });
        }
    }
}
```

### Graph Evaluation

```rust
fn evaluate_graph(snarl: &Snarl<NodeData>) -> HashMap<NodeId, Value> {
    let mut results = HashMap::new();
    let mut to_evaluate: Vec<NodeId> = snarl.node_ids().collect();
    
    // Topological sort to evaluate in correct order
    while !to_evaluate.is_empty() {
        // Find nodes with all inputs evaluated
        for &node_id in &to_evaluate.clone() {
            if can_evaluate(node_id, snarl, &results) {
                let value = evaluate_node(node_id, snarl, &results);
                results.insert(node_id, value);
                to_evaluate.retain(|&id| id != node_id);
            }
        }
    }
    
    results
}
```

## Style Configuration

```rust
let mut style = SnarlStyle::default();

// Node appearance
style.node_frame = egui::Frame::default()
    .fill(egui::Color32::from_gray(40))
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
    .rounding(5.0);

// Wire appearance
style.wire_frame_style = egui::Stroke::new(2.0, egui::Color32::from_gray(150));
style.wire_thickness = 3.0;

// Selection
style.selection_frame = egui::Stroke::new(2.0, egui::Color32::BLUE);

// Background
style.background_pattern = Some(BackgroundPattern::Grid);

// Collapsible nodes
style.collapsible = true;
```

## Best Practices

1. **Keep node data simple** - Store only essential state in nodes
2. **Validate connections** - Check type compatibility in `connect()`
3. **Use enums for node types** - Makes pattern matching easy
4. **Cache evaluation results** - Don't re-evaluate unchanged subgraphs
5. **Provide clear visual feedback** - Use colors and shapes to indicate types
6. **Handle cycles gracefully** - Detect and prevent infinite loops
7. **Serialize carefully** - Ensure node data is serializable
8. **Test with complex graphs** - Ensure performance scales

## Common Patterns

### Node Palette

```rust
struct NodePalette {
    categories: Vec<NodeCategory>,
}

struct NodeCategory {
    name: String,
    nodes: Vec<NodeTemplate>,
}

impl GraphViewer {
    fn show_palette(&self, ui: &mut egui::Ui) -> Option<NodeData> {
        let mut selected = None;
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for category in &self.palette.categories {
                ui.collapsing(&category.name, |ui| {
                    for template in &category.nodes {
                        if ui.button(&template.name).clicked() {
                            selected = Some(template.create_node());
                        }
                    }
                });
            }
        });
        
        selected
    }
}
```

### Undo/Redo

```rust
struct GraphHistory {
    states: Vec<Snarl<NodeData>>,
    current: usize,
}

impl GraphHistory {
    fn push(&mut self, state: Snarl<NodeData>) {
        self.states.truncate(self.current + 1);
        self.states.push(state);
        self.current += 1;
    }
    
    fn undo(&mut self) -> Option<&Snarl<NodeData>> {
        if self.current > 0 {
            self.current -= 1;
            Some(&self.states[self.current])
        } else {
            None
        }
    }
    
    fn redo(&mut self) -> Option<&Snarl<NodeData>> {
        if self.current + 1 < self.states.len() {
            self.current += 1;
            Some(&self.states[self.current])
        } else {
            None
        }
    }
}
```

## Troubleshooting

### Wires Not Connecting
- Ensure `connect()` method calls `snarl.connect()`
- Check that pin IDs are valid
- Verify node has the expected input/output count

### Nodes Not Rendering
- Implement all required SnarlViewer methods
- Check that `title()`, `inputs()`, and `outputs()` return correct values
- Ensure node positions are within visible area

### Performance Issues
- Minimize allocations in hot paths
- Cache pin information when possible
- Only evaluate dirty subgraphs
- Use `has_body()` to avoid unnecessary widget creation
