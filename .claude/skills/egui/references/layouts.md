# Advanced Layout Techniques

Comprehensive guide to layout patterns and techniques in egui.

## Layout Fundamentals

### Direction and Alignment
```rust
// Left-to-right (default)
ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
    ui.button("Left");
    ui.button("Right");
});

// Right-to-left
ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
    ui.button("Settings");
    ui.button("Profile");
});

// Top-to-bottom (default for vertical)
ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    ui.button("Top");
    ui.button("Bottom");
});

// Bottom-to-top
ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
    ui.button("Footer Item 1");
    ui.button("Footer Item 2");
});
```

### Alignment Options
```rust
// Horizontal alignment
egui::Align::Min     // Left
egui::Align::Center  // Center
egui::Align::Max     // Right

// Vertical alignment
egui::Align::Min     // Top
egui::Align::Center  // Middle
egui::Align::Max     // Bottom
```

## Common Layout Patterns

### Split Panel (Left/Right)
```rust
egui::SidePanel::left("left_panel")
    .min_width(200.0)
    .max_width(400.0)
    .resizable(true)
    .show(ctx, |ui| {
        ui.heading("Sidebar");
        // Left content
    });

egui::CentralPanel::default().show(ctx, |ui| {
    ui.heading("Main Content");
    // Right content
});
```

### Header and Footer
```rust
egui::TopBottomPanel::top("header")
    .show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("App Title");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Settings").clicked() { }
                if ui.button("Help").clicked() { }
            });
        });
    });

egui::TopBottomPanel::bottom("footer")
    .show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("© 2025");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("Privacy", "https://example.com/privacy");
                ui.hyperlink_to("Terms", "https://example.com/terms");
            });
        });
    });

egui::CentralPanel::default().show(ctx, |ui| {
    // Main content
});
```

### Three-Column Layout
```rust
ui.columns(3, |columns| {
    columns[0].label("Left Column");
    columns[0].button("Button 1");
    
    columns[1].label("Middle Column");
    columns[1].button("Button 2");
    
    columns[2].label("Right Column");
    columns[2].button("Button 3");
});
```

### Card/Panel Grid
```rust
let num_columns = 3;
let spacing = 10.0;

egui::Grid::new("card_grid")
    .num_columns(num_columns)
    .spacing([spacing, spacing])
    .show(ui, |ui| {
        for (i, item) in self.items.iter().enumerate() {
            ui.group(|ui| {
                ui.set_min_size(egui::vec2(150.0, 100.0));
                ui.vertical_centered(|ui| {
                    ui.heading(&item.title);
                    ui.label(&item.description);
                    if ui.button("Action").clicked() {
                        // Handle click
                    }
                });
            });
            
            if (i + 1) % num_columns == 0 {
                ui.end_row();
            }
        }
    });
```

## Grid Layouts

### Basic Grid
```rust
egui::Grid::new("my_grid")
    .num_columns(2)
    .spacing([40.0, 4.0])
    .striped(true)
    .show(ui, |ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut self.name);
        ui.end_row();
        
        ui.label("Age:");
        ui.add(egui::DragValue::new(&mut self.age));
        ui.end_row();
        
        ui.label("Email:");
        ui.text_edit_singleline(&mut self.email);
        ui.end_row();
    });
```

### Form Layout
```rust
egui::Grid::new("form")
    .num_columns(2)
    .spacing([10.0, 10.0])
    .min_col_width(100.0)
    .show(ui, |ui| {
        // Label column auto-sizes
        ui.label("Username:");
        ui.text_edit_singleline(&mut self.username);
        ui.end_row();
        
        ui.label("Password:");
        ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
        ui.end_row();
        
        ui.label("Remember me:");
        ui.checkbox(&mut self.remember, "");
        ui.end_row();
        
        // Span entire row for submit button
        ui.label("");
        if ui.button("Login").clicked() {
            // Handle login
        }
        ui.end_row();
    });
```

### Table-like Grid
```rust
egui::Grid::new("table")
    .striped(true)
    .min_col_width(100.0)
    .show(ui, |ui| {
        // Header
        ui.heading("Name");
        ui.heading("Age");
        ui.heading("City");
        ui.end_row();
        
        // Data rows
        for person in &self.people {
            ui.label(&person.name);
            ui.label(person.age.to_string());
            ui.label(&person.city);
            ui.end_row();
        }
    });
```

## Spacing and Sizing

### Manual Spacing
```rust
// Vertical space
ui.add_space(10.0);

// Horizontal space
ui.horizontal(|ui| {
    ui.label("Left");
    ui.add_space(50.0);
    ui.label("Right");
});

// Adjust spacing globally
ui.spacing_mut().item_spacing = egui::vec2(8.0, 4.0);
ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0);
```

### Widget Sizing
```rust
// Fixed size
ui.add_sized([200.0, 30.0], egui::Button::new("Fixed Size"));

// Minimum size
ui.set_min_size(egui::vec2(300.0, 200.0));

// Maximum size
ui.set_max_size(egui::vec2(500.0, 400.0));

// Available space
let size = ui.available_size();
ui.allocate_space(size);
```

### Responsive Width
```rust
// Fill available width
ui.horizontal(|ui| {
    ui.label("Label:");
    ui.add_sized(
        [ui.available_width(), 20.0],
        egui::TextEdit::singleline(&mut self.text)
    );
});

// Percentage-based
let width = ui.available_width() * 0.5;
ui.add_sized([width, 30.0], egui::Button::new("50% Width"));
```

## Advanced Patterns

### Centered Content
```rust
ui.with_layout(
    egui::Layout::top_down(egui::Align::Center),
    |ui| {
        ui.heading("Centered Heading");
        ui.label("Centered text");
        if ui.button("Centered Button").clicked() { }
    }
);

// Or use vertical_centered
ui.vertical_centered(|ui| {
    ui.heading("Centered Content");
});
```

### Justified Layout (Space Between)
```rust
ui.horizontal(|ui| {
    ui.label("Left");
    
    // Push content to the right
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button("Right Button").clicked() { }
        ui.label("Right Label");
    });
});
```

### Expanding Spacer
```rust
ui.horizontal(|ui| {
    ui.button("Left");
    
    // Allocate all remaining space
    ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
    
    ui.button("Right");
});
```

### Multi-line Horizontal
```rust
ui.horizontal_wrapped(|ui| {
    for i in 0..20 {
        ui.button(format!("Button {}", i));
    }
});
```

### Nested Layouts
```rust
ui.horizontal(|ui| {
    ui.vertical(|ui| {
        ui.label("Left Panel");
        ui.separator();
        for i in 0..5 {
            ui.button(format!("Item {}", i));
        }
    });
    
    ui.separator();
    
    ui.vertical(|ui| {
        ui.label("Right Panel");
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..20 {
                ui.label(format!("Row {}", i));
            }
        });
    });
});
```

### Collapsible Sidebar
```rust
struct App {
    sidebar_open: bool,
}

// Toggle button in header
egui::TopBottomPanel::top("header").show(ctx, |ui| {
    ui.horizontal(|ui| {
        if ui.button("☰").clicked() {
            self.sidebar_open = !self.sidebar_open;
        }
        ui.heading("App Title");
    });
});

// Conditional sidebar
if self.sidebar_open {
    egui::SidePanel::left("sidebar")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Sidebar");
            // Sidebar content
        });
}

egui::CentralPanel::default().show(ctx, |ui| {
    // Main content
});
```

## Scrollable Regions

### Vertical Scroll with Header
```rust
ui.vertical(|ui| {
    // Fixed header
    ui.heading("List Header");
    ui.separator();
    
    // Scrollable content
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for i in 0..100 {
                ui.label(format!("Item {}", i));
            }
        });
});
```

### Horizontal Scroll
```rust
egui::ScrollArea::horizontal().show(ui, |ui| {
    ui.horizontal(|ui| {
        for i in 0..50 {
            ui.button(format!("Button {}", i));
        }
    });
});
```

### Scroll to Item
```rust
let mut scroll_to = None;

egui::ScrollArea::vertical()
    .id_salt("my_scroll_area")
    .show(ui, |ui| {
        for i in 0..100 {
            if ui.button(format!("Item {}", i)).clicked() {
                scroll_to = Some(i);
            }
        }
    });

// Scroll to item
if let Some(index) = scroll_to {
    ui.ctx().animate_value_with_time(
        egui::Id::new("scroll_offset"),
        index as f32 * 30.0,
        0.3
    );
}
```

## Dynamic Layouts

### Responsive Grid Columns
```rust
let panel_width = ui.available_width();
let min_item_width = 150.0;
let num_columns = (panel_width / min_item_width).floor().max(1.0) as usize;

egui::Grid::new("responsive_grid")
    .num_columns(num_columns)
    .show(ui, |ui| {
        for (i, item) in self.items.iter().enumerate() {
            ui.group(|ui| {
                ui.label(&item.name);
            });
            
            if (i + 1) % num_columns == 0 {
                ui.end_row();
            }
        }
    });
```

### Conditional Layout
```rust
let wide_layout = ui.available_width() > 800.0;

if wide_layout {
    // Wide layout: side-by-side
    ui.horizontal(|ui| {
        ui.group(|ui| { /* Left content */ });
        ui.group(|ui| { /* Right content */ });
    });
} else {
    // Narrow layout: stacked
    ui.vertical(|ui| {
        ui.group(|ui| { /* Top content */ });
        ui.group(|ui| { /* Bottom content */ });
    });
}
```

## Custom Allocations

### Manual Positioning
```rust
let (rect, response) = ui.allocate_exact_size(
    egui::vec2(100.0, 50.0),
    egui::Sense::click()
);

if ui.is_rect_visible(rect) {
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        5.0,
        egui::Color32::BLUE
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Custom",
        egui::FontId::default(),
        egui::Color32::WHITE,
    );
}

if response.clicked() {
    // Handle click
}
```

### Absolute Positioning
```rust
egui::Area::new("floating_element")
    .fixed_pos(egui::pos2(100.0, 100.0))
    .show(ctx, |ui| {
        ui.label("Absolutely positioned");
    });
```

### Layered Content
```rust
// Background layer
ui.painter().rect_filled(
    ui.max_rect(),
    0.0,
    egui::Color32::from_gray(30)
);

// Content layer
ui.vertical_centered(|ui| {
    ui.heading("Layered Content");
});
```
