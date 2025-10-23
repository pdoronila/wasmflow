//! Component loading and management
//!
//! This module handles loading WASM components and managing the component registry.

use super::WasmFlowApp;

impl WasmFlowApp {
    /// Load a custom WASM component
    pub(super) fn load_component(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WebAssembly Component", &["wasm"])
            .pick_file()
        {
            // Get the engine's component manager
            let component_manager = self.engine.component_manager();
            let mut cm = component_manager.lock().unwrap();

            // Load component and register with registry
            match cm.load_component_sync(&path) {
                Ok(component_spec) => {
                    let component_name = component_spec.name.clone();

                    // Register with the component registry
                    match self.registry.register_component(component_spec) {
                        Ok(()) => {
                            self.status_message = format!("Loaded component: {}", component_name);
                            self.error_message = None;
                        }
                        Err(e) => {
                            self.error_message =
                                Some(format!("Failed to register component: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load component: {}", e));
                }
            }
        }
    }

    /// Reload all components from the components/ directory
    pub(super) fn reload_components(&mut self) {
        let components_dir = std::path::Path::new("components/bin");

        if !components_dir.exists() {
            self.error_message = Some("Components directory not found: components/bin/. Create this directory and place .wasm component files there.".to_string());
            return;
        }

        let mut loaded_count = 0;
        let mut error_count = 0;

        // Get the engine's component manager
        let component_manager = self.engine.component_manager();
        let mut cm = component_manager.lock().unwrap();

        // Scan for .wasm files
        if let Ok(entries) = std::fs::read_dir(components_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    // Load component and register with registry
                    match cm.load_component_sync(&path) {
                        Ok(component_spec) => {
                            // Register with the component registry
                            match self.registry.register_component(component_spec) {
                                Ok(()) => {
                                    loaded_count += 1;
                                }
                                Err(e) => {
                                    error_count += 1;
                                    log::warn!(
                                        "Failed to register component {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error_count += 1;
                            log::warn!("Failed to load component {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        if error_count > 0 {
            self.status_message = format!(
                "Loaded {} components ({} errors)",
                loaded_count, error_count
            );
        } else if loaded_count > 0 {
            self.status_message = format!("Loaded {} components", loaded_count);
        } else {
            self.status_message = "No components found in components/bin/ directory".to_string();
        }

        self.error_message = None;
    }
}
