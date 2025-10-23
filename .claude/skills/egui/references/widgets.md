# Widget Catalog

Complete reference for all egui widgets with practical examples.

## Text and Labels

### Basic Labels
```rust
ui.label("Simple text");
ui.heading("Large heading");
ui.monospace("code_style");
ui.small("Small text");
ui.weak("Weak (gray) text");
```

### Rich Text Formatting
```rust
use egui::RichText;

ui.label(RichText::new("Bold").strong());
ui.label(RichText::new("Italic").italics());
ui.label(RichText::new("Underline").underline());
ui.label(RichText::new("Strikethrough").strikethrough());
ui.label(RichText::new("Red").color(egui::Color32::RED));
ui.label(RichText::new("Large").size(24.0));
ui.label(RichText::new("Monospace").monospace());

// Combine multiple formats
ui.label(
    RichText::new("Bold Red Large")
        .strong()
        .color(egui::Color32::RED)
        .size(20.0)
);
```

### Code Blocks
```rust
ui.code("inline code");

ui.code_editor(&mut self.code_string);
```

## Input Widgets

### Text Input
```rust
// Single line
ui.text_edit_singleline(&mut self.text);

// Multi-line
ui.text_edit_multiline(&mut self.text);

// With hint text
ui.add(
    egui::TextEdit::singleline(&mut self.text)
        .hint_text("Type here...")
);

// Password field
ui.add(
    egui::TextEdit::singleline(&mut self.password)
        .password(true)
);

// Read-only
ui.add(
    egui::TextEdit::multiline(&mut self.text)
        .interactive(false)
);

// With custom width
ui.add(
    egui::TextEdit::singleline(&mut self.text)
        .desired_width(200.0)
);
```

### Numeric Input

#### Sliders
```rust
// Basic slider
ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0));

// With text label
ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text("Volume"));

// Integer slider
ui.add(egui::Slider::new(&mut self.count, 0..=10).text("Count"));

// Logarithmic slider
ui.add(egui::Slider::new(&mut self.value, 0.1..=1000.0).logarithmic(true));

// Custom formatting
ui.add(
    egui::Slider::new(&mut self.value, 0.0..=100.0)
        .text("Percentage")
        .suffix("%")
);
```

#### DragValue
```rust
// Basic drag value
ui.add(egui::DragValue::new(&mut self.value));

// With speed (how fast it changes)
ui.add(egui::DragValue::new(&mut self.value).speed(0.1));

// With range
ui.add(
    egui::DragValue::new(&mut self.value)
        .clamp_range(0.0..=100.0)
);

// With prefix/suffix
ui.add(
    egui::DragValue::new(&mut self.price)
        .prefix("$")
        .suffix(" USD")
);
```

### Checkbox
```rust
ui.checkbox(&mut self.enabled, "Enable feature");

// Without text
ui.add(egui::Checkbox::without_text(&mut self.enabled));
```

### Radio Buttons
```rust
#[derive(PartialEq)]
enum Mode {
    A, B, C
}

ui.radio_value(&mut self.mode, Mode::A, "Mode A");
ui.radio_value(&mut self.mode, Mode::B, "Mode B");
ui.radio_value(&mut self.mode, Mode::C, "Mode C");
```

### Combo Box (Dropdown)
```rust
egui::ComboBox::from_label("Select option")
    .selected_text(format!("{:?}", self.selected))
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut self.selected, Option::A, "Option A");
        ui.selectable_value(&mut self.selected, Option::B, "Option B");
        ui.selectable_value(&mut self.selected, Option::C, "Option C");
    });

// With ID (for multiple combo boxes)
egui::ComboBox::from_id_salt("my_combo")
    .selected_text(format!("{:?}", self.selected))
    .show_ui(ui, |ui| {
        // Options...
    });
```

### Color Picker
```rust
ui.color_edit_button_rgb(&mut self.color);
ui.color_edit_button_rgba_unmultiplied(&mut self.color_with_alpha);

// Custom color picker
egui::color_picker::color_edit_button_srgba(
    ui,
    &mut self.color,
    egui::color_picker::Alpha::Opaque,
);
```

## Buttons

### Basic Buttons
```rust
if ui.button("Click me").clicked() {
    // Handle click
}

// Small button
if ui.small_button("Small").clicked() { }

// Image button
if ui.add(egui::ImageButton::new(egui::include_image!("icon.png"))).clicked() { }

// Button with custom size
if ui.add_sized([100.0, 40.0], egui::Button::new("Wide")).clicked() { }

// Styled button
if ui.add(
    egui::Button::new("Styled")
        .fill(egui::Color32::BLUE)
        .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
).clicked() { }
```

### Links
```rust
// Hyperlink that opens URL
ui.hyperlink("https://example.com");

// Custom text with URL
ui.hyperlink_to("Click here", "https://example.com");

// Just styled as link (no action)
if ui.link("Looks like link").clicked() {
    // Custom action
}
```

### Selectable Labels
```rust
let mut selected = false;
ui.selectable_label(selected, "Selectable");

// With state
ui.selectable_value(&mut self.choice, Choice::A, "Choice A");
```

## Display Widgets

### Images
```rust
// From embedded asset
ui.image(egui::include_image!("../assets/image.png"));

// With custom size
ui.add(
    egui::Image::new(egui::include_image!("../assets/image.png"))
        .max_width(200.0)
);

// From bytes
let texture: egui::TextureHandle = ui.ctx().load_texture(
    "my-image",
    egui::ColorImage::example(),
    Default::default()
);
ui.image(&texture);
```

### Progress Bar
```rust
ui.add(egui::ProgressBar::new(self.progress));

// With percentage text
ui.add(egui::ProgressBar::new(self.progress).show_percentage());

// Custom text
ui.add(egui::ProgressBar::new(self.progress).text("Loading..."));
```

### Spinner
```rust
ui.spinner();

// Custom size
ui.add(egui::Spinner::new().size(32.0));
```

### Separator
```rust
ui.separator();

// With spacing
ui.add_space(5.0);
ui.separator();
ui.add_space(5.0);
```

## Container Widgets

### Collapsing Header
```rust
ui.collapsing("Advanced Settings", |ui| {
    ui.label("Hidden content");
    ui.checkbox(&mut self.option, "Option");
});

// Initially open
ui.collapsing("Open by default", |ui| {
    ui.label("Content");
}).open(Some(true));

// With header response
let header = ui.collapsing("Header", |ui| {
    ui.label("Content");
});
if header.header_response.clicked() {
    println!("Header clicked");
}
```

### Groups
```rust
ui.group(|ui| {
    ui.label("Content in a frame");
    ui.button("Button");
});

// Vertical group
ui.vertical(|ui| {
    ui.label("Line 1");
    ui.label("Line 2");
});

// Horizontal group
ui.horizontal(|ui| {
    ui.label("Left");
    ui.label("Right");
});
```

### Scroll Areas
```rust
egui::ScrollArea::vertical().show(ui, |ui| {
    for i in 0..100 {
        ui.label(format!("Line {}", i));
    }
});

// Both directions
egui::ScrollArea::both().show(ui, |ui| {
    // Large content
});

// Auto-shrink
egui::ScrollArea::vertical()
    .auto_shrink([false; 2])
    .show(ui, |ui| {
        // Content
    });

// With max height
egui::ScrollArea::vertical()
    .max_height(200.0)
    .show(ui, |ui| {
        // Content
    });
```

### Frames
```rust
egui::Frame::none()
    .fill(egui::Color32::DARK_GRAY)
    .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
    .inner_margin(10.0)
    .show(ui, |ui| {
        ui.label("Framed content");
    });
```

## Advanced Widgets

### Menu Bar
```rust
egui::menu::bar(ui, |ui| {
    ui.menu_button("File", |ui| {
        if ui.button("Open").clicked() {
            // Handle open
        }
        if ui.button("Save").clicked() {
            // Handle save
        }
    });
    
    ui.menu_button("Edit", |ui| {
        if ui.button("Copy").clicked() { }
        if ui.button("Paste").clicked() { }
    });
});
```

### Context Menu
```rust
let response = ui.button("Right-click me");
response.context_menu(|ui| {
    if ui.button("Option 1").clicked() {
        ui.close_menu();
    }
    if ui.button("Option 2").clicked() {
        ui.close_menu();
    }
});
```

### Tooltips
```rust
ui.button("Hover me").on_hover_text("Tooltip text");

// Rich tooltip
ui.button("Hover").on_hover_ui(|ui| {
    ui.heading("Tooltip heading");
    ui.label("More detailed information");
});

// Custom position
ui.button("Hover").on_hover_text_at_pointer("At cursor");
```

### Resize Handle
```rust
let mut size = ui.available_size();
ui.allocate_ui(size, |ui| {
    ui.label("Resizable content");
});

// Handle in corner
if ui.add(egui::widgets::ResizeHandle::new()).dragged() {
    // Handle resize
}
```

## Plotting (requires egui_plot)

```rust
use egui_plot::{Line, Plot, PlotPoints};

let sin = (0..1000).map(|i| {
    let x = i as f64 * 0.01;
    [x, x.sin()]
});

Plot::new("my_plot")
    .view_aspect(2.0)
    .show(ui, |plot_ui| {
        plot_ui.line(Line::new(PlotPoints::from_iter(sin)));
    });
```

## Tables (requires egui_extras)

```rust
use egui_extras::{TableBuilder, Column};

TableBuilder::new(ui)
    .striped(true)
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
                row.col(|ui| { ui.label(&item.value.to_string()); });
            });
        }
    });
```
