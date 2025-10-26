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

    /// Start async component reload with progress tracking
    ///
    /// Shows a splash screen and loads components in the background.
    /// Similar to startup loading but for manual reload.
    pub(super) fn reload_components(&mut self) {
        use crate::ui::ComponentLoadProgress;
        use std::sync::{Arc, Mutex};

        log::info!("Starting async component reload");

        // Clear existing components from ComponentManager for a clean reload
        let component_manager = self.engine.component_manager();
        {
            let mut cm = component_manager.lock().unwrap();
            cm.clear_all_components();
        }

        // Create progress tracker
        let progress = Arc::new(Mutex::new(ComponentLoadProgress::new()));

        // Create splash screen
        self.splash_screen = Some(crate::ui::SplashScreen::new(progress.clone()));

        // Create a new registry for the background thread
        let registry = Arc::new(Mutex::new(ComponentRegistry::new()));

        // Update loading state to show we're reloading
        self.loading_state = crate::ui::LoadingState::Loading {
            progress: progress.clone(),
            registry: registry.clone(),
        };

        // Create cache
        let cache_dir = std::path::PathBuf::from("components/bin/.cache");
        let cache = match ComponentCache::new(cache_dir) {
            Ok(cache) => Arc::new(cache),
            Err(e) => {
                log::error!("Failed to create component cache: {}", e);
                self.loading_state = crate::ui::LoadingState::Failed {
                    error: format!("Failed to create component cache: {}", e),
                };
                self.splash_screen = None;
                return;
            }
        };

        // Clone for thread
        let registry_clone = registry.clone();
        let progress_clone = progress.clone();

        // Spawn loading thread
        std::thread::spawn(move || {
            async_component_loader(
                std::path::PathBuf::from("components/bin"),
                registry_clone,
                component_manager,
                cache,
                progress_clone,
            );
        });

        // Note: The main registry will be updated in poll_loading_progress() when complete
    }

    /// Reload all components synchronously (legacy, kept for compatibility)
    #[allow(dead_code)]
    pub(super) fn reload_components_sync(&mut self) {
        let components_dir = std::path::Path::new("components/bin");

        if !components_dir.exists() {
            self.error_message = Some("Components directory not found: components/bin/. Create this directory and place .wasm component files there.".to_string());
            return;
        }

        let mut loaded_count = 0;
        let mut error_count = 0;
        let mut cache_hits = 0;

        // Create cache (same as async loading)
        let cache_dir = std::path::PathBuf::from("components/bin/.cache");
        let cache = match ComponentCache::new(cache_dir) {
            Ok(cache) => Some(cache),
            Err(e) => {
                log::warn!("Failed to create component cache, loading without cache: {}", e);
                None
            }
        };

        // Get the engine's component manager
        let component_manager = self.engine.component_manager();

        // Scan for .wasm files
        if let Ok(entries) = std::fs::read_dir(components_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    let component_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Try to load with cache (if available)
                    let spec = if let Some(ref cache) = cache {
                        match load_component_with_cache(&path, &component_manager, cache) {
                            Ok((spec, from_cache)) => {
                                if from_cache {
                                    cache_hits += 1;
                                    log::debug!("Cache hit: {}", component_name);
                                } else {
                                    log::debug!("Cache miss: {}", component_name);
                                }
                                Some(spec)
                            }
                            Err(e) => {
                                error_count += 1;
                                log::warn!("Failed to load component {}: {}", path.display(), e);
                                None
                            }
                        }
                    } else {
                        // Fallback to direct loading without cache
                        let mut cm = component_manager.lock().unwrap();
                        match cm.load_component_sync(&path) {
                            Ok(spec) => Some(spec),
                            Err(e) => {
                                error_count += 1;
                                log::warn!("Failed to load component {}: {}", path.display(), e);
                                None
                            }
                        }
                    };

                    // Register component if loaded successfully
                    if let Some(component_spec) = spec {
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
                }
            }
        }

        if error_count > 0 {
            self.status_message = format!(
                "Loaded {} components ({} errors, {} cached)",
                loaded_count, error_count, cache_hits
            );
        } else if loaded_count > 0 {
            self.status_message = format!(
                "Loaded {} components ({} cached)",
                loaded_count, cache_hits
            );
        } else {
            self.status_message = "No components found in components/bin/ directory".to_string();
        }

        self.error_message = None;
    }
}

/// Load a component with caching support
///
/// Returns (ComponentSpec, from_cache: bool)
/// - from_cache=true means loaded from cache (fast, no metadata extraction)
/// - from_cache=false means extracted metadata and updated cache
fn load_component_with_cache(
    wasm_path: &std::path::Path,
    component_manager: &Arc<Mutex<ComponentManager>>,
    cache: &ComponentCache,
) -> Result<(crate::graph::node::ComponentSpec, bool), String> {
    match cache.get_cached_spec(wasm_path) {
        Ok(Some(spec)) => {
            // Cache hit - only need to load bytecode (fast, no metadata extraction)
            let mut cm = component_manager.lock().map_err(|e| {
                format!("Failed to lock ComponentManager: {}", e)
            })?;

            // Load bytecode only (no metadata extraction needed)
            cm.load_bytecode_only(wasm_path, &spec.id).map_err(|e| {
                format!("Failed to load component bytecode: {}", e)
            })?;

            Ok((spec, true))
        }
        Ok(None) => {
            // Cache miss - extract metadata and save to cache
            let mut cm = component_manager.lock().map_err(|e| {
                format!("Failed to lock ComponentManager: {}", e)
            })?;

            let spec = cm.load_component_sync(wasm_path).map_err(|e| {
                format!("Failed to load component: {}", e)
            })?;

            // Save to cache for next time
            if let Err(e) = cache.save_spec(wasm_path, &spec) {
                log::warn!(
                    "Failed to cache {}: {}",
                    wasm_path.display(),
                    e
                );
            }

            Ok((spec, false))
        }
        Err(e) => {
            log::warn!("Cache error for {}: {}", wasm_path.display(), e);

            // Fallback to direct loading
            let mut cm = component_manager.lock().map_err(|e| {
                format!("Failed to lock ComponentManager: {}", e)
            })?;

            let spec = cm.load_component_sync(wasm_path).map_err(|e| {
                format!("Failed to load component: {}", e)
            })?;

            Ok((spec, false))
        }
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
    log::info!(
        "Starting async component loading from: {}",
        components_dir.display()
    );

    // Check if directory exists
    if !components_dir.exists() {
        let mut p = progress.lock().unwrap();
        p.record_error(
            "components/bin".to_string(),
            format!("Directory not found: {}", components_dir.display()),
        );
        log::error!(
            "Components directory not found: {}",
            components_dir.display()
        );
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

    // Debug: List all files we found
    for (i, path) in wasm_files.iter().enumerate() {
        log::debug!("  [{}] {}", i + 1, path.display());
    }

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

        // Try to load with cache using helper function
        log::debug!("Attempting to load: {}", wasm_path.display());
        let spec = match load_component_with_cache(&wasm_path, &component_manager, &cache) {
            Ok((spec, from_cache)) => {
                if from_cache {
                    log::info!("✓ Cache hit: {} (loaded from cache)", component_name);
                } else {
                    log::info!("✓ Cache miss: {} (extracting metadata)", component_name);
                }
                log::debug!("  Component ID: {}", spec.id);
                Some(spec)
            }
            Err(e) => {
                let error_msg = format!("Failed to load: {}", e);
                log::error!("✗ Failed to load {}: {}", component_name, e);
                let mut p = progress.lock().unwrap();
                p.record_error(component_name.clone(), error_msg);
                None
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
