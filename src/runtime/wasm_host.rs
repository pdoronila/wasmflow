//! WASM Component Host
//!
//! This module implements the WebAssembly component runtime using wasmtime.
//! It provides:
//! - Component loading and validation
//! - Host function implementations (logging, temp directory access)
//! - WASI context configuration based on capability grants
//! - Component metadata extraction

use crate::graph::node::{ComponentSpec, NodeValue};
use crate::runtime::capabilities::{Capability, CapabilitySet};
use crate::ui::wit_ui_renderer::{
    ColoredText, FooterView as WitFooterView, HorizontalLayout, KeyValuePair, UiElement,
    UiElementItem, VerticalLayout, WitFooterViewAdapter,
};
use crate::ComponentError;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wasmtime::component::{Component as WasmComponent, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

// Generate bindings from WIT files - base component world
wasmtime::component::bindgen!({
    path: "wit",
    world: "component",
    async: true,
});

// Generate bindings for component-with-ui world in a separate module
pub mod with_ui {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "component-with-ui",
        async: true,
    });
}

/// Maximum WASM component file size (50MB)
const MAX_COMPONENT_SIZE: u64 = 50 * 1024 * 1024;

/// Maximum memory limit per component (500MB)
#[allow(dead_code)]
const MAX_MEMORY_BYTES: usize = 500 * 1024 * 1024;

/// Host state for component execution
pub struct HostState {
    /// WASI context for filesystem/network access
    pub wasi: WasiCtx,
    /// WASI HTTP context for HTTP client functionality
    pub http: WasiHttpCtx,
    /// Resource table for WASI resources
    pub table: ResourceTable,
    /// Component ID for logging
    pub component_id: String,
    /// Granted capabilities
    pub capabilities: CapabilitySet,
}

impl HostState {
    /// Create new host state with specified capabilities
    pub fn new(component_id: String, capabilities: CapabilitySet) -> Result<Self> {
        let mut builder = WasiCtxBuilder::new();

        // Configure WASI based on granted capabilities
        Self::configure_wasi(&mut builder, &capabilities)?;

        let wasi = builder.build();
        let http = WasiHttpCtx::new();
        let table = ResourceTable::new();

        Ok(Self {
            wasi,
            http,
            table,
            component_id,
            capabilities,
        })
    }

    /// Configure WASI context based on capability set
    /// T067-T070: Implementation of WASI context configuration
    fn configure_wasi(builder: &mut WasiCtxBuilder, capabilities: &CapabilitySet) -> Result<()> {
        // Always inherit stdout/stderr for host::log() functionality
        builder.inherit_stdout().inherit_stderr();

        // Configure based on capability set variant
        match capabilities {
            CapabilitySet::None => {
                // No additional capabilities - pure computation only
            }
            CapabilitySet::FileRead { paths }
            | CapabilitySet::FileWrite { paths }
            | CapabilitySet::FileReadWrite { paths } => {
                // T068-T069: File capability enforcement
                use wasmtime_wasi::{DirPerms, FilePerms};

                for path in paths {
                    if !path.is_absolute() {
                        anyhow::bail!("Path must be absolute: {:?}", path);
                    }
                    let path_str = path
                        .to_str()
                        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path"))?;

                    // Note: For now using all() permissions
                    // TODO: Differentiate between FileRead, FileWrite, and FileReadWrite
                    builder.preopened_dir(
                        path.clone(),
                        path_str,
                        DirPerms::all(),
                        FilePerms::all(),
                    )?;
                }
            }
            CapabilitySet::Network { allowed_hosts: _ } => {
                // T070: Network capability enforcement
                // Note: Host allowlisting is enforced at the WASI outgoing handler level
                // For now, we inherit network access
                // TODO: Implement custom outgoing handler to validate allowed_hosts
                builder.inherit_network();
            }
            CapabilitySet::Full => {
                // Full access - inherit everything
                builder.inherit_stdio().inherit_env().inherit_network();
            }
        }

        Ok(())
    }

    /// Log a message from the component
    pub fn log(&mut self, level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        match level {
            "error" => log::error!("[{}] [{}] {}", timestamp, self.component_id, message),
            "warn" => log::warn!("[{}] [{}] {}", timestamp, self.component_id, message),
            "info" => log::info!("[{}] [{}] {}", timestamp, self.component_id, message),
            "debug" => log::debug!("[{}] [{}] {}", timestamp, self.component_id, message),
            _ => log::info!("[{}] [{}] {}", timestamp, self.component_id, message),
        }
    }

    /// Get temporary directory path
    pub fn get_temp_dir(&self) -> Result<String, String> {
        // Check if component has permission for temp storage
        // For now, we'll allow temp access if the component has any file write capability
        if !self.capabilities.has(Capability::FileWrite) {
            return Err("Component does not have permission for temporary storage".to_string());
        }

        std::env::temp_dir()
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Failed to get temp directory".to_string())
    }
}

impl WasiView for HostState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for HostState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

// Implement the Host trait from the generated WIT bindings
impl wasmflow::node::host::Host for HostState {
    fn log<'life0, 'async_trait>(
        &'life0 mut self,
        level: String,
        message: String,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            HostState::log(self, &level, &message);
        })
    }

    fn get_temp_dir<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<String, String>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { HostState::get_temp_dir(self) })
    }
}

// Also implement for component-with-ui bindings
impl with_ui::wasmflow::node::host::Host for HostState {
    fn log<'life0, 'async_trait>(
        &'life0 mut self,
        level: String,
        message: String,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            HostState::log(self, &level, &message);
        })
    }

    fn get_temp_dir<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<String, String>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { HostState::get_temp_dir(self) })
    }
}

/// Maximum number of compiled modules to cache (T083: LRU eviction)
const MAX_COMPILED_MODULES: usize = 50;

/// T083: Component data for lazy compilation
struct ComponentData {
    /// Component bytecode (always stored)
    bytecode: Arc<Vec<u8>>,
    /// Cached compiled component (lazily compiled)
    compiled: Option<Arc<WasmComponent>>,
}

/// Component manager for loading and executing WASM components
pub struct ComponentManager {
    /// Wasmtime engine (shared across all components)
    engine: Engine,
    /// Linker for host functions
    linker: Arc<Mutex<Linker<HostState>>>,
    /// T083: Component data with lazy compilation
    components: HashMap<String, ComponentData>,
    /// T083: LRU cache for tracking compilation order
    lru_order: Vec<String>,
}

impl ComponentManager {
    /// Create a new component manager
    pub fn new() -> Result<Self> {
        // Configure wasmtime engine
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);

        // Set memory limits to handle large data (e.g., HTTP responses)
        config.max_wasm_stack(2 * 1024 * 1024); // 2MB stack
        // Note: Linear memory limit is per-instance and controlled by WASM module itself

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);

        // Add WASI imports
        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        // T010: Add WASI HTTP support
        wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;

        // Add custom host functions
        Self::add_host_functions(&mut linker)?;

        Ok(Self {
            engine,
            linker: Arc::new(Mutex::new(linker)),
            components: HashMap::new(),
            lru_order: Vec::new(),
        })
    }

    /// Add custom host functions to the linker
    fn add_host_functions(linker: &mut Linker<HostState>) -> Result<()> {
        // Add WasmFlow host functions using generated bindings
        // Both component and component-with-ui worlds import the same host interface,
        // so we only need to add it once
        wasmflow::node::host::add_to_linker(linker, |state| state)?;

        Ok(())
    }

    /// T083: Load a component from a file (lazy compilation)
    /// Stores bytecode, defers compilation until first execution
    pub async fn load_component(&mut self, path: &Path) -> Result<ComponentSpec, ComponentError> {
        // Validate file size
        let metadata = std::fs::metadata(path).map_err(|e| ComponentError::LoadFailed {
            path: path.to_path_buf(),
            reason: format!("Failed to read component metadata: {}", e),
        })?;

        if metadata.len() > MAX_COMPONENT_SIZE {
            return Err(ComponentError::ValidationFailed(format!(
                "Component file too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_COMPONENT_SIZE
            )));
        }

        // T083: Read bytecode instead of compiling immediately
        let bytecode = std::fs::read(path).map_err(|e| ComponentError::LoadFailed {
            path: path.to_path_buf(),
            reason: format!("Failed to read component file: {}", e),
        })?;

        // Create component spec from basic metadata
        let component_spec = self.create_basic_spec_from_bytecode(&bytecode, path)?;

        // T083: Store bytecode only, defer compilation
        self.components.insert(
            component_spec.id.clone(),
            ComponentData {
                bytecode: Arc::new(bytecode),
                compiled: None,
            },
        );

        log::debug!("Loaded component bytecode (lazy): {}", component_spec.id);
        Ok(component_spec)
    }

    /// T083: Load a component synchronously (for UI integration)
    /// Stores bytecode, defers compilation until first execution
    pub fn load_component_sync(&mut self, path: &Path) -> Result<ComponentSpec, ComponentError> {
        // Validate file size
        let metadata = std::fs::metadata(path).map_err(|e| ComponentError::LoadFailed {
            path: path.to_path_buf(),
            reason: format!("Failed to read component metadata: {}", e),
        })?;

        if metadata.len() > MAX_COMPONENT_SIZE {
            return Err(ComponentError::ValidationFailed(format!(
                "Component file too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_COMPONENT_SIZE
            )));
        }

        // T083: Read bytecode instead of compiling immediately
        let bytecode = std::fs::read(path).map_err(|e| ComponentError::LoadFailed {
            path: path.to_path_buf(),
            reason: format!("Failed to read component file: {}", e),
        })?;

        // Create component spec from basic metadata
        let component_spec = self.create_basic_spec_from_bytecode(&bytecode, path)?;

        // T083: Store bytecode only, defer compilation
        self.components.insert(
            component_spec.id.clone(),
            ComponentData {
                bytecode: Arc::new(bytecode),
                compiled: None,
            },
        );

        log::debug!(
            "Loaded component bytecode (lazy, sync): {}",
            component_spec.id
        );
        Ok(component_spec)
    }

    /// T083: Create a component spec by extracting metadata from the component
    fn create_basic_spec_from_bytecode(
        &self,
        bytecode: &[u8],
        path: &Path,
    ) -> Result<ComponentSpec, ComponentError> {
        // Compile component to extract metadata
        let component = WasmComponent::from_binary(&self.engine, bytecode).map_err(|e| {
            ComponentError::LoadFailed {
                path: path.to_path_buf(),
                reason: format!("Failed to compile component for metadata extraction: {}", e),
            }
        })?;

        // Create a minimal host state for metadata extraction (no capabilities needed)
        let host_state = HostState::new("metadata-extraction".to_string(), CapabilitySet::none())
            .map_err(|e| ComponentError::LoadFailed {
            path: path.to_path_buf(),
            reason: format!("Failed to create host state for metadata: {}", e),
        })?;

        let mut store = Store::new(&self.engine, host_state);

        // Instantiate component
        // Clone linker to avoid holding MutexGuard across await
        let linker = {
            let guard = self.linker.lock().unwrap();
            guard.clone()
        };
        let instance = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { Component::instantiate_async(&mut store, &component, &linker).await })
            .map_err(|e| ComponentError::LoadFailed {
                path: path.to_path_buf(),
                reason: format!("Failed to instantiate component for metadata: {}", e),
            })?;

        // Extract metadata using WIT bindings
        let metadata_interface = instance.wasmflow_node_metadata();

        let component_info = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { metadata_interface.call_get_info(&mut store).await })
            .map_err(|e| ComponentError::LoadFailed {
                path: path.to_path_buf(),
                reason: format!("Failed to call get-info: {}", e),
            })?;

        let inputs = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { metadata_interface.call_get_inputs(&mut store).await })
            .map_err(|e| ComponentError::LoadFailed {
                path: path.to_path_buf(),
                reason: format!("Failed to call get-inputs: {}", e),
            })?;

        let outputs = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { metadata_interface.call_get_outputs(&mut store).await })
            .map_err(|e| ComponentError::LoadFailed {
                path: path.to_path_buf(),
                reason: format!("Failed to call get-outputs: {}", e),
            })?;

        let capabilities = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { metadata_interface.call_get_capabilities(&mut store).await })
            .map_err(|e| ComponentError::LoadFailed {
                path: path.to_path_buf(),
                reason: format!("Failed to call get-capabilities: {}", e),
            })?;

        // Create component ID
        let component_id = format!(
            "user:{}",
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        );

        // Create component spec from extracted metadata
        let mut spec = ComponentSpec::new_user_defined(
            component_id,
            component_info.name,
            component_info.description,
            component_info.category,
            path.to_path_buf(),
        );

        // Add input ports
        for input in inputs {
            spec = spec.with_input(
                input.name,
                Self::wit_data_type_to_node_data_type(&input.data_type),
                input.description,
            );
        }

        // Add output ports
        for output in &outputs {
            spec = spec.with_output(
                output.name.clone(),
                Self::wit_data_type_to_node_data_type(&output.data_type),
                output.description.clone(),
            );
        }

        // Store capabilities in spec
        if let Some(caps) = capabilities {
            spec.required_capabilities = caps;
        }

        // Try to extract custom footer view from component (if it exports UI interface)
        // Clone outputs so we don't move the value
        let outputs_for_ui = outputs.clone();
        if let Ok(footer_view) = Self::try_extract_footer_view_with_ui(
            &self.engine,
            &self.linker,
            &component,
            &outputs_for_ui,
            path,
        ) {
            log::debug!("Component has custom UI footer view");
            let adapter = WitFooterViewAdapter::new(footer_view);
            spec = spec.with_footer_view(Arc::new(adapter));
        }

        Ok(spec)
    }

    /// Try to extract footer view from a component with UI interface
    /// Attempts to instantiate with component-with-ui bindings and extract footer view
    fn try_extract_footer_view_with_ui(
        engine: &Engine,
        linker: &Arc<Mutex<Linker<HostState>>>,
        component: &WasmComponent,
        outputs: &[wasmflow::node::types::PortSpec],
        _path: &Path,
    ) -> Result<WitFooterView, Box<dyn std::error::Error>> {
        // Create host state for UI extraction
        let host_state = HostState::new("ui-extraction".to_string(), CapabilitySet::none())?;

        let mut store = Store::new(engine, host_state);

        // Try to instantiate with component-with-ui bindings
        let linker_clone = {
            let guard = linker.lock().unwrap();
            guard.clone()
        };

        // Instantiate using component-with-ui bindings
        let instance = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                with_ui::ComponentWithUi::instantiate_async(&mut store, component, &linker_clone)
                    .await
            })
            .map_err(|e| format!("Failed to instantiate with UI bindings: {}", e))?;

        // Create sample outputs to pass to get-footer-view
        let sample_outputs: Vec<(String, with_ui::wasmflow::node::types::Value)> = outputs
            .iter()
            .map(|output| {
                let value = match output.data_type {
                    wasmflow::node::types::DataType::U32Type => {
                        with_ui::wasmflow::node::types::Value::U32Val(0)
                    }
                    wasmflow::node::types::DataType::I32Type => {
                        with_ui::wasmflow::node::types::Value::I32Val(0)
                    }
                    wasmflow::node::types::DataType::F32Type => {
                        with_ui::wasmflow::node::types::Value::F32Val(0.0)
                    }
                    wasmflow::node::types::DataType::StringType => {
                        with_ui::wasmflow::node::types::Value::StringVal(String::new())
                    }
                    wasmflow::node::types::DataType::BinaryType => {
                        with_ui::wasmflow::node::types::Value::BinaryVal(vec![])
                    }
                    _ => with_ui::wasmflow::node::types::Value::StringVal(String::new()),
                };
                (output.name.clone(), value)
            })
            .collect();

        // Get UI interface and call get-footer-view
        let ui_interface = instance.wasmflow_node_ui();
        let wit_footer_view_opt = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                ui_interface
                    .call_get_footer_view(&mut store, &sample_outputs)
                    .await
            })
            .map_err(|e| format!("Failed to call get-footer-view: {}", e))?;

        // Convert Option<WIT FooterView> to our FooterView
        if let Some(wit_view) = wit_footer_view_opt {
            Ok(Self::convert_wit_footer_view(wit_view))
        } else {
            Err("Component returned None for footer view".into())
        }
    }

    /// Convert WIT FooterView to our internal representation
    fn convert_wit_footer_view(
        wit_view: with_ui::exports::wasmflow::node::ui::FooterView,
    ) -> WitFooterView {
        WitFooterView {
            elements: wit_view
                .elements
                .iter()
                .filter_map(Self::convert_wit_ui_element)
                .collect(),
        }
    }

    /// Convert WIT UiElement to our internal representation
    fn convert_wit_ui_element(
        elem: &with_ui::exports::wasmflow::node::ui::UiElement,
    ) -> Option<UiElement> {
        match elem {
            with_ui::exports::wasmflow::node::ui::UiElement::Label(text) => {
                Some(UiElement::Label(text.clone()))
            }
            with_ui::exports::wasmflow::node::ui::UiElement::ColoredLabel(colored) => {
                Some(UiElement::ColoredLabel(ColoredText {
                    text: colored.text.clone(),
                    r: colored.r,
                    g: colored.g,
                    b: colored.b,
                }))
            }
            with_ui::exports::wasmflow::node::ui::UiElement::KeyValue(kv) => {
                Some(UiElement::KeyValue(KeyValuePair {
                    key: kv.key.clone(),
                    value: kv.value.clone(),
                }))
            }
            with_ui::exports::wasmflow::node::ui::UiElement::Horizontal(layout) => {
                Some(UiElement::Horizontal(HorizontalLayout {
                    elements: layout
                        .elements
                        .iter()
                        .filter_map(Self::convert_wit_ui_element_item)
                        .collect(),
                }))
            }
            with_ui::exports::wasmflow::node::ui::UiElement::Vertical(layout) => {
                Some(UiElement::Vertical(VerticalLayout {
                    elements: layout
                        .elements
                        .iter()
                        .filter_map(Self::convert_wit_ui_element_item)
                        .collect(),
                }))
            }
            with_ui::exports::wasmflow::node::ui::UiElement::Separator => {
                Some(UiElement::Separator)
            }
        }
    }

    /// Convert WIT UiElementItem to our internal representation
    fn convert_wit_ui_element_item(
        item: &with_ui::exports::wasmflow::node::ui::UiElementItem,
    ) -> Option<UiElementItem> {
        match item {
            with_ui::exports::wasmflow::node::ui::UiElementItem::Label(text) => {
                Some(UiElementItem::Label(text.clone()))
            }
            with_ui::exports::wasmflow::node::ui::UiElementItem::ColoredLabel(colored) => {
                Some(UiElementItem::ColoredLabel(ColoredText {
                    text: colored.text.clone(),
                    r: colored.r,
                    g: colored.g,
                    b: colored.b,
                }))
            }
            with_ui::exports::wasmflow::node::ui::UiElementItem::KeyValue(kv) => {
                Some(UiElementItem::KeyValue(KeyValuePair {
                    key: kv.key.clone(),
                    value: kv.value.clone(),
                }))
            }
            with_ui::exports::wasmflow::node::ui::UiElementItem::Separator => {
                Some(UiElementItem::Separator)
            }
        }
    }

    /// Convert WIT DataType to Node DataType
    fn wit_data_type_to_node_data_type(
        wit_type: &wasmflow::node::types::DataType,
    ) -> crate::graph::node::DataType {
        use self::wasmflow::node::types::DataType as WitDataType;
        match wit_type {
            WitDataType::U32Type => crate::graph::node::DataType::U32,
            WitDataType::I32Type => crate::graph::node::DataType::I32,
            WitDataType::F32Type => crate::graph::node::DataType::F32,
            WitDataType::StringType => crate::graph::node::DataType::String,
            WitDataType::BoolType => crate::graph::node::DataType::Bool,
            WitDataType::BinaryType => crate::graph::node::DataType::Binary,
            WitDataType::ListType => {
                crate::graph::node::DataType::List(Box::new(crate::graph::node::DataType::Any))
            }
            WitDataType::AnyType => crate::graph::node::DataType::Any,
        }
    }

    /// Extract metadata from a component (deprecated in favor of lazy compilation)
    /// Kept for compatibility but now uses basic spec creation
    #[allow(dead_code)]
    async fn extract_metadata(
        &self,
        _component: &WasmComponent,
        path: &Path,
    ) -> Result<ComponentSpec, ComponentError> {
        // T083: With lazy compilation, we don't need to instantiate for metadata
        // Just create a basic spec from the path
        self.create_basic_spec_from_bytecode(&[], path)
    }

    /// Validate a component meets requirements
    pub fn validate_component(&self, path: &Path) -> Result<(), ComponentError> {
        // Check file exists
        if !path.exists() {
            return Err(ComponentError::ValidationFailed(format!(
                "Component file not found: {}",
                path.display()
            )));
        }

        // Check file extension
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            return Err(ComponentError::ValidationFailed(
                "Component file must have .wasm extension".to_string(),
            ));
        }

        // Check file size
        let metadata = std::fs::metadata(path).map_err(|e| {
            ComponentError::ValidationFailed(format!("Failed to read file metadata: {}", e))
        })?;

        if metadata.len() > MAX_COMPONENT_SIZE {
            return Err(ComponentError::ValidationFailed(format!(
                "Component file too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_COMPONENT_SIZE
            )));
        }

        Ok(())
    }

    /// T083: Get a loaded component with lazy compilation and LRU eviction
    pub fn get_component(
        &mut self,
        component_id: &str,
    ) -> Result<Arc<WasmComponent>, ComponentError> {
        // First check if we have a compiled version - clone it if we do
        let compiled_opt = self
            .components
            .get(component_id)
            .and_then(|data| data.compiled.as_ref().map(|c| c.clone()));

        if let Some(compiled) = compiled_opt {
            // Already compiled - update LRU and return
            self.update_lru(component_id);
            return Ok(compiled);
        }

        // Check if component exists at all
        if !self.components.contains_key(component_id) {
            return Err(ComponentError::ExecutionError(format!(
                "Component not loaded: {}",
                component_id
            )));
        }

        // Component exists but not compiled yet - compile it
        log::debug!("Compiling component on first use: {}", component_id);

        // Get bytecode (separate borrow to avoid conflicts)
        let bytecode = self
            .components
            .get(component_id)
            .unwrap() // Safe: we checked existence above
            .bytecode
            .clone();

        // Compile the component
        let compiled = WasmComponent::from_binary(&self.engine, &bytecode).map_err(|e| {
            ComponentError::ExecutionError(format!(
                "Failed to compile component {}: {}",
                component_id, e
            ))
        })?;

        let compiled_arc = Arc::new(compiled);

        // Store compiled component
        if let Some(data) = self.components.get_mut(component_id) {
            data.compiled = Some(compiled_arc.clone());
        }

        // T083: Update LRU and evict if needed
        self.update_lru_and_evict(component_id);

        Ok(compiled_arc)
    }

    /// T083: Update LRU order for a component
    fn update_lru(&mut self, component_id: &str) {
        // Remove from current position
        self.lru_order.retain(|id| id != component_id);
        // Add to end (most recently used)
        self.lru_order.push(component_id.to_string());
    }

    /// T083: Update LRU and evict oldest if cache is full
    fn update_lru_and_evict(&mut self, component_id: &str) {
        // Update LRU order
        self.update_lru(component_id);

        // Evict if cache is full
        while self.lru_order.len() > MAX_COMPILED_MODULES {
            if let Some(evict_id) = self.lru_order.first().cloned() {
                log::debug!("Evicting compiled component (LRU): {}", evict_id);
                // Remove from LRU list
                self.lru_order.remove(0);
                // Clear compiled component from cache
                if let Some(data) = self.components.get_mut(&evict_id) {
                    data.compiled = None;
                }
            } else {
                break;
            }
        }
    }

    /// Execute a component with given inputs
    /// T075: Permission enforcement - capabilities are enforced via WASI context
    /// T083: Now uses lazy compilation - compiles on first execution
    pub async fn execute_component(
        &mut self,
        component_id: &str,
        inputs: &HashMap<String, NodeValue>,
        capabilities: CapabilitySet,
    ) -> Result<HashMap<String, NodeValue>, ComponentError> {
        // T083: Get component (lazy compilation happens here)
        let component = self.get_component(component_id)?;

        // T075: Create host state with granted capabilities (WASI context configured here)
        let host_state =
            HostState::new(component_id.to_string(), capabilities.clone()).map_err(|e| {
                ComponentError::ExecutionError(format!("Failed to create host state: {}", e))
            })?;

        let mut store = Store::new(&self.engine, host_state);

        // Instantiate component using generated WIT bindings
        // Clone linker Arc to avoid holding MutexGuard across await
        let linker = {
            let guard = self.linker.lock().unwrap();
            guard.clone()
        };
        let instance = Component::instantiate_async(&mut store, &component, &linker)
            .await
            .map_err(|e| {
                // T076: Check for permission-related errors during instantiation
                Self::map_wasi_error_to_permission_denied(e, component_id, &capabilities)
            })?;

        // Convert inputs from NodeValue to WIT Value format
        let wit_inputs: Vec<(String, wasmflow::node::types::Value)> = inputs
            .iter()
            .map(|(name, value)| (name.clone(), node_value_to_wit(value)))
            .collect();

        // Call the component's execute() function via WIT bindings
        let result = instance
            .wasmflow_node_execution()
            .call_execute(&mut store, &wit_inputs)
            .await
            .map_err(|e| {
                // T076: Check for permission-related errors during execution
                Self::map_wasi_error_to_permission_denied(e, component_id, &capabilities)
            })?;

        // Handle execution result
        match result {
            Ok(wit_outputs) => {
                // Convert WIT outputs back to NodeValue
                let outputs: HashMap<String, NodeValue> = wit_outputs
                    .iter()
                    .map(|(name, value)| (name.clone(), wit_to_node_value(value)))
                    .collect();

                log::debug!(
                    "Component {} executed successfully with {} outputs",
                    component_id,
                    outputs.len()
                );
                Ok(outputs)
            }
            Err(err) => Err(ComponentError::ExecutionError(format!(
                "Component execution error: {}",
                err.message
            ))),
        }
    }

    /// Get footer view for a node with current output values
    ///
    /// For WASM components with custom UI, this calls get-footer-view() with
    /// the node's actual output values and returns the result.
    pub fn get_footer_view_for_node(
        &self,
        node: &crate::graph::node::GraphNode,
    ) -> Result<Option<WitFooterView>, Box<dyn std::error::Error>> {
        // Only handle user-defined components
        if !node.component_id.starts_with("user:") {
            return Ok(None);
        }

        // Get the component data to access bytecode
        let component_data = self
            .components
            .get(&node.component_id)
            .ok_or("Component not found")?;

        // Get or compile the component
        let compiled = if let Some(compiled) = &component_data.compiled {
            compiled.clone()
        } else {
            // Component not yet compiled - compile from bytecode
            Arc::new(WasmComponent::from_binary(
                &self.engine,
                &component_data.bytecode,
            )?)
        };

        // Create host state for UI extraction
        let host_state =
            HostState::new(format!("{}-ui", node.component_id), CapabilitySet::none())?;

        let mut store = Store::new(&self.engine, host_state);

        // Try to instantiate with component-with-ui bindings
        let linker_clone = {
            let guard = self.linker.lock().unwrap();
            guard.clone()
        };

        // Instantiate using component-with-ui bindings
        let instance = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                with_ui::ComponentWithUi::instantiate_async(&mut store, &compiled, &linker_clone)
                    .await
            })
            .map_err(|e| format!("Failed to instantiate with UI bindings: {}", e))?;

        // Convert node outputs to WIT format
        let wit_outputs: Vec<(String, with_ui::wasmflow::node::types::Value)> = node
            .outputs
            .iter()
            .filter_map(|port| {
                port.current_value.as_ref().map(|value| {
                    let wit_value = Self::node_value_to_wit_ui_value(value);
                    (port.name.clone(), wit_value)
                })
            })
            .collect();

        // Get UI interface and call get-footer-view
        let ui_interface = instance.wasmflow_node_ui();
        let wit_footer_view_opt = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                ui_interface
                    .call_get_footer_view(&mut store, &wit_outputs)
                    .await
            })
            .map_err(|e| format!("Failed to call get-footer-view: {}", e))?;

        // Convert Option<WIT FooterView> to our FooterView
        if let Some(wit_view) = wit_footer_view_opt {
            Ok(Some(Self::convert_wit_footer_view(wit_view)))
        } else {
            Ok(None)
        }
    }

    /// Convert NodeValue to WIT UI Value (for component-with-ui bindings)
    fn node_value_to_wit_ui_value(value: &NodeValue) -> with_ui::wasmflow::node::types::Value {
        use with_ui::wasmflow::node::types::Value;
        match value {
            NodeValue::U32(v) => Value::U32Val(*v),
            NodeValue::I32(v) => Value::I32Val(*v),
            NodeValue::F32(v) => Value::F32Val(*v),
            NodeValue::String(s) => Value::StringVal(s.clone()),
            NodeValue::Binary(b) => Value::BinaryVal(b.clone()),
            NodeValue::List(_) => Value::StringVal("<list>".to_string()),
            NodeValue::Record(_) => Value::StringVal("<record>".to_string()),
        }
    }

    /// T076: Map WASI errors to permission denied errors
    /// Detects permission-related failures and converts them to PermissionDenied
    fn map_wasi_error_to_permission_denied<E: std::fmt::Display>(
        error: E,
        component_id: &str,
        capabilities: &CapabilitySet,
    ) -> ComponentError {
        let error_msg = error.to_string().to_lowercase();

        // Check for common WASI permission error patterns
        if error_msg.contains("permission denied")
            || error_msg.contains("eperm")
            || error_msg.contains("eacces")
            || error_msg.contains("access denied")
            || error_msg.contains("not permitted")
            || error_msg.contains("preopened")
        {
            let capability_desc = capabilities.description();
            ComponentError::PermissionDenied {
                node_id: uuid::Uuid::nil(), // Will be filled in by engine.rs
                capability: format!(
                    "{} attempted to access resources not granted in permissions: {}",
                    component_id, capability_desc
                ),
            }
        } else {
            ComponentError::ExecutionError(format!("Component error: {}", error))
        }
    }
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new().expect("Failed to create component manager")
    }
}

/// Convert NodeValue to WIT Value
fn node_value_to_wit(value: &NodeValue) -> self::wasmflow::node::types::Value {
    use self::wasmflow::node::types::Value;
    match value {
        NodeValue::U32(v) => Value::U32Val(*v),
        NodeValue::I32(v) => Value::I32Val(*v),
        NodeValue::F32(v) => Value::F32Val(*v),
        NodeValue::String(s) => Value::StringVal(s.clone()),
        NodeValue::Bool(b) => Value::BoolVal(*b),
        NodeValue::Binary(b) => Value::BinaryVal(b.clone()),
        NodeValue::List(items) => {
            // Try to detect homogeneous list type and convert to typed list
            if items.is_empty() {
                // Empty list - default to string list
                Value::StringListVal(vec![])
            } else {
                // Check first item to determine list type
                match &items[0] {
                    NodeValue::String(_) => {
                        // String list
                        let strings: Vec<String> = items
                            .iter()
                            .filter_map(|v| {
                                if let NodeValue::String(s) = v {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        Value::StringListVal(strings)
                    }
                    NodeValue::U32(_) => {
                        // U32 list
                        let nums: Vec<u32> = items
                            .iter()
                            .filter_map(|v| {
                                if let NodeValue::U32(n) = v {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        Value::U32ListVal(nums)
                    }
                    NodeValue::F32(_) => {
                        // F32 list
                        let nums: Vec<f32> = items
                            .iter()
                            .filter_map(|v| {
                                if let NodeValue::F32(n) = v {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        Value::F32ListVal(nums)
                    }
                    _ => {
                        // Heterogeneous or unsupported list type - convert to string placeholder
                        Value::StringVal("<mixed-list>".to_string())
                    }
                }
            }
        }
        NodeValue::Record(_) => {
            // Records need special handling - not implemented yet
            // For now, convert to a placeholder string
            Value::StringVal("<record>".to_string())
        }
    }
}

/// Convert WIT Value to NodeValue
fn wit_to_node_value(value: &self::wasmflow::node::types::Value) -> NodeValue {
    use self::wasmflow::node::types::Value;
    match value {
        Value::U32Val(v) => NodeValue::U32(*v),
        Value::I32Val(v) => NodeValue::I32(*v),
        Value::F32Val(v) => NodeValue::F32(*v),
        Value::StringVal(s) => NodeValue::String(s.clone()),
        Value::BoolVal(b) => NodeValue::Bool(*b),
        Value::BinaryVal(b) => NodeValue::Binary(b.clone()),
        Value::StringListVal(items) => {
            NodeValue::List(items.iter().map(|s| NodeValue::String(s.clone())).collect())
        }
        Value::U32ListVal(items) => {
            NodeValue::List(items.iter().map(|v| NodeValue::U32(*v)).collect())
        }
        Value::F32ListVal(items) => {
            NodeValue::List(items.iter().map(|v| NodeValue::F32(*v)).collect())
        }
    }
}

/// Parse capability string format (e.g., "file-read:/path", "network:example.com")
pub fn parse_capability_string(cap_str: &str) -> Result<(Capability, Option<String>), String> {
    let parts: Vec<&str> = cap_str.splitn(2, ':').collect();

    match parts.as_slice() {
        ["file-read", path] => Ok((Capability::FileRead, Some(path.to_string()))),
        ["file-read"] => Ok((Capability::FileRead, None)),
        ["file-write", path] => Ok((Capability::FileWrite, Some(path.to_string()))),
        ["file-write"] => Ok((Capability::FileWrite, None)),
        ["network", host] => Ok((Capability::NetworkHttp, Some(host.to_string()))),
        ["network"] => Ok((Capability::NetworkHttp, None)),
        ["process"] => Ok((Capability::ProcessSpawn, None)),
        ["env"] => Ok((Capability::EnvAccess, None)),
        ["time"] => Ok((Capability::TimeAccess, None)),
        ["crypto"] => Ok((Capability::CryptoRandom, None)),
        _ => Err(format!("Unknown capability format: {}", cap_str)),
    }
}

/// T071: Parse capability request strings into CapabilitySet
/// Converts component capability requests (from get-capabilities()) into a CapabilitySet
pub fn parse_capability_requests(capability_strings: &[String]) -> Result<CapabilitySet, String> {
    use std::path::PathBuf;

    if capability_strings.is_empty() {
        return Ok(CapabilitySet::none());
    }

    let mut file_read_paths = Vec::new();
    let mut file_write_paths = Vec::new();
    let mut network_hosts = Vec::new();
    let mut has_full = false;

    for cap_str in capability_strings {
        if cap_str == "full" {
            has_full = true;
            break; // Full access overrides everything else
        }

        let (capability, param) = parse_capability_string(cap_str)?;

        match (capability, param) {
            (Capability::FileRead, Some(path)) => {
                file_read_paths.push(PathBuf::from(path));
            }
            (Capability::FileWrite, Some(path)) => {
                file_write_paths.push(PathBuf::from(path));
            }
            (Capability::NetworkHttp, Some(host)) => {
                network_hosts.push(host);
            }
            _ => {
                // Other capabilities not yet mapped to CapabilitySet variants
                // For now, we ignore them or could return an error
            }
        }
    }

    if has_full {
        return Ok(CapabilitySet::full());
    }

    // Prioritize the most specific capability set
    if !file_write_paths.is_empty() {
        // If both read and write paths, use FileReadWrite
        if !file_read_paths.is_empty() {
            // Combine both sets of paths
            let mut all_paths = file_write_paths;
            all_paths.extend(file_read_paths);
            all_paths.sort();
            all_paths.dedup();
            Ok(CapabilitySet::file_read_write(all_paths))
        } else {
            Ok(CapabilitySet::file_write(file_write_paths))
        }
    } else if !file_read_paths.is_empty() {
        Ok(CapabilitySet::file_read(file_read_paths))
    } else if !network_hosts.is_empty() {
        Ok(CapabilitySet::network(network_hosts))
    } else {
        Ok(CapabilitySet::none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_capability_string() {
        let (cap, path) = parse_capability_string("file-read:/data").unwrap();
        assert_eq!(cap, Capability::FileRead);
        assert_eq!(path, Some("/data".to_string()));

        let (cap, host) = parse_capability_string("network:api.example.com").unwrap();
        assert_eq!(cap, Capability::NetworkHttp);
        assert_eq!(host, Some("api.example.com".to_string()));

        let (cap, _) = parse_capability_string("time").unwrap();
        assert_eq!(cap, Capability::TimeAccess);

        assert!(parse_capability_string("invalid:format:too:many:colons").is_err());
    }

    #[test]
    fn test_host_state_logging() {
        let mut state =
            HostState::new("test-component".to_string(), CapabilitySet::none()).unwrap();

        // This should not panic
        state.log("info", "Test message");
        state.log("error", "Test error");
    }

    #[test]
    fn test_host_state_temp_dir_no_permission() {
        let state = HostState::new("test-component".to_string(), CapabilitySet::none()).unwrap();

        // Should fail without file write capability
        assert!(state.get_temp_dir().is_err());
    }

    #[test]
    fn test_host_state_temp_dir_with_permission() {
        use std::path::PathBuf;
        let state = HostState::new(
            "test-component".to_string(),
            CapabilitySet::file_write(vec![PathBuf::from("/tmp")]),
        )
        .unwrap();

        // Should succeed with file write capability
        assert!(state.get_temp_dir().is_ok());
    }

    #[test]
    fn test_parse_capability_requests() {
        use std::path::PathBuf;

        // Test empty list
        let result = parse_capability_requests(&[]).unwrap();
        assert_eq!(result, CapabilitySet::None);

        // Test file-read
        let result = parse_capability_requests(&[
            "file-read:/data".to_string(),
            "file-read:/tmp".to_string(),
        ])
        .unwrap();
        match result {
            CapabilitySet::FileRead { paths } => {
                assert_eq!(paths.len(), 2);
                assert!(paths.contains(&PathBuf::from("/data")));
            }
            _ => panic!("Expected FileRead capability set"),
        }

        // Test file-write
        let result = parse_capability_requests(&["file-write:/output".to_string()]).unwrap();
        match result {
            CapabilitySet::FileWrite { paths } => {
                assert_eq!(paths.len(), 1);
                assert_eq!(paths[0], PathBuf::from("/output"));
            }
            _ => panic!("Expected FileWrite capability set"),
        }

        // Test mixed read/write (should create FileReadWrite)
        let result = parse_capability_requests(&[
            "file-read:/data".to_string(),
            "file-write:/output".to_string(),
        ])
        .unwrap();
        match result {
            CapabilitySet::FileReadWrite { paths } => {
                assert_eq!(paths.len(), 2);
            }
            _ => panic!("Expected FileReadWrite capability set"),
        }

        // Test network
        let result = parse_capability_requests(&[
            "network:api.example.com".to_string(),
            "network:cdn.example.com".to_string(),
        ])
        .unwrap();
        match result {
            CapabilitySet::Network { allowed_hosts } => {
                assert_eq!(allowed_hosts.len(), 2);
                assert!(allowed_hosts.contains(&"api.example.com".to_string()));
            }
            _ => panic!("Expected Network capability set"),
        }

        // Test full access
        let result = parse_capability_requests(&["full".to_string()]).unwrap();
        assert_eq!(result, CapabilitySet::Full);
    }
}
