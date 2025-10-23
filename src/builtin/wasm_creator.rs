//! T031-T038: WASM Component Creator Node
//!
//! A special development node that allows users to create custom WASM components
//! directly in the visual editor using Rust code and structured comments.

use crate::graph::node::{CompilationState, ComponentSpec, NodeValue, GraphNode, Language};
use crate::runtime::{CompilationConfig, ComponentCompiler, TemplateGenerator};
use crate::ui::code_editor::{CodeEditorWidget, CodeTheme};
use crate::ui::component_view::ComponentFooterView;
use crate::ComponentError;
use egui::{RichText, Color32};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// T073: Helper function to capitalize first letter
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// WASM Component Creator Node
///
/// This node provides a code editor interface for creating custom WASM components.
/// Users can write Rust code with structured comments to define component metadata,
/// and the node will compile it into a WASM component that appears in the palette.
///
/// T060: Multiple instances of this node can coexist independently. Each instance:
/// - Has its own unique ID (Uuid)
/// - Maintains independent source code and compilation state
/// - Tracks its own generated component ID
/// - Uses shared ComponentRegistry (expected behavior for component sharing)
pub struct WasmCreatorNode {
    /// Unique node identifier
    pub id: Uuid,

    /// User-specified component name (PascalCase, e.g., "TripleNumber")
    pub component_name: String,

    /// User's Rust code with annotations
    pub source_code: String,

    /// Whether to save code in graph file (default: true)
    pub save_code: bool,

    /// Current compilation state
    pub compilation_state: CompilationState,

    /// Last error message (if compilation failed)
    pub last_error: Option<String>,

    /// ID of the generated component (if compilation succeeded)
    pub generated_component_id: Option<String>,

    /// Code editor widget (not serialized)
    #[allow(dead_code)]
    code_editor: CodeEditorWidget,

    /// Selected color theme for code editor
    pub editor_theme: CodeTheme,

    /// Programming language for this component
    pub language: Language,
}

impl WasmCreatorNode {
    /// T032: Create a new WASM Creator Node with default values
    ///
    /// Defaults:
    /// - Empty component name
    /// - Default template code
    /// - save_code = true (persist code in graph)
    /// - compilation_state = Idle
    pub fn new() -> Self {
        let default_code = Self::default_template(Language::Rust);

        Self {
            id: Uuid::new_v4(),
            component_name: String::new(),
            source_code: default_code,
            save_code: true,
            compilation_state: CompilationState::Idle,
            last_error: None,
            generated_component_id: None,
            code_editor: CodeEditorWidget::new().with_rows(25),
            editor_theme: CodeTheme::default(),
            language: Language::Rust,
        }
    }

    /// Get default template for a language
    pub fn default_template(language: Language) -> String {
        match language {
            Language::Rust => Self::default_rust_template(),
            Language::Python => Self::default_python_template(),
            Language::JavaScript => Self::default_javascript_template(),
        }
    }

    /// Default Rust template
    fn default_rust_template() -> String {
        r#"// @description My custom component
// @category User-Defined
// @input value:F32 Input number
// @output result:F32 Output number

// Your code here
let result = value * 2.0;
"#.to_string()
    }

    /// Default Python template
    fn default_python_template() -> String {
        r#"# @description My custom component
# @category User-Defined
# @input value:F32 Input number
# @output result:F32 Output number

# Your code here
result = value * 2.0
"#.to_string()
    }

    /// Default JavaScript template
    fn default_javascript_template() -> String {
        r#"// @description My custom component
// @category User-Defined
// @input value:F32 Input number
// @output result:F32 Output number

// Your code here
const result = value * 2.0;
"#.to_string()
    }

    /// Get the component spec for this creator node
    ///
    /// This is used to add the creator node to the palette
    pub fn spec() -> ComponentSpec {
        ComponentSpec::new_builtin(
            "builtin:development:wasm-creator".to_string(),
            "WASM Creator".to_string(),
            "Create custom WASM components from Rust code".to_string(),
            Some("Development".to_string()),
        )
        // Creator nodes don't have traditional inputs/outputs
        // They create other components instead
    }

    /// T033: Render the node's UI
    ///
    /// Displays:
    /// - Component name text input
    /// - Code editor with Rust syntax highlighting
    /// - Save code checkbox
    /// - Execute button (or status indicator if compiling/compiled)
    pub fn render_ui(&mut self, ui: &mut egui::Ui) -> Result<(), String> {
        ui.vertical(|ui| {
            ui.heading("WASM Component Creator");
            ui.separator();

            // Component name input
            ui.horizontal(|ui| {
                ui.label("Component Name:");
                let name_response = ui.text_edit_singleline(&mut self.component_name);

                // Show validation hint
                if !self.component_name.is_empty() {
                    if let Err(error) = self.validate_name() {
                        ui.label(RichText::new("⚠").color(Color32::from_rgb(255, 180, 0)))
                            .on_hover_text(error);
                    } else {
                        ui.label(RichText::new("✓").color(Color32::from_rgb(100, 200, 100)));
                    }
                }

                // Reset state when name changes
                if name_response.changed() {
                    self.reset_state();
                }
            });

            ui.add_space(5.0);

            // Code editor with syntax highlighting
            ui.label(format!("{} Code:", self.language.display_name()));
            let code_response = self.code_editor.show_with_theme_selector(
                ui,
                &mut self.source_code,
                self.language,
                &mut self.editor_theme,
            );

            // Reset state when code changes
            if code_response.changed() {
                self.reset_state();
            }

            ui.add_space(5.0);

            // Save code checkbox
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.save_code, "Save code in graph file");
                ui.label("(Uncheck to save only component name)");
            });

            ui.add_space(5.0);

            // Execute button or status display
            match &self.compilation_state {
                CompilationState::Idle => {
                    if ui.button("🔨 Execute (Compile Component)").clicked() {
                        if let Err(error) = self.on_execute_clicked() {
                            self.last_error = Some(error);
                        }
                    }
                }
                CompilationState::Compiling { started_at, .. } => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        let elapsed = chrono::Utc::now().signed_duration_since(*started_at);
                        ui.label(format!("Compiling... ({}s)", elapsed.num_seconds()));
                    });
                }
                CompilationState::Success { build_time_ms, .. } => {
                    ui.label(RichText::new(format!("✓ Compiled successfully in {}ms", build_time_ms))
                        .color(Color32::from_rgb(100, 200, 100)));

                    if ui.button("Compile Again").clicked() {
                        if let Err(error) = self.on_execute_clicked() {
                            self.last_error = Some(error);
                        }
                    }
                }
                CompilationState::Failed { error_message, line_number, .. } => {
                    ui.label(RichText::new("✗ Compilation failed")
                        .color(Color32::from_rgb(255, 100, 100)));

                    if let Some(line) = line_number {
                        ui.label(format!("Error at line {}: {}", line, error_message));
                    } else {
                        ui.label(error_message);
                    }

                    if ui.button("Try Again").clicked() {
                        if let Err(error) = self.on_execute_clicked() {
                            self.last_error = Some(error);
                        }
                    }
                }
            }

            // Show last error if any
            if let Some(error) = &self.last_error {
                ui.separator();
                ui.colored_label(Color32::from_rgb(255, 100, 100), error);
            }

            // Show statistics
            ui.separator();
            ui.horizontal(|ui| {
                let line_count = CodeEditorWidget::line_count(&self.source_code);
                let byte_count = self.source_code.len();
                ui.label(format!("Lines: {} | Size: {} bytes", line_count, byte_count));
            });
        });

        Ok(())
    }

    /// T034, T073: Validate component name with comprehensive error messages
    ///
    /// Rules:
    /// - Must be PascalCase (starts with uppercase letter)
    /// - Can contain letters, numbers, underscores
    /// - Length: 3-50 characters
    /// - Regex: ^[A-Z][a-zA-Z0-9_]*$
    fn validate_name(&self) -> Result<(), String> {
        let name = &self.component_name;

        // T073: Check for empty name
        if name.is_empty() {
            return Err("Component name cannot be empty. Please enter a name in PascalCase (e.g., MyComponent, TripleNumber)".to_string());
        }

        // Check length
        if name.len() < 3 {
            return Err(format!("Component name '{}' is too short ({}  characters). Must be at least 3 characters (e.g., 'Add', 'Sum', 'HttpFetch')", name, name.len()));
        }

        if name.len() > 50 {
            return Err(format!("Component name is too long ({} characters). Must be at most 50 characters. Consider abbreviating.", name.len()));
        }

        // T073: Check PascalCase format with helpful error messages
        let pascal_case_regex = Regex::new(r"^[A-Z][a-zA-Z0-9_]*$").unwrap();
        if !pascal_case_regex.is_match(name) {
            // Provide specific error based on the problem
            if name.chars().next().unwrap().is_lowercase() {
                return Err(format!("Component name '{}' must start with an uppercase letter (PascalCase). Try '{}' instead.", name, capitalize_first(name)));
            } else if name.contains('-') {
                return Err(format!("Component name '{}' cannot contain hyphens. Use PascalCase instead (e.g., '{}').", name, name.replace('-', "")));
            } else if name.contains(' ') {
                return Err(format!("Component name '{}' cannot contain spaces. Use PascalCase instead (e.g., '{}').", name, name.replace(' ', "")));
            } else if name.contains(|c: char| !c.is_alphanumeric() && c != '_') {
                return Err(format!("Component name '{}' contains invalid characters. Only letters, numbers, and underscores are allowed (PascalCase).", name));
            } else {
                return Err(format!("Component name '{}' must be in PascalCase (e.g., MyComponent, TripleNumber, HttpFetch).", name));
            }
        }

        Ok(())
    }

    /// T035: Validate code size
    ///
    /// Rules:
    /// - Maximum 10,000 lines
    /// - Maximum 500KB (500,000 bytes)
    fn validate_code(&self) -> Result<(), String> {
        let line_count = CodeEditorWidget::line_count(&self.source_code);
        let byte_count = self.source_code.len();

        const MAX_LINES: usize = 10_000;
        const MAX_BYTES: usize = 500_000; // 500KB

        if line_count > MAX_LINES {
            return Err(format!("Code exceeds maximum {} lines (current: {})", MAX_LINES, line_count));
        }

        if byte_count > MAX_BYTES {
            return Err(format!("Code exceeds maximum {}KB (current: {}KB)", MAX_BYTES / 1024, byte_count / 1024));
        }

        Ok(())
    }

    /// T036: Handle Execute button click
    ///
    /// Workflow:
    /// 1. Validate name and code
    /// 2. Parse annotations using template generator
    /// 3. Generate complete component code
    /// 4. Invoke compiler
    /// 5. Update compilation state
    fn on_execute_clicked(&mut self) -> Result<(), String> {
        // Validate inputs
        self.validate_name()?;
        self.validate_code()?;

        if self.source_code.trim().is_empty() {
            return Err("Code cannot be empty".to_string());
        }

        // T054: Log compilation start
        // Note: If a component with this name already exists, it will be replaced
        // during registration (see register_dynamic_component in ComponentRegistry)
        log::info!("Starting compilation for component: {}", self.component_name);

        // Set state to Compiling
        self.compilation_state = CompilationState::Compiling {
            started_at: chrono::Utc::now(),
            pid: None, // Will be set by compiler
        };
        self.last_error = None;

        // Parse annotations and generate metadata (T006, T007)
        // Note: This old struct doesn't have language field, defaulting to Rust
        let metadata = TemplateGenerator::parse_annotations(&self.component_name, &self.source_code, Language::Rust)
            .map_err(|e| format!("Failed to parse annotations: {}", e))?;

        // Select template type (T008)
        let template_type = TemplateGenerator::select_template(&metadata);

        // Generate complete component code (T009)
        let generated_code = TemplateGenerator::generate_component_code(&metadata, &self.source_code, template_type, Language::Rust);

        // Generate WIT interface (T010)
        let wit_definition = TemplateGenerator::generate_wit(&metadata);

        // Generate Cargo.toml (T011)
        let cargo_toml = TemplateGenerator::generate_cargo_toml(&self.component_name);

        // Create compilation config
        let config = CompilationConfig {
            component_name: self.component_name.clone(),
            source_code: generated_code,
            wit_definition,
            cargo_toml,
            timeout: Duration::from_secs(120), // 2 minute timeout (T015)
            language: Language::Rust, // Old struct defaults to Rust
        };

        // Compile the component (T014)
        let compiler = ComponentCompiler::with_default_workspace();
        let compilation_result = compiler.compile(config)
            .map_err(|e| format!("Compilation failed: {}", e))?;

        // T037: Update state based on compilation result
        match compilation_result {
            crate::runtime::CompilationResult::Success { wasm_path, build_time_ms, .. } => {
                log::info!("Compilation succeeded for {} in {}ms", self.component_name, build_time_ms);

                self.compilation_state = CompilationState::Success {
                    compiled_at: chrono::Utc::now(),
                    component_path: wasm_path.clone(),
                    build_time_ms,
                };

                // T038: Register the dynamic component
                // TODO: This will be implemented once we have access to the registry
                // For now, store the component ID
                self.generated_component_id = Some(format!("user:{}", self.component_name));

                Ok(())
            }
            crate::runtime::CompilationResult::Failure { error_message, line_number, .. } => {
                log::error!("Compilation failed for {}: {}", self.component_name, error_message);

                self.compilation_state = CompilationState::Failed {
                    error_message: error_message.clone(),
                    line_number,
                    failed_at: chrono::Utc::now(),
                };

                Err(error_message)
            }
            crate::runtime::CompilationResult::Timeout { elapsed } => {
                let error = format!("Compilation timed out after {:?}", elapsed);
                log::error!("{}", error);

                self.compilation_state = CompilationState::Failed {
                    error_message: error.clone(),
                    line_number: None,
                    failed_at: chrono::Utc::now(),
                };

                Err(error)
            }
        }
    }

    /// T037: Reset state to Idle when code or name changes
    fn reset_state(&mut self) {
        if !matches!(self.compilation_state, CompilationState::Idle) {
            self.compilation_state = CompilationState::Idle;
            self.last_error = None;
        }
    }

    /// T046: Format error message with optional line number
    ///
    /// Formats compilation errors in a user-friendly way:
    /// - With line number: "Line 5: expected expression, found `;`"
    /// - Without line number: "expected expression, found `;`"
    ///
    /// Also truncates very long error messages to avoid UI overflow.
    pub fn format_error(error_message: &str, line_number: Option<usize>) -> String {
        // Truncate very long error messages
        const MAX_ERROR_LENGTH: usize = 200;
        let truncated = if error_message.len() > MAX_ERROR_LENGTH {
            format!("{}...", &error_message[..MAX_ERROR_LENGTH])
        } else {
            error_message.to_string()
        };

        // Format with line number if available
        match line_number {
            Some(line) => format!("Line {}: {}", line, truncated),
            None => truncated,
        }
    }

    /// Execute method for the node (no-op for creator nodes)
    ///
    /// Creator nodes don't execute in the traditional sense - they create components instead.
    pub fn execute(&self, _inputs: &HashMap<String, NodeValue>) -> Result<HashMap<String, NodeValue>, ComponentError> {
        // Creator nodes don't produce outputs
        Ok(HashMap::new())
    }
}

impl Default for WasmCreatorNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Footer view for WASM Creator nodes
///
/// Renders the code editor, component name input, and compilation controls
pub struct WasmCreatorFooterView {
    /// Code editor widget (created per-instance, not serialized)
    code_editor: CodeEditorWidget,
}

impl WasmCreatorFooterView {
    /// Create a new WASM Creator footer view
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            code_editor: CodeEditorWidget::new().with_rows(25),
        })
    }

    /// Convert component name to filename (PascalCase -> snake_case)
    fn component_name_to_filename(component_name: &str) -> String {
        component_name
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_ascii_lowercase().to_string()
                } else if c.is_ascii_uppercase() {
                    format!("_{}", c.to_ascii_lowercase())
                } else {
                    c.to_string()
                }
            })
            .collect()
    }

    /// Execute compilation workflow
    fn execute_compilation(creator_data: &mut crate::graph::node::WasmCreatorNodeData) -> Result<(), String> {
        // Validate inputs
        validate_name(&creator_data.component_name)?;
        validate_code(&creator_data.source_code)?;

        if creator_data.source_code.trim().is_empty() {
            return Err("Code cannot be empty".to_string());
        }

        // Log compilation start
        log::info!("Starting compilation for component: {}", creator_data.component_name);

        // Set state to Compiling
        creator_data.compilation_state = CompilationState::Compiling {
            started_at: chrono::Utc::now(),
            pid: None,
        };

        // Parse annotations and generate metadata
        let metadata = TemplateGenerator::parse_annotations(&creator_data.component_name, &creator_data.source_code, creator_data.language)
            .map_err(|e| format!("Failed to parse annotations: {}", e))?;

        // Select template type
        let template_type = TemplateGenerator::select_template(&metadata);

        // Generate complete component code
        let generated_code = TemplateGenerator::generate_component_code(&metadata, &creator_data.source_code, template_type, creator_data.language);

        // Generate WIT interface
        let wit_definition = TemplateGenerator::generate_wit(&metadata);

        // Generate Cargo.toml
        let cargo_toml = TemplateGenerator::generate_cargo_toml(&creator_data.component_name);

        // Create compilation config
        let config = CompilationConfig {
            component_name: creator_data.component_name.clone(),
            source_code: generated_code,
            wit_definition,
            cargo_toml,
            timeout: Duration::from_secs(120),
            language: creator_data.language,
        };

        // Compile the component
        let compiler = ComponentCompiler::with_default_workspace();
        let compilation_result = compiler.compile(config)
            .map_err(|e| format!("Compilation failed: {}", e))?;

        // Update state based on compilation result
        match compilation_result {
            crate::runtime::CompilationResult::Success { wasm_path, build_time_ms, .. } => {
                log::info!("Compilation succeeded for {} in {}ms", creator_data.component_name, build_time_ms);

                // Copy the compiled .wasm file to components/bin/ directory
                // Use current directory + "components/bin" for reliable path resolution
                let components_dir = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("components")
                    .join("bin");

                // Ensure directory exists
                match std::fs::create_dir_all(&components_dir) {
                    Ok(_) => {
                        let dest_filename = format!("{}.wasm", Self::component_name_to_filename(&creator_data.component_name));
                        let dest_path = components_dir.join(&dest_filename);

                        match std::fs::copy(&wasm_path, &dest_path) {
                            Ok(_) => {
                                log::info!("Copied compiled component to: {}", dest_path.display());
                                creator_data.compilation_state = CompilationState::Success {
                                    compiled_at: chrono::Utc::now(),
                                    component_path: dest_path, // Use the copied path
                                    build_time_ms,
                                };
                            }
                            Err(e) => {
                                log::error!("Failed to copy component to components/bin/{}: {}", dest_filename, e);
                                creator_data.compilation_state = CompilationState::Success {
                                    compiled_at: chrono::Utc::now(),
                                    component_path: wasm_path.clone(),
                                    build_time_ms,
                                };
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create components/bin/ directory at {}: {}", components_dir.display(), e);
                        creator_data.compilation_state = CompilationState::Success {
                            compiled_at: chrono::Utc::now(),
                            component_path: wasm_path.clone(),
                            build_time_ms,
                        };
                    }
                }

                // Store the component ID
                creator_data.generated_component_id = Some(format!("user:{}", creator_data.component_name));

                Ok(())
            }
            crate::runtime::CompilationResult::Failure { error_message, line_number, .. } => {
                log::error!("Compilation failed for {}: {}", creator_data.component_name, error_message);

                creator_data.compilation_state = CompilationState::Failed {
                    error_message: error_message.clone(),
                    line_number,
                    failed_at: chrono::Utc::now(),
                };

                Err(error_message)
            }
            crate::runtime::CompilationResult::Timeout { elapsed } => {
                let error = format!("Compilation timed out after {:?}", elapsed);
                log::error!("{}", error);

                creator_data.compilation_state = CompilationState::Failed {
                    error_message: error.clone(),
                    line_number: None,
                    failed_at: chrono::Utc::now(),
                };

                Err(error)
            }
        }
    }
}

impl ComponentFooterView for WasmCreatorFooterView {
    fn render_footer(&self, ui: &mut egui::Ui, node: &mut GraphNode) -> Result<(), String> {
        // Get the creator data from the node
        let creator_data = node.creator_data.as_mut().ok_or_else(|| {
            "WasmCreatorFooterView requires creator_data in GraphNode".to_string()
        })?;

        ui.vertical(|ui| {
            ui.heading("WASM Component Creator");
            ui.separator();

            // Language selector
            ui.horizontal(|ui| {
                ui.label("Language:");
                ui.label("ⓘ").on_hover_text(
                    "JavaScript components include a full JS runtime (~10-20MB) and may load/execute slower than Rust components.\n\
                    Rust components are much smaller and faster."
                );
                let current_language = creator_data.language;

                egui::ComboBox::from_id_salt("language_selector")
                    .selected_text(current_language.display_name())
                    .show_ui(ui, |ui| {
                        let mut language_changed = false;

                        if ui.selectable_value(&mut creator_data.language, Language::Rust, "Rust").clicked() {
                            language_changed = current_language != Language::Rust;
                        }
                        // Python temporarily disabled due to componentize-py compatibility issues
                        // if ui.selectable_value(&mut creator_data.language, Language::Python, "Python").clicked() {
                        //     language_changed = current_language != Language::Python;
                        // }
                        if ui.selectable_value(&mut creator_data.language, Language::JavaScript, "JavaScript").clicked() {
                            language_changed = current_language != Language::JavaScript;
                        }

                        // If language changed, update template
                        if language_changed {
                            creator_data.source_code = WasmCreatorNode::default_template(creator_data.language);
                            // Reset compilation state
                            creator_data.compilation_state = CompilationState::Idle;
                        }
                    });
            });

            ui.add_space(5.0);

            // Component name input
            ui.horizontal(|ui| {
                ui.label("Component Name:");
                let name_response = ui.text_edit_singleline(&mut creator_data.component_name);

                // Show validation hint
                if !creator_data.component_name.is_empty() {
                    if let Err(error) = validate_name(&creator_data.component_name) {
                        ui.label(RichText::new("⚠").color(Color32::from_rgb(255, 180, 0)))
                            .on_hover_text(error);
                    } else {
                        ui.label(RichText::new("✓").color(Color32::from_rgb(100, 200, 100)));
                    }
                }

                // Reset state when name changes
                if name_response.changed() {
                    if !matches!(creator_data.compilation_state, CompilationState::Idle) {
                        creator_data.compilation_state = CompilationState::Idle;
                    }
                }
            });

            ui.add_space(5.0);

            // Code editor with syntax highlighting
            ui.label(format!("{} Code:", creator_data.language.display_name()));
            let code_response = self.code_editor.show_with_theme_selector(
                ui,
                &mut creator_data.source_code,
                creator_data.language,
                &mut creator_data.editor_theme,
            );

            // Reset state when code changes
            if code_response.changed() {
                if !matches!(creator_data.compilation_state, CompilationState::Idle) {
                    creator_data.compilation_state = CompilationState::Idle;
                }
            }

            ui.add_space(5.0);

            // Save code checkbox
            ui.horizontal(|ui| {
                ui.checkbox(&mut creator_data.save_code, "Save code in graph file");
                ui.label("(Uncheck to save only component name)");
            });

            ui.add_space(5.0);

            // Execute button or status display
            match &creator_data.compilation_state {
                CompilationState::Idle => {
                    if ui.button("🔨 Execute (Compile Component)").clicked() {
                        match Self::execute_compilation(creator_data) {
                            Ok(()) => {
                                // Compilation started successfully
                            }
                            Err(e) => {
                                // Show validation error
                                ui.colored_label(Color32::from_rgb(255, 100, 100), format!("Error: {}", e));
                            }
                        }
                    }

                    ui.add_space(3.0);
                    ui.label(RichText::new("Note: Compilation may take a few seconds and will freeze the UI.")
                        .color(Color32::GRAY)
                        .size(10.0));
                }
                CompilationState::Compiling { started_at, .. } => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        let elapsed = chrono::Utc::now().signed_duration_since(*started_at);
                        ui.label(format!("Compiling... ({}s)", elapsed.num_seconds()));
                    });
                }
                CompilationState::Success { build_time_ms, component_path, .. } => {
                    ui.label(RichText::new(format!("✓ Compiled successfully in {}ms", build_time_ms))
                        .color(Color32::from_rgb(100, 200, 100)));

                    // Show where the component was saved
                    ui.label(RichText::new(format!("Saved to: {}", component_path.display()))
                        .color(Color32::GRAY));

                    ui.add_space(5.0);

                    // Instructions to load the component
                    ui.label(RichText::new("To use this component:")
                        .color(Color32::from_rgb(200, 200, 100)));
                    ui.label("1. Go to File → Reload Components");
                    ui.label(format!("2. Look for '{}' in the palette", creator_data.component_name));

                    ui.add_space(5.0);

                    // Clone values needed in closures to avoid borrowing issues
                    let component_path_clone = component_path.clone();
                    let component_name_clone = creator_data.component_name.clone();

                    ui.horizontal(|ui| {
                        if ui.button("Compile Again").clicked() {
                            match Self::execute_compilation(creator_data) {
                                Ok(()) => {
                                    // Compilation started successfully
                                }
                                Err(e) => {
                                    // Show validation error
                                    ui.colored_label(Color32::from_rgb(255, 100, 100), format!("Error: {}", e));
                                }
                            }
                        }

                        if ui.button("💾 Save File...").clicked() {
                            // Open file dialog to save the WASM file
                            let default_filename = format!("{}.wasm", Self::component_name_to_filename(&component_name_clone));

                            if let Some(save_path) = rfd::FileDialog::new()
                                .add_filter("WebAssembly Component", &["wasm"])
                                .set_file_name(&default_filename)
                                .save_file()
                            {
                                match std::fs::copy(&component_path_clone, &save_path) {
                                    Ok(_) => {
                                        log::info!("Saved component to: {}", save_path.display());
                                        // Show success feedback (could store in a temp field for display)
                                    }
                                    Err(e) => {
                                        log::error!("Failed to save component: {}", e);
                                        ui.colored_label(
                                            Color32::from_rgb(255, 100, 100),
                                            format!("Failed to save: {}", e)
                                        );
                                    }
                                }
                            }
                        }
                    });
                }
                CompilationState::Failed { error_message, line_number, .. } => {
                    ui.label(RichText::new("✗ Compilation failed")
                        .color(Color32::from_rgb(255, 100, 100)));

                    if let Some(line) = line_number {
                        ui.label(format!("Error at line {}: {}", line, error_message));
                    } else {
                        ui.label(error_message);
                    }

                    if ui.button("Try Again").clicked() {
                        match Self::execute_compilation(creator_data) {
                            Ok(()) => {
                                // Compilation started successfully
                            }
                            Err(e) => {
                                // Show validation error
                                ui.colored_label(Color32::from_rgb(255, 100, 100), format!("Error: {}", e));
                            }
                        }
                    }
                }
            }

            // Show statistics
            ui.separator();
            ui.horizontal(|ui| {
                let line_count = CodeEditorWidget::line_count(&creator_data.source_code);
                let byte_count = creator_data.source_code.len();
                ui.label(format!("Lines: {} | Size: {} bytes", line_count, byte_count));
            });
        });

        Ok(())
    }
}

/// Helper function to validate code size
fn validate_code(source_code: &str) -> Result<(), String> {
    let line_count = CodeEditorWidget::line_count(source_code);
    let byte_count = source_code.len();

    const MAX_LINES: usize = 10_000;
    const MAX_BYTES: usize = 500_000; // 500KB

    if line_count > MAX_LINES {
        return Err(format!("Code exceeds maximum {} lines (current: {})", MAX_LINES, line_count));
    }

    if byte_count > MAX_BYTES {
        return Err(format!("Code exceeds maximum {}KB (current: {}KB)", MAX_BYTES / 1024, byte_count / 1024));
    }

    Ok(())
}

/// Helper function to validate component name (same logic as WasmCreatorNode)
fn validate_name(name: &str) -> Result<(), String> {
    // T073: Check for empty name
    if name.is_empty() {
        return Err("Component name cannot be empty. Please enter a name in PascalCase (e.g., MyComponent, TripleNumber)".to_string());
    }

    // Check length
    if name.len() < 3 {
        return Err(format!("Component name '{}' is too short ({}  characters). Must be at least 3 characters (e.g., 'Add', 'Sum', 'HttpFetch')", name, name.len()));
    }

    if name.len() > 50 {
        return Err(format!("Component name is too long ({} characters). Must be at most 50 characters. Consider abbreviating.", name.len()));
    }

    // T073: Check PascalCase format with helpful error messages
    let pascal_case_regex = Regex::new(r"^[A-Z][a-zA-Z0-9_]*$").unwrap();
    if !pascal_case_regex.is_match(name) {
        // Provide specific error based on the problem
        if name.chars().next().unwrap().is_lowercase() {
            return Err(format!("Component name '{}' must start with an uppercase letter (PascalCase). Try '{}' instead.", name, capitalize_first(name)));
        } else if name.contains('-') {
            return Err(format!("Component name '{}' cannot contain hyphens. Use PascalCase instead (e.g., '{}').", name, name.replace('-', "")));
        } else if name.contains(' ') {
            return Err(format!("Component name '{}' cannot contain spaces. Use PascalCase instead (e.g., '{}').", name, name.replace(' ', "")));
        } else if name.contains(|c: char| !c.is_alphanumeric() && c != '_') {
            return Err(format!("Component name '{}' contains invalid characters. Only letters, numbers, and underscores are allowed (PascalCase).", name));
        } else {
            return Err(format!("Component name '{}' must be in PascalCase (e.g., MyComponent, TripleNumber, HttpFetch).", name));
        }
    }

    Ok(())
}

/// T040: Register the WASM Creator Node in the component registry
pub fn register_wasm_creator_node(registry: &mut crate::graph::node::ComponentRegistry) {
    let spec = WasmCreatorNode::spec()
        .with_footer_view(WasmCreatorFooterView::new());
    registry.register_builtin(spec);
    log::info!("Registered WASM Creator Node with footer view");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creator_has_defaults() {
        let node = WasmCreatorNode::new();
        assert_eq!(node.component_name, "");
        assert_eq!(node.save_code, true);
        assert!(matches!(node.compilation_state, CompilationState::Idle));
        assert!(node.last_error.is_none());
        assert!(node.generated_component_id.is_none());
    }

    #[test]
    fn test_validate_name_too_short() {
        let mut node = WasmCreatorNode::new();
        node.component_name = "AB".to_string();
        assert!(node.validate_name().is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let mut node = WasmCreatorNode::new();
        node.component_name = "A".repeat(51);
        assert!(node.validate_name().is_err());
    }

    #[test]
    fn test_validate_name_not_pascal_case() {
        let mut node = WasmCreatorNode::new();

        node.component_name = "lowercase".to_string();
        assert!(node.validate_name().is_err());

        node.component_name = "snake_case".to_string();
        assert!(node.validate_name().is_err());

        node.component_name = "123Number".to_string();
        assert!(node.validate_name().is_err());
    }

    #[test]
    fn test_validate_name_valid() {
        let mut node = WasmCreatorNode::new();

        node.component_name = "TripleNumber".to_string();
        assert!(node.validate_name().is_ok());

        node.component_name = "HTTPFetcher".to_string();
        assert!(node.validate_name().is_ok());

        node.component_name = "MyComponent123".to_string();
        assert!(node.validate_name().is_ok());
    }

    #[test]
    fn test_validate_code_empty() {
        let mut node = WasmCreatorNode::new();
        node.source_code = "".to_string();
        assert!(node.validate_code().is_ok()); // Empty is valid for size check
    }

    #[test]
    fn test_validate_code_too_many_lines() {
        let mut node = WasmCreatorNode::new();
        let mut huge_code = String::new();
        for i in 0..10_001 {
            huge_code.push_str(&format!("// Line {}\n", i));
        }
        node.source_code = huge_code;
        assert!(node.validate_code().is_err());
    }

    #[test]
    fn test_validate_code_too_large() {
        let mut node = WasmCreatorNode::new();
        node.source_code = "a".repeat(500_001); // 500KB + 1 byte
        assert!(node.validate_code().is_err());
    }

    #[test]
    fn test_reset_state() {
        let mut node = WasmCreatorNode::new();
        node.compilation_state = CompilationState::Success {
            compiled_at: chrono::Utc::now(),
            component_path: std::path::PathBuf::from("/tmp/test.wasm"),
            build_time_ms: 1000,
        };
        node.last_error = Some("Old error".to_string());

        node.reset_state();

        assert!(matches!(node.compilation_state, CompilationState::Idle));
        assert!(node.last_error.is_none());
    }

    #[test]
    fn test_execute_returns_empty() {
        let node = WasmCreatorNode::new();
        let result = node.execute(&HashMap::new()).unwrap();
        assert!(result.is_empty());
    }
}
