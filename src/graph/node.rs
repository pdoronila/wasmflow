//! Core graph node types and data structures

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use uuid::Uuid;

/// Typed data flowing through connections between nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeValue {
    /// Unsigned 32-bit integer
    U32(u32),
    /// Signed 32-bit integer
    I32(i32),
    /// 32-bit floating point number
    F32(f32),
    /// UTF-8 text string
    String(String),
    /// Boolean value
    Bool(bool),
    /// Raw binary data for efficient large payloads
    Binary(Vec<u8>),
    /// Homogeneous or heterogeneous list of values
    List(Vec<NodeValue>),
    /// Key-value structured data (BTreeMap for deterministic serialization)
    Record(BTreeMap<String, NodeValue>),
}

impl NodeValue {
    /// Get the type name of this value for display purposes
    pub fn type_name(&self) -> &'static str {
        match self {
            NodeValue::U32(_) => "u32",
            NodeValue::I32(_) => "i32",
            NodeValue::F32(_) => "f32",
            NodeValue::String(_) => "string",
            NodeValue::Bool(_) => "bool",
            NodeValue::Binary(_) => "binary",
            NodeValue::List(_) => "list",
            NodeValue::Record(_) => "record",
        }
    }

    /// Format value for display in UI
    pub fn format_display(&self) -> String {
        match self {
            NodeValue::U32(v) => format!("{}", v),
            NodeValue::I32(v) => format!("{}", v),
            NodeValue::F32(v) => format!("{:.2}", v),
            NodeValue::String(s) => format!("\"{}\"", s),
            NodeValue::Bool(b) => format!("{}", b),
            NodeValue::Binary(b) => format!("<{} bytes>", b.len()),
            NodeValue::List(items) => format!("[{} items]", items.len()),
            NodeValue::Record(fields) => format!("{{{} fields}}", fields.len()),
        }
    }
}

/// Data type specification for ports
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    U32,
    I32,
    F32,
    String,
    Bool,
    Binary,
    List(Box<DataType>),
    Record(Vec<(String, DataType)>),
    /// Accepts any type (for generic nodes like passthrough, logger)
    Any,
}

impl DataType {
    /// Get the type name for display
    pub fn name(&self) -> String {
        match self {
            DataType::U32 => "u32".to_string(),
            DataType::I32 => "i32".to_string(),
            DataType::F32 => "f32".to_string(),
            DataType::String => "string".to_string(),
            DataType::Bool => "bool".to_string(),
            DataType::Binary => "binary".to_string(),
            DataType::List(inner) => format!("list<{}>", inner.name()),
            DataType::Record(_) => "record".to_string(),
            DataType::Any => "any".to_string(),
        }
    }
}

/// Port direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

/// Connection point on a node for receiving (input) or providing (output) data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Unique identifier (composite: NodeId + local index)
    pub id: Uuid,
    /// Human-readable label (e.g., "a", "sum", "input_file")
    pub name: String,
    /// Expected data type
    pub data_type: DataType,
    /// Input or Output
    pub direction: PortDirection,
    /// Whether connection is required
    pub optional: bool,
    /// Current value (present only for outputs after execution)
    pub current_value: Option<NodeValue>,
}

impl Port {
    /// Create a new port
    pub fn new(
        name: String,
        data_type: DataType,
        direction: PortDirection,
        optional: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            data_type,
            direction,
            optional,
            current_value: None,
        }
    }
}

/// Node execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionState {
    Idle,
    Running,
    Completed,
    Failed,
}

impl Default for ExecutionState {
    fn default() -> Self {
        ExecutionState::Idle
    }
}

/// Continuous execution state for long-running nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuousExecutionState {
    /// Not running, waiting for user action
    Idle,
    /// Play button clicked, spawning async task
    Starting,
    /// Actively executing, processing inputs
    Running,
    /// Stop button clicked, graceful shutdown in progress
    Stopping,
    /// Successfully stopped by user
    Stopped,
    /// Execution failed, error details available
    Error,
}

impl Default for ContinuousExecutionState {
    fn default() -> Self {
        ContinuousExecutionState::Idle
    }
}

/// Runtime state for continuous execution (NOT serialized)
#[derive(Debug, Clone)]
pub struct ContinuousRuntimeState {
    /// Whether execution is currently active
    pub is_running: bool,
    /// When execution began
    pub started_at: Option<std::time::Instant>,
    /// Number of execution cycles completed
    pub iterations: u64,
    /// Most recent error message
    pub last_error: Option<String>,
    /// Current execution state
    pub execution_state: ContinuousExecutionState,
}

impl Default for ContinuousRuntimeState {
    fn default() -> Self {
        Self {
            is_running: false,
            started_at: None,
            iterations: 0,
            last_error: None,
            execution_state: ContinuousExecutionState::Idle,
        }
    }
}

/// Configuration for continuous execution nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousNodeConfig {
    /// Whether this node type can run continuously
    pub supports_continuous: bool,
    /// User preference for showing play/stop controls
    pub enabled: bool,
    /// Runtime state (NOT serialized - always reset on load)
    #[serde(skip)]
    pub runtime_state: ContinuousRuntimeState,
}

impl Default for ContinuousNodeConfig {
    fn default() -> Self {
        Self {
            supports_continuous: false,
            enabled: false,
            runtime_state: ContinuousRuntimeState::default(),
        }
    }
}

/// Node metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub author: String,
    pub version: String,
    pub description: String,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self {
            author: "Unknown".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
        }
    }
}

/// Programming language for WASM Creator nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
}

impl Default for Language {
    fn default() -> Self {
        Language::Rust
    }
}

impl Language {
    /// Get display name for the language
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
        }
    }

    /// Get file extension for the language
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Python => "py",
            Language::JavaScript => "js",
        }
    }
}

/// T057: Data specific to WASM Creator nodes
///
/// This struct stores the code editor state and compilation state for creator nodes.
/// The source_code field is optionally saved based on the save_code flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCreatorNodeData {
    /// User-specified component name (PascalCase)
    pub component_name: String,
    /// Whether to save source code in graph file
    pub save_code: bool,
    /// User's Rust code with annotations (optional - may be empty if save_code was false)
    #[serde(default)]
    pub source_code: String,
    /// Current compilation state
    pub compilation_state: CompilationState,
    /// ID of generated component (if compilation succeeded)
    pub generated_component_id: Option<String>,
    /// Programming language for this component
    #[serde(default)]
    pub language: Language,
    /// Code editor theme preference
    #[serde(default, skip)]
    pub editor_theme: crate::ui::code_editor::CodeTheme,
}

impl WasmCreatorNodeData {
    pub fn new(component_name: String, source_code: String) -> Self {
        Self {
            component_name,
            save_code: true,
            source_code,
            compilation_state: CompilationState::Idle,
            generated_component_id: None,
            language: Language::Rust, // Default to Rust for backward compatibility
            editor_theme: crate::ui::code_editor::CodeTheme::default(),
        }
    }

    /// Create with specific language
    pub fn new_with_language(component_name: String, source_code: String, language: Language) -> Self {
        Self {
            component_name,
            save_code: true,
            source_code,
            compilation_state: CompilationState::Idle,
            generated_component_id: None,
            language,
            editor_theme: crate::ui::code_editor::CodeTheme::default(),
        }
    }

    /// T057: Prepare for serialization by clearing source_code if save_code is false
    pub fn prepare_for_save(&mut self) {
        if !self.save_code {
            self.source_code = String::new();
        }
    }

    /// Check if source code needs to be loaded (was saved)
    pub fn has_saved_code(&self) -> bool {
        self.save_code && !self.source_code.is_empty()
    }
}

/// T027: Port mapping from external composite port to internal node port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Name of the port as exposed on the composite node
    pub external_name: String,
    /// ID of the internal node this port connects to
    pub internal_node_id: Uuid,
    /// Name of the port on the internal node
    pub internal_port_name: String,
    /// Data type of the port (for validation)
    pub port_type: DataType,
}

/// T027: Metadata about a composition operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionMetadata {
    /// When the composition was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Number of components composed together
    pub component_count: usize,
    /// Names of all composed components (for footer display)
    pub component_names: Vec<String>,
    /// Size of the composed WASM binary in bytes
    pub composition_size_bytes: usize,
    /// Hash of composition inputs (for cache invalidation)
    pub composition_hash: u64,
}

/// T027: Data specific to composite nodes created from composition
///
/// This struct stores the internal structure of a composite node,
/// allowing drill-down inspection while preserving the original node layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionData {
    /// Name of the composite
    pub name: String,
    /// Socket (main) component path
    pub socket_path: std::path::PathBuf,
    /// Plug (dependency) component paths
    pub plug_paths: Vec<std::path::PathBuf>,
    /// Internal nodes preserved for drill-down (BTreeMap for deterministic serialization)
    pub internal_nodes: BTreeMap<Uuid, GraphNode>,
    /// Internal connections between nodes
    pub internal_edges: Vec<crate::graph::connection::Connection>,
    /// Exposed inputs mapped to internal node inputs (BTreeMap for deterministic serialization)
    pub exposed_inputs: BTreeMap<String, PortMapping>,
    /// Exposed outputs mapped to internal node outputs (BTreeMap for deterministic serialization)
    pub exposed_outputs: BTreeMap<String, PortMapping>,
    /// Composition metadata
    pub metadata: CompositionMetadata,
    /// Cached composition binary (not serialized - regenerated on load if needed)
    #[serde(skip)]
    pub cached_composition: Option<Vec<u8>>,
}

impl CompositionData {
    /// Create new composition data
    pub fn new(
        name: String,
        socket_path: std::path::PathBuf,
        plug_paths: Vec<std::path::PathBuf>,
        internal_nodes: BTreeMap<Uuid, GraphNode>,
        internal_edges: Vec<crate::graph::connection::Connection>,
        component_names: Vec<String>,
        composed_binary: Vec<u8>,
    ) -> Self {
        let composition_hash = crc::Crc::<u64>::new(&crc::CRC_64_ECMA_182)
            .checksum(&composed_binary);

        Self {
            name,
            socket_path,
            plug_paths,
            internal_nodes,
            internal_edges,
            exposed_inputs: BTreeMap::new(), // To be populated by port aggregation
            exposed_outputs: BTreeMap::new(), // To be populated by port aggregation
            metadata: CompositionMetadata {
                created_at: chrono::Utc::now(),
                component_count: component_names.len(),
                component_names,
                composition_size_bytes: composed_binary.len(),
                composition_hash,
            },
            cached_composition: Some(composed_binary),
        }
    }
}

/// A computational unit in the visual programming graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier
    pub id: Uuid,
    /// Reference to ComponentSpec (determines behavior)
    pub component_id: String,
    /// User-visible label on canvas
    pub display_name: String,
    /// Canvas coordinates for UI rendering
    #[serde(with = "pos2_serde")]
    pub position: egui::Pos2,
    /// Input ports
    pub inputs: Vec<Port>,
    /// Output ports
    pub outputs: Vec<Port>,
    /// Node metadata
    pub metadata: NodeMetadata,
    /// Security permissions
    pub capabilities: crate::runtime::CapabilitySet,
    /// Execution state
    #[serde(default)]
    pub execution_state: ExecutionState,
    /// T084: Dirty flag for incremental execution
    /// True if node needs re-execution (inputs changed, node modified, or upstream dependency changed)
    #[serde(default = "default_dirty")]
    pub dirty: bool,
    /// Cached footer view for WASM components with custom UI
    /// Computed after execution with actual output values
    /// Not serialized (UI-only concern, regenerated on load)
    #[serde(skip)]
    pub cached_footer_view: Option<crate::ui::wit_ui_renderer::FooterView>,
    /// Selection state for rectangle selection
    /// Not serialized (UI-only state)
    #[serde(skip, default)]
    pub selected: bool,
    /// T057: Creator node data (only present for WasmCreatorNode type)
    /// Stores the code editor state and compilation state for creator nodes
    #[serde(default)]
    pub creator_data: Option<WasmCreatorNodeData>,
    /// T059: Flag indicating component was updated and node may need refresh
    #[serde(default)]
    pub needs_component_refresh: bool,
    /// Timestamp when execution started (for showing spinner on long-running nodes)
    /// Not serialized as it's runtime-only state
    #[serde(skip)]
    pub execution_started_at: Option<std::time::Instant>,
    /// Timestamp when execution completed (for showing brief completion indicator)
    /// Not serialized as it's runtime-only state
    #[serde(skip)]
    pub execution_completed_at: Option<std::time::Instant>,
    /// Continuous execution configuration (if node supports continuous mode)
    #[serde(default)]
    pub continuous_config: Option<ContinuousNodeConfig>,
    /// T026: Composition data (only present for composite nodes)
    /// Stores internal structure for drill-down inspection
    #[serde(default)]
    pub composition_data: Option<CompositionData>,
}

/// T084: Default dirty flag to true for new nodes
fn default_dirty() -> bool {
    true
}

impl GraphNode {
    /// Create a new graph node from a component spec
    pub fn new(component_id: String, display_name: String, position: egui::Pos2) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_id,
            display_name,
            position,
            inputs: Vec::new(),
            outputs: Vec::new(),
            metadata: NodeMetadata::default(),
            capabilities: crate::runtime::CapabilitySet::None,
            execution_state: ExecutionState::Idle,
            dirty: true, // T084: New nodes are always dirty
            cached_footer_view: None,
            creator_data: None, // T057: Creator data only present for WasmCreatorNode type
            needs_component_refresh: false, // T059: No refresh needed for new nodes
            execution_started_at: None,
            execution_completed_at: None,
            continuous_config: None, // Continuous config set later if node supports it
            selected: false, // T019: New nodes start unselected
            composition_data: None, // T026: Composition data only present for composite nodes
        }
    }

    /// Get input port by name
    pub fn get_input(&self, name: &str) -> Option<&Port> {
        self.inputs.iter().find(|p| p.name == name)
    }

    /// Get mutable input port by name
    pub fn get_input_mut(&mut self, name: &str) -> Option<&mut Port> {
        self.inputs.iter_mut().find(|p| p.name == name)
    }

    /// Get output port by name
    pub fn get_output(&self, name: &str) -> Option<&Port> {
        self.outputs.iter().find(|p| p.name == name)
    }

    /// Get mutable output port by name
    pub fn get_output_mut(&mut self, name: &str) -> Option<&mut Port> {
        self.outputs.iter_mut().find(|p| p.name == name)
    }

    /// T043: Compute a hash of the node's current input values
    /// Returns None if any inputs don't have values yet
    pub fn compute_input_hash(&self) -> Option<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash each input's current value in a deterministic order (by name)
        let mut inputs_with_values: Vec<_> = self.inputs.iter()
            .filter_map(|port| {
                port.current_value.as_ref().map(|value| (&port.name, value))
            })
            .collect();

        // Sort by name for deterministic hashing
        inputs_with_values.sort_by(|a, b| a.0.cmp(b.0));

        // If we don't have all inputs with values, return None
        if inputs_with_values.len() != self.inputs.len() {
            return None;
        }

        // Hash each (name, value) pair
        for (name, value) in inputs_with_values {
            name.hash(&mut hasher);
            // Hash the discriminant and value
            match value {
                NodeValue::U32(v) => {
                    "U32".hash(&mut hasher);
                    v.hash(&mut hasher);
                }
                NodeValue::I32(v) => {
                    "I32".hash(&mut hasher);
                    v.hash(&mut hasher);
                }
                NodeValue::F32(v) => {
                    "F32".hash(&mut hasher);
                    v.to_bits().hash(&mut hasher);
                }
                NodeValue::String(v) => {
                    "String".hash(&mut hasher);
                    v.hash(&mut hasher);
                }
                NodeValue::Bool(v) => {
                    "Bool".hash(&mut hasher);
                    v.hash(&mut hasher);
                }
                NodeValue::Binary(v) => {
                    "Binary".hash(&mut hasher);
                    v.hash(&mut hasher);
                }
                NodeValue::List(list) => {
                    "List".hash(&mut hasher);
                    list.len().hash(&mut hasher);
                    // Simplified: just hash length for now to avoid deep recursion
                }
                NodeValue::Record(map) => {
                    "Record".hash(&mut hasher);
                    // BTreeMap is already sorted, so iteration is deterministic
                    for (k, v) in map {
                        k.hash(&mut hasher);
                        // Recursively hash nested values (simplified - only handles primitives)
                        match v {
                            NodeValue::U32(val) => val.hash(&mut hasher),
                            NodeValue::I32(val) => val.hash(&mut hasher),
                            NodeValue::F32(val) => val.to_bits().hash(&mut hasher),
                            NodeValue::String(val) => val.hash(&mut hasher),
                            NodeValue::Bool(val) => val.hash(&mut hasher),
                            NodeValue::Binary(val) => val.hash(&mut hasher),
                            _ => {} // Skip complex nested types for now
                        }
                    }
                }
            }
        }

        Some(hasher.finish())
    }
}

/// Custom serde for egui::Pos2
mod pos2_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(pos: &egui::Pos2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (pos.x, pos.y).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<egui::Pos2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (x, y) = <(f32, f32)>::deserialize(deserializer)?;
        Ok(egui::Pos2::new(x, y))
    }
}

/// Component type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    /// Built-in component (compiled into application)
    Builtin,
    /// User-defined component loaded from external file
    UserDefined(std::path::PathBuf),
    /// T026: Composite component created from multiple composed components
    Composed {
        /// Socket (main) component path
        socket_path: std::path::PathBuf,
        /// Plug (dependency) component paths
        plug_paths: Vec<std::path::PathBuf>,
    },
}

/// Node type classification (T018)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Regular computational node
    Regular,
    /// WASM component creator node with code editor
    WasmCreatorNode,
}

/// Compilation state for WASM creator nodes (T020)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompilationState {
    /// No compilation in progress
    Idle,
    /// Compilation running
    Compiling {
        started_at: chrono::DateTime<chrono::Utc>,
        #[serde(skip)]
        pid: Option<u32>,
    },
    /// Compilation succeeded
    Success {
        compiled_at: chrono::DateTime<chrono::Utc>,
        component_path: std::path::PathBuf,
        build_time_ms: u64,
    },
    /// Compilation failed
    Failed {
        error_message: String,
        line_number: Option<usize>,
        failed_at: chrono::DateTime<chrono::Utc>,
    },
}

/// Generated component metadata (T019)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedComponent {
    /// Component name (PascalCase)
    pub name: String,
    /// Path to compiled .wasm file
    pub wasm_path: std::path::PathBuf,
    /// Component metadata
    pub metadata: crate::runtime::ComponentMetadata,
    /// When the component was loaded
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    /// ID of the creator node that generated this component
    pub source_creator_node: Uuid,
}

/// Port specification for component interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    pub name: String,
    pub data_type: DataType,
    pub optional: bool,
    pub description: String,
}

/// Specification of a node's behavior, interface, and system requirements
#[derive(Clone, Serialize, Deserialize)]
pub struct ComponentSpec {
    /// Unique component identifier (e.g., "builtin:math:add", "user:text-processor")
    pub id: String,
    /// Display name for UI
    pub name: String,
    /// User-visible documentation
    pub description: String,
    /// Creator attribution
    pub author: String,
    /// Semantic version (major.minor.patch)
    pub version: String,
    /// Builtin or UserDefined
    pub component_type: ComponentType,
    /// Expected inputs
    pub input_spec: Vec<PortSpec>,
    /// Produced outputs
    pub output_spec: Vec<PortSpec>,
    /// Required system capabilities
    pub required_capabilities: Vec<String>,
    /// Category for organization (e.g., "Math", "Text", "File I/O")
    pub category: Option<String>,
    /// Optional custom footer view implementation
    ///
    /// Components can provide a custom UI view for displaying in the canvas
    /// footer when a node is selected. This enables colocation of component
    /// logic with its UI presentation.
    ///
    /// Note: This field is not serialized (UI-only concern). Views are
    /// re-registered when components are loaded from the registry.
    #[serde(skip)]
    pub footer_view: Option<Arc<dyn crate::ui::component_view::ComponentFooterView>>,
}

// Manual Debug implementation because view trait objects don't implement Debug
impl std::fmt::Debug for ComponentSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentSpec")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("author", &self.author)
            .field("version", &self.version)
            .field("component_type", &self.component_type)
            .field("input_spec", &self.input_spec)
            .field("output_spec", &self.output_spec)
            .field("required_capabilities", &self.required_capabilities)
            .field("category", &self.category)
            .field("footer_view", &self.footer_view.as_ref().map(|_| "<view>"))
            .finish()
    }
}

impl ComponentSpec {
    /// Create a new builtin component specification
    pub fn new_builtin(
        id: String,
        name: String,
        description: String,
        category: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            author: "WasmFlow".to_string(),
            version: "1.0.0".to_string(),
            component_type: ComponentType::Builtin,
            input_spec: Vec::new(),
            output_spec: Vec::new(),
            required_capabilities: Vec::new(),
            category,
            footer_view: None,
        }
    }

    /// Create a new user-defined component specification
    pub fn new_user_defined(
        id: String,
        name: String,
        description: String,
        category: Option<String>,
        path: std::path::PathBuf,
    ) -> Self {
        Self {
            id,
            name,
            description,
            author: "User".to_string(),
            version: "1.0.0".to_string(),
            component_type: ComponentType::UserDefined(path),
            input_spec: Vec::new(),
            output_spec: Vec::new(),
            required_capabilities: Vec::new(),
            category,
            footer_view: None,
        }
    }

    /// Add an input port specification
    pub fn with_input(mut self, name: String, data_type: DataType, description: String) -> Self {
        self.input_spec.push(PortSpec {
            name,
            data_type,
            optional: false,
            description,
        });
        self
    }

    /// Add an output port specification
    pub fn with_output(mut self, name: String, data_type: DataType, description: String) -> Self {
        self.output_spec.push(PortSpec {
            name,
            data_type,
            optional: false,
            description,
        });
        self
    }

    /// Set custom footer view for this component
    ///
    /// Enables the component to provide custom UI rendering in the canvas
    /// footer when a node instance is selected.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let spec = ComponentSpec::new_builtin(...)
    ///     .with_footer_view(Arc::new(MyCustomFooterView));
    /// ```
    pub fn with_footer_view(
        mut self,
        view: Arc<dyn crate::ui::component_view::ComponentFooterView>,
    ) -> Self {
        self.footer_view = Some(view);
        self
    }

    /// Check if this component has a custom footer view
    ///
    /// Returns `true` if a footer view implementation is available.
    pub fn has_footer_view(&self) -> bool {
        self.footer_view.is_some()
    }

    /// Get footer view if available
    ///
    /// Returns a reference to the footer view implementation, if one exists.
    pub fn get_footer_view(&self) -> Option<&Arc<dyn crate::ui::component_view::ComponentFooterView>> {
        self.footer_view.as_ref()
    }

    /// Create a GraphNode instance from this spec
    pub fn create_node(&self, position: egui::Pos2) -> GraphNode {
        let mut node = GraphNode::new(self.id.clone(), self.name.clone(), position);

        // Create input ports
        for spec in &self.input_spec {
            node.inputs.push(Port::new(
                spec.name.clone(),
                spec.data_type.clone(),
                PortDirection::Input,
                spec.optional,
            ));
        }

        // Create output ports
        for spec in &self.output_spec {
            let mut port = Port::new(
                spec.name.clone(),
                spec.data_type.clone(),
                PortDirection::Output,
                spec.optional,
            );

            // For constant nodes, initialize with default value based on type
            if self.id.starts_with("builtin:constant:") {
                port.current_value = Some(match spec.data_type {
                    DataType::F32 => NodeValue::F32(0.0),
                    DataType::I32 => NodeValue::I32(0),
                    DataType::U32 => NodeValue::U32(0),
                    DataType::String => NodeValue::String(String::new()),
                    _ => NodeValue::F32(0.0),
                });
            }

            node.outputs.push(port);
        }

        node.metadata = NodeMetadata {
            author: self.author.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
        };

        // T057: Initialize creator_data for WASM Creator nodes
        if self.id == "builtin:development:wasm-creator" {
            let default_code = r#"// @description My custom component
// @category User-Defined
// @input value:F32 Input number
// @output result:F32 Output number

// Your code here
let result = value * 2.0;
"#;
            node.creator_data = Some(WasmCreatorNodeData::new(
                String::new(),
                default_code.to_string(),
            ));
        }

        // Initialize continuous_config for continuous timer nodes
        if self.id == "builtin:continuous:timer" {
            node.continuous_config = Some(ContinuousNodeConfig {
                supports_continuous: true,
                enabled: true,
                runtime_state: ContinuousRuntimeState::default(),
            });

            // Set default interval value (100ms)
            if let Some(interval_port) = node.get_input_mut("interval") {
                interval_port.current_value = Some(NodeValue::U32(100));
            }
        }

        // T050: Initialize continuous_config for continuous combiner nodes
        if self.id == "builtin:continuous:combiner" {
            node.continuous_config = Some(ContinuousNodeConfig {
                supports_continuous: true,
                enabled: true,
                runtime_state: ContinuousRuntimeState::default(),
            });

            // Set default input values
            if let Some(input_a) = node.get_input_mut("input_a") {
                input_a.current_value = Some(NodeValue::String("Hello".to_string()));
            }
            if let Some(input_b) = node.get_input_mut("input_b") {
                input_b.current_value = Some(NodeValue::String("World".to_string()));
            }
            if let Some(separator) = node.get_input_mut("separator") {
                separator.current_value = Some(NodeValue::String(" ".to_string()));
            }
        }

        node
    }
}

/// Component registry for managing available node types
#[derive(Debug, Default)]
pub struct ComponentRegistry {
    components: HashMap<String, ComponentSpec>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a builtin component
    pub fn register_builtin(&mut self, spec: ComponentSpec) {
        self.components.insert(spec.id.clone(), spec);
    }

    /// Register a user-defined component
    pub fn register_component(&mut self, spec: ComponentSpec) -> Result<(), crate::ComponentError> {
        // Validate version format
        if !Self::is_valid_version(&spec.version) {
            return Err(crate::ComponentError::ValidationFailed(format!(
                "Invalid version format: {}",
                spec.version
            )));
        }

        self.components.insert(spec.id.clone(), spec);
        Ok(())
    }

    /// Get component by ID
    pub fn get_by_id(&self, id: &str) -> Option<&ComponentSpec> {
        self.components.get(id)
    }

    /// List all components
    pub fn list_all(&self) -> Vec<&ComponentSpec> {
        self.components.values().collect()
    }

    /// List components by category
    pub fn list_by_category(&self, category: &str) -> Vec<&ComponentSpec> {
        self.components
            .values()
            .filter(|spec| spec.category.as_deref() == Some(category))
            .collect()
    }

    /// Validate semantic version format
    fn is_valid_version(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok())
    }

    /// Register a dynamically compiled component from WASM creator node (T021, T055)
    ///
    /// If a component with the same name already exists, it will be replaced.
    /// This enables recompilation workflow where users can update their components.
    /// T059: Returns (component_id, was_replaced) where was_replaced indicates
    /// if an existing component was replaced. Callers with access to NodeGraph
    /// should call `graph.mark_component_users_for_refresh(&component_id)` when was_replaced is true.
    pub fn register_dynamic_component(
        &mut self,
        name: String,
        wasm_path: std::path::PathBuf,
        metadata: crate::runtime::ComponentMetadata,
    ) -> Result<(String, bool), crate::ComponentError> {
        // Generate component ID
        let component_id = format!("user:{}", name);

        // T055: Check if component already exists (for replacement)
        let was_replaced = if self.has_component(&component_id) {
            log::info!("Component '{}' already exists - replacing with new version", name);
            self.unregister_component(&component_id);
            true
        } else {
            log::info!("Registering new component '{}'", name);
            false
        };

        // Create component spec from metadata
        let mut spec = ComponentSpec::new_user_defined(
            component_id.clone(),
            name,
            metadata.description,
            Some(metadata.category),
            wasm_path,
        );

        // Add input ports from metadata
        for input in metadata.inputs {
            spec = spec.with_input(
                input.name,
                input.data_type,
                input.description,
            );
        }

        // Add output ports from metadata
        for output in metadata.outputs {
            spec = spec.with_output(
                output.name,
                output.data_type,
                output.description,
            );
        }

        // Set capabilities if any
        if !metadata.capabilities.is_empty() {
            spec.required_capabilities = metadata.capabilities;
        }

        // Register the component
        self.register_component(spec)?;

        log::info!("Successfully registered component '{}'", component_id);

        Ok((component_id, was_replaced))
    }

    /// Unregister a component by ID (T022)
    pub fn unregister_component(&mut self, id: &str) -> bool {
        self.components.remove(id).is_some()
    }

    /// Check if a component exists by ID (T022)
    pub fn has_component(&self, id: &str) -> bool {
        self.components.contains_key(id)
    }
}
