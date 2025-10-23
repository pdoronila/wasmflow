use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "egui App",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct MyApp {
    name: String,
    age: u32,
    enabled: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: String::new(),
            age: 0,
            enabled: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel for menu/header
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.label("Theme:");
                    if ui.button("Dark").clicked() {
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                    if ui.button("Light").clicked() {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                });
            });
        });

        // Side panel (optional)
        egui::SidePanel::left("side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Sidebar");
                ui.separator();
                
                ui.label("Some controls:");
                ui.checkbox(&mut self.enabled, "Enable feature");
                
                ui.separator();
                
                if ui.button("Click me!").clicked() {
                    println!("Button clicked!");
                }
            });

        // Central panel (main content area)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to egui!");
            
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            
            if ui.button("Submit").clicked() {
                println!("Name: {}, Age: {}", self.name, self.age);
            }
            
            ui.separator();
            
            ui.label(format!("Hello, {}!", if self.name.is_empty() { "World" } else { &self.name }));
            
            if self.enabled {
                ui.label("Feature is enabled");
            }
            
            ui.separator();
            
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
                });
            });
        });
    }
}
