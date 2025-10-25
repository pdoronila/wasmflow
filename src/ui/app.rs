//! Main application state and UI logic

// Sub-modules for organized application logic
mod components;
mod composition;
mod execution;
mod permissions;
mod state;

use super::canvas::NodeCanvas;
use super::dialogs::{
    AboutDialog, GraphMetadataDialog, PermissionDialog, PermissionsViewDialog,
    UnsavedChangesAction, UnsavedChangesDialog,
};
use super::palette::{Palette, PaletteAction};
use super::spotlight::{SpotlightAction, SpotlightSearch};
use super::theme::Theme;
use crate::builtin::{
    register_constant_nodes, register_continuous_example,
    register_http_server_listener, register_wasm_creator_node,
};
use crate::graph::command::CommandHistory;
use crate::graph::graph::NodeGraph;
use crate::graph::node::ComponentRegistry;
use crate::runtime::capabilities::{CapabilityGrant, CapabilitySet};
use crate::runtime::continuous::{ContinuousExecutionManager, ExecutionResult};
use crate::runtime::engine::{register_builtin_executors, ExecutionEngine};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use uuid::Uuid;

/// Main WasmFlow application
pub struct WasmFlowApp {
    /// Current graph being edited
    graph: NodeGraph,
    /// Component registry for available nodes
    registry: ComponentRegistry,
    /// Execution engine
    engine: ExecutionEngine,
    /// Visual node editor canvas
    canvas: NodeCanvas,
    /// Command history for undo/redo
    history: CommandHistory,
    /// Status message
    status_message: String,
    /// Error message (if any)
    error_message: Option<String>,
    /// Current file path (if graph has been saved)
    current_file: Option<std::path::PathBuf>,
    /// Dirty flag tracking unsaved changes
    dirty: bool,
    /// Unsaved changes confirmation dialog
    unsaved_changes_dialog: UnsavedChangesDialog,
    /// Pending action after unsaved changes dialog
    pending_action: Option<PendingAction>,
    /// Recent files list (up to 10)
    recent_files: Vec<PathBuf>,
    /// T073: Permission request dialog
    permission_dialog: PermissionDialog,
    /// T073: Pending component awaiting permission approval
    pending_permission_request: Option<PendingPermissionRequest>,
    /// T078: Permission view dialog
    permissions_view_dialog: PermissionsViewDialog,
    /// T100: About dialog
    about_dialog: AboutDialog,
    /// T091: Component palette with search
    palette: Palette,
    /// T092: Graph metadata editor dialog
    metadata_dialog: GraphMetadataDialog,
    /// Spotlight search for quick node creation
    spotlight: SpotlightSearch,
    /// Last space key press time for double-space detection
    last_space_time: Option<std::time::Instant>,
    /// Application theme
    theme: Theme,
    /// Incremental execution state
    execution_state: Option<IncrementalExecutionState>,
    /// Continuous execution manager for long-running nodes
    continuous_manager: ContinuousExecutionManager,
    /// Channel for receiving continuous execution results
    continuous_result_rx: Receiver<ExecutionResult>,
    /// Channel for sending continuous execution results
    continuous_result_tx: std::sync::mpsc::Sender<ExecutionResult>,
    /// Channel for receiving downstream node execution results (triggered by continuous nodes)
    downstream_result_rx: Receiver<(Uuid, NodeExecutionResult)>,
    /// Channel for sending downstream node execution results
    downstream_result_tx: std::sync::mpsc::Sender<(Uuid, NodeExecutionResult)>,
    /// T028: Component composer for WAC composition
    composer: crate::runtime::wac_integration::ComponentComposer,
    /// T032: Error dialog for composition failures
    composition_error: Option<String>,
    /// T037: View stack for drill-down navigation
    view_stack: crate::graph::drill_down::ViewStack,
}

/// State for incremental execution on the main thread
struct IncrementalExecutionState {
    /// List of nodes to execute in order
    execution_order: Vec<Uuid>,
    /// Index of the current node being executed
    current_index: usize,
    /// Channel for receiving execution results from background thread
    execution_receiver: Option<
        Receiver<Result<std::collections::HashMap<String, crate::graph::node::NodeValue>, String>>,
    >,
}

/// Result from background node execution
type NodeExecutionResult =
    Result<std::collections::HashMap<String, crate::graph::node::NodeValue>, String>;

/// T073: Component awaiting permission approval
#[derive(Debug, Clone)]
struct PendingPermissionRequest {
    /// Component specification
    component_spec: crate::graph::node::ComponentSpec,
    /// Requested capabilities
    capabilities: CapabilitySet,
    /// Position where to create the node
    position: egui::Pos2,
}

/// Actions that can be pending confirmation from unsaved changes dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingAction {
    /// User wants to create a new graph
    NewGraph,
    /// User wants to quit the application
    Quit,
}

impl WasmFlowApp {
    /// Create a new WasmFlow application
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create component registry and register builtin nodes
        let mut registry = ComponentRegistry::new();
        register_constant_nodes(&mut registry);
        register_wasm_creator_node(&mut registry);
        register_continuous_example(&mut registry);
        register_http_server_listener(&mut registry);

        // Create execution engine and register executors
        // The engine creates its own ComponentManager internally
        let mut engine = ExecutionEngine::new();
        register_builtin_executors(&mut engine);

        // Create initial graph
        let graph = NodeGraph::new("Untitled Graph".to_string(), "User".to_string());

        let recent_files = Self::load_recent_files();

        // Create channel for continuous execution results
        let (continuous_result_tx, continuous_result_rx) = channel();

        // Create channel for downstream execution results
        let (downstream_result_tx, downstream_result_rx) = channel();

        let mut app = Self {
            graph,
            registry,
            engine,
            canvas: NodeCanvas::new(),
            history: CommandHistory::new(),
            status_message: "Welcome to WasmFlow! Create nodes from the palette.".to_string(),
            error_message: None,
            current_file: None,
            dirty: false,
            unsaved_changes_dialog: UnsavedChangesDialog::new(),
            pending_action: None,
            recent_files,
            permission_dialog: PermissionDialog::new(),
            pending_permission_request: None,
            permissions_view_dialog: PermissionsViewDialog::new(),
            about_dialog: AboutDialog::new(),
            palette: Palette::new(),
            metadata_dialog: GraphMetadataDialog::new(),
            spotlight: SpotlightSearch::new(),
            last_space_time: None,
            theme: Theme::dark(),
            execution_state: None,
            continuous_manager: ContinuousExecutionManager::new(),
            continuous_result_rx,
            continuous_result_tx,
            downstream_result_rx,
            downstream_result_tx,
            composer: crate::runtime::wac_integration::ComponentComposer::new(), // T028
            composition_error: None,                                             // T032
            view_stack: crate::graph::drill_down::ViewStack::new(),              // T037
        };

        // Auto-load components from components/ directory on startup
        app.reload_components();

        app
    }



    /// Handle quit action (with confirmation if dirty)
    fn quit(&mut self, ctx: &egui::Context) {
        if self.dirty {
            // Show unsaved changes dialog
            self.unsaved_changes_dialog.open();
            self.pending_action = Some(PendingAction::Quit);
        } else {
            // Quit immediately if no unsaved changes
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Handle unsaved changes dialog response
    fn handle_unsaved_changes_dialog(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.unsaved_changes_dialog.show(ctx) {
            match action {
                UnsavedChangesAction::Save => {
                    // Save the graph first
                    self.save_graph();

                    // Only proceed with pending action if save succeeded (dirty flag cleared)
                    if !self.dirty {
                        self.execute_pending_action(ctx);
                    }
                }
                UnsavedChangesAction::Discard => {
                    // Discard changes and execute pending action
                    self.dirty = false;
                    self.execute_pending_action(ctx);
                }
                UnsavedChangesAction::Cancel => {
                    // Cancel the pending action
                    self.pending_action = None;
                }
            }
        }
    }

    /// Execute the pending action after unsaved changes dialog
    fn execute_pending_action(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.pending_action.take() {
            match action {
                PendingAction::NewGraph => {
                    self.create_new_graph();
                }
                PendingAction::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }








    /// T099: Control palette visibility (for CLI support)
    pub fn set_palette_visible(&mut self, _visible: bool) {
        // TODO: Implement palette visibility toggle
        // For now, this is a placeholder - full implementation would require
        // adding a boolean field to WasmFlowApp and checking it in render_palette()
        log::warn!("--no-palette flag is not yet fully implemented");
    }

    /// T092: Open the metadata editor dialog
    fn open_metadata_dialog(&mut self) {
        self.metadata_dialog.open(
            self.graph.name.clone(),
            self.graph.metadata.author.clone(),
            self.graph.metadata.description.clone(),
            self.graph.metadata.created_at.clone(),
            self.graph.metadata.modified_at.clone(),
        );
    }

    /// T092: Handle metadata dialog response
    fn handle_metadata_dialog(&mut self, ctx: &egui::Context) {
        self.metadata_dialog.show(ctx);

        // Check if user saved changes
        if self.metadata_dialog.saved() {
            let (name, author, description) = self.metadata_dialog.get_metadata();

            // Update graph metadata
            self.graph.name = name;
            self.graph.metadata.author = author;
            self.graph.metadata.description = description;
            self.graph.metadata.touch();

            // Mark as dirty
            self.dirty = true;
            self.status_message = "Graph metadata updated".to_string();

            // Reset dialog
            self.metadata_dialog.reset();
        }
    }

    /// Render the top menu bar
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Graph").clicked() {
                        self.new_graph();
                        ui.close();
                    }
                    if ui.button("Open... (Ctrl+O)").clicked() {
                        self.load_graph();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Save (Ctrl+S)").clicked() {
                        self.save_graph();
                        ui.close();
                    }
                    if ui.button("Save As... (Ctrl+Shift+S)").clicked() {
                        self.save_graph_as();
                        ui.close();
                    }
                    ui.separator();

                    // Recent Files submenu
                    ui.menu_button("Recent Files", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label("No recent files");
                        } else {
                            // Clone to avoid borrow checker issues
                            let recent_files = self.recent_files.clone();
                            for path in recent_files {
                                let file_name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");

                                if ui
                                    .button(file_name)
                                    .on_hover_text(path.display().to_string())
                                    .clicked()
                                {
                                    self.open_recent_file(path);
                                    ui.close();
                                }
                            }
                        }
                    });

                    ui.separator();

                    // Component loading menu items
                    if ui.button("Load Component...").clicked() {
                        self.load_component();
                        ui.close();
                    }
                    if ui.button("Reload Components").clicked() {
                        self.reload_components();
                        ui.close();
                    }

                    ui.separator();
                    if ui.button("Quit").clicked() {
                        self.quit(ctx);
                        ui.close();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    ui.add_enabled_ui(self.history.can_undo(), |ui| {
                        if ui.button("Undo (Ctrl+Z)").clicked() {
                            self.undo();
                            ui.close();
                        }
                    });
                    ui.add_enabled_ui(self.history.can_redo(), |ui| {
                        if ui.button("Redo (Ctrl+Y)").clicked() {
                            self.redo();
                            ui.close();
                        }
                    });

                    ui.separator();

                    // T092: Graph metadata editor
                    if ui.button("Graph Properties...").clicked() {
                        self.open_metadata_dialog();
                        ui.close();
                    }
                });

                // T100: Help menu with About dialog
                ui.menu_button("Help", |ui| {
                    if ui.button("About WasmFlow").clicked() {
                        self.about_dialog.open();
                        ui.close();
                    }
                });

                ui.separator();

                // T041: Back button - only show in drill-down mode
                if self.view_stack.is_drill_down() {
                    if ui.button("⬅ Back").clicked() {
                        self.handle_back_navigation();
                    }
                    ui.separator();
                }

                if ui.button("▶ Execute").clicked() {
                    self.execute_graph();
                }

                // T031: Compose button - only enabled when 2+ nodes are selected AND viewing main canvas
                let selected_count = self.graph.nodes.values().filter(|n| n.selected).count();
                let can_compose = selected_count >= 2 && self.view_stack.is_main_canvas();
                ui.add_enabled_ui(can_compose, |ui| {
                    let compose_button = ui.button("🔧 Compose");
                    if compose_button.clicked() {
                        self.handle_compose_action();
                    }
                    if !self.view_stack.is_main_canvas() {
                        compose_button.on_hover_text("Return to main canvas to compose nodes");
                    } else if selected_count < 2 {
                        compose_button.on_hover_text("Select at least 2 nodes to compose");
                    } else {
                        compose_button.on_hover_text(format!(
                            "Compose {} selected nodes into a composite",
                            selected_count
                        ));
                    }
                });

                ui.separator();

                // Show dirty indicator
                if self.dirty {
                    ui.label("●"); // Dirty indicator
                }

                // Show current file name
                if let Some(path) = &self.current_file {
                    ui.label(path.file_name().unwrap().to_string_lossy().to_string());
                } else {
                    ui.label("Untitled");
                }

                ui.separator();

                // T039: Breadcrumb navigation
                if self.view_stack.is_drill_down() {
                    ui.separator();
                    ui.label("📍");
                    let breadcrumbs = self.view_stack.breadcrumb_path();
                    for (i, (name, _depth)) in breadcrumbs.iter().enumerate() {
                        if i > 0 {
                            ui.label("›");
                        }
                        ui.label(egui::RichText::new(name).strong());
                    }
                    ui.separator();
                }

                // T040: Show node count from current view context
                let (node_count, connection_count) = match self.view_stack.current() {
                    crate::graph::drill_down::ViewContext::MainCanvas => {
                        (self.graph.nodes.len(), self.graph.connections.len())
                    }
                    crate::graph::drill_down::ViewContext::DrillDown {
                        internal_nodes,
                        internal_edges,
                        ..
                    } => (internal_nodes.len(), internal_edges.len()),
                };
                ui.label(format!("Nodes: {}", node_count));
                ui.label(format!("Connections: {}", connection_count));
            });
        });
    }

    /// Render the status bar at the bottom
    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
                } else {
                    ui.label(&self.status_message);
                }
            });
        });
    }

    /// T091: Render the component palette on the left with search functionality
    fn render_palette(&mut self, ctx: &egui::Context) {
        // Show the palette widget and handle any actions
        if let Some(action) = self.palette.show(ctx, &self.registry, &self.theme) {
            match action {
                PaletteAction::AddComponent { spec, position } => {
                    // T073: Check if this is a user-defined component that needs permission
                    if let crate::graph::node::ComponentType::UserDefined(_) = spec.component_type {
                        // Extract capabilities from component metadata
                        let requested_capabilities =
                            Self::parse_capabilities(&spec.required_capabilities);

                        // T080: Check for capability escalation
                        // Look for existing grants for this component_id
                        let existing_grant = self
                            .graph
                            .nodes
                            .values()
                            .find(|node| node.component_id == spec.id)
                            .and_then(|node| self.graph.get_capability_grant(node.id));

                        let needs_approval = if let Some(grant) = existing_grant {
                            // Compare requested vs granted capabilities
                            // If they differ, we need re-approval
                            grant.capability_set != requested_capabilities
                        } else {
                            // No existing grant - always need approval
                            true
                        };

                        if needs_approval {
                            // Show permission dialog for approval
                            self.permission_dialog.open(
                                spec.name.clone(),
                                spec.description.clone(),
                                requested_capabilities.clone(),
                            );

                            // Store pending request for later approval
                            self.pending_permission_request = Some(PendingPermissionRequest {
                                component_spec: spec,
                                capabilities: requested_capabilities,
                                position,
                            });
                        } else {
                            // Reuse existing grant - add node directly
                            let node = spec.create_node(position);
                            let grant = existing_grant.unwrap(); // Safe because we checked above

                            // Create a new grant for this node instance
                            let new_grant = CapabilityGrant {
                                node_id: node.id,
                                capability_set: grant.capability_set.clone(),
                                granted_at: chrono::Utc::now().to_rfc3339(),
                                scope: grant.scope.clone(),
                            };
                            self.graph.grant_capability(new_grant);

                            // Add the node
                            let cmd = crate::graph::command::Command::AddNode { node };
                            if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                                self.error_message = Some(format!("Failed to add node: {}", e));
                            } else {
                                self.status_message = format!(
                                    "Added {} node (reusing existing permissions)",
                                    spec.name
                                );
                                self.error_message = None;
                                self.dirty = true;
                            }
                        }
                    } else {
                        // Builtin component - add directly without permission dialog
                        let node = spec.create_node(position);
                        let cmd = crate::graph::command::Command::AddNode { node };

                        if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                            self.error_message = Some(format!("Failed to add node: {}", e));
                        } else {
                            self.status_message = format!("Added {} node", spec.name);
                            self.error_message = None;
                            self.dirty = true;
                        }
                    }
                }
            }
        }
    }

    /// Render the main canvas area
    fn render_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // T040: Check if we're in drill-down mode
            match self.view_stack.current() {
                crate::graph::drill_down::ViewContext::MainCanvas => {
                    // Show the main graph
                    self.canvas.show(ui, &mut self.graph, &self.registry);
                }
                crate::graph::drill_down::ViewContext::DrillDown {
                    composite_node_id: _,
                    composite_node_name,
                    internal_nodes,
                    internal_edges,
                } => {
                    // Create a temporary graph from the internal structure
                    let mut temp_graph = crate::graph::NodeGraph::new(
                        format!("Internal: {}", composite_node_name),
                        "Composite".to_string(),
                    );

                    // Clone internal nodes into the temp graph
                    temp_graph.nodes = internal_nodes.clone();
                    temp_graph.connections = internal_edges.clone();

                    // Show the internal structure
                    self.canvas.show(ui, &mut temp_graph, &self.registry);
                }
            }
        });

        // T043: Only process modifications when viewing main canvas (read-only drill-down)
        if self.view_stack.is_main_canvas() {
            // Process pending node deletions through command history
            if !self.canvas.pending_deletions.is_empty() {
                for node_id in self.canvas.pending_deletions.drain(..) {
                    // Create RemoveNode command
                    let cmd = crate::graph::command::Command::RemoveNode {
                        node_id,
                        node: crate::graph::node::GraphNode::new(
                            String::new(),
                            String::new(),
                            egui::Pos2::ZERO,
                        ), // Placeholder, will be filled by execute()
                        connections: Vec::new(), // Placeholder, will be filled by execute()
                    };

                    if let Err(e) = self.history.execute(cmd, &mut self.graph) {
                        self.error_message = Some(format!("Failed to delete node: {}", e));
                    } else {
                        self.status_message = "Node deleted".to_string();
                        self.error_message = None;
                        self.dirty = true;
                    }
                }
            }
        } else {
            // In drill-down mode - discard any modification attempts
            if !self.canvas.pending_deletions.is_empty() {
                self.canvas.pending_deletions.clear();
                self.status_message =
                    "Drill-down view is read-only. Return to main canvas to make changes."
                        .to_string();
            }
        }

        // T078: Process pending permission view request
        if let Some(node_id) = self.canvas.pending_permission_view.take() {
            if let Some(node) = self.graph.nodes.get(&node_id) {
                let grant = self.graph.get_capability_grant(node_id).cloned();
                self.permissions_view_dialog
                    .open(node_id, node.display_name.clone(), grant);
            }
        }

        // T040: Process pending drill-down request
        if let Some(composite_node_id) = self.canvas.pending_drill_down.take() {
            self.handle_drill_down(composite_node_id);
        }

        // Handle pending continuous node start requests
        let pending_starts: Vec<Uuid> = self.canvas.pending_continuous_start.drain(..).collect();
        if !pending_starts.is_empty() {
            // T044: Pass Arc<Mutex<NodeGraph>> so continuous nodes can read updated input values
            let graph_arc = std::sync::Arc::new(std::sync::Mutex::new(self.graph.clone()));
            let component_manager = self.engine.component_manager();
            let result_tx = self.continuous_result_tx.clone();

            for node_id in pending_starts {
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    if let Some(config) = &mut node.continuous_config {
                        // T041: Clear error state if restarting from error
                        if matches!(
                            config.runtime_state.execution_state,
                            crate::graph::node::ContinuousExecutionState::Error
                        ) {
                            config.runtime_state.last_error = None;
                            log::info!("Clearing error state for continuous node {}", node_id);
                        }

                        // Update state to Starting
                        config.runtime_state.execution_state =
                            crate::graph::node::ContinuousExecutionState::Starting;

                        // Start the continuous execution
                        if let Err(e) = self.continuous_manager.start_node(
                            node_id,
                            graph_arc.clone(),
                            component_manager.clone(),
                            result_tx.clone(),
                        ) {
                            self.error_message =
                                Some(format!("Failed to start continuous node: {}", e));
                            config.runtime_state.execution_state =
                                crate::graph::node::ContinuousExecutionState::Error;
                            config.runtime_state.last_error = Some(e.to_string());
                        } else {
                            self.status_message = format!("Started continuous node");
                        }
                    }
                }
            }
            self.canvas.mark_dirty();
        }

        // Handle pending continuous node stop requests
        let pending_stops: Vec<Uuid> = self.canvas.pending_continuous_stop.drain(..).collect();
        if !pending_stops.is_empty() {
            for node_id in pending_stops {
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    if let Some(config) = &mut node.continuous_config {
                        // Update state to Stopping
                        config.runtime_state.execution_state =
                            crate::graph::node::ContinuousExecutionState::Stopping;

                        // Stop the continuous execution
                        if let Err(e) = self.continuous_manager.stop_node(node_id) {
                            self.error_message =
                                Some(format!("Failed to stop continuous node: {}", e));
                            config.runtime_state.execution_state =
                                crate::graph::node::ContinuousExecutionState::Error;
                            config.runtime_state.last_error = Some(e.to_string());
                        } else {
                            self.status_message = format!("Stopping continuous node...");
                        }
                    }
                }
            }
            self.canvas.mark_dirty();
        }
    }
}

impl eframe::App for WasmFlowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process incremental execution step on main thread
        if self.execution_state.is_some() {
            self.process_execution_step();
            ctx.request_repaint(); // Keep processing until done
        }

        // Poll continuous execution results
        self.poll_continuous_results();

        // Poll downstream execution results (triggered by continuous nodes)
        self.poll_downstream_results();

        // Request continuous repaints when there are running nodes (for spinner/elapsed time)
        let has_running_nodes = self.graph.nodes.values().any(|n| {
            matches!(
                n.execution_state,
                crate::graph::node::ExecutionState::Running
            )
        });

        // Also check for continuous nodes in running state
        let has_continuous_running = self.graph.nodes.values().any(|n| {
            if let Some(config) = &n.continuous_config {
                matches!(
                    config.runtime_state.execution_state,
                    crate::graph::node::ContinuousExecutionState::Running
                        | crate::graph::node::ContinuousExecutionState::Starting
                        | crate::graph::node::ContinuousExecutionState::Stopping
                )
            } else {
                false
            }
        });

        if has_running_nodes || has_continuous_running {
            ctx.request_repaint();
        }

        // Intercept window close event to show unsaved changes dialog
        if ctx.input(|i| i.viewport().close_requested())
            && self.dirty
            && !self.unsaved_changes_dialog.is_open()
        {
            // Prevent close and show dialog
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.unsaved_changes_dialog.open();
            self.pending_action = Some(PendingAction::Quit);
        }

        // Handle unsaved changes dialog
        self.handle_unsaved_changes_dialog(ctx);

        // T073: Handle permission dialog
        self.handle_permission_dialog(ctx);

        // T078: Handle permissions view dialog
        self.handle_permissions_view_dialog(ctx);

        // T092: Handle graph metadata dialog
        self.handle_metadata_dialog(ctx);

        // T100: Show about dialog
        self.about_dialog.show(ctx);

        // T032: Show composition error dialog
        self.show_composition_error_dialog(ctx);

        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command) {
            if ctx.input(|i| i.modifiers.shift) {
                // Ctrl+Shift+Z -> Redo (alternative to Ctrl+Y)
                self.redo();
            } else {
                // Ctrl+Z -> Undo
                self.undo();
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::Y) && i.modifiers.command) {
            // Ctrl+Y -> Redo
            self.redo();
        } else if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.command) {
            if ctx.input(|i| i.modifiers.shift) {
                // Ctrl+Shift+S -> Save As
                self.save_graph_as();
            } else {
                // Ctrl+S -> Save
                self.save_graph();
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.command) {
            // Ctrl+O -> Open
            self.load_graph();
        }

        // Handle double-space for spotlight search (only when no dialog is open)
        if !self.spotlight.is_visible()
            && !self.unsaved_changes_dialog.is_open()
            && !self.permission_dialog.is_open()
            && !self.permissions_view_dialog.is_open()
            && !self.about_dialog.is_open()
            && !self.metadata_dialog.is_open()
        {
            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                let now = std::time::Instant::now();

                if let Some(last_time) = self.last_space_time {
                    // Check if this is within 300ms of the last space press
                    if now.duration_since(last_time).as_millis() <= 300 {
                        // Double-space detected! Open spotlight
                        self.spotlight.open();
                        self.last_space_time = None;
                    } else {
                        // Too slow, reset timer
                        self.last_space_time = Some(now);
                    }
                } else {
                    // First space press
                    self.last_space_time = Some(now);
                }
            }
        }

        self.render_menu_bar(ctx);
        self.render_status_bar(ctx);
        self.render_palette(ctx);
        self.render_canvas(ctx);

        // Render spotlight search (must be after canvas to overlay on top)
        let mouse_pos = ctx.input(|i| i.pointer.hover_pos());
        if let Some(action) = self.spotlight.show(ctx, &self.registry, mouse_pos) {
            match action {
                SpotlightAction::AddComponent { spec, position } => {
                    // Use the existing permission handling logic
                    self.handle_add_component_with_permissions(spec, position);
                }
            }
        }
    }
}

impl Drop for WasmFlowApp {
    fn drop(&mut self) {
        // Gracefully shutdown all running continuous nodes
        log::info!("Shutting down WasmFlow application");
        self.continuous_manager.shutdown();
    }
}
