# egui App Template

A minimal but functional egui application template using eframe.

## Features

- Menu bar with File and View menus
- Side panel with controls
- Central content area
- Theme switching (dark/light)
- Example widgets: text input, slider, buttons, checkboxes

## Building and Running

### Development
```bash
cargo run
```

### Release
```bash
cargo run --release
```

## Project Structure

```
egui-app/
├── Cargo.toml          # Dependencies and project configuration
└── src/
    └── main.rs         # Main application code
```

## Customization

### Adding Dependencies

Edit `Cargo.toml` to add more egui crates:

```toml
[dependencies]
eframe = "0.29"
egui = "0.29"
egui_extras = { version = "0.29", features = ["all_loaders"] }  # For images, tables
egui_plot = "0.29"                                               # For plotting
```

### Modifying the App

The `MyApp` struct holds your application state:

```rust
struct MyApp {
    // Add your state fields here
    name: String,
    age: u32,
    // ...
}
```

The `update` method is called every frame:

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Add your UI code here
    }
}
```

## WebAssembly Support

To compile for web, install trunk and build:

```bash
cargo install trunk
trunk build --release
trunk serve  # For development server
```

## Resources

- [egui Documentation](https://docs.rs/egui)
- [eframe Documentation](https://docs.rs/eframe)
- [egui GitHub](https://github.com/emilk/egui)
- [Examples](https://github.com/emilk/egui/tree/master/examples)
