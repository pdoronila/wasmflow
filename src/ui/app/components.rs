//! Component loading and management
//!
//! This module handles loading WASM components and managing the component registry.

use super::WasmFlowApp;
use crate::graph::node::ComponentRegistry;
use crate::runtime::wasm_host::ComponentManager;
use crate::runtime::ComponentCache;
use crate::ui::ComponentLoadProgress;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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

/// Async component loader for background thread
///
/// Loads all components from the components directory with caching support.
/// Updates progress atomically as components are loaded.
///
/// # Arguments
///
/// * `components_dir` - Path to the components directory
/// * `registry` - Shared component registry
/// * `component_manager` - Shared component manager
/// * `cache` - Component metadata cache
/// * `progress` - Shared progress tracker
pub fn async_component_loader(
    components_dir: PathBuf,
    registry: Arc<Mutex<ComponentRegistry>>,
    component_manager: Arc<Mutex<ComponentManager>>,
    cache: Arc<ComponentCache>,
    progress: Arc<Mutex<ComponentLoadProgress>>,
) {
    log::info!("Starting async component loading from: {}", components_dir.display());

    // Check if directory exists
    if !components_dir.exists() {
        let mut p = progress.lock().unwrap();
        p.record_error(
            "components/bin".to_string(),
            format!("Directory not found: {}", components_dir.display()),
        );
        log::error!("Components directory not found: {}", components_dir.display());
        return;
    }

    // Scan directory for .wasm files
    let entries = match std::fs::read_dir(&components_dir) {
        Ok(entries) => entries,
        Err(e) => {
            let mut p = progress.lock().unwrap();
            p.record_error(
                "components/bin".to_string(),
                format!("Failed to read directory: {}", e),
            );
            log::error!("Failed to read components directory: {}", e);
            return;
        }
    };

    // Collect all .wasm files
    let wasm_files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("wasm"))
        .collect();

    log::info!("Found {} WASM component files", wasm_files.len());

    // Update total count
    {
        let mut p = progress.lock().unwrap();
        p.total_components = wasm_files.len();
    }

    // Load each component
    for wasm_path in wasm_files {
        let component_name = wasm_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Update current component
        {
            let mut p = progress.lock().unwrap();
            p.set_current(Some(component_name.clone()));
        }

        log::debug!("Loading component: {}", component_name);

        // Try to load from cache first
        let spec = match cache.get_cached_spec(&wasm_path) {
            Ok(Some(spec)) => {
                log::info!("Cache hit: {} (loaded from cache)", component_name);
                Some(spec)
            }
            Ok(None) => {
                log::info!("Cache miss: {} (extracting metadata)", component_name);

                // Load from ComponentManager
                let mut cm = component_manager.lock().unwrap();
                match cm.load_component_sync(&wasm_path) {
                    Ok(spec) => {
                        // Save to cache for next time
                        if let Err(e) = cache.save_spec(&wasm_path, &spec) {
                            log::warn!("Failed to cache {}: {}", component_name, e);
                        }
                        Some(spec)
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to load: {}", e);
                        let mut p = progress.lock().unwrap();
                        p.record_error(component_name.clone(), error_msg);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Cache error for {}: {}", component_name, e);

                // Fallback to direct loading
                let mut cm = component_manager.lock().unwrap();
                match cm.load_component_sync(&wasm_path) {
                    Ok(spec) => Some(spec),
                    Err(e) => {
                        let error_msg = format!("Failed to load: {}", e);
                        let mut p = progress.lock().unwrap();
                        p.record_error(component_name.clone(), error_msg);
                        None
                    }
                }
            }
        };

        // Register component if loaded successfully
        if let Some(spec) = spec {
            let mut reg = registry.lock().unwrap();
            match reg.register_component(spec) {
                Ok(()) => {
                    let mut p = progress.lock().unwrap();
                    p.record_success(component_name);
                }
                Err(e) => {
                    let error_msg = format!("Failed to register: {}", e);
                    let mut p = progress.lock().unwrap();
                    p.record_error(component_name, error_msg);
                }
            }
        }
    }

    // Clear current component
    {
        let mut p = progress.lock().unwrap();
        p.set_current(None);
    }

    log::info!(
        "Component loading complete: {}/{} loaded, {} errors",
        {
            let p = progress.lock().unwrap();
            p.loaded_count - p.errors.len()
        },
        {
            let p = progress.lock().unwrap();
            p.total_components
        },
        {
            let p = progress.lock().unwrap();
            p.errors.len()
        }
    );
}
