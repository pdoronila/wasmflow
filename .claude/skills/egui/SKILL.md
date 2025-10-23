---
name: egui
description: Build cross-platform GUI applications in Rust using egui, an immediate mode GUI library. Use when creating desktop applications, tools with graphical interfaces, game editor UIs, data visualization apps, or when integrating GUI into Rust projects. Covers eframe for standalone apps, layout management, widgets, styling, and cross-platform deployment including WebAssembly.
---

# egui - Immediate Mode GUI for Rust

## Overview

egui is a lightweight, fast immediate mode GUI library for Rust. Use this skill when building cross-platform desktop applications, developer tools, game editors, or any Rust project requiring a graphical interface. egui excels at responsive, dynamic UIs and works on Windows, macOS, Linux, and web via WebAssembly.

## Quick Start

### Basic eframe Application

The most common way to use egui is through eframe, which handles window creation and event loops:

```rust
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My App",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            age: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to egui!");
            ui.horizontal(|ui| {
                ui.label("Name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click me").clicked() {
                println!("Clicked!");
            }
        });
    }
}
```

### Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
eframe = "0.29"  # Or latest version
egui = "0.29"
```

For WebAssembly support:
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-futures = "0.4"
```

### Asset Template

Use the boilerplate template in `assets/eframe-template/` to quickly start new projects. Copy the entire directory and customize.

## Core Concepts

### Immediate Mode Paradigm

In immediate mode, UI code runs every frame (~60 FPS). No callbacks or state synchronization needed:

```rust
// This runs every frame
if ui.button("Increment").clicked() {
    counter += 1;  // Direct state mutation
}
ui.label(format!("Count: {}", counter));
```

Benefits:
- Simpler code flow - no callbacks
- State always in sync with UI
- Easy to reason about

Tradeoffs:
- Window sizing may have 1-frame delay
- Must manage persistent state explicitly via `Context` or app struct

### Context and UI Objects

- `egui::Context` (`ctx`): Main entry point, provided by the integration
- `egui::Ui` (`ui`): Received from panels/windows, used to add widgets

Always structure apps around panels to get `Ui`:

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Use panels/windows to get Ui
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add widgets here using ui
        });
    }
}
```

## Layout and Panels

### Panel Types

**CentralPanel**: Main content area (fills remaining space)
```rust
egui::CentralPanel::default().show(ctx, |ui| {
    ui.label("Main content");
});
```

**SidePanel**: Left or right sidebars
```rust
egui::SidePanel::left("left_panel").show(ctx, |ui| {
    ui.heading("Sidebar");
});
```

**TopBottomPanel**: Headers or footers
```rust
egui::TopBottomPanel::top("header").show(ctx, |ui| {
    ui.horizontal(|ui| {
        ui.heading("My App");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Settings").clicked() {
                // Handle click
            }
        });
    });
});
```

**Window**: Floating, movable windows
```rust
egui::Window::new("Settings")
    .resizable(true)
    .show(ctx, |ui| {
        ui.label("Window content");
    });
```

**Area**: Custom positioned regions
```rust
egui::Area::new("floating")
    .fixed_pos([100.0, 100.0])
    .show(ctx, |ui| {
        ui.label("Floating content");
    });
```

### Layout Patterns

**Horizontal layout**:
```rust
ui.horizontal(|ui| {
    ui.label("Label");
    ui.button("Button");
});
```

**Vertical layout** (default):
```rust
ui.vertical(|ui| {
    ui.button("Button 1");
    ui.button("Button 2");
});
```

**Grid layout**:
```rust
egui::Grid::new("my_grid")
    .num_columns(2)
    .spacing([10.0, 10.0])
    .show(ui, |ui| {
        ui.label("Row 1, Col 1");
        ui.label("Row 1, Col 2");
        ui.end_row();

        ui.label("Row 2, Col 1");
        ui.label("Row 2, Col 2");
        ui.end_row();
    });
```

**Custom spacing**:
```rust
ui.add_space(10.0);  // Add vertical space
ui.spacing_mut().item_spacing = egui::vec2(10.0, 5.0);  // Set spacing
```

## Common Widgets

### Input Widgets

**Text input**:
```rust
ui.text_edit_singleline(&mut self.text);
ui.text_edit_multiline(&mut self.text);
```

**Numeric input**:
```rust
ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text("Value"));
ui.add(egui::DragValue::new(&mut self.value).speed(0.1));
```

**Checkbox**:
```rust
ui.checkbox(&mut self.enabled, "Enable feature");
```

**Radio buttons**:
```rust
ui.radio_value(&mut self.mode, Mode::A, "Mode A");
ui.radio_value(&mut self.mode, Mode::B, "Mode B");
```

**Combo box (dropdown)**:
```rust
egui::ComboBox::from_label("Select option")
    .selected_text(format!("{:?}", self.selected))
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut self.selected, Option::A, "A");
        ui.selectable_value(&mut self.selected, Option::B, "B");
    });
```

### Display Widgets

**Labels**:
```rust
ui.label("Simple text");
ui.heading("Heading");
ui.monospace("Monospace");
ui.small("Small text");
```

**Formatted text**:
```rust
use egui::RichText;
ui.label(RichText::new("Bold text").strong());
ui.label(RichText::new("Red text").color(egui::Color32::RED));
ui.label(RichText::new("Large text").size(20.0));
```

**Images**:
```rust
ui.image(egui::include_image!("../assets/icon.png"));
```

**Separators**:
```rust
ui.separator();
```

### Interactive Widgets

**Buttons**:
```rust
if ui.button("Click me").clicked() {
    // Handle click
}

if ui.small_button("Small").clicked() { }
```

**Collapsing sections**:
```rust
ui.collapsing("Advanced", |ui| {
    ui.label("Hidden content");
});
```

**Progress bar**:
```rust
ui.add(egui::ProgressBar::new(self.progress).show_percentage());
```

**Hyperlinks**:
```rust
ui.hyperlink_to("Documentation", "https://docs.rs/egui");
```

## Response Handling

All widgets return a `Response` object with interaction information:

```rust
let response = ui.button("Click");

if response.clicked() { }
if response.double_clicked() { }
if response.hovered() { }
if response.dragged() { }
if response.has_focus() { }

// Tooltip on hover
response.on_hover_text("Hover text");

// Context menu (right-click)
response.context_menu(|ui| {
    if ui.button("Option 1").clicked() { }
});
```

## State Management

### App-Level State

Store state in the app struct:
```rust
struct MyApp {
    counter: i32,
    items: Vec<String>,
}
```

### Persistent State with Context

Use `ctx.data_mut()` or `ctx.memory_mut()` for state that persists between frames but doesn't belong in the app struct:

```rust
// Store window open state
let open = ctx.data_mut(|d| {
    *d.get_temp_mut_or_default(egui::Id::new("window_open"))
});
```

### IDs for Unique Components

Provide unique IDs when you have multiple similar components:
```rust
egui::Window::new("Window")
    .id(egui::Id::new("my_unique_id"))
    .show(ctx, |ui| { });
```

## Styling and Theming

### Modifying Visuals

```rust
// Dark mode
ctx.set_visuals(egui::Visuals::dark());

// Light mode
ctx.set_visuals(egui::Visuals::light());

// Custom colors
let mut visuals = egui::Visuals::dark();
visuals.override_text_color = Some(egui::Color32::WHITE);
ctx.set_visuals(visuals);
```

### Custom Styles

```rust
ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 10.0);
ui.style_mut().visuals.widgets.noninteractive.bg_fill =
    egui::Color32::from_rgb(50, 50, 50);
```

### Per-Widget Styling

```rust
ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::BLUE;
let button = egui::Button::new("Styled")
    .fill(egui::Color32::RED);
ui.add(button);
```

## Advanced Features

### Custom Widgets

Implement the `Widget` trait:
```rust
struct MyWidget {
    value: f32,
}

impl egui::Widget for MyWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(100.0, 20.0),
            egui::Sense::click(),
        );

        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 0.0, egui::Color32::BLUE);
        }

        response
    }
}
```

### Painting Custom Graphics

```rust
let painter = ui.painter();
painter.circle_filled(
    egui::pos2(100.0, 100.0),
    20.0,
    egui::Color32::RED,
);
painter.line_segment(
    [egui::pos2(0.0, 0.0), egui::pos2(100.0, 100.0)],
    egui::Stroke::new(2.0, egui::Color32::WHITE),
);
```

### Frame Pacing and Repaint

```rust
// Request repaint after delay
ctx.request_repaint_after(std::time::Duration::from_millis(16));

// Request repaint immediately (for animations)
ctx.request_repaint();
```

## Integration Patterns

### With Other Game Engines

**Bevy**:
```rust
use bevy_egui::{egui, EguiContexts, EguiPlugin};

app.add_plugins(EguiPlugin);

fn ui_system(mut contexts: EguiContexts) {
    egui::Window::new("Game UI").show(contexts.ctx_mut(), |ui| {
        ui.label("In-game menu");
    });
}
```

**Macroquad**:
```rust
use macroquad::prelude::*;
use egui_macroquad;

#[macroquad::main("egui with macroquad")]
async fn main() {
    loop {
        egui_macroquad::ui(|ctx| {
            egui::Window::new("Game").show(ctx, |ui| {
                ui.label("UI");
            });
        });

        egui_macroquad::draw();
        next_frame().await;
    }
}
```

### WebAssembly Deployment

Build for web:
```bash
cargo install trunk
trunk build --release
trunk serve  # For development
```

Add to `index.html`:
```html
<link data-trunk rel="rust" data-wasm-opt="z"/>
<link data-trunk rel="icon" href="assets/favicon.ico"/>
```

## Common Patterns

### Tab/Page Navigation

```rust
enum Page {
    Home,
    Settings,
    About,
}

struct MyApp {
    current_page: Page,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("nav").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_page, Page::Home, "Home");
                ui.selectable_value(&mut self.current_page, Page::Settings, "Settings");
                ui.selectable_value(&mut self.current_page, Page::About, "About");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Home => ui.heading("Home"),
                Page::Settings => ui.heading("Settings"),
                Page::About => ui.heading("About"),
            };
        });
    }
}
```

### Modal Dialogs

```rust
struct MyApp {
    show_dialog: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.show_dialog {
            egui::Window::new("Confirm")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Are you sure?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            self.show_dialog = false;
                        }
                        if ui.button("No").clicked() {
                            self.show_dialog = false;
                        }
                    });
                });
        }
    }
}
```

### Lists and Tables

```rust
use egui_extras::{TableBuilder, Column};

TableBuilder::new(ui)
    .column(Column::auto())
    .column(Column::remainder())
    .header(20.0, |mut header| {
        header.col(|ui| { ui.heading("Name"); });
        header.col(|ui| { ui.heading("Value"); });
    })
    .body(|mut body| {
        for item in &self.items {
            body.row(20.0, |mut row| {
                row.col(|ui| { ui.label(&item.name); });
                row.col(|ui| { ui.label(&item.value); });
            });
        }
    });
```

## Node Graphs with egui-snarl

See `references/egui_snarl.md` for comprehensive node-graph implementation guide.

Quick example:

```toml
[dependencies]
egui-snarl = "0.8"
```

```rust
use egui_snarl::{Snarl, SnarlViewer, ui::SnarlStyle};

// Define node data type
#[derive(Clone)]
enum NodeData {
    Input(f32),
    Math(MathOp),
    Output,
}

// Implement SnarlViewer trait
impl SnarlViewer<NodeData> for MyViewer {
    fn title(&mut self, node: &NodeData) -> String {
        match node {
            NodeData::Input(_) => "Input".into(),
            NodeData::Math(_) => "Math".into(),
            NodeData::Output => "Output".into(),
        }
    }

    fn inputs(&mut self, node: &NodeData) -> usize {
        match node {
            NodeData::Input(_) => 0,
            NodeData::Math(_) => 2,
            NodeData::Output => 1,
        }
    }

    fn outputs(&mut self, node: &NodeData) -> usize {
        match node {
            NodeData::Input(_) => 1,
            NodeData::Math(_) => 1,
            NodeData::Output => 0,
        }
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut egui::Ui,
                  _scale: f32, snarl: &mut Snarl<NodeData>) -> PinInfo {
        ui.label("In");
        PinInfo::square()
    }

    fn show_output(&mut self, pin: &OutPin, ui: &mut egui::Ui,
                   _scale: f32, snarl: &mut Snarl<NodeData>) -> PinInfo {
        ui.label("Out");
        PinInfo::square()
    }
}

// Usage
let mut snarl = Snarl::new();
snarl.insert_node(egui::pos2(100.0, 100.0), NodeData::Input(0.0));

snarl.show(&mut my_viewer, &SnarlStyle::default(), "snarl", ui);
```

## Performance Tips

- Minimize allocations in hot paths
- Use `ui.ctx().request_repaint()` only when needed
- For large lists, only render visible items in scroll areas
- Cache computed layouts when possible
- Profile with `puffin` profiler integration

## Troubleshooting

### Common Issues

**Window not showing**: Ensure panels/windows are created inside `update()`

**State not persisting**: Store in app struct, not as local variables

**Performance issues**: Use `ui.is_rect_visible()` before painting, limit `request_repaint()` calls

**Text not rendering**: Check that font loading succeeded, use default fonts initially

**Layout flickering**: Expected for first frame of grids/windows, use `ctx.request_discard()` if critical

## Resources

### References
- `references/widgets.md` - Comprehensive widget catalog with examples
- `references/layouts.md` - Advanced layout techniques and patterns
- `references/egui_snarl.md` For detailed node-graph implementation patterns.

### Assets
- `assets/eframe-template/` - Ready-to-use project template for new egui apps

### External Resources
- [Official docs](https://docs.rs/egui)
- [GitHub repository](https://github.com/emilk/egui)
- [Online demo](https://www.egui.rs/)
- [egui Discord](https://discord.gg/JFcEma9bJq)
